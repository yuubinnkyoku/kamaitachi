//! トランスコーダーモジュール

mod error;
mod hwaccel;
mod job;
mod preset;
pub mod progress;

pub use error::{FfmpegError, FfmpegErrorKind};
pub use hwaccel::{HwAccelDetector, HwAccelType};
pub use job::TranscodeJob;
pub use preset::{
    AmfQuality, AmfUsage, AqMode, AudioCodec, ContainerFormat, NvencBRefMode, NvencMultipass,
    NvencTune, RateControlMode, TranscodeSettings, VideoCodec, VideoPreset, VideoResolution,
    X264Profile, X264Tune,
};
pub use progress::{
    estimate_compression_ratio, estimate_compression_ratio_advanced, format_duration, format_size,
    ContentType, FfmpegProgressInfo, TranscodeProgress, VideoMetadata,
};
