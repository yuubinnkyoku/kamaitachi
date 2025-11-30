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
                total_duration.map(|d| d.as_micros() as u64).unwrap_or(0)
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
        self.total_duration_us.store(duration.as_micros() as u64, Ordering::SeqCst);
    }

    /// 現在の進捗を取得
    pub fn get_progress(&self) -> TranscodeProgress {
        let frames_processed = self.frames_processed.load(Ordering::SeqCst);
        let total_frames = {
            let tf = self.total_frames.load(Ordering::SeqCst);
            if tf > 0 { Some(tf) } else { None }
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

/// CRFと設定から予測圧縮率を計算
/// この値は大まかな目安であり、実際のサイズは動画の内容によって大きく変わる
pub fn estimate_compression_ratio(
    crf: u8,
    source_resolution: (u32, u32),
    target_resolution: (u32, u32),
) -> f64 {
    // CRFベースの圧縮率（CRF 23をベースライン1.0として）
    // CRFが上がるとファイルサイズは小さくなる
    let crf_factor = 1.0 / (1.0 + (crf as f64 - 23.0) * 0.15);

    // 解像度による係数
    let source_pixels = source_resolution.0 as f64 * source_resolution.1 as f64;
    let target_pixels = target_resolution.0 as f64 * target_resolution.1 as f64;
    let resolution_factor = if source_pixels > 0.0 && target_pixels > 0.0 {
        target_pixels / source_pixels
    } else {
        1.0
    };

    // 最終的な圧縮率（元のサイズに対する比率）
    (crf_factor * resolution_factor).max(0.05).min(2.0)
}
