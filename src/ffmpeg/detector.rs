//! 既存FFmpeg検出

use anyhow::{anyhow, Context, Result};
use log::{debug, info, warn};
use std::path::PathBuf;
use std::process::Command;

/// FFmpeg検出器
pub struct FfmpegDetector;

/// FFmpegの検出結果
#[derive(Debug, Clone)]
pub struct FfmpegInfo {
    /// FFmpegの実行ファイルパス
    pub ffmpeg_path: PathBuf,
    /// FFprobeの実行ファイルパス
    pub ffprobe_path: Option<PathBuf>,
    /// バージョン文字列
    pub version: String,
    /// メジャーバージョン
    pub major_version: u32,
    /// マイナーバージョン
    pub minor_version: u32,
    /// GPLビルドかどうか
    pub is_gpl: bool,
}

impl FfmpegDetector {
    /// システム上のFFmpegを検出
    pub fn detect() -> Result<FfmpegInfo> {
        // 1. 環境変数 FFMPEG_DIR をチェック
        if let Ok(ffmpeg_dir) = std::env::var("FFMPEG_DIR") {
            debug!("Checking FFMPEG_DIR: {}", ffmpeg_dir);
            if let Ok(info) = Self::check_ffmpeg_in_dir(&PathBuf::from(ffmpeg_dir)) {
                info!("Found FFmpeg via FFMPEG_DIR: {:?}", info.ffmpeg_path);
                return Ok(info);
            }
        }

        // 2. ビルド時に設定されたパスをチェック
        if let Some(bin_path) = option_env!("FFMPEG_BIN_PATH") {
            debug!("Checking FFMPEG_BIN_PATH: {}", bin_path);
            if let Ok(info) = Self::check_ffmpeg_in_dir(&PathBuf::from(bin_path)) {
                info!("Found FFmpeg via FFMPEG_BIN_PATH: {:?}", info.ffmpeg_path);
                return Ok(info);
            }
        }

        // 3. PATH上のFFmpegをチェック
        if let Ok(info) = Self::check_ffmpeg_in_path() {
            info!("Found FFmpeg in PATH: {:?}", info.ffmpeg_path);
            return Ok(info);
        }

        // 4. よくあるインストール場所をチェック (Windows)
        #[cfg(target_os = "windows")]
        {
            let common_paths = vec![
                PathBuf::from(r"C:\Program Files\FFmpeg\bin"),
                PathBuf::from(r"C:\FFmpeg\bin"),
                dirs::data_local_dir()
                    .map(|p| p.join("Programs").join("ffmpeg").join("bin"))
                    .unwrap_or_default(),
                // Chocolatey
                dirs::data_local_dir()
                    .map(|p| p.join("UniGetUI").join("Chocolatey").join("bin"))
                    .unwrap_or_default(),
                // Scoop
                dirs::home_dir()
                    .map(|p| p.join("scoop").join("apps").join("ffmpeg").join("current").join("bin"))
                    .unwrap_or_default(),
            ];

            for path in common_paths {
                if path.exists() {
                    debug!("Checking common path: {:?}", path);
                    if let Ok(info) = Self::check_ffmpeg_in_dir(&path) {
                        info!("Found FFmpeg at common location: {:?}", info.ffmpeg_path);
                        return Ok(info);
                    }
                }
            }
        }

        Err(anyhow!("FFmpeg not found on this system"))
    }

    /// 指定したパスのFFmpegをチェック
    pub fn check_ffmpeg_at_path(path: &PathBuf) -> Result<FfmpegInfo> {
        Self::get_ffmpeg_info(path)
    }

