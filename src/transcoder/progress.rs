//! 進捗フィルター

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// トランスコード進捗情報
#[derive(Clone, Debug)]
pub struct TranscodeProgress {
    /// 処理済みフレーム数
    pub frames_processed: u64,
    /// 総フレーム数（推定）
    pub total_frames: Option<u64>,
    /// 進捗率 (0.0 - 1.0)
    pub progress: f32,
    /// 処理速度 (fps)
    pub fps: f32,
    /// 経過時間
    pub elapsed: Duration,
    /// 残り時間（推定）
    pub remaining: Option<Duration>,
    /// 現在の処理時間位置
    pub current_time: Duration,
    /// 総時間
    pub total_time: Option<Duration>,
    /// 現在の出力サイズ（バイト）
    pub current_size: u64,
    /// 予測最終サイズ（バイト）
    pub estimated_size: Option<u64>,
}

impl Default for TranscodeProgress {
    fn default() -> Self {
        Self {
            frames_processed: 0,
            total_frames: None,
            progress: 0.0,
            fps: 0.0,
            elapsed: Duration::ZERO,
            remaining: None,
            current_time: Duration::ZERO,
            total_time: None,
            current_size: 0,
            estimated_size: None,
        }
    }
}

/// 進捗追跡用のフィルター
/// ez_ffmpeg::FrameFilter と連携して使用
pub struct ProgressFilter {
    /// 処理済みフレーム数
    frames_processed: AtomicU64,
    /// 総フレーム数
    total_frames: AtomicU64,
    /// 開始時刻
    start_time: Instant,
    /// キャンセルフラグ
    cancelled: Arc<AtomicBool>,
    /// 総時間（マイクロ秒）
    total_duration_us: AtomicU64,
    /// 現在時刻（マイクロ秒）
    current_time_us: AtomicU64,
    /// 現在の出力サイズ（バイト）
    current_size: AtomicU64,
    /// 入力ファイルサイズ（バイト）
    input_size: AtomicU64,
}

impl ProgressFilter {
    /// 新しい進捗フィルターを作成
    pub fn new(total_frames: Option<u64>, total_duration: Option<Duration>) -> Self {
        Self {
            frames_processed: AtomicU64::new(0),
            total_frames: AtomicU64::new(total_frames.unwrap_or(0)),
            start_time: Instant::now(),
            cancelled: Arc::new(AtomicBool::new(false)),
            total_duration_us: AtomicU64::new(
                total_duration.map(|d| d.as_micros() as u64).unwrap_or(0),
            ),
            current_time_us: AtomicU64::new(0),
            current_size: AtomicU64::new(0),
            input_size: AtomicU64::new(0),
        }
    }

    /// 入力ファイルサイズを設定
    pub fn set_input_size(&self, size: u64) {
        self.input_size.store(size, Ordering::SeqCst);
    }

    /// 現在の出力サイズを更新
    pub fn set_current_size(&self, size: u64) {
        self.current_size.store(size, Ordering::SeqCst);
    }

    /// キャンセルフラグを取得
    pub fn cancel_flag(&self) -> Arc<AtomicBool> {
        self.cancelled.clone()
    }

    /// フレーム処理を通知
    pub fn on_frame(&self, pts_us: Option<u64>) {
        self.frames_processed.fetch_add(1, Ordering::SeqCst);
        if let Some(pts) = pts_us {
            self.current_time_us.store(pts, Ordering::SeqCst);
        }
    }

    /// 総フレーム数を設定
    pub fn set_total_frames(&self, frames: u64) {
        self.total_frames.store(frames, Ordering::SeqCst);
    }

    /// 総時間を設定
    pub fn set_total_duration(&self, duration: Duration) {
        self.total_duration_us
            .store(duration.as_micros() as u64, Ordering::SeqCst);
    }

