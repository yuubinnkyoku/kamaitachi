//! HWアクセラレーション検出

use anyhow::Result;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::process::Command;
use std::sync::OnceLock;

use super::VideoCodec;

/// 利用可能なエンコーダーのキャッシュ
static AVAILABLE_ENCODERS: OnceLock<HashSet<String>> = OnceLock::new();

/// HWアクセラレーションタイプ
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HwAccelType {
    /// 自動検出
    Auto,
    /// NVIDIA NVENC
    Nvenc,
    /// Intel Quick Sync Video
    Qsv,
    /// AMD AMF
    Amf,
    /// ソフトウェアエンコード
    Software,
}

impl HwAccelType {
    /// 表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            HwAccelType::Auto => "自動検出",
            HwAccelType::Nvenc => "NVIDIA NVENC",
            HwAccelType::Qsv => "Intel QSV",
            HwAccelType::Amf => "AMD AMF",
            HwAccelType::Software => "ソフトウェア",
        }
    }

    /// すべてのバリアントを取得
    pub fn all() -> &'static [HwAccelType] {
        &[
            HwAccelType::Auto,
            HwAccelType::Nvenc,
            HwAccelType::Qsv,
            HwAccelType::Amf,
            HwAccelType::Software,
        ]
    }
}

impl Default for HwAccelType {
    fn default() -> Self {
        HwAccelType::Auto
    }
}

/// HWアクセラレーション検出器
pub struct HwAccelDetector;

/// 検出されたHWアクセラレーション情報
#[derive(Debug, Clone)]
pub struct HwAccelInfo {
    /// 利用可能なHWアクセラレーションタイプ
    pub available: Vec<HwAccelType>,
    /// 推奨されるHWアクセラレーション
    pub recommended: HwAccelType,
}

impl HwAccelDetector {
    /// 利用可能なHWアクセラレーションを検出
    pub fn detect(ffmpeg_path: Option<&std::path::PathBuf>) -> Result<HwAccelInfo> {
        let mut available = Vec::new();

        // NVIDIA NVENCを検出
        if Self::detect_nvenc(ffmpeg_path) {
            info!("NVIDIA NVENC detected");
            available.push(HwAccelType::Nvenc);
        }

        // Intel QSVを検出
        if Self::detect_qsv(ffmpeg_path) {
            info!("Intel QSV detected");
            available.push(HwAccelType::Qsv);
        }

        // AMD AMFを検出
        if Self::detect_amf(ffmpeg_path) {
            info!("AMD AMF detected");
            available.push(HwAccelType::Amf);
        }

        // ソフトウェアは常に利用可能
        available.push(HwAccelType::Software);

        // 推奨を決定（優先順位: NVENC > QSV > AMF > Software）
        let recommended = if available.contains(&HwAccelType::Nvenc) {
            HwAccelType::Nvenc
        } else if available.contains(&HwAccelType::Qsv) {
            HwAccelType::Qsv
        } else if available.contains(&HwAccelType::Amf) {
            HwAccelType::Amf
        } else {
            HwAccelType::Software
        };

        Ok(HwAccelInfo {
            available,
            recommended,
        })
    }

    /// NVIDIA NVENCを検出
    fn detect_nvenc(ffmpeg_path: Option<&std::path::PathBuf>) -> bool {
        // nvidia-smiで確認
        #[cfg(target_os = "windows")]
        {
            let nvidia_smi = Command::new("nvidia-smi").output();
            if let Ok(output) = nvidia_smi {
                if output.status.success() {
                    debug!("nvidia-smi succeeded");
                    return Self::check_encoder_available("h264_nvenc", ffmpeg_path);
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Linux/macOS: /proc/driver/nvidia/version をチェック
            if std::path::Path::new("/proc/driver/nvidia/version").exists() {
                return Self::check_encoder_available("h264_nvenc", ffmpeg_path);
            }
        }

        false
    }

    /// Intel QSVを検出
    fn detect_qsv(ffmpeg_path: Option<&std::path::PathBuf>) -> bool {
        Self::check_encoder_available("h264_qsv", ffmpeg_path)
    }

    /// AMD AMFを検出
    fn detect_amf(ffmpeg_path: Option<&std::path::PathBuf>) -> bool {
        Self::check_encoder_available("h264_amf", ffmpeg_path)
    }

    /// FFmpegでエンコーダーが利用可能かチェック
    fn check_encoder_available(encoder: &str, ffmpeg_path: Option<&std::path::PathBuf>) -> bool {
        let ffmpeg = ffmpeg_path
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "ffmpeg".to_string());

        let output = Command::new(&ffmpeg).args(["-encoders"]).output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.contains(encoder);
        }

        false
    }

    /// 自動選択されたHWアクセラレーションを解決
    pub fn resolve_auto(
        hwaccel: HwAccelType,
        ffmpeg_path: Option<&std::path::PathBuf>,
    ) -> HwAccelType {
        if hwaccel != HwAccelType::Auto {
            return hwaccel;
        }

        match Self::detect(ffmpeg_path) {
            Ok(info) => info.recommended,
            Err(_) => HwAccelType::Software,
        }
    }

