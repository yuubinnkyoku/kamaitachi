//! 進捗表示

use gpui::*;

use crate::app::AppState;
use crate::transcoder::format_duration;
use std::time::Duration;

/// 進捗ビュー
pub struct ProgressView {
    /// アプリケーション状態
    app_state: AppState,
}

impl ProgressView {
    pub fn new(app_state: AppState, _cx: &mut Context<Self>) -> Self {
        Self { app_state }
    }

    /// キャンセル処理
    fn cancel_transcode(&mut self, cx: &mut Context<Self>) {
        log::info!("Cancel button clicked");
        self.app_state.current_progress.cancel();
        cx.notify();
    }
}

impl Render for ProgressView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current_job = self.app_state.current_job.read(cx);
        let ffmpeg_status = if self.app_state.ffmpeg_path.read(cx).is_some() {
            "FFmpeg: 準備完了"
        } else {
            "FFmpeg: 未検出"
        };

        match current_job.as_ref() {
            Some(job) => {
                // 進捗情報を取得
                let progress = self.app_state.current_progress.get_progress();
                let elapsed_secs = self.app_state.current_progress.get_elapsed_secs();
                let remaining_secs = self.app_state.current_progress.get_remaining_secs();
                let fps = self.app_state.current_progress.get_fps();
                
                // 表示用の文字列を作成
                let progress_percent = (progress * 100.0) as u32;
                let elapsed_str = format_duration(Duration::from_secs_f32(elapsed_secs));
                let remaining_str = remaining_secs
                    .map(|s| format_duration(Duration::from_secs_f32(s)))
                    .unwrap_or_else(|| "--:--".to_string());
                
                let status_text = if fps > 0.0 {
                    format!("{}% | {} 経過 | {} 残り | {:.1} fps", progress_percent, elapsed_str, remaining_str, fps)
                } else {
                    format!("{}% | {} 経過 | {} 残り", progress_percent, elapsed_str, remaining_str)
                };

                // ジョブ実行中の進捗表示
                div()
                    .w_full()
                    .h(px(60.0))
                    .px(px(16.0))
                    .flex()
                    .items_center()
                    .gap(px(16.0))
                    .bg(rgb(0x181825))
                    // 進捗バー
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(
                                        div().text_sm().child(
                                            job.input_path
                                                .file_name()
                                                .and_then(|n| n.to_str())
                                                .unwrap_or("Processing...")
                                                .to_string(),
                                        ),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0x6c7086))
                                            .child(status_text),
                                    ),
                            )
                            .child(
                                div()
                                    .w_full()
                                    .h(px(4.0))
                                    .rounded(px(2.0))
                                    .bg(rgb(0x313244))
                                    .child(
                                        div()
                                            .h_full()
                                            .rounded(px(2.0))
                                            .bg(rgb(0x89b4fa))
                                            .w(relative(progress)),
                                    ),
                            ),
                    )
                    // キャンセルボタン
                    .child(
                        div()
                            .id("cancel-button")
                            .px(px(12.0))
                            .py(px(6.0))
                            .rounded(px(4.0))
                            .bg(rgb(0xf38ba8))
                            .text_color(rgb(0x1e1e2e))
                            .text_sm()
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(0xeba0ac)))
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.cancel_transcode(cx);
                            }))
                            .child("キャンセル"),
                    )
            }
            None => {
                // 待機中のステータスバー
                div()
                    .w_full()
                    .h(px(32.0))
                    .px(px(16.0))
                    .flex()
                    .items_center()
                    .justify_between()
                    .bg(rgb(0x181825))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x6c7086))
                            .child(ffmpeg_status),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x6c7086))
                            .child(format!("kamaitachi v{}", env!("CARGO_PKG_VERSION"))),
                    )
            }
        }
    }
}
