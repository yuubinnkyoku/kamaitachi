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
        }
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
