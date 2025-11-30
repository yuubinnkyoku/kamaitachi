//! アプリケーション状態管理

use crate::config::Settings;
use crate::transcoder::{HwAccelType, TranscodeJob, TranscodeSettings};
use gpui::*;
use std::path::PathBuf;
use std::sync::Arc;

/// アプリケーションのグローバル状態
#[derive(Clone)]
pub struct AppState {
    /// ファイルキュー
    pub files: Model<Vec<FileEntry>>,
    /// トランスコード設定
    pub transcode_settings: Model<TranscodeSettings>,
    /// 現在のジョブ
    pub current_job: Model<Option<TranscodeJob>>,
    /// アプリケーション設定
    pub settings: Model<Settings>,
    /// FFmpegパス
    pub ffmpeg_path: Model<Option<PathBuf>>,
}

impl AppState {
    pub fn new(cx: &mut AppContext) -> Self {
        // 設定をロード
        let settings = Settings::load().unwrap_or_default();

        Self {
            files: cx.new_model(|_| Vec::new()),
            transcode_settings: cx.new_model(|_| TranscodeSettings::default()),
            current_job: cx.new_model(|_| None),
            settings: cx.new_model(|_| settings),
            ffmpeg_path: cx.new_model(|_| None),
        }
    }

    /// ファイルをキューに追加
    pub fn add_files(&self, paths: Vec<PathBuf>, cx: &mut AppContext) {
        self.files.update(cx, |files, _| {
            for path in paths {
                if Self::is_supported_format(&path) {
                    let entry = FileEntry::new(path);
                    files.push(entry);
                }
            }
        });
    }

    /// ファイルをキューから削除
    pub fn remove_file(&self, index: usize, cx: &mut AppContext) {
        self.files.update(cx, |files, _| {
            if index < files.len() {
                files.remove(index);
            }
        });
    }

    /// キューをクリア
    pub fn clear_files(&self, cx: &mut AppContext) {
        self.files.update(cx, |files, _| {
            files.clear();
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
        }
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
