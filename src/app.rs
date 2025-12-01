//! アプリケーション状態管理

use crate::config::Settings;
use crate::ffmpeg::{FfmpegDetector, FfmpegInfo, ProbeResult};
use crate::transcoder::{
    estimate_compression_ratio_advanced, ContentType, TranscodeJob, TranscodeSettings,
    VideoMetadata,
};
use gpui::*;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;

/// 現在の進捗情報（スレッド間共有用）
#[derive(Clone)]
pub struct CurrentProgress {
    /// 進捗率 (0-10000 で 0.00% - 100.00% を表現)
    pub progress_permyriad: Arc<AtomicU32>,
    /// 経過時間（秒 * 100）
    pub elapsed_centisecs: Arc<AtomicU32>,
    /// 残り時間（秒 * 100、0xFFFFFFFF = 不明）
    pub remaining_centisecs: Arc<AtomicU32>,
    /// 現在のFPS * 100
    pub fps_centi: Arc<AtomicU32>,
    /// 総時間（秒 * 100）
    pub total_duration_centisecs: Arc<AtomicU32>,
    /// 現在の処理時間位置（秒 * 100）
    pub current_time_centisecs: Arc<AtomicU32>,
    /// キャンセルフラグ
    pub cancelled: Arc<AtomicBool>,
}

