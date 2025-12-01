//! トランスコードジョブ

use anyhow::{Context, Result};
use log::{info, warn};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use super::{
    AqMode, HwAccelDetector, HwAccelType, RateControlMode, TranscodeProgress, TranscodeSettings,
    VideoCodec,
};

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
    /// ffmpeg_pathを渡すとエンコーダーの利用可能性をチェックしてフォールバック
    pub fn build_ffmpeg_args(&self) -> Vec<String> {
        self.build_ffmpeg_args_with_path(None)
    }

    /// FFmpegコマンド引数を生成（FFmpegパス指定版）
    pub fn build_ffmpeg_args_with_path(
        &self,
        ffmpeg_path: Option<&std::path::PathBuf>,
    ) -> Vec<String> {
        let mut args = Vec::new();

        // 実際に使用するエンコーダーとHWアクセラレーションを決定
        let (actual_encoder, actual_hwaccel) = HwAccelDetector::get_available_encoder(
            &self.settings.video_codec,
            &self.settings.hwaccel,
            ffmpeg_path,
        );

        // HWアクセラレーション設定（入力オプションなので -i の前に配置）
        self.add_hwaccel_args(&mut args, &actual_hwaccel);

        // 入力ファイル
        args.push("-i".to_string());
        args.push(self.input_path.to_string_lossy().to_string());

        // ビデオコーデック設定
        self.add_video_args_with_encoder(&mut args, &actual_encoder, &actual_hwaccel);

        // オーディオコーデック設定
        self.add_audio_args(&mut args);

        // 進捗情報をstdoutに構造化フォーマットで出力
        args.push("-progress".to_string());
        args.push("pipe:1".to_string());

        // 進捗更新間隔を0.5秒に設定
        args.push("-stats_period".to_string());
        args.push("0.5".to_string());

        // 上書き確認なし
        args.push("-y".to_string());

        // 出力ファイル
        args.push(self.output_path.to_string_lossy().to_string());

        args
    }

    /// HWアクセラレーション引数を追加
    fn add_hwaccel_args(&self, args: &mut Vec<String>, hwaccel: &HwAccelType) {
        match hwaccel {
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

    /// ビデオコーデック引数を追加（エンコーダー指定版）
    fn add_video_args_with_encoder(
        &self,
        args: &mut Vec<String>,
        encoder: &str,
        hwaccel: &HwAccelType,
    ) {
        use super::VideoResolution;

        // コーデック
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

        // エンコーダー固有のオプション設定
        match encoder {
            // NVIDIA NVENC H.264
            "h264_nvenc" => {
                self.add_nvenc_args(args);
            }

            // NVIDIA NVENC HEVC
            "hevc_nvenc" => {
                self.add_nvenc_args(args);
            }

            // NVIDIA NVENC AV1
            "av1_nvenc" => {
                self.add_nvenc_args(args);
            }

            // Intel QSV H.264
            "h264_qsv" => {
                self.add_qsv_args(args);
            }

            // Intel QSV HEVC
            "hevc_qsv" => {
                self.add_qsv_args(args);
            }

            // Intel QSV AV1
            "av1_qsv" => {
                self.add_qsv_args(args);
            }

            // Intel QSV VP9
            "vp9_qsv" => {
                self.add_qsv_args(args);
            }

            // AMD AMF H.264
            "h264_amf" => {
                self.add_amf_args(args);
            }

            // AMD AMF HEVC
            "hevc_amf" => {
                self.add_amf_args(args);
            }

            // AMD AMF AV1
            "av1_amf" => {
                self.add_amf_args(args);
            }

            // libx264
            "libx264" => {
                self.add_x264_args(args);
            }

            // libx265
            "libx265" => {
                self.add_x265_args(args);
            }

            // VP9 (libvpx-vp9)
            "libvpx-vp9" => {
                self.add_vp9_args(args);
            }

            // libaom-av1
            "libaom-av1" => {
                self.add_libaom_av1_args(args);
            }

            // libsvtav1 (SVT-AV1)
            "libsvtav1" => {
                self.add_svtav1_args(args);
            }

            // その他のエンコーダー（フォールバック）
            _ => {
                // CRF設定のみ
                self.add_rate_control_args(args, hwaccel);
                args.push("-preset".to_string());
                args.push(self.settings.preset.ffmpeg_name().to_string());
            }
        }

        // 共通GOP設定
        if self.settings.gop_size > 0 {
            args.push("-g".to_string());
            args.push(self.settings.gop_size.to_string());
        }
    }

    /// レートコントロール引数を追加
    fn add_rate_control_args(&self, args: &mut Vec<String>, hwaccel: &HwAccelType) {
        match self.settings.rate_control {
            RateControlMode::Crf => {
                match hwaccel {
                    HwAccelType::Nvenc | HwAccelType::Qsv | HwAccelType::Amf => {
                        // HWエンコーダーでは-cqを使用
                        args.push("-cq".to_string());
                        args.push(self.settings.crf.to_string());
                    }
                    _ => {
                        args.push("-crf".to_string());
                        args.push(self.settings.crf.to_string());
                    }
                }
            }
            RateControlMode::Cbr => {
                args.push("-b:v".to_string());
                args.push(format!("{}k", self.settings.target_bitrate));
                args.push("-maxrate".to_string());
                args.push(format!("{}k", self.settings.target_bitrate));
                args.push("-bufsize".to_string());
                args.push(format!("{}k", self.settings.target_bitrate * 2));
            }
            RateControlMode::Vbr => {
                args.push("-b:v".to_string());
                args.push(format!("{}k", self.settings.target_bitrate));
                args.push("-maxrate".to_string());
                args.push(format!("{}k", self.settings.max_bitrate));
                args.push("-bufsize".to_string());
                args.push(format!("{}k", self.settings.max_bitrate));
            }
            RateControlMode::Cqp => {
                args.push("-qp".to_string());
                args.push(self.settings.crf.to_string());
            }
        }
    }

    /// NVENC固有引数を追加
    fn add_nvenc_args(&self, args: &mut Vec<String>) {
        // チューニング
        args.push("-tune".to_string());
        args.push(self.settings.nvenc_tune.ffmpeg_value().to_string());

        // レートコントロール
        match self.settings.rate_control {
            RateControlMode::Crf => {
                args.push("-rc".to_string());
                args.push("vbr".to_string());
                args.push("-cq".to_string());
                args.push(self.settings.crf.to_string());
            }
            RateControlMode::Cbr => {
                args.push("-rc".to_string());
                args.push("cbr".to_string());
                args.push("-b:v".to_string());
                args.push(format!("{}k", self.settings.target_bitrate));
            }
            RateControlMode::Vbr => {
                args.push("-rc".to_string());
                args.push("vbr".to_string());
                args.push("-b:v".to_string());
                args.push(format!("{}k", self.settings.target_bitrate));
                args.push("-maxrate".to_string());
                args.push(format!("{}k", self.settings.max_bitrate));
            }
            RateControlMode::Cqp => {
                args.push("-rc".to_string());
                args.push("constqp".to_string());
                args.push("-qp".to_string());
                args.push(self.settings.crf.to_string());
            }
        }

        // プリセット
        args.push("-preset".to_string());
        let preset = match self.settings.preset {
            super::VideoPreset::Ultrafast => "p1",
            super::VideoPreset::Fast => "p3",
            super::VideoPreset::Medium => "p4",
            super::VideoPreset::Slow => "p6",
            super::VideoPreset::Veryslow => "p7",
        };
        args.push(preset.to_string());

        // マルチパス
        args.push("-multipass".to_string());
        args.push(self.settings.nvenc_multipass.ffmpeg_value().to_string());

        // Bフレーム
        if self.settings.bframes > 0 {
            args.push("-bf".to_string());
            args.push(self.settings.bframes.to_string());

            // B参照モード
            args.push("-b_ref_mode".to_string());
            args.push(self.settings.nvenc_b_ref_mode.ffmpeg_value().to_string());
        }

        // ルックアヘッド
        if self.settings.lookahead > 0 {
            args.push("-rc-lookahead".to_string());
            args.push(self.settings.lookahead.to_string());
        }

        // AQ設定
        if self.settings.aq_mode != AqMode::None {
            args.push("-spatial-aq".to_string());
            args.push("1".to_string());
            args.push("-aq-strength".to_string());
            args.push(self.settings.aq_strength.to_string());
        }
    }

    /// QSV固有引数を追加
    fn add_qsv_args(&self, args: &mut Vec<String>) {
        // レートコントロール
        match self.settings.rate_control {
            RateControlMode::Crf => {
                args.push("-global_quality".to_string());
                args.push(self.settings.crf.to_string());
            }
            RateControlMode::Cbr => {
                args.push("-b:v".to_string());
                args.push(format!("{}k", self.settings.target_bitrate));
                args.push("-maxrate".to_string());
                args.push(format!("{}k", self.settings.target_bitrate));
            }
            RateControlMode::Vbr => {
                args.push("-b:v".to_string());
                args.push(format!("{}k", self.settings.target_bitrate));
                args.push("-maxrate".to_string());
                args.push(format!("{}k", self.settings.max_bitrate));
            }
            RateControlMode::Cqp => {
                args.push("-q".to_string());
                args.push(self.settings.crf.to_string());
            }
        }

        // プリセット
        args.push("-preset".to_string());
        let preset = match self.settings.preset {
            super::VideoPreset::Ultrafast => "veryfast",
            super::VideoPreset::Fast => "faster",
            super::VideoPreset::Medium => "medium",
            super::VideoPreset::Slow => "slower",
            super::VideoPreset::Veryslow => "veryslow",
        };
        args.push(preset.to_string());

        // ルックアヘッド
        if self.settings.qsv_la_depth > 0 {
            args.push("-look_ahead".to_string());
            args.push("1".to_string());
            args.push("-look_ahead_depth".to_string());
            args.push(self.settings.qsv_la_depth.to_string());
        }

        // アダプティブI/B
        if self.settings.qsv_adaptive_i {
            args.push("-adaptive_i".to_string());
            args.push("1".to_string());
        }
        if self.settings.qsv_adaptive_b {
            args.push("-adaptive_b".to_string());
            args.push("1".to_string());
        }

        // Bフレーム
        if self.settings.bframes > 0 {
            args.push("-bf".to_string());
            args.push(self.settings.bframes.to_string());
        }

        // 参照フレーム
        if self.settings.ref_frames > 0 {
            args.push("-refs".to_string());
            args.push(self.settings.ref_frames.to_string());
        }
    }

    /// AMF固有引数を追加
    fn add_amf_args(&self, args: &mut Vec<String>) {
        // 使用法
        args.push("-usage".to_string());
        args.push(self.settings.amf_usage.ffmpeg_value().to_string());

        // 品質プリセット
        args.push("-quality".to_string());
        args.push(self.settings.amf_quality.ffmpeg_value().to_string());

        // レートコントロール
        match self.settings.rate_control {
            RateControlMode::Crf => {
                args.push("-rc".to_string());
                args.push("cqp".to_string());
                args.push("-qp_i".to_string());
                args.push(self.settings.crf.to_string());
                args.push("-qp_p".to_string());
                args.push(self.settings.crf.to_string());
                args.push("-qp_b".to_string());
                args.push((self.settings.crf + 2).to_string());
            }
            RateControlMode::Cbr => {
                args.push("-rc".to_string());
                args.push("cbr".to_string());
                args.push("-b:v".to_string());
                args.push(format!("{}k", self.settings.target_bitrate));
            }
            RateControlMode::Vbr => {
                args.push("-rc".to_string());
                args.push("vbr_peak".to_string());
                args.push("-b:v".to_string());
                args.push(format!("{}k", self.settings.target_bitrate));
                args.push("-maxrate".to_string());
                args.push(format!("{}k", self.settings.max_bitrate));
            }
            RateControlMode::Cqp => {
                args.push("-rc".to_string());
                args.push("cqp".to_string());
                args.push("-qp_i".to_string());
                args.push(self.settings.crf.to_string());
                args.push("-qp_p".to_string());
                args.push(self.settings.crf.to_string());
            }
        }

        // Bフレーム
        if self.settings.bframes > 0 {
            args.push("-bf".to_string());
            args.push(self.settings.bframes.to_string());
        }
    }

    /// libx264固有引数を追加
    fn add_x264_args(&self, args: &mut Vec<String>) {
        // プロファイル
        args.push("-profile:v".to_string());
        args.push(self.settings.x264_profile.ffmpeg_value().to_string());

        // レートコントロール
        self.add_rate_control_args(args, &HwAccelType::Software);

        // プリセット
        args.push("-preset".to_string());
        args.push(self.settings.preset.ffmpeg_name().to_string());

        // チューニング
        if let Some(tune) = self.settings.x264_tune.ffmpeg_value() {
            args.push("-tune".to_string());
            args.push(tune.to_string());
        }

        // Bフレーム
        if self.settings.bframes > 0 {
            args.push("-bf".to_string());
            args.push(self.settings.bframes.to_string());
        }

        // 参照フレーム
        if self.settings.ref_frames > 0 {
            args.push("-refs".to_string());
            args.push(self.settings.ref_frames.to_string());
        }

        // AQ設定
        if self.settings.aq_mode != AqMode::None {
            args.push("-aq-mode".to_string());
            args.push(self.settings.aq_mode.ffmpeg_value().to_string());
            args.push("-aq-strength".to_string());
            args.push(format!("{:.1}", self.settings.aq_strength as f32 / 10.0));
        }

        // ルックアヘッド
        if self.settings.lookahead > 0 {
            args.push("-rc-lookahead".to_string());
            args.push(self.settings.lookahead.to_string());
        }
    }

    /// libx265固有引数を追加
    fn add_x265_args(&self, args: &mut Vec<String>) {
        // レートコントロール
        self.add_rate_control_args(args, &HwAccelType::Software);

        // プリセット
        args.push("-preset".to_string());
        args.push(self.settings.preset.ffmpeg_name().to_string());

        // チューニング（x265は一部異なる）
        if let Some(tune) = self.settings.x264_tune.ffmpeg_value() {
            // x265でサポートされるtuneのみ使用
            if matches!(
                tune,
                "grain" | "animation" | "zerolatency" | "fastdecode" | "psnr" | "ssim"
            ) {
                args.push("-tune".to_string());
                args.push(tune.to_string());
            }
        }

        // x265固有パラメータをx265-paramsで渡す
        let mut x265_params = Vec::new();

        // Bフレーム
        if self.settings.bframes > 0 {
            x265_params.push(format!("bframes={}", self.settings.bframes));
        }

        // 参照フレーム
        if self.settings.ref_frames > 0 {
            x265_params.push(format!("ref={}", self.settings.ref_frames));
        }

        // AQ設定
        if self.settings.aq_mode != AqMode::None {
            x265_params.push(format!("aq-mode={}", self.settings.aq_mode.ffmpeg_value()));
            x265_params.push(format!(
                "aq-strength={:.1}",
                self.settings.aq_strength as f32 / 10.0
            ));
        }

        // ルックアヘッド
        if self.settings.lookahead > 0 {
            x265_params.push(format!("rc-lookahead={}", self.settings.lookahead));
        }

        if !x265_params.is_empty() {
            args.push("-x265-params".to_string());
            args.push(x265_params.join(":"));
        }
    }

    /// VP9固有引数を追加
    fn add_vp9_args(&self, args: &mut Vec<String>) {
        // VP9では -b:v 0 + -crf でCRFモードを使用
        match self.settings.rate_control {
            RateControlMode::Crf | RateControlMode::Cqp => {
                args.push("-b:v".to_string());
                args.push("0".to_string());
                args.push("-crf".to_string());
                args.push(self.settings.crf.to_string());
            }
            RateControlMode::Cbr => {
                args.push("-b:v".to_string());
                args.push(format!("{}k", self.settings.target_bitrate));
                args.push("-minrate".to_string());
                args.push(format!("{}k", self.settings.target_bitrate));
                args.push("-maxrate".to_string());
                args.push(format!("{}k", self.settings.target_bitrate));
            }
            RateControlMode::Vbr => {
                args.push("-b:v".to_string());
                args.push(format!("{}k", self.settings.target_bitrate));
                args.push("-maxrate".to_string());
                args.push(format!("{}k", self.settings.max_bitrate));
            }
        }

        // VP9は -cpu-used オプションを使用（0-8、値が大きいほど高速）
        args.push("-cpu-used".to_string());
        let cpu_used = match self.settings.preset {
            super::VideoPreset::Ultrafast => "8",
            super::VideoPreset::Fast => "6",
            super::VideoPreset::Medium => "4",
            super::VideoPreset::Slow => "2",
            super::VideoPreset::Veryslow => "0",
        };
        args.push(cpu_used.to_string());

        // VP9は -deadline を設定
        args.push("-deadline".to_string());
        args.push("good".to_string());

        // row-mt を有効にしてマルチスレッド化
        args.push("-row-mt".to_string());
        args.push("1".to_string());

        // タイル設定
        if self.settings.vp9_tile_columns > 0 {
            args.push("-tile-columns".to_string());
            args.push(self.settings.vp9_tile_columns.to_string());
        }
        if self.settings.vp9_tile_rows > 0 {
            args.push("-tile-rows".to_string());
            args.push(self.settings.vp9_tile_rows.to_string());
        }

        // フレーム並列処理
        if self.settings.vp9_frame_parallel {
            args.push("-frame-parallel".to_string());
            args.push("1".to_string());
        }

        // 自動ALTフレーム
        if self.settings.vp9_auto_alt_ref {
            args.push("-auto-alt-ref".to_string());
            args.push("1".to_string());

            // ラグインフレーム
            if self.settings.vp9_lag_in_frames > 0 {
                args.push("-lag-in-frames".to_string());
                args.push(self.settings.vp9_lag_in_frames.to_string());
            }
        }
    }

    /// libaom-av1固有引数を追加
    fn add_libaom_av1_args(&self, args: &mut Vec<String>) {
        // レートコントロール
        match self.settings.rate_control {
            RateControlMode::Crf | RateControlMode::Cqp => {
                args.push("-crf".to_string());
                args.push(self.settings.crf.to_string());
            }
            RateControlMode::Cbr => {
                args.push("-b:v".to_string());
                args.push(format!("{}k", self.settings.target_bitrate));
            }
            RateControlMode::Vbr => {
                args.push("-b:v".to_string());
                args.push(format!("{}k", self.settings.target_bitrate));
                args.push("-maxrate".to_string());
                args.push(format!("{}k", self.settings.max_bitrate));
            }
        }

        // cpu-used オプション（0-8、値が大きいほど高速）
        args.push("-cpu-used".to_string());
        let cpu_used = match self.settings.preset {
            super::VideoPreset::Ultrafast => "8",
            super::VideoPreset::Fast => "6",
            super::VideoPreset::Medium => "4",
            super::VideoPreset::Slow => "2",
            super::VideoPreset::Veryslow => "1",
        };
        args.push(cpu_used.to_string());

        // row-mt を有効にしてマルチスレッド化（高速化に重要）
        args.push("-row-mt".to_string());
        args.push("1".to_string());

        // タイル設定
        let tile_setting = format!(
            "{}x{}",
            self.settings.av1_tile_columns, self.settings.av1_tile_rows
        );
        args.push("-tiles".to_string());
        args.push(tile_setting);
    }

    /// SVT-AV1固有引数を追加
    fn add_svtav1_args(&self, args: &mut Vec<String>) {
        // レートコントロール
        match self.settings.rate_control {
            RateControlMode::Crf | RateControlMode::Cqp => {
                args.push("-crf".to_string());
                args.push(self.settings.crf.to_string());
            }
            RateControlMode::Cbr => {
                args.push("-b:v".to_string());
                args.push(format!("{}k", self.settings.target_bitrate));
                args.push("-rc".to_string());
                args.push("1".to_string()); // CBR mode
            }
            RateControlMode::Vbr => {
                args.push("-b:v".to_string());
                args.push(format!("{}k", self.settings.target_bitrate));
                args.push("-maxrate".to_string());
                args.push(format!("{}k", self.settings.max_bitrate));
            }
        }

        // SVT-AV1はpresetオプションを使用（0-13、値が大きいほど高速）
        args.push("-preset".to_string());
        let preset = match self.settings.preset {
            super::VideoPreset::Ultrafast => "12",
            super::VideoPreset::Fast => "10",
            super::VideoPreset::Medium => "8",
            super::VideoPreset::Slow => "5",
            super::VideoPreset::Veryslow => "2",
        };
        args.push(preset.to_string());

        // フィルムグレイン
        if self.settings.svtav1_film_grain > 0 {
            args.push("-svtav1-params".to_string());
            let mut params = format!("film-grain={}", self.settings.svtav1_film_grain);
            if self.settings.svtav1_film_grain_denoise {
                params.push_str(":film-grain-denoise=1");
            }
            args.push(params);
        }

        // タイル設定
        if self.settings.av1_tile_columns > 0 || self.settings.av1_tile_rows > 0 {
            args.push("-tile_columns".to_string());
            args.push(self.settings.av1_tile_columns.to_string());
            args.push("-tile_rows".to_string());
            args.push(self.settings.av1_tile_rows.to_string());
        }
    }

    /// ビデオコーデック引数を追加（互換性のため残す）
    fn add_video_args(&self, args: &mut Vec<String>) {
        let encoder = self
            .settings
            .video_codec
            .encoder_name(&self.settings.hwaccel);
        self.add_video_args_with_encoder(args, encoder, &self.settings.hwaccel);
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
