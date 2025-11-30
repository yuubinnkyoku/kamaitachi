//! アプリケーション状態管理

use crate::config::Settings;
use crate::transcoder::{TranscodeJob, TranscodeSettings, estimate_compression_ratio};
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
}

impl AppState {
    pub fn new(cx: &mut App) -> Self {
        // 設定をロード
        let settings = Settings::load().unwrap_or_default();

        Self {
            files: cx.new(|_| Vec::new()),
            transcode_settings: cx.new(|_| TranscodeSettings::default()),
            current_job: cx.new(|_| None),
            settings: cx.new(|_| settings),
            ffmpeg_path: cx.new(|_| None),
        }
    }

    /// ファイルをキューに追加
    pub fn add_files(&self, paths: Vec<PathBuf>, cx: &mut App) {
        let settings = self.transcode_settings.read(cx).clone();
        self.files.update(cx, |files, _| {
            for path in paths {
                if Self::is_supported_format(&path) {
                    let mut entry = FileEntry::new(path);
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
    /// 動画の解像度（幅, 高さ）
    pub resolution: Option<(u32, u32)>,
    /// 動画の長さ（秒）
    pub duration: Option<f64>,
}

impl FileEntry {
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let size = std::fs::metadata(&path)
            .map(|m| m.len())
            .unwrap_or(0);

        Self {
            path,
            name,
            size,
            status: FileStatus::Pending,
            progress: 0.0,
            estimated_size: None,
            resolution: None,
            duration: None,
        }
    }

    /// 予測サイズを計算・更新
    pub fn update_estimated_size(&mut self, settings: &TranscodeSettings) {
        // 解像度が不明な場合は予測不可
        let source_res = self.resolution.unwrap_or((1920, 1080)); // デフォルト1080p
        
        let target_res = match settings.resolution {
            crate::transcoder::VideoResolution::Original => source_res,
            res => res.dimensions(),
        };
        
        // 圧縮率を計算
        let ratio = estimate_compression_ratio(settings.crf, source_res, target_res);
        
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