    /// 現在の進捗を取得
    pub fn get_progress(&self) -> TranscodeProgress {
        let frames_processed = self.frames_processed.load(Ordering::SeqCst);
        let total_frames = {
            let tf = self.total_frames.load(Ordering::SeqCst);
            if tf > 0 {
                Some(tf)
            } else {
                None
            }
        };
        let elapsed = self.start_time.elapsed();
        let current_time_us = self.current_time_us.load(Ordering::SeqCst);
        let total_duration_us = self.total_duration_us.load(Ordering::SeqCst);
        let current_size = self.current_size.load(Ordering::SeqCst);

        // FPS計算
        let fps = if elapsed.as_secs_f32() > 0.0 {
            frames_processed as f32 / elapsed.as_secs_f32()
        } else {
            0.0
        };

        // 進捗計算（時間ベースまたはフレームベース）
        let progress = if total_duration_us > 0 {
            (current_time_us as f32 / total_duration_us as f32).min(1.0)
        } else if let Some(total) = total_frames {
            if total > 0 {
                (frames_processed as f32 / total as f32).min(1.0)
            } else {
                0.0
            }
        } else {
            0.0
        };

        // 残り時間推定
        let remaining = if progress > 0.0 && progress < 1.0 {
            let total_estimated = elapsed.as_secs_f32() / progress;
            let remaining_secs = total_estimated - elapsed.as_secs_f32();
            Some(Duration::from_secs_f32(remaining_secs.max(0.0)))
        } else {
            None
        };

        // 予測最終サイズを計算
        let estimated_size = if progress > 0.05 && current_size > 0 {
            // 進捗が5%以上で現在サイズがある場合のみ予測
            Some((current_size as f64 / progress as f64) as u64)
        } else {
            None
        };

        TranscodeProgress {
            frames_processed,
            total_frames,
            progress,
            fps,
            elapsed,
            remaining,
            current_time: Duration::from_micros(current_time_us),
            total_time: if total_duration_us > 0 {
                Some(Duration::from_micros(total_duration_us))
            } else {
                None
            },
            current_size,
            estimated_size,
        }
    }

    /// キャンセル
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// キャンセルされたか確認
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

/// 時間をフォーマット
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

/// ファイルサイズをフォーマット
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

use super::preset::{AudioCodec, TranscodeSettings, VideoCodec, VideoPreset, VideoResolution};
use super::HwAccelType;

/// コンテンツタイプ（動き量補正用）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ContentType {
    /// 静止画・スライドショー（動きほぼなし）
    Static,
    /// アニメーション（限定的な動き）
    Anime,
    /// 通常の実写（標準的な動き）
    #[default]
    Normal,
    /// ゲーム実況・スポーツ（激しい動き）
    HighMotion,
    /// スクリーンレコード（デスクトップキャプチャ）
    ScreenRecord,
}

impl ContentType {
    /// 動き量補正係数を取得
    pub fn motion_factor(&self) -> f64 {
        match self {
            ContentType::Static => 0.20,       // 静止画: 80%削減
            ContentType::ScreenRecord => 0.35, // スクリーンレコード: 65%削減
            ContentType::Anime => 0.50,        // アニメ: 50%削減
            ContentType::Normal => 1.00,       // 実写（基準）
            ContentType::HighMotion => 1.60,   // ゲーム/スポーツ: 60%増加
        }
    }

    /// 表示名を取得
    pub fn display_name(&self) -> &'static str {
        match self {
            ContentType::Static => "静止画/スライド",
            ContentType::ScreenRecord => "画面録画",
            ContentType::Anime => "アニメ",
            ContentType::Normal => "実写（通常）",
            ContentType::HighMotion => "ゲーム/スポーツ",
        }
    }

    /// すべてのバリアントを取得
    pub fn all() -> &'static [ContentType] {
        &[
            ContentType::Static,
            ContentType::ScreenRecord,
            ContentType::Anime,
            ContentType::Normal,
            ContentType::HighMotion,
        ]
    }
}