    /// 指定したディレクトリ内のFFmpegをチェック
    fn check_ffmpeg_in_dir(dir: &PathBuf) -> Result<FfmpegInfo> {
        #[cfg(target_os = "windows")]
        let ffmpeg_name = "ffmpeg.exe";
        #[cfg(not(target_os = "windows"))]
        let ffmpeg_name = "ffmpeg";

        let ffmpeg_path = dir.join(ffmpeg_name);

        if ffmpeg_path.exists() {
            Self::get_ffmpeg_info(&ffmpeg_path)
        } else {
            // binサブディレクトリをチェック
            let bin_path = dir.join("bin").join(ffmpeg_name);
            if bin_path.exists() {
                Self::get_ffmpeg_info(&bin_path)
            } else {
                Err(anyhow!("FFmpeg not found in {:?}", dir))
            }
        }
    }

    /// PATH上のFFmpegをチェック
    fn check_ffmpeg_in_path() -> Result<FfmpegInfo> {
        #[cfg(target_os = "windows")]
        let ffmpeg_name = "ffmpeg";
        #[cfg(not(target_os = "windows"))]
        let ffmpeg_name = "ffmpeg";

        let output = Command::new(ffmpeg_name)
            .arg("-version")
            .output()
            .context("Failed to execute ffmpeg")?;

        if output.status.success() {
            // whereコマンドでパスを取得
            #[cfg(target_os = "windows")]
            let path_output = Command::new("where").arg("ffmpeg").output()?;
            #[cfg(not(target_os = "windows"))]
            let path_output = Command::new("which").arg("ffmpeg").output()?;

            let path_str = String::from_utf8_lossy(&path_output.stdout);
            let ffmpeg_path = PathBuf::from(path_str.lines().next().unwrap_or("ffmpeg").trim());

            Self::parse_ffmpeg_output(&output.stdout, ffmpeg_path)
        } else {
            Err(anyhow!("FFmpeg not found in PATH"))
        }
    }

    /// FFmpegの情報を取得
    fn get_ffmpeg_info(ffmpeg_path: &PathBuf) -> Result<FfmpegInfo> {
        let output = Command::new(ffmpeg_path)
            .arg("-version")
            .output()
            .context(format!("Failed to execute {:?}", ffmpeg_path))?;

        if output.status.success() {
            Self::parse_ffmpeg_output(&output.stdout, ffmpeg_path.clone())
        } else {
            Err(anyhow!("FFmpeg execution failed: {:?}", ffmpeg_path))
        }
    }

    /// FFmpegの出力をパース
    fn parse_ffmpeg_output(output: &[u8], ffmpeg_path: PathBuf) -> Result<FfmpegInfo> {
        let output_str = String::from_utf8_lossy(output);

        // バージョン抽出 (例: "ffmpeg version 7.0.1 Copyright ...")
        let version_line = output_str
            .lines()
            .next()
            .context("Empty ffmpeg output")?;

        let version = version_line
            .split_whitespace()
            .find(|s| s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false))
            .unwrap_or("unknown")
            .to_string();

        // メジャー・マイナーバージョン抽出
        let mut version_parts = version.split('.');
        let major_version = version_parts
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let minor_version = version_parts
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        // GPLビルドかどうか
        let is_gpl = output_str.contains("--enable-gpl");

        // FFprobeパスを推測
        let ffprobe_path = {
            let parent = ffmpeg_path.parent();
            #[cfg(target_os = "windows")]
            let probe_name = "ffprobe.exe";
            #[cfg(not(target_os = "windows"))]
            let probe_name = "ffprobe";

            parent
                .map(|p| p.join(probe_name))
                .filter(|p| p.exists())
        };

        Ok(FfmpegInfo {
            ffmpeg_path,
            ffprobe_path,
            version,
            major_version,
            minor_version,
            is_gpl,
        })
    }

    /// FFmpegのバージョンが要件を満たしているかチェック
    pub fn check_version_requirement(info: &FfmpegInfo, min_major: u32) -> bool {
        info.major_version >= min_major
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_ffmpeg() {
        // このテストは実際のFFmpegがインストールされている環境でのみ成功する
        if let Ok(info) = FfmpegDetector::detect() {
            println!("FFmpeg found: {:?}", info);
            assert!(!info.version.is_empty());
        }
    }
}
