//! プリセット定義

use serde::{Deserialize, Serialize};

use super::HwAccelType;

/// トランスコード設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TranscodeSettings {
    /// 出力コンテナ形式
    pub container: ContainerFormat,
    /// ビデオコーデック
    pub video_codec: VideoCodec,
    /// 解像度
    pub resolution: VideoResolution,
    /// CRF値（品質）
    pub crf: u8,
    /// エンコードプリセット
    pub preset: VideoPreset,
    /// HWアクセラレーション
    pub hwaccel: HwAccelType,
    /// オーディオコーデック
    pub audio_codec: AudioCodec,
    /// オーディオビットレート (kbps)
    pub audio_bitrate: u32,
    /// 出力ディレクトリ
    pub output_dir: Option<std::path::PathBuf>,
    /// 出力ファイル名サフィックス
    pub output_suffix: String,
}

impl Default for TranscodeSettings {
    fn default() -> Self {
        Self {
            container: ContainerFormat::Mp4,
            video_codec: VideoCodec::H264,
            resolution: VideoResolution::Original,
            crf: 23,
            preset: VideoPreset::Medium,
            hwaccel: HwAccelType::Auto,
            audio_codec: AudioCodec::Aac,
            audio_bitrate: 192,
            output_dir: None,
            output_suffix: "_transcoded".to_string(),
        }
    }
}

/// コンテナ形式
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerFormat {
    Mp4,
    Mkv,
}

impl ContainerFormat {
    /// 拡張子を取得
    pub fn extension(&self) -> &'static str {
        match self {
            ContainerFormat::Mp4 => "mp4",
            ContainerFormat::Mkv => "mkv",
        }
    }

    /// 表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            ContainerFormat::Mp4 => "MP4",
            ContainerFormat::Mkv => "MKV",
        }
    }

    /// すべてのバリアントを取得
    pub fn all() -> &'static [ContainerFormat] {
        &[ContainerFormat::Mp4, ContainerFormat::Mkv]
    }
}

/// ビデオコーデック
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoCodec {
    H264,
    H265,
    Vp9,
    Av1,
}

impl VideoCodec {
    /// 表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            VideoCodec::H264 => "H.264 (AVC)",
            VideoCodec::H265 => "H.265 (HEVC)",
            VideoCodec::Vp9 => "VP9",
            VideoCodec::Av1 => "AV1",
        }
    }

    /// FFmpegエンコーダー名を取得
    pub fn encoder_name(&self, hwaccel: &HwAccelType) -> &'static str {
        match (self, hwaccel) {
            // NVIDIA NVENC
            (VideoCodec::H264, HwAccelType::Nvenc) => "h264_nvenc",
            (VideoCodec::H265, HwAccelType::Nvenc) => "hevc_nvenc",
            (VideoCodec::Av1, HwAccelType::Nvenc) => "av1_nvenc",
            (VideoCodec::Vp9, HwAccelType::Nvenc) => "libvpx-vp9", // NVENCはVP9非対応

            // Intel QSV
            (VideoCodec::H264, HwAccelType::Qsv) => "h264_qsv",
            (VideoCodec::H265, HwAccelType::Qsv) => "hevc_qsv",
            (VideoCodec::Av1, HwAccelType::Qsv) => "av1_qsv",
            (VideoCodec::Vp9, HwAccelType::Qsv) => "vp9_qsv",

            // AMD AMF
            (VideoCodec::H264, HwAccelType::Amf) => "h264_amf",
            (VideoCodec::H265, HwAccelType::Amf) => "hevc_amf",
            (VideoCodec::Av1, HwAccelType::Amf) => "av1_amf",
            (VideoCodec::Vp9, HwAccelType::Amf) => "libvpx-vp9", // AMFはVP9非対応

            // ソフトウェア / 自動
            (VideoCodec::H264, _) => "libx264",
            (VideoCodec::H265, _) => "libx265",
            (VideoCodec::Vp9, _) => "libvpx-vp9",
            (VideoCodec::Av1, _) => "libsvtav1",
        }
    }

    /// すべてのバリアントを取得
    pub fn all() -> &'static [VideoCodec] {
        &[VideoCodec::H264, VideoCodec::H265, VideoCodec::Vp9, VideoCodec::Av1]
    }
}