/// 動画メタデータ（予測精度向上のため）
#[derive(Clone, Debug, Default)]
pub struct VideoMetadata {
    /// 解像度（幅, 高さ）
    pub resolution: Option<(u32, u32)>,
    /// フレームレート
    pub fps: Option<f64>,
    /// 動画の長さ（秒）
    pub duration: Option<f64>,
    /// コンテンツタイプ
    pub content_type: ContentType,
    /// 元の映像ビットレート（bps）
    pub source_video_bitrate: Option<u64>,
    /// 元の音声ビットレート（bps）
    pub source_audio_bitrate: Option<u64>,
    /// 元の全体ビットレート（bps）
    pub source_overall_bitrate: Option<u64>,
}

/// 設定から予測圧縮率を計算（2024-2025年実測値準拠の改良版）
/// この値は大まかな目安であり、実際のサイズは動画の内容によって変わる
/// 誤差目標: ±10-15%程度
pub fn estimate_compression_ratio(settings: &TranscodeSettings, source_resolution: (u32, u32)) -> f64 {
    // デフォルトのメタデータで計算
    let metadata = VideoMetadata {
        resolution: Some(source_resolution),
        ..Default::default()
    };
    estimate_compression_ratio_advanced(settings, &metadata)
}

/// 詳細なメタデータを使用した高精度予測
pub fn estimate_compression_ratio_advanced(
    settings: &TranscodeSettings,
    metadata: &VideoMetadata,
) -> f64 {
    let source_resolution = metadata.resolution.unwrap_or((1920, 1080));
    let source_fps = metadata.fps.unwrap_or(30.0);

    // ターゲット解像度を計算
    let target_resolution = match settings.resolution {
        VideoResolution::Original => source_resolution,
        res => {
            let dims = res.dimensions();
            if dims.0 == 0 || dims.1 == 0 {
                source_resolution
            } else {
                dims
            }
        }
    };

    // === ソースビットレートがある場合、より正確な予測を行う ===
    if let Some(source_video_bitrate) = metadata.source_video_bitrate {
        return estimate_from_source_bitrate(
            settings,
            metadata,
            source_video_bitrate,
            source_resolution,
            target_resolution,
            source_fps,
        );
    }

    // === ソースビットレートがない場合、従来の圧縮率ベースの予測 ===
    estimate_from_compression_ratio(
        settings,
        metadata,
        source_resolution,
        target_resolution,
        source_fps,
    )
}

