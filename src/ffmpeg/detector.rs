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
                    .map(|p| {
                        p.join("scoop")
                            .join("apps")
                            .join("ffmpeg")
                            .join("current")
                            .join("bin")
                    })
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
        let version_line = output_str.lines().next().context("Empty ffmpeg output")?;

        let version = version_line
            .split_whitespace()
            .find(|s| {
                s.chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
            })
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

            parent.map(|p| p.join(probe_name)).filter(|p| p.exists())
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

/// ffprobeで取得した動画メタデータ
#[derive(Debug, Clone, Default)]
pub struct ProbeResult {
    /// 動画の長さ（秒）
    pub duration: Option<f64>,
    /// 映像ビットレート（bps）
    pub video_bitrate: Option<u64>,
    /// 音声ビットレート（bps）
    pub audio_bitrate: Option<u64>,
    /// 全体ビットレート（bps）
    pub overall_bitrate: Option<u64>,
    /// 解像度（幅, 高さ）
    pub resolution: Option<(u32, u32)>,
    /// フレームレート
    pub fps: Option<f64>,
    /// 映像コーデック
    pub video_codec: Option<String>,
    /// 音声コーデック
    pub audio_codec: Option<String>,
}

impl FfmpegInfo {
    /// ffprobeで動画のメタデータを取得
    pub fn probe_video(&self, path: &std::path::Path) -> Result<ProbeResult> {
        let ffprobe_path = self
            .ffprobe_path
            .as_ref()
            .ok_or_else(|| anyhow!("ffprobe not found"))?;

        // JSON形式で詳細情報を取得
        let output = Command::new(ffprobe_path)
            .args([
                "-v",
                "quiet",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
            ])
            .arg(path)
            .output()
            .context("Failed to execute ffprobe")?;

        if !output.status.success() {
            return Err(anyhow!(
                "ffprobe failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        Self::parse_probe_json(&json_str)
    }

    /// ffprobeのJSON出力をパース
    fn parse_probe_json(json_str: &str) -> Result<ProbeResult> {
        use std::collections::HashMap;

        let mut result = ProbeResult::default();

        // 簡易JSONパース（serde_json依存を避けるため）
        // format セクションから duration と bit_rate を取得
        if let Some(format_start) = json_str.find("\"format\"") {
            let format_section = &json_str[format_start..];

            // duration
            if let Some(duration) = Self::extract_json_number(format_section, "duration") {
                result.duration = Some(duration);
            }

            // bit_rate (全体)
            if let Some(bitrate) = Self::extract_json_string_number(format_section, "bit_rate") {
                result.overall_bitrate = Some(bitrate);
            }
        }

        // streams セクションから映像・音声情報を取得
        let mut in_streams = false;
        let mut brace_count = 0;
        let mut current_stream = String::new();

        for line in json_str.lines() {
            let trimmed = line.trim();

            if trimmed.contains("\"streams\"") {
                in_streams = true;
                continue;
            }

            if in_streams {
                current_stream.push_str(line);
                brace_count += line.chars().filter(|&c| c == '{').count() as i32;
                brace_count -= line.chars().filter(|&c| c == '}').count() as i32;

                // ストリーム1つ分が完了
                if brace_count == 0
                    && !current_stream.is_empty()
                    && current_stream.contains("codec_type")
                {
                    if current_stream.contains("\"codec_type\": \"video\"")
                        || current_stream.contains("\"codec_type\":\"video\"")
                    {
                        // 映像ストリーム
                        if let Some(w) = Self::extract_json_int(&current_stream, "width") {
                            if let Some(h) = Self::extract_json_int(&current_stream, "height") {
                                result.resolution = Some((w as u32, h as u32));
                            }
                        }

                        if let Some(bitrate) =
                            Self::extract_json_string_number(&current_stream, "bit_rate")
                        {
                            result.video_bitrate = Some(bitrate);
                        }

                        // フレームレート (r_frame_rate: "30/1" or "30000/1001")
                        if let Some(fps_str) =
                            Self::extract_json_string(&current_stream, "r_frame_rate")
                        {
                            if let Some(fps) = Self::parse_frame_rate(&fps_str) {
                                result.fps = Some(fps);
                            }
                        }

                        if let Some(codec) =
                            Self::extract_json_string(&current_stream, "codec_name")
                        {
                            result.video_codec = Some(codec);
                        }
                    } else if current_stream.contains("\"codec_type\": \"audio\"")
                        || current_stream.contains("\"codec_type\":\"audio\"")
                    {
                        // 音声ストリーム
                        if result.audio_bitrate.is_none() {
                            if let Some(bitrate) =
                                Self::extract_json_string_number(&current_stream, "bit_rate")
                            {
                                result.audio_bitrate = Some(bitrate);
                            }
                        }

                        if result.audio_codec.is_none() {
                            if let Some(codec) =
                                Self::extract_json_string(&current_stream, "codec_name")
                            {
                                result.audio_codec = Some(codec);
                            }
                        }
                    }

                    current_stream.clear();
                }
            }
        }

        // 映像ビットレートが取れなかった場合、全体から音声を引いて推定
        if result.video_bitrate.is_none() {
            if let (Some(overall), Some(audio)) = (result.overall_bitrate, result.audio_bitrate) {
                if overall > audio {
                    result.video_bitrate = Some(overall - audio);
                }
            } else if let Some(overall) = result.overall_bitrate {
                // 音声ビットレートが不明な場合、全体の90%を映像と仮定
                result.video_bitrate = Some((overall as f64 * 0.90) as u64);
            }
        }

        Ok(result)
    }

    /// JSON文字列から数値を抽出 ("key": 123.45)
    fn extract_json_number(json: &str, key: &str) -> Option<f64> {
        let pattern = format!("\"{}\":", key);
        if let Some(pos) = json.find(&pattern) {
            let start = pos + pattern.len();
            let rest = json[start..].trim_start();
            let end = rest
                .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
                .unwrap_or(rest.len());
            rest[..end].parse().ok()
        } else {
            None
        }
    }

    /// JSON文字列から整数を抽出 ("key": 123)
    fn extract_json_int(json: &str, key: &str) -> Option<i64> {
        let pattern = format!("\"{}\":", key);
        if let Some(pos) = json.find(&pattern) {
            let start = pos + pattern.len();
            let rest = json[start..].trim_start();
            let end = rest
                .find(|c: char| !c.is_ascii_digit() && c != '-')
                .unwrap_or(rest.len());
            rest[..end].parse().ok()
        } else {
            None
        }
    }

    /// JSON文字列から文字列値を抽出 ("key": "value")
    fn extract_json_string(json: &str, key: &str) -> Option<String> {
        let pattern = format!("\"{}\":", key);
        if let Some(pos) = json.find(&pattern) {
            let start = pos + pattern.len();
            let rest = json[start..].trim_start();
            if rest.starts_with('"') {
                let content = &rest[1..];
                if let Some(end) = content.find('"') {
                    return Some(content[..end].to_string());
                }
            }
        }
        None
    }

    /// JSON文字列から数値文字列を抽出して数値に変換 ("key": "12345")
    fn extract_json_string_number(json: &str, key: &str) -> Option<u64> {
        Self::extract_json_string(json, key)?.parse().ok()
    }

    /// フレームレート文字列をパース ("30/1" -> 30.0)
    fn parse_frame_rate(fps_str: &str) -> Option<f64> {
        let parts: Vec<&str> = fps_str.split('/').collect();
        if parts.len() == 2 {
            let num: f64 = parts[0].parse().ok()?;
            let den: f64 = parts[1].parse().ok()?;
            if den > 0.0 {
                return Some(num / den);
            }
        }
        fps_str.parse().ok()
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
