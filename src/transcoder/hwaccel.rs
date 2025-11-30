//! HWアクセラレーション検出

use anyhow::Result;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::process::Command;

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

        let output = Command::new(&ffmpeg)
            .args(["-encoders"])
            .output();

        if let Ok(output) = output {
            let stdout = String::from_utf8_lossy(&output.stdout);
            return stdout.contains(encoder);
        }

        false
    }

    /// 自動選択されたHWアクセラレーションを解決
    pub fn resolve_auto(hwaccel: HwAccelType, ffmpeg_path: Option<&std::path::PathBuf>) -> HwAccelType {
        if hwaccel != HwAccelType::Auto {
            return hwaccel;
        }

        match Self::detect(ffmpeg_path) {
            Ok(info) => info.recommended,
            Err(_) => HwAccelType::Software,
        }
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