/// 解像度
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoResolution {
    /// 元の解像度を維持
    Original,
    /// 4K (3840x2160)
    Uhd4K,
    /// 1080p (1920x1080)
    Fhd1080,
    /// 720p (1280x720)
    Hd720,
    /// 480p (854x480)
    Sd480,
    /// カスタム解像度
    Custom(u32, u32),
}

impl VideoResolution {
    /// 表示名を取得
    pub fn display_name(&self) -> String {
        match self {
            VideoResolution::Original => "元の解像度".to_string(),
            VideoResolution::Uhd4K => "4K (2160p)".to_string(),
            VideoResolution::Fhd1080 => "1080p".to_string(),
            VideoResolution::Hd720 => "720p".to_string(),
            VideoResolution::Sd480 => "480p".to_string(),
            VideoResolution::Custom(w, h) => format!("{}x{}", w, h),
        }
    }

    /// 解像度（幅, 高さ）を取得
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            VideoResolution::Original => (0, 0),
            VideoResolution::Uhd4K => (3840, 2160),
            VideoResolution::Fhd1080 => (1920, 1080),
            VideoResolution::Hd720 => (1280, 720),
            VideoResolution::Sd480 => (854, 480),
            VideoResolution::Custom(w, h) => (*w, *h),
        }
    }

    /// すべてのバリアントを取得（カスタムを除く）
    pub fn all() -> &'static [VideoResolution] {
        &[
            VideoResolution::Original,
            VideoResolution::Uhd4K,
            VideoResolution::Fhd1080,
            VideoResolution::Hd720,
            VideoResolution::Sd480,
        ]
    }
}

/// エンコードプリセット
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoPreset {
    Ultrafast,
    Fast,
    Medium,
    Slow,
    Veryslow,
}

impl VideoPreset {
    /// 表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            VideoPreset::Ultrafast => "最速 (ultrafast)",
            VideoPreset::Fast => "高速 (fast)",
            VideoPreset::Medium => "標準 (medium)",
            VideoPreset::Slow => "高品質 (slow)",
            VideoPreset::Veryslow => "最高品質 (veryslow)",
        }
    }

    /// FFmpegプリセット名を取得
    pub fn ffmpeg_name(&self) -> &'static str {
        match self {
            VideoPreset::Ultrafast => "ultrafast",
            VideoPreset::Fast => "fast",
            VideoPreset::Medium => "medium",
            VideoPreset::Slow => "slow",
            VideoPreset::Veryslow => "veryslow",
        }
    }

    /// すべてのバリアントを取得
    pub fn all() -> &'static [VideoPreset] {
        &[
            VideoPreset::Ultrafast,
            VideoPreset::Fast,
            VideoPreset::Medium,
            VideoPreset::Slow,
            VideoPreset::Veryslow,
        ]
    }
}

/// オーディオコーデック
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioCodec {
    /// AAC
    Aac,
    /// MP3
    Mp3,
    /// FLAC（ロスレス）
    Flac,
    /// コピー（無変換）
    Copy,
}

impl AudioCodec {
    /// 表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            AudioCodec::Aac => "AAC",
            AudioCodec::Mp3 => "MP3",
            AudioCodec::Flac => "FLAC (ロスレス)",
            AudioCodec::Copy => "コピー (無変換)",
        }
    }

    /// すべてのバリアントを取得
    pub fn all() -> &'static [AudioCodec] {
        &[AudioCodec::Aac, AudioCodec::Mp3, AudioCodec::Flac, AudioCodec::Copy]
    }
}

/// オーディオビットレートの選択肢
pub fn audio_bitrate_options() -> &'static [(u32, &'static str)] {
    &[
        (128, "128 kbps"),
        (192, "192 kbps"),
        (256, "256 kbps"),
        (320, "320 kbps"),
    ]
}