    /// 利用可能なすべてのエンコーダーをキャッシュから取得
    /// 初回呼び出し時にFFmpegから取得してキャッシュ
    pub fn get_available_encoders(
        ffmpeg_path: Option<&std::path::PathBuf>,
    ) -> &'static HashSet<String> {
        AVAILABLE_ENCODERS.get_or_init(|| Self::fetch_available_encoders(ffmpeg_path))
    }

    /// FFmpegから利用可能なエンコーダー一覧を取得
    fn fetch_available_encoders(ffmpeg_path: Option<&std::path::PathBuf>) -> HashSet<String> {
        let ffmpeg = ffmpeg_path
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "ffmpeg".to_string());

        let output = Command::new(&ffmpeg).args(["-encoders"]).output();

        let mut encoders = HashSet::new();
        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                // エンコーダー行のパース: " V..... h264_nvenc           NVIDIA NVENC H.264 encoder"
                let trimmed = line.trim();
                if trimmed.len() > 7 && (trimmed.starts_with("V") || trimmed.starts_with(" V")) {
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() >= 2 {
                        encoders.insert(parts[1].to_string());
                    }
                }
            }
        }

        info!("Available encoders detected: {:?}", encoders);
        encoders
    }

    /// 特定のエンコーダーが実際に動作するかをテスト
    /// (エンコーダーがリストにあってもGPUがサポートしていない場合がある)
    pub fn test_encoder_availability(
        encoder: &str,
        ffmpeg_path: Option<&std::path::PathBuf>,
    ) -> bool {
        // まずエンコーダーがFFmpegに含まれているかチェック
        let available_encoders = Self::get_available_encoders(ffmpeg_path);
        if !available_encoders.contains(encoder) {
            debug!("Encoder {} not found in FFmpeg encoder list", encoder);
            return false;
        }

        // 既知のソフトウェアエンコーダーはFFmpegに含まれていれば動作する
        let known_software_encoders = [
            "libx264",
            "libx265",
            "libvpx-vp9",
            "libsvtav1",
            "libaom-av1",
        ];
        if known_software_encoders.contains(&encoder) {
            return true;
        }

        // HWエンコーダーは実際にテストが必要
        let ffmpeg = ffmpeg_path
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "ffmpeg".to_string());

        // ダミーの入力でエンコーダーの初期化をテスト
        let result = Command::new(&ffmpeg)
            .args([
                "-f",
                "lavfi",
                "-i",
                "nullsrc=s=256x256:d=0.1",
                "-c:v",
                encoder,
                "-frames:v",
                "1",
                "-f",
                "null",
                "-",
            ])
            .output();

        match result {
            Ok(output) => {
                let success = output.status.success();
                if !success {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    // GPU非対応エラーを検出
                    if stderr.contains("not supported")
                        || stderr.contains("doesn't support")
                        || stderr.contains("Codec not supported")
                        || stderr.contains("Error while opening encoder")
                    {
                        warn!(
                            "Encoder {} is not supported by this GPU: {}",
                            encoder,
                            stderr
                                .lines()
                                .find(|l| l.contains("not supported") || l.contains("Error"))
                                .unwrap_or("")
                        );
                        return false;
                    }
                }
                success
            }
            Err(e) => {
                warn!("Failed to test encoder {}: {}", encoder, e);
                false
            }
        }
    }

    /// ビデオコーデックに対するフォールバックエンコーダー候補を取得
    /// 優先度順にリストを返す
    fn get_fallback_encoders(video_codec: &VideoCodec) -> Vec<&'static str> {
        match video_codec {
            VideoCodec::H264 => vec!["libx264"],
            VideoCodec::H265 => vec!["libx265"],
            VideoCodec::Vp9 => vec!["libvpx-vp9"],
            // AV1は複数のソフトウェアエンコーダーがある
            // libsvtav1: 高速、品質良好（推奨）
            // libaom-av1: 遅いが高品質、互換性高い
            VideoCodec::Av1 => vec!["libsvtav1", "libaom-av1"],
        }
    }

    /// 指定されたビデオコーデックとHWアクセラレーションの組み合わせが利用可能かチェック
    /// 利用不可の場合は代替エンコーダーを返す
    pub fn get_available_encoder(
        video_codec: &VideoCodec,
        hwaccel: &HwAccelType,
        ffmpeg_path: Option<&std::path::PathBuf>,
    ) -> (String, HwAccelType) {
        let preferred_encoder = video_codec.encoder_name(hwaccel);

        // 優先エンコーダーが利用可能かテスト
        if Self::test_encoder_availability(preferred_encoder, ffmpeg_path) {
            info!("Using preferred encoder: {}", preferred_encoder);
            return (preferred_encoder.to_string(), *hwaccel);
        }

        // フォールバック: ソフトウェアエンコーダーを順番に試す
        let fallback_encoders = Self::get_fallback_encoders(video_codec);

        for fallback in &fallback_encoders {
            if Self::test_encoder_availability(fallback, ffmpeg_path) {
                warn!(
                    "Encoder {} not available, falling back to {}",
                    preferred_encoder, fallback
                );
                return (fallback.to_string(), HwAccelType::Software);
            }
        }

        // すべてのフォールバックが失敗した場合、最初のフォールバックを返す（エラーになる可能性あり）
        let last_resort = fallback_encoders.first().unwrap_or(&"libx264");
        warn!(
            "No available encoder found for {:?}, using {} (may fail)",
            video_codec, last_resort
        );
        (last_resort.to_string(), HwAccelType::Software)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_hwaccel() {
        if let Ok(info) = HwAccelDetector::detect(None) {
            println!("Available HW acceleration: {:?}", info.available);
            println!("Recommended: {:?}", info.recommended);
            assert!(!info.available.is_empty());
        }
    }
}
