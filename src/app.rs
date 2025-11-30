//! アプリケーション状態管理

use crate::config::Settings;
use crate::ffmpeg::{FfmpegDetector, FfmpegInfo, ProbeResult};
use crate::transcoder::{
    estimate_compression_ratio_advanced, ContentType, TranscodeJob, TranscodeSettings,
    VideoMetadata,
};
use gpui::*;
use std::path::PathBuf;

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
        }
    }

    /// ファイルをキューに追加
    pub fn add_files(&self, paths: Vec<PathBuf>, cx: &mut App) {
        let settings = self.transcode_settings.read(cx).clone();
        let ffmpeg_info = self.ffmpeg_info.read(cx).clone();
        self.files.update(cx, |files, _| {
            for path in paths {
                if Self::is_supported_format(&path) {
                    let mut entry = FileEntry::new(path.clone());
                    // ffprobeでメタデータを取得
                    if let Some(ref info) = ffmpeg_info {
                        entry.probe_metadata(info);
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
