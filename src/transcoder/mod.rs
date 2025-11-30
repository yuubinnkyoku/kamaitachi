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
    AudioCodec, ContainerFormat, TranscodeSettings, VideoCodec, VideoPreset, VideoResolution,
};
pub use progress::{
    estimate_compression_ratio, estimate_compression_ratio_advanced, format_duration, format_size,
    ContentType, TranscodeProgress, VideoMetadata,
};
