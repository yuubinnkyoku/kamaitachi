//! FFmpeg自動ダウンロード

use anyhow::{anyhow, Context, Result};
use log::{debug, info, warn};
use std::fs::{self, File};
use std::io::{self, BufReader, Read, Write};
use std::path::PathBuf;

use crate::config::Settings;

/// FFmpegダウンローダー
pub struct FfmpegDownloader;

/// ダウンロード進捗コールバック
pub type ProgressCallback = Box<dyn Fn(DownloadProgress) + Send + Sync>;

/// ダウンロード進捗
#[derive(Clone, Debug)]
pub struct DownloadProgress {
    /// ダウンロード済みバイト数
    pub downloaded: u64,
    /// 総バイト数（不明な場合はNone）
    pub total: Option<u64>,
    /// 進捗率（0.0 - 1.0）
    pub progress: f32,
    /// 現在のステータス
    pub status: DownloadStatus,
}

/// ダウンロードステータス
#[derive(Clone, Debug, PartialEq)]
pub enum DownloadStatus {
    /// 準備中
    Preparing,
    /// ダウンロード中
    Downloading,
    /// 展開中
    Extracting,
    /// 完了
    Completed,
    /// エラー
    Error(String),
}

impl FfmpegDownloader {
    /// FFmpegダウンロードURL (gyan.dev GPL build)
    #[cfg(target_os = "windows")]
    const DOWNLOAD_URL: &'static str =
        "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-full.zip";

    #[cfg(target_os = "linux")]
    const DOWNLOAD_URL: &'static str =
        "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz";

    #[cfg(target_os = "macos")]
    const DOWNLOAD_URL: &'static str =
        "https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip";

    /// FFmpegをダウンロードして展開
    pub fn download(progress_callback: Option<ProgressCallback>) -> Result<PathBuf> {
        let ffmpeg_dir = Settings::ffmpeg_dir()?;
        let archive_path = ffmpeg_dir.join("ffmpeg-download.zip");

        // 進捗通知: 準備中
        if let Some(ref cb) = progress_callback {
            cb(DownloadProgress {
                downloaded: 0,
                total: None,
                progress: 0.0,
                status: DownloadStatus::Preparing,
            });
        }

        info!("Downloading FFmpeg from {}", Self::DOWNLOAD_URL);

        // ダウンロード
        let response = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(600))
            .build()?
            .get(Self::DOWNLOAD_URL)
            .send()
            .context("Failed to start download")?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Download failed with status: {}",
                response.status()
            ));
        }

        let total_size = response.content_length();
        let mut downloaded: u64 = 0;

        // ファイルに保存
        let mut file = File::create(&archive_path)?;
        let mut reader = BufReader::new(response);
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }

            file.write_all(&buffer[..bytes_read])?;
            downloaded += bytes_read as u64;

            // 進捗通知
            if let Some(ref cb) = progress_callback {
                let progress = total_size
                    .map(|t| downloaded as f32 / t as f32)
                    .unwrap_or(0.0);
                cb(DownloadProgress {
                    downloaded,
                    total: total_size,
                    progress,
                    status: DownloadStatus::Downloading,
                });
            }
        }

        info!("Download complete: {} bytes", downloaded);

        // 進捗通知: 展開中
        if let Some(ref cb) = progress_callback {
            cb(DownloadProgress {
                downloaded,
                total: total_size,
                progress: 0.5,
                status: DownloadStatus::Extracting,
            });
        }

        // アーカイブを展開
        let ffmpeg_bin_path = Self::extract_archive(&archive_path, &ffmpeg_dir)?;

        // アーカイブを削除
        let _ = fs::remove_file(&archive_path);

        // 進捗通知: 完了
        if let Some(ref cb) = progress_callback {
            cb(DownloadProgress {
                downloaded,
                total: total_size,
                progress: 1.0,
                status: DownloadStatus::Completed,
            });
        }

        info!("FFmpeg extracted to: {:?}", ffmpeg_bin_path);

        Ok(ffmpeg_bin_path)
    }

    /// アーカイブを展開
    #[cfg(target_os = "windows")]
    fn extract_archive(archive_path: &PathBuf, dest_dir: &PathBuf) -> Result<PathBuf> {
        let file = File::open(archive_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        // 最初のディレクトリ名を取得（通常は ffmpeg-x.x-full_build）
        let mut root_dir_name = String::new();
        for i in 0..archive.len() {
            let file = archive.by_index(i)?;
            let name = file.name();
            if let Some(first_component) = name.split('/').next() {
                if !first_component.is_empty() {
                    root_dir_name = first_component.to_string();
                    break;
                }
            }
        }

        info!("Extracting {} files...", archive.len());

        // 展開
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = dest_dir.join(file.name());

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)?;
                    }
                }
                let mut outfile = File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;
            }
        }

        // binディレクトリのパスを返す
        let bin_dir = dest_dir.join(&root_dir_name).join("bin");
        if bin_dir.exists() {
            Ok(bin_dir)
        } else {
            // フラットな構造の場合
            Ok(dest_dir.join(&root_dir_name))
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn extract_archive(archive_path: &PathBuf, dest_dir: &PathBuf) -> Result<PathBuf> {
        use std::process::Command;

        // tar.xz の展開 (Linux/macOS)
        let output = Command::new("tar")
            .arg("-xf")
            .arg(archive_path)
            .arg("-C")
            .arg(dest_dir)
            .output()
            .context("Failed to extract archive")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Extraction failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // 展開されたディレクトリを探す
        for entry in fs::read_dir(dest_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && path.file_name().map(|n| n.to_string_lossy().contains("ffmpeg")).unwrap_or(false) {
                return Ok(path);
            }
        }

        Err(anyhow!("Could not find extracted ffmpeg directory"))
    }

    /// ダウンロード済みのFFmpegがあるかチェック
    pub fn is_downloaded() -> Result<Option<PathBuf>> {
        let ffmpeg_dir = Settings::ffmpeg_dir()?;

        #[cfg(target_os = "windows")]
        let ffmpeg_name = "ffmpeg.exe";
        #[cfg(not(target_os = "windows"))]
        let ffmpeg_name = "ffmpeg";

        // ディレクトリ内を再帰的に検索
        fn find_ffmpeg(dir: &PathBuf, name: &str) -> Option<PathBuf> {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Some(found) = find_ffmpeg(&path, name) {
                            return Some(found);
                        }
                    } else if path.file_name().map(|n| n == name).unwrap_or(false) {
                        return Some(path);
                    }
                }
            }
            None
        }

        Ok(find_ffmpeg(&ffmpeg_dir, ffmpeg_name))
    }
}
