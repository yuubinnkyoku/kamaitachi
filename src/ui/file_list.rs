//! ファイルリスト（Table）

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonVariant, ButtonVariants};
use gpui_component::Disableable;

use crate::app::{AppState, FileEntry, FileStatus};
use crate::transcoder::format_size;

/// ファイルリスト
pub struct FileList {
    /// アプリケーション状態
    app_state: AppState,
    /// 選択されたインデックス
    selected_index: Option<usize>,
}

impl FileList {
    pub fn new(app_state: AppState, _cx: &mut Context<Self>) -> Self {
        Self {
            app_state,
            selected_index: None,
        }
    }

    /// ファイルを削除
    fn remove_selected(&mut self, cx: &mut Context<Self>) {
        if let Some(index) = self.selected_index {
            self.app_state.files.update(cx, |files, _| {
                if index < files.len() {
                    files.remove(index);
                }
            });
            self.selected_index = None;
            cx.notify();
        }
    }
}

impl Render for FileList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let files = self.app_state.files.read(cx).clone();
        let files_len = files.len();
        let is_empty = files.is_empty();
        let selected = self.selected_index;

        // 合計サイズを計算
        let total_size: u64 = files.iter().map(|f| f.size).sum();
        let total_estimated: u64 = files.iter().filter_map(|f| f.estimated_size).sum();

        // 合計サイズのフォーマット
        let size_summary = if total_size > 0 && total_estimated > 0 {
            let compression_ratio = (total_estimated as f64 / total_size as f64) * 100.0;
            format!(
                "{} → {} ({:.0}%)",
                format_size(total_size),
                format_size(total_estimated),
                compression_ratio
            )
        } else if total_size > 0 {
            format_size(total_size)
        } else {
            String::new()
        };

        div()
            .size_full()
            .flex()
            .flex_col()
            // ヘッダー
            .child(
                div()
                    .w_full()
                    .h(px(40.0))
                    .px(px(16.0))
                    .flex()
                    .items_center()
                    .justify_between()
                    .bg(rgb(0x1e1e2e))
                    .border_b_1()
                    .border_color(rgb(0x313244))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(16.0))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(format!("ファイル ({} 件)", files_len)),
                            )
                            .when(!size_summary.is_empty(), |this| {
                                this.child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0xa6e3a1))
                                        .child(size_summary),
                                )
                            }),
                    )
                    .child(
                        Button::new("remove-selected")
                            .label("削除")
                            .with_variant(ButtonVariant::Ghost)
                            .disabled(selected.is_none())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.remove_selected(cx);
                            })),
                    ),
            )
            // ファイルリスト
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .overflow_hidden()
                    .children(if is_empty {
                        vec![div()
                            .size_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(rgb(0x6c7086))
                            .child("ファイルをドラッグ＆ドロップまたは「ファイル追加」ボタンで追加")
                            .into_any_element()]
                    } else {
                        files
                            .iter()
                            .enumerate()
                            .map(|(index, file)| self.render_file_row(index, file, cx))
                            .collect()
                    }),
            )
    }
}

impl FileList {
    /// ファイル行をレンダリング
    fn render_file_row(
        &self,
        index: usize,
        file: &FileEntry,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let is_selected = self.selected_index == Some(index);
        let status_color = match file.status {
            FileStatus::Pending => rgb(0x6c7086),
            FileStatus::Processing => rgb(0x89b4fa),
            FileStatus::Completed => rgb(0xa6e3a1),
            FileStatus::Error(_) => rgb(0xf38ba8),
            FileStatus::Cancelled => rgb(0xfab387),
        };

        // ライフタイムの問題を避けるため、所有権を持つ値に変換
        let file_name = file.name.clone();
        let file_path = file.path.to_string_lossy().to_string();
        let file_size = file.formatted_size();
        let estimated_size = file.estimated_size.map(format_size);
        let status_label = file.status.label().to_string();
        let is_processing = file.status == FileStatus::Processing;
        let progress = file.progress;

        // エラーメッセージを取得
        let error_message = match &file.status {
            FileStatus::Error(msg) => Some(msg.clone()),
            _ => None,
        };
        let has_error = error_message.is_some();
        // エラーメッセージの行数に応じて高さを調整
        let error_height = if let Some(ref msg) = error_message {
            // 提案が含まれている場合（💡があるか、改行が多い場合）は高さを増やす
            let line_count = msg.lines().count().max(1);
            if line_count > 2 {
                px(48.0 + 20.0 * line_count as f32)
            } else {
                px(96.0)
            }
        } else {
            px(48.0)
        };

        div()
            .w_full()
            .h(error_height)
            .px(px(16.0))
            .flex()
            .flex_col()
            .relative()
            .bg(if is_selected {
                rgb(0x313244)
            } else {
                rgb(0x1e1e2e)
            })
            .hover(|s| s.bg(rgb(0x45475a)))
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, cx| {
                    this.selected_index = Some(index);
                    cx.notify();
                }),
            )
            // メイン行
            .child(
                div()
                    .w_full()
                    .h(px(48.0))
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    // ステータスインジケーター
                    .child(div().w(px(8.0)).h(px(8.0)).rounded_full().bg(status_color))
                    // ファイル情報
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .overflow_hidden()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .truncate()
                                    .child(file_name),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x6c7086))
                                    .truncate()
                                    .child(file_path),
                            ),
                    )
                    // サイズ（元サイズ → 予測サイズ）
                    .child(
                        div()
                            .w(px(140.0))
                            .flex()
                            .flex_col()
                            .gap(px(1.0))
                            .child(div().text_sm().text_color(rgb(0x6c7086)).child(file_size))
                            .when_some(estimated_size, |this, est| {
                                this.child(
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0xa6e3a1))
                                        .child(format!("→ {}", est)),
                                )
                            }),
                    )
                    // ステータス
                    .child(
                        div()
                            .w(px(80.0))
                            .text_sm()
                            .text_color(status_color)
                            .child(status_label),
                    )
                    // 進捗バー（処理中のみ表示）
                    .when(is_processing, |this| {
                        this.child(
                            div()
                                .w(px(100.0))
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
                        )
                    }),
            )
            // エラーメッセージ（エラー時のみ表示）
            .when_some(error_message, |this, msg| {
                this.child(
                    div()
                        .w_full()
                        .px(px(20.0))
                        .py(px(6.0))
                        .bg(rgb(0x302030))
                        .border_1()
                        .border_color(rgb(0xf38ba8))
                        .rounded(px(4.0))
                        .text_xs()
                        .text_color(rgb(0xf5c2e7))
                        .child(msg),
                )
            })
            .into_any_element()
    }
}
