//! アプリケーション設定（JSON保存）

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// アプリケーション設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    /// FFmpegのカスタムパス
    pub ffmpeg_custom_path: Option<PathBuf>,
    /// 最後に使用した出力ディレクトリ
    pub last_output_dir: Option<PathBuf>,
    /// 最後に使用した入力ディレクトリ
    pub last_input_dir: Option<PathBuf>,
    /// ウィンドウ位置X
    pub window_x: Option<i32>,
    /// ウィンドウ位置Y
    pub window_y: Option<i32>,
    /// ウィンドウ幅
    pub window_width: Option<u32>,
    /// ウィンドウ高さ
    pub window_height: Option<u32>,
    /// ダークモード
    pub dark_mode: bool,
    /// 処理完了時に通知
    pub notify_on_complete: bool,
    /// 処理完了後にシャットダウン
    pub shutdown_on_complete: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ffmpeg_custom_path: None,
            last_output_dir: None,
            last_input_dir: None,
            window_x: None,
            window_y: None,
            window_width: Some(1200),
            window_height: Some(800),
            dark_mode: true,
            notify_on_complete: true,
            shutdown_on_complete: false,
        }
    }
}

impl Settings {
    /// 設定ファイルのパスを取得
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("kamaitachi");

        // ディレクトリが存在しない場合は作成
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)?;
        }

        Ok(config_dir.join("settings.json"))
    }

    /// アプリケーションデータディレクトリを取得
    pub fn app_data_dir() -> Result<PathBuf> {
        let data_dir = dirs::data_local_dir()
            .context("Failed to get data directory")?
            .join("kamaitachi");

        // ディレクトリが存在しない場合は作成
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)?;
        }

        Ok(data_dir)
    }

    /// FFmpegのダウンロード先ディレクトリを取得
    pub fn ffmpeg_dir() -> Result<PathBuf> {
        let ffmpeg_dir = Self::app_data_dir()?.join("ffmpeg");

        // ディレクトリが存在しない場合は作成
        if !ffmpeg_dir.exists() {
            std::fs::create_dir_all(&ffmpeg_dir)?;
        }

        Ok(ffmpeg_dir)
    }

    /// 設定をファイルからロード
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let settings: Settings = serde_json::from_str(&content)?;
            Ok(settings)
        } else {
            Ok(Self::default())
        }
    }

    /// 設定をファイルに保存
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}