/// ソースビットレートを基にした予測（高精度）
fn estimate_from_source_bitrate(
    settings: &TranscodeSettings,
    metadata: &VideoMetadata,
    source_video_bitrate: u64,
    source_resolution: (u32, u32),
    target_resolution: (u32, u32),
    source_fps: f64,
) -> f64 {
    // === 1. ターゲットビットレートを推定 ===
    // CRF→ビットレートの変換は動画の特性に依存するが、
    // 元のビットレートを基準にすることで精度が向上する

    // CRF係数（CRFが高いほどビットレートが低下）
    // 基準CRF=23として、CRF±6で約2倍の変化
    let crf_divisor = match settings.video_codec {
        VideoCodec::H264 => 6.0,
        VideoCodec::H265 => 5.5,
        VideoCodec::Vp9 => 5.8,
        VideoCodec::Av1 => 5.0,
    };
    let crf_factor = 2.0_f64.powf((23.0 - settings.crf as f64) / crf_divisor);

    // === 2. 解像度変換による影響 ===
    let source_pixels = source_resolution.0 as f64 * source_resolution.1 as f64;
    let target_pixels = target_resolution.0 as f64 * target_resolution.1 as f64;
    let resolution_factor = if source_pixels > 0.0 {
        (target_pixels / source_pixels).powf(0.85) // 解像度変更の影響は0.85乗
    } else {
        1.0
    };

    // === 3. コーデック効率 ===
    // 元のコーデックが分かれば比較できるが、ここでは出力コーデックの絶対効率を使用
    // 1080p 30fps CRF23 medium での典型的なビットレート（Mbps）
    let typical_bitrate_mbps = match settings.video_codec {
        VideoCodec::H264 => 8.0,  // H.264: 約8Mbps
        VideoCodec::H265 => 4.0,  // H.265: 約4Mbps
        VideoCodec::Vp9 => 3.6,   // VP9: 約3.6Mbps
        VideoCodec::Av1 => 2.3,   // AV1: 約2.3Mbps
    };

    // 元のビットレートと典型値の比率から、動画の複雑さを推定
    let source_mbps = source_video_bitrate as f64 / 1_000_000.0;
    // 解像度・フレームレートを正規化した実効ビットレート
    let normalized_source_mbps = source_mbps * (1920.0 * 1080.0 / source_pixels) * (30.0 / source_fps);
    
    // 複雑さ係数（元が高ビットレートなら複雑な動画）
    // 典型的な値（10Mbps程度）との比較
    let complexity_factor = (normalized_source_mbps / 10.0).sqrt().clamp(0.5, 2.0);

    // === 4. ターゲットビットレートを計算 ===
    let base_target_mbps = typical_bitrate_mbps * crf_factor * resolution_factor * complexity_factor;

    // === 5. プリセット係数 ===
    let preset_factor = match settings.preset {
        VideoPreset::Ultrafast => 1.55,
        VideoPreset::Fast => 1.18,
        VideoPreset::Medium => 1.00,
        VideoPreset::Slow => 0.89,
        VideoPreset::Veryslow => 0.81,
    };

    // === 6. HWアクセラレーション係数 ===
    let hwaccel_factor = match (&settings.hwaccel, &settings.video_codec) {
        (HwAccelType::Software, _) => 1.00,
        (HwAccelType::Nvenc, VideoCodec::Av1) => 1.10,
        (HwAccelType::Nvenc, _) => 1.30,
        (HwAccelType::Qsv, _) => 1.18,
        (HwAccelType::Amf, _) => 1.20,
        (HwAccelType::Auto, VideoCodec::Av1) => 1.05,
        (HwAccelType::Auto, _) => 1.15,
    };

    // === 7. 動き量補正 ===
    let motion_factor = metadata.content_type.motion_factor();

    // 最終的なターゲットビットレート
    let target_video_mbps = base_target_mbps * preset_factor * hwaccel_factor * motion_factor;
    let target_video_bitrate = target_video_mbps * 1_000_000.0;

    // === 8. 映像部分の圧縮率 ===
    let video_ratio = target_video_bitrate / source_video_bitrate as f64;

    // === 9. 音声部分の処理 ===
    let source_audio_bitrate = metadata.source_audio_bitrate.unwrap_or(192_000); // デフォルト192kbps
    let target_audio_bitrate = match settings.audio_codec {
        AudioCodec::Copy => source_audio_bitrate as f64,
        AudioCodec::Aac | AudioCodec::Mp3 => settings.audio_bitrate as f64 * 1000.0,
        AudioCodec::Flac => source_audio_bitrate as f64 * 2.5, // FLACは約2.5倍
    };

    // 全体に対する音声の割合
    let total_source_bitrate = metadata.source_overall_bitrate
        .unwrap_or(source_video_bitrate + source_audio_bitrate);
    let audio_portion = source_audio_bitrate as f64 / total_source_bitrate as f64;
    let video_portion = 1.0 - audio_portion;

    // === 10. 最終的な圧縮率 ===
    let audio_ratio = target_audio_bitrate / source_audio_bitrate as f64;
    let total_ratio = video_portion * video_ratio + audio_portion * audio_ratio;

    total_ratio.clamp(0.03, 5.0)
}

