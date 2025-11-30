//! トランスコードジョブ

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use super::{HwAccelDetector, HwAccelType, TranscodeProgress, TranscodeSettings};

/// トランスコードジョブ
#[derive(Clone)]
pub struct TranscodeJob {
    /// 入力ファイルパス
    pub input_path: PathBuf,
    /// 出力ファイルパス
    pub output_path: PathBuf,
    /// トランスコード設定
    pub settings: TranscodeSettings,
    /// キャンセルフラグ
    pub cancelled: Arc<AtomicBool>,
    /// ジョブ状態
    pub state: JobState,
}

/// ジョブ状態
#[derive(Clone, Debug, PartialEq)]
pub enum JobState {
    /// 待機中
    Pending,
    /// 実行中
    Running,
    /// 完了
    Completed,
    /// エラー
    Failed(String),
    /// キャンセル
    Cancelled,
}

impl TranscodeJob {
    /// 新しいジョブを作成
    pub fn new(input_path: PathBuf, output_path: PathBuf, settings: TranscodeSettings) -> Self {
        Self {
            input_path,
            output_path,
            settings,
            cancelled: Arc::new(AtomicBool::new(false)),
            state: JobState::Pending,
        }
    }

    /// 出力パスを生成
    pub fn generate_output_path(
        input_path: &PathBuf,
        output_dir: &PathBuf,
        suffix: &str,
        settings: &TranscodeSettings,
    ) -> PathBuf {
        let stem = input_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");

        let extension = settings.container.extension();

        output_dir.join(format!("{}{}.{}", stem, suffix, extension))
    }

    /// ジョブをキャンセル
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// キャンセルされたかチェック
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// FFmpegコマンド引数を生成
    pub fn build_ffmpeg_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        // 入力ファイル
        args.push("-i".to_string());
        args.push(self.input_path.to_string_lossy().to_string());

        // HWアクセラレーション設定
        self.add_hwaccel_args(&mut args);

        // ビデオコーデック設定
        self.add_video_args(&mut args);

        // オーディオコーデック設定
        self.add_audio_args(&mut args);

        // 上書き確認なし
        args.push("-y".to_string());

        // 出力ファイル
        args.push(self.output_path.to_string_lossy().to_string());

        args
    }

    /// HWアクセラレーション引数を追加
    fn add_hwaccel_args(&self, args: &mut Vec<String>) {
        match self.settings.hwaccel {
            HwAccelType::Auto => {
                // 自動検出は実行時に決定
            }
            HwAccelType::Nvenc => {
                args.push("-hwaccel".to_string());
                args.push("cuda".to_string());
            }
            HwAccelType::Qsv => {
                args.push("-hwaccel".to_string());
                args.push("qsv".to_string());
            }
            HwAccelType::Amf => {
                args.push("-hwaccel".to_string());
                args.push("d3d11va".to_string());
            }
            HwAccelType::Software => {
                // ソフトウェアエンコード - 特別な引数なし
            }
        }
    }

    /// ビデオコーデック引数を追加
    fn add_video_args(&self, args: &mut Vec<String>) {
        use super::{VideoCodec, VideoResolution};

        // コーデック
        let encoder = self
            .settings
            .video_codec
            .encoder_name(&self.settings.hwaccel);
        args.push("-c:v".to_string());
        args.push(encoder.to_string());

        // 解像度
        if let VideoResolution::Custom(w, h) = self.settings.resolution {
            args.push("-vf".to_string());
            args.push(format!("scale={}:{}", w, h));
        } else if self.settings.resolution != VideoResolution::Original {
            let (w, h) = self.settings.resolution.dimensions();
            args.push("-vf".to_string());
            args.push(format!("scale={}:{}", w, h));
        }

        // CRF（品質）- HWエンコーダーでは-qpを使用
        match self.settings.hwaccel {
            HwAccelType::Nvenc | HwAccelType::Qsv | HwAccelType::Amf => {
                args.push("-qp".to_string());
                args.push(self.settings.crf.to_string());
            }
            _ => {
                args.push("-crf".to_string());
                args.push(self.settings.crf.to_string());
            }
        }

        // プリセット
        let preset_arg = match self.settings.hwaccel {
            HwAccelType::Nvenc => "-preset",
            HwAccelType::Qsv => "-preset",
            HwAccelType::Amf => "-quality",
            _ => "-preset",
        };
        args.push(preset_arg.to_string());
        args.push(self.settings.preset.ffmpeg_name().to_string());
    }

    /// オーディオコーデック引数を追加
    fn add_audio_args(&self, args: &mut Vec<String>) {
        use super::AudioCodec;

        match self.settings.audio_codec {
            AudioCodec::Copy => {
                args.push("-c:a".to_string());
                args.push("copy".to_string());
            }
            AudioCodec::Aac => {
                args.push("-c:a".to_string());
                args.push("aac".to_string());
                args.push("-b:a".to_string());
                args.push(format!("{}k", self.settings.audio_bitrate));
            }
            AudioCodec::Mp3 => {
                args.push("-c:a".to_string());
                args.push("libmp3lame".to_string());
                args.push("-b:a".to_string());
                args.push(format!("{}k", self.settings.audio_bitrate));
            }
            AudioCodec::Flac => {
                args.push("-c:a".to_string());
                args.push("flac".to_string());
            }
        }
    }
}
