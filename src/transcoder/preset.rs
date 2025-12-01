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

    // === エンコーダー固有設定 ===
    /// レートコントロールモード
    pub rate_control: RateControlMode,
    /// ターゲットビットレート (kbps) - CBR/VBRモード用
    pub target_bitrate: u32,
    /// 最大ビットレート (kbps) - VBRモード用
    pub max_bitrate: u32,
    /// Bフレーム数
    pub bframes: u8,
    /// 参照フレーム数
    pub ref_frames: u8,
    /// GOP（キーフレーム間隔）
    pub gop_size: u32,
    /// ルックアヘッドフレーム数
    pub lookahead: u8,
    /// AQモード（適応的量子化）
    pub aq_mode: AqMode,
    /// AQ強度 (0-15、デフォルト8)
    pub aq_strength: u8,

    // === NVENC固有設定 ===
    /// NVENCチューニング
    pub nvenc_tune: NvencTune,
    /// NVENCマルチパス
    pub nvenc_multipass: NvencMultipass,
    /// NVENC B参照モード
    pub nvenc_b_ref_mode: NvencBRefMode,

    // === QSV固有設定 ===
    /// QSVルックアヘッド深度
    pub qsv_la_depth: u8,
    /// QSVアダプティブI
    pub qsv_adaptive_i: bool,
    /// QSVアダプティブB
    pub qsv_adaptive_b: bool,

    // === AMF固有設定 ===
    /// AMF使用法（品質 vs 速度）
    pub amf_usage: AmfUsage,
    /// AMF品質プリセット
    pub amf_quality: AmfQuality,

    // === libx264/libx265固有設定 ===
    /// チューニング設定
    pub x264_tune: X264Tune,
    /// プロファイル
    pub x264_profile: X264Profile,

    // === VP9固有設定 ===
    /// タイル列数
    pub vp9_tile_columns: u8,
    /// タイル行数
    pub vp9_tile_rows: u8,
    /// フレーム並列処理
    pub vp9_frame_parallel: bool,
    /// 自動ALTフレーム
    pub vp9_auto_alt_ref: bool,
    /// ラグインフレーム数
    pub vp9_lag_in_frames: u8,

    // === AV1固有設定 ===
    /// SVT-AV1フィルムグレイン
    pub svtav1_film_grain: u8,
    /// SVT-AV1フィルムグレイン合成
    pub svtav1_film_grain_denoise: bool,
    /// AV1タイル設定
    pub av1_tile_columns: u8,
    /// AV1タイル行数
    pub av1_tile_rows: u8,
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

            // エンコーダー固有設定のデフォルト
            rate_control: RateControlMode::Crf,
            target_bitrate: 5000,
            max_bitrate: 10000,
            bframes: 3,
            ref_frames: 4,
            gop_size: 250,
            lookahead: 20,
            aq_mode: AqMode::Variance,
            aq_strength: 8,

            // NVENC
            nvenc_tune: NvencTune::HighQuality,
            nvenc_multipass: NvencMultipass::Qres,
            nvenc_b_ref_mode: NvencBRefMode::Each,

            // QSV
            qsv_la_depth: 40,
            qsv_adaptive_i: true,
            qsv_adaptive_b: true,

            // AMF
            amf_usage: AmfUsage::Transcoding,
            amf_quality: AmfQuality::Balanced,

            // libx264/libx265
            x264_tune: X264Tune::None,
            x264_profile: X264Profile::High,

            // VP9
            vp9_tile_columns: 2,
            vp9_tile_rows: 1,
            vp9_frame_parallel: true,
            vp9_auto_alt_ref: true,
            vp9_lag_in_frames: 25,

            // AV1
            svtav1_film_grain: 0,
            svtav1_film_grain_denoise: false,
            av1_tile_columns: 2,
            av1_tile_rows: 2,
        }
    }
}

/// レートコントロールモード
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RateControlMode {
    /// CRF（固定品質）
    Crf,
    /// CBR（固定ビットレート）
    Cbr,
    /// VBR（可変ビットレート）
    Vbr,
    /// CQP（固定量子化パラメータ）- HWエンコーダー向け
    Cqp,
}

impl RateControlMode {
    /// 表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            RateControlMode::Crf => "CRF (固定品質)",
            RateControlMode::Cbr => "CBR (固定レート)",
            RateControlMode::Vbr => "VBR (可変レート)",
            RateControlMode::Cqp => "CQP (固定QP)",
        }
    }

    /// すべてのバリアントを取得
    pub fn all() -> &'static [RateControlMode] {
        &[
            RateControlMode::Crf,
            RateControlMode::Cbr,
            RateControlMode::Vbr,
            RateControlMode::Cqp,
        ]
    }
}

impl Default for RateControlMode {
    fn default() -> Self {
        RateControlMode::Crf
    }
}

/// 適応的量子化モード
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AqMode {
    /// 無効
    None,
    /// 分散ベースAQ
    Variance,
    /// 自動分散ベースAQ（ダーク領域にバイアス）
    AutoVariance,
    /// 自動分散ベースAQ（ダーク領域に強いバイアス）
    AutoVarianceBiased,
}