/// 従来の圧縮率ベースの予測（ソースビットレートがない場合）
fn estimate_from_compression_ratio(
    settings: &TranscodeSettings,
    metadata: &VideoMetadata,
    source_resolution: (u32, u32),
    target_resolution: (u32, u32),
    source_fps: f64,
) -> f64 {
    // === 1. CRF係数（コーデック別の減衰率を使用）===
    let crf_divisor = match settings.video_codec {
        VideoCodec::H264 => 6.0,
        VideoCodec::H265 => 5.5,
        VideoCodec::Vp9 => 5.8,
        VideoCodec::Av1 => 5.0,
    };
    let crf_factor = 2.0_f64.powf((23.0 - settings.crf as f64) / crf_divisor);

    // === 2. 解像度係数（改良版）===
    // ピクセル数比率^0.95 × 短辺比率^0.05（極端なアスペクト比で補正）
    let source_pixels = source_resolution.0 as f64 * source_resolution.1 as f64;
    let target_pixels = target_resolution.0 as f64 * target_resolution.1 as f64;
    let base_1080p_pixels = 1920.0 * 1080.0;

    let resolution_factor = if source_pixels > 0.0 && target_pixels > 0.0 {
        let pixel_ratio = target_pixels / source_pixels;
        let short_side_source = source_resolution.0.min(source_resolution.1) as f64;
        let short_side_target = target_resolution.0.min(target_resolution.1) as f64;
        let short_side_ratio = short_side_target / short_side_source.max(1.0);

        // メイン: ピクセル比率^0.95、補正: 短辺比率^0.05
        pixel_ratio.powf(0.95) * short_side_ratio.powf(0.05)
    } else {
        1.0
    };

    // === 3. フレームレート係数 ===
    // (fps / 30)^0.9 - 60fpsでも単純に2倍にはならない（GOP効率）
    let fps_factor = (source_fps / 30.0).powf(0.9);

    // === 4. 動き量補正（最重要）===
    let motion_factor = metadata.content_type.motion_factor();

    // === 5. コーデック効率（CRF23 medium software基準の実測値）===
    // H.265を基準1.0として設定
    let codec_efficiency = match settings.video_codec {
        VideoCodec::H264 => 2.00, // H.264は約2倍大きい
        VideoCodec::H265 => 1.00, // H.265を基準
        VideoCodec::Vp9 => 0.90,  // VP9は10%小さい
        VideoCodec::Av1 => 0.57,  // AV1は43%小さい（SVT-AV1基準）
    };

    // === 6. プリセット係数（実測値）===
    let preset_factor = match settings.preset {
        VideoPreset::Ultrafast => 1.55, // 55%大きい
        VideoPreset::Fast => 1.18,      // 18%大きい
        VideoPreset::Medium => 1.00,    // 基準
        VideoPreset::Slow => 0.89,      // 11%小さい
        VideoPreset::Veryslow => 0.81,  // 19%小さい
    };

    // === 7. HWアクセラレーション係数（品質劣化を考慮）===
    let hwaccel_factor = match (&settings.hwaccel, &settings.video_codec) {
        // ソフトウェアエンコード
        (HwAccelType::Software, _) => 1.00,

        // NVENC
        (HwAccelType::Nvenc, VideoCodec::Av1) => 1.10,  // NVENC AV1は比較的効率良い
        (HwAccelType::Nvenc, _) => 1.30,               // NVENC H.264/H.265は品質落ちる

        // QSV
        (HwAccelType::Qsv, _) => 1.18,

        // AMF
        (HwAccelType::Amf, _) => 1.20,

        // Auto（平均的な値）
        (HwAccelType::Auto, VideoCodec::Av1) => 1.05,
        (HwAccelType::Auto, _) => 1.15,
    };

    // === 8. オーディオサイズ計算（別途加算）===
    // 動画の長さが分かる場合は正確に計算、不明な場合は比率で概算
    let duration_hours = metadata.duration.unwrap_or(0.0) / 3600.0;

    let audio_size_mb = if duration_hours > 0.0 {
        // 長さが分かる場合: MB/時間で計算
        match settings.audio_codec {
            AudioCodec::Copy => 0.0, // 後で元ファイルの比率から計算
            AudioCodec::Aac => {
                let mb_per_hour = match settings.audio_bitrate {
                    b if b <= 128 => 8.0,
                    b if b <= 192 => 12.0,
                    b if b <= 256 => 18.0,
                    _ => 22.0,
                };
                mb_per_hour * duration_hours
            }
            AudioCodec::Mp3 => {
                let mb_per_hour = match settings.audio_bitrate {
                    b if b <= 128 => 9.0,
                    b if b <= 192 => 13.0,
                    b if b <= 256 => 19.0,
                    _ => 23.0,
                };
                mb_per_hour * duration_hours
            }
            AudioCodec::Flac => 100.0 * duration_hours, // FLAC: 約100MB/時間
        }
    } else {
        0.0 // 長さ不明の場合は後で比率で計算
    };

    // === ビデオ部分の圧縮率計算 ===
    // 基準: H.265 CRF23 medium software で 1080p 30fps 通常実写
    let video_compression =
        crf_factor * resolution_factor * fps_factor * motion_factor * codec_efficiency * preset_factor * hwaccel_factor;

    // === 最終計算 ===
    if duration_hours > 0.0 && audio_size_mb > 0.0 {
        // オーディオサイズを絶対値で計算できる場合
        // TODO: 元ファイルサイズが必要なので、ここでは比率ベースで計算
        let audio_ratio = 0.10; // 元ファイルの約10%がオーディオと仮定
        let video_ratio = 1.0 - audio_ratio;

        let audio_factor = match settings.audio_codec {
            AudioCodec::Copy => audio_ratio,
            AudioCodec::Aac => {
                let bitrate_ratio = settings.audio_bitrate as f64 / 192.0;
                audio_ratio * bitrate_ratio * 0.7
            }
            AudioCodec::Mp3 => {
                let bitrate_ratio = settings.audio_bitrate as f64 / 192.0;
                audio_ratio * bitrate_ratio * 0.8
            }
            AudioCodec::Flac => audio_ratio * 2.0,
        };

        (video_ratio * video_compression + audio_factor).max(0.03).min(5.0)
    } else {
        // 長さ不明の場合は比率ベースで計算
        let audio_ratio = 0.10;
        let video_ratio = 1.0 - audio_ratio;

        let audio_factor = match settings.audio_codec {
            AudioCodec::Copy => audio_ratio,
            AudioCodec::Aac => {
                let bitrate_ratio = settings.audio_bitrate as f64 / 192.0;
                audio_ratio * bitrate_ratio * 0.7
            }
            AudioCodec::Mp3 => {
                let bitrate_ratio = settings.audio_bitrate as f64 / 192.0;
                audio_ratio * bitrate_ratio * 0.8
            }
            AudioCodec::Flac => audio_ratio * 2.0,
        };

        (video_ratio * video_compression + audio_factor).max(0.03).min(5.0)
    }
}

/// シンプルな圧縮率計算（後方互換性のため）
pub fn estimate_compression_ratio_simple(
    crf: u8,
    source_resolution: (u32, u32),
    target_resolution: (u32, u32),
) -> f64 {
    let crf_factor = 2.0_f64.powf((23.0 - crf as f64) / 6.0);

    let source_pixels = source_resolution.0 as f64 * source_resolution.1 as f64;
    let target_pixels = target_resolution.0 as f64 * target_resolution.1 as f64;
    let resolution_factor = if source_pixels > 0.0 && target_pixels > 0.0 {
        target_pixels / source_pixels
    } else {
        1.0
    };

    (crf_factor * resolution_factor).max(0.05).min(2.0)
}