impl Default for CurrentProgress {
    fn default() -> Self {
        Self {
            progress_permyriad: Arc::new(AtomicU32::new(0)),
            elapsed_centisecs: Arc::new(AtomicU32::new(0)),
            remaining_centisecs: Arc::new(AtomicU32::new(0xFFFFFFFF)),
            fps_centi: Arc::new(AtomicU32::new(0)),
            total_duration_centisecs: Arc::new(AtomicU32::new(0)),
            current_time_centisecs: Arc::new(AtomicU32::new(0)),
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl CurrentProgress {
    /// 進捗率を取得 (0.0 - 1.0)
    pub fn get_progress(&self) -> f32 {
        self.progress_permyriad.load(Ordering::Relaxed) as f32 / 10000.0
    }

    /// 進捗率を設定 (0.0 - 1.0)
    pub fn set_progress(&self, progress: f32) {
        let permyriad = (progress * 10000.0).clamp(0.0, 10000.0) as u32;
        self.progress_permyriad.store(permyriad, Ordering::Relaxed);
    }

    /// 経過時間を取得（秒）
    pub fn get_elapsed_secs(&self) -> f32 {
        self.elapsed_centisecs.load(Ordering::Relaxed) as f32 / 100.0
    }

    /// 経過時間を設定（秒）
    pub fn set_elapsed_secs(&self, secs: f32) {
        let centisecs = (secs * 100.0) as u32;
        self.elapsed_centisecs.store(centisecs, Ordering::Relaxed);
    }

    /// 残り時間を取得（秒、Noneは不明）
    pub fn get_remaining_secs(&self) -> Option<f32> {
        let centisecs = self.remaining_centisecs.load(Ordering::Relaxed);
        if centisecs == 0xFFFFFFFF {
            None
        } else {
            Some(centisecs as f32 / 100.0)
        }
    }

    /// 残り時間を設定（秒）
    pub fn set_remaining_secs(&self, secs: Option<f32>) {
        let centisecs = secs.map(|s| (s * 100.0) as u32).unwrap_or(0xFFFFFFFF);
        self.remaining_centisecs.store(centisecs, Ordering::Relaxed);
    }

    /// FPSを取得
    pub fn get_fps(&self) -> f32 {
        self.fps_centi.load(Ordering::Relaxed) as f32 / 100.0
    }

    /// FPSを設定
    pub fn set_fps(&self, fps: f32) {
        let centi = (fps * 100.0) as u32;
        self.fps_centi.store(centi, Ordering::Relaxed);
    }

    /// リセット
    pub fn reset(&self) {
        self.progress_permyriad.store(0, Ordering::Relaxed);
        self.elapsed_centisecs.store(0, Ordering::Relaxed);
        self.remaining_centisecs
            .store(0xFFFFFFFF, Ordering::Relaxed);
        self.fps_centi.store(0, Ordering::Relaxed);
        self.total_duration_centisecs.store(0, Ordering::Relaxed);
        self.current_time_centisecs.store(0, Ordering::Relaxed);
        self.cancelled.store(false, Ordering::Relaxed);
    }

    /// キャンセルフラグを設定
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// キャンセルされたか確認
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// 総時間を設定（秒）
    pub fn set_total_duration_secs(&self, secs: f64) {
        let centisecs = (secs * 100.0) as u32;
        self.total_duration_centisecs
            .store(centisecs, Ordering::Relaxed);
    }

    /// 総時間を取得（秒）
    pub fn get_total_duration_secs(&self) -> f64 {
        self.total_duration_centisecs.load(Ordering::Relaxed) as f64 / 100.0
    }

    /// 現在の処理時間位置を設定（秒）
    pub fn set_current_time_secs(&self, secs: f64) {
        let centisecs = (secs * 100.0) as u32;
        self.current_time_centisecs
            .store(centisecs, Ordering::Relaxed);
    }

    /// 現在の処理時間位置を取得（秒）
    pub fn get_current_time_secs(&self) -> f64 {
        self.current_time_centisecs.load(Ordering::Relaxed) as f64 / 100.0
    }

    /// time_secsベースで進捗率を計算・更新
    pub fn update_progress_from_time(&self, current_time_secs: f64) {
        self.set_current_time_secs(current_time_secs);
        let total = self.get_total_duration_secs();
        if total > 0.0 {
            let progress = (current_time_secs / total).min(1.0) as f32;
            self.set_progress(progress);
        }
    }
}

/// アプリケーションのグローバル状態
#[derive(Clone)]
pub struct AppState {
    /// ファイルキュー
    pub files: Entity<Vec<FileEntry>>,
    /// トランスコード設定
    pub transcode_settings: Entity<TranscodeSettings>,
    /// 現在のジョブ
    pub current_job: Entity<Option<TranscodeJob>>,
    /// アプリケーション設定
    pub settings: Entity<Settings>,
    /// FFmpegパス
    pub ffmpeg_path: Entity<Option<PathBuf>>,
    /// FFmpeg情報（ffprobe用）
    pub ffmpeg_info: Entity<Option<FfmpegInfo>>,
    /// 現在の進捗情報（スレッド間共有）
    pub current_progress: CurrentProgress,
}

impl AppState {
    pub fn new(cx: &mut App) -> Self {
        // 設定をロード
        let settings = Settings::load().unwrap_or_default();

        // FFmpegを検出
        let ffmpeg_info = FfmpegDetector::detect().ok();

        Self {
            files: cx.new(|_| Vec::new()),
            transcode_settings: cx.new(|_| TranscodeSettings::default()),
            current_job: cx.new(|_| None),
            settings: cx.new(|_| settings),
            ffmpeg_path: cx.new(|_| None),
            ffmpeg_info: cx.new(|_| ffmpeg_info),
            current_progress: CurrentProgress::default(),
        }
    }

    /// ファイルをキューに追加
    pub fn add_files(&self, paths: Vec<PathBuf>, cx: &mut App) {
        let settings = self.transcode_settings.read(cx).clone();
        let ffmpeg_info = self.ffmpeg_info.read(cx).clone();
        log::info!(
            "Adding {} files, ffmpeg_info available: {}",
            paths.len(),
            ffmpeg_info.is_some()
        );
        self.files.update(cx, |files, _| {
            for path in paths {
                if Self::is_supported_format(&path) {
                    let mut entry = FileEntry::new(path.clone());
                    // ffprobeでメタデータを取得
                    if let Some(ref info) = ffmpeg_info {
                        entry.probe_metadata(info);
                        log::info!(
                            "Probed {}: duration={:?}",
                            entry.name,
                            entry.metadata.duration
                        );
                    } else {
                        log::warn!(
                            "ffmpeg_info not available, skipping probe for {}",
                            entry.name
                        );
                    }
                    entry.update_estimated_size(&settings);
                    files.push(entry);
                }
            }
        });
    }

    /// ファイルをキューから削除
    pub fn remove_file(&self, index: usize, cx: &mut App) {
        self.files.update(cx, |files, _| {
            if index < files.len() {
                files.remove(index);
            }
        });
    }

    /// キューをクリア
    pub fn clear_files(&self, cx: &mut App) {
        self.files.update(cx, |files, _| {
            files.clear();
        });
    }

    /// すべてのファイルの予測サイズを更新
    pub fn update_all_estimated_sizes(&self, cx: &mut App) {
        let settings = self.transcode_settings.read(cx).clone();
        self.files.update(cx, |files, _| {
            for file in files.iter_mut() {
                file.update_estimated_size(&settings);
            }
        });
    }

    /// サポートされている入力形式かチェック
    fn is_supported_format(path: &PathBuf) -> bool {
        const SUPPORTED_EXTENSIONS: &[&str] = &[
            "mp4", "mkv", "avi", "mov", "webm", "flv", "wmv", "m4v", "ts",
        ];

        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
            .unwrap_or(false)
    }
}

/// ファイルエントリ
#[derive(Clone, Debug)]
pub struct FileEntry {
    /// ファイルパス
    pub path: PathBuf,
    /// ファイル名
    pub name: String,
    /// ファイルサイズ（バイト）
    pub size: u64,
    /// 処理状態
    pub status: FileStatus,
    /// 進捗（0.0 - 1.0）
    pub progress: f32,
    /// 予測出力サイズ（バイト）
    pub estimated_size: Option<u64>,
    /// 動画メタデータ
    pub metadata: VideoMetadata,
}

impl FileEntry {
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

        Self {
            path,
            name,
            size,
            status: FileStatus::Pending,
            progress: 0.0,
            estimated_size: None,
            metadata: VideoMetadata::default(),
        }
    }

    /// ffprobeでメタデータを取得
    pub fn probe_metadata(&mut self, ffmpeg_info: &FfmpegInfo) {
        if let Ok(probe) = ffmpeg_info.probe_video(&self.path) {
            // 解像度
            if let Some((w, h)) = probe.resolution {
                self.metadata.resolution = Some((w, h));
            }
            // フレームレート
            if let Some(fps) = probe.fps {
                self.metadata.fps = Some(fps);
            }
            // 動画の長さ
            if let Some(duration) = probe.duration {
                self.metadata.duration = Some(duration);
            }
            // ビットレート
            if let Some(video_br) = probe.video_bitrate {
                self.metadata.source_video_bitrate = Some(video_br);
            }
            if let Some(audio_br) = probe.audio_bitrate {
                self.metadata.source_audio_bitrate = Some(audio_br);
            }
            if let Some(overall_br) = probe.overall_bitrate {
                self.metadata.source_overall_bitrate = Some(overall_br);
            }

            log::debug!(
                "Probed {}: resolution={:?}, fps={:?}, duration={:?}, video_br={:?}, audio_br={:?}",
                self.name,
                self.metadata.resolution,
                self.metadata.fps,
                self.metadata.duration,
                self.metadata.source_video_bitrate,
                self.metadata.source_audio_bitrate
            );
        }
    }

    /// コンテンツタイプを設定
    pub fn set_content_type(&mut self, content_type: ContentType) {
        self.metadata.content_type = content_type;
    }

    /// フレームレートを設定
    pub fn set_fps(&mut self, fps: f64) {
        self.metadata.fps = Some(fps);
    }

    /// 解像度を設定
    pub fn set_resolution(&mut self, width: u32, height: u32) {
        self.metadata.resolution = Some((width, height));
    }

    /// 動画の長さを設定
    pub fn set_duration(&mut self, duration: f64) {
        self.metadata.duration = Some(duration);
    }

    /// 予測サイズを計算・更新（高精度版）
    pub fn update_estimated_size(&mut self, settings: &TranscodeSettings) {
        // メタデータが不完全な場合はデフォルト値を使用
        let mut metadata = self.metadata.clone();
        if metadata.resolution.is_none() {
            metadata.resolution = Some((1920, 1080)); // デフォルト1080p
        }
        if metadata.fps.is_none() {
            metadata.fps = Some(30.0); // デフォルト30fps
        }

        // 高精度予測モデルを使用
        let ratio = estimate_compression_ratio_advanced(settings, &metadata);

        // 予測サイズを計算
        self.estimated_size = Some((self.size as f64 * ratio) as u64);
    }

    /// ファイルサイズを人間が読める形式にフォーマット
    pub fn formatted_size(&self) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;

        if self.size >= GB {
            format!("{:.2} GB", self.size as f64 / GB as f64)
        } else if self.size >= MB {
            format!("{:.2} MB", self.size as f64 / MB as f64)
        } else if self.size >= KB {
            format!("{:.2} KB", self.size as f64 / KB as f64)
        } else {
            format!("{} B", self.size)
        }
    }
}

/// ファイル処理状態
#[derive(Clone, Debug, PartialEq)]
pub enum FileStatus {
    /// 待機中
    Pending,
    /// 処理中
    Processing,
    /// 完了
    Completed,
    /// エラー
    Error(String),
    /// キャンセル
    Cancelled,
}

impl FileStatus {
    pub fn label(&self) -> &str {
        match self {
            FileStatus::Pending => "待機中",
            FileStatus::Processing => "処理中",
            FileStatus::Completed => "完了",
            FileStatus::Error(_) => "エラー",
            FileStatus::Cancelled => "キャンセル",
        }
    }
}