impl AqMode {
    /// 表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            AqMode::None => "無効",
            AqMode::Variance => "分散ベース",
            AqMode::AutoVariance => "自動分散",
            AqMode::AutoVarianceBiased => "自動分散 (ダーク強調)",
        }
    }

    /// FFmpeg引数値を取得
    pub fn ffmpeg_value(&self) -> u8 {
        match self {
            AqMode::None => 0,
            AqMode::Variance => 1,
            AqMode::AutoVariance => 2,
            AqMode::AutoVarianceBiased => 3,
        }
    }

    /// すべてのバリアントを取得
    pub fn all() -> &'static [AqMode] {
        &[
            AqMode::None,
            AqMode::Variance,
            AqMode::AutoVariance,
            AqMode::AutoVarianceBiased,
        ]
    }
}

impl Default for AqMode {
    fn default() -> Self {
        AqMode::Variance
    }
}

/// NVENCチューニング
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NvencTune {
    /// 高品質
    HighQuality,
    /// 低遅延
    LowLatency,
    /// 超低遅延
    UltraLowLatency,
    /// ロスレス
    Lossless,
}

impl NvencTune {
    /// 表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            NvencTune::HighQuality => "高品質 (hq)",
            NvencTune::LowLatency => "低遅延 (ll)",
            NvencTune::UltraLowLatency => "超低遅延 (ull)",
            NvencTune::Lossless => "ロスレス (lossless)",
        }
    }

    /// FFmpeg引数値を取得
    pub fn ffmpeg_value(&self) -> &'static str {
        match self {
            NvencTune::HighQuality => "hq",
            NvencTune::LowLatency => "ll",
            NvencTune::UltraLowLatency => "ull",
            NvencTune::Lossless => "lossless",
        }
    }

    /// すべてのバリアントを取得
    pub fn all() -> &'static [NvencTune] {
        &[
            NvencTune::HighQuality,
            NvencTune::LowLatency,
            NvencTune::UltraLowLatency,
            NvencTune::Lossless,
        ]
    }
}

impl Default for NvencTune {
    fn default() -> Self {
        NvencTune::HighQuality
    }
}

/// NVENCマルチパス
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NvencMultipass {
    /// 無効
    Disabled,
    /// 1/4解像度
    Qres,
    /// フル解像度
    Fullres,
}

impl NvencMultipass {
    /// 表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            NvencMultipass::Disabled => "無効",
            NvencMultipass::Qres => "1/4解像度",
            NvencMultipass::Fullres => "フル解像度",
        }
    }

    /// FFmpeg引数値を取得
    pub fn ffmpeg_value(&self) -> &'static str {
        match self {
            NvencMultipass::Disabled => "disabled",
            NvencMultipass::Qres => "qres",
            NvencMultipass::Fullres => "fullres",
        }
    }

    /// すべてのバリアントを取得
    pub fn all() -> &'static [NvencMultipass] {
        &[
            NvencMultipass::Disabled,
            NvencMultipass::Qres,
            NvencMultipass::Fullres,
        ]
    }
}

impl Default for NvencMultipass {
    fn default() -> Self {
        NvencMultipass::Qres
    }
}

/// NVENC B参照モード
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NvencBRefMode {
    /// 無効
    Disabled,
    /// 各Bフレーム
    Each,
    /// 中間のみ
    Middle,
}

impl NvencBRefMode {
    /// 表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            NvencBRefMode::Disabled => "無効",
            NvencBRefMode::Each => "各Bフレーム",
            NvencBRefMode::Middle => "中間のみ",
        }
    }

    /// FFmpeg引数値を取得
    pub fn ffmpeg_value(&self) -> &'static str {
        match self {
            NvencBRefMode::Disabled => "disabled",
            NvencBRefMode::Each => "each",
            NvencBRefMode::Middle => "middle",
        }
    }

    /// すべてのバリアントを取得
    pub fn all() -> &'static [NvencBRefMode] {
        &[
            NvencBRefMode::Disabled,
            NvencBRefMode::Each,
            NvencBRefMode::Middle,
        ]
    }
}

impl Default for NvencBRefMode {
    fn default() -> Self {
        NvencBRefMode::Each
    }
}

/// AMF使用法
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmfUsage {
    /// トランスコーディング
    Transcoding,
    /// 超低遅延
    UltraLowLatency,
    /// 低遅延
    LowLatency,
    /// ウェブカメラ
    Webcam,
}

impl AmfUsage {
    /// 表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            AmfUsage::Transcoding => "トランスコーディング",
            AmfUsage::UltraLowLatency => "超低遅延",
            AmfUsage::LowLatency => "低遅延",
            AmfUsage::Webcam => "ウェブカメラ",
        }
    }

    /// FFmpeg引数値を取得
    pub fn ffmpeg_value(&self) -> &'static str {
        match self {
            AmfUsage::Transcoding => "transcoding",
            AmfUsage::UltraLowLatency => "ultralowlatency",
            AmfUsage::LowLatency => "lowlatency",
            AmfUsage::Webcam => "webcam",
        }
    }

    /// すべてのバリアントを取得
    pub fn all() -> &'static [AmfUsage] {
        &[
            AmfUsage::Transcoding,
            AmfUsage::UltraLowLatency,
            AmfUsage::LowLatency,
            AmfUsage::Webcam,
        ]
    }
}

impl Default for AmfUsage {
    fn default() -> Self {
        AmfUsage::Transcoding
    }
}

/// AMF品質プリセット
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmfQuality {
    /// 速度重視
    Speed,
    /// バランス
    Balanced,
    /// 品質重視
    Quality,
}

impl AmfQuality {
    /// 表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            AmfQuality::Speed => "速度重視",
            AmfQuality::Balanced => "バランス",
            AmfQuality::Quality => "品質重視",
        }
    }

    /// FFmpeg引数値を取得
    pub fn ffmpeg_value(&self) -> &'static str {
        match self {
            AmfQuality::Speed => "speed",
            AmfQuality::Balanced => "balanced",
            AmfQuality::Quality => "quality",
        }
    }

    /// すべてのバリアントを取得
    pub fn all() -> &'static [AmfQuality] {
        &[AmfQuality::Speed, AmfQuality::Balanced, AmfQuality::Quality]
    }
}

impl Default for AmfQuality {
    fn default() -> Self {
        AmfQuality::Balanced
    }
}

/// x264チューニング
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum X264Tune {
    /// なし（デフォルト）
    None,
    /// 映画
    Film,
    /// アニメーション
    Animation,
    /// グレイン
    Grain,
    /// 静止画
    StillImage,
    /// PSNR最適化
    Psnr,
    /// SSIM最適化
    Ssim,
    /// 高速デコード
    FastDecode,
    /// ゼロレイテンシー
    ZeroLatency,
}

impl X264Tune {
    /// 表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            X264Tune::None => "なし",
            X264Tune::Film => "映画",
            X264Tune::Animation => "アニメーション",
            X264Tune::Grain => "グレイン (ノイズ保持)",
            X264Tune::StillImage => "静止画",
            X264Tune::Psnr => "PSNR最適化",
            X264Tune::Ssim => "SSIM最適化",
            X264Tune::FastDecode => "高速デコード",
            X264Tune::ZeroLatency => "ゼロレイテンシー",
        }
    }

    /// FFmpeg引数値を取得
    pub fn ffmpeg_value(&self) -> Option<&'static str> {
        match self {
            X264Tune::None => None,
            X264Tune::Film => Some("film"),
            X264Tune::Animation => Some("animation"),
            X264Tune::Grain => Some("grain"),
            X264Tune::StillImage => Some("stillimage"),
            X264Tune::Psnr => Some("psnr"),
            X264Tune::Ssim => Some("ssim"),
            X264Tune::FastDecode => Some("fastdecode"),
            X264Tune::ZeroLatency => Some("zerolatency"),
        }
    }

    /// すべてのバリアントを取得
    pub fn all() -> &'static [X264Tune] {
        &[
            X264Tune::None,
            X264Tune::Film,
            X264Tune::Animation,
            X264Tune::Grain,
            X264Tune::StillImage,
            X264Tune::Psnr,
            X264Tune::Ssim,
            X264Tune::FastDecode,
            X264Tune::ZeroLatency,
        ]
    }
}

impl Default for X264Tune {
    fn default() -> Self {
        X264Tune::None
    }
}

/// x264プロファイル
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum X264Profile {
    /// Baseline（低負荷デバイス向け）
    Baseline,
    /// Main（標準）
    Main,
    /// High（高品質）
    High,
    /// High 10（10ビット）
    High10,
    /// High 4:4:4 Predictive
    High444,
}

impl X264Profile {
    /// 表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            X264Profile::Baseline => "Baseline (互換性重視)",
            X264Profile::Main => "Main (標準)",
            X264Profile::High => "High (高品質)",
            X264Profile::High10 => "High 10 (10ビット)",
            X264Profile::High444 => "High 4:4:4",
        }
    }

    /// FFmpeg引数値を取得
    pub fn ffmpeg_value(&self) -> &'static str {
        match self {
            X264Profile::Baseline => "baseline",
            X264Profile::Main => "main",
            X264Profile::High => "high",
            X264Profile::High10 => "high10",
            X264Profile::High444 => "high444",
        }
    }

    /// すべてのバリアントを取得
    pub fn all() -> &'static [X264Profile] {
        &[
            X264Profile::Baseline,
            X264Profile::Main,
            X264Profile::High,
            X264Profile::High10,
            X264Profile::High444,
        ]
    }
}

impl Default for X264Profile {
    fn default() -> Self {
        X264Profile::High
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
        &[
            VideoCodec::H264,
            VideoCodec::H265,
            VideoCodec::Vp9,
            VideoCodec::Av1,
        ]
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
        &[
            AudioCodec::Aac,
            AudioCodec::Mp3,
            AudioCodec::Flac,
            AudioCodec::Copy,
        ]
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
