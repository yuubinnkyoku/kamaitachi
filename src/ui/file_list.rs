//! ファイルリスト（Table）

use gpui::*;
use gpui_component::button::{Button, ButtonVariant};
use gpui_component::table::{Table, TableDelegate, ColFixed, ColSort};

use crate::app::{AppState, FileEntry, FileStatus};

/// ファイルリスト
pub struct FileList {
    /// アプリケーション状態
    app_state: AppState,
    /// 選択されたインデックス
    selected_index: Option<usize>,
}

impl FileList {
    pub fn new(app_state: AppState, _cx: &mut ViewContext<Self>) -> Self {
        Self {
            app_state,
            selected_index: None,
        }
    }

    /// ファイルを削除
    fn remove_selected(&mut self, cx: &mut ViewContext<Self>) {
        if let Some(index) = self.selected_index {
            self.app_state.remove_file(index, cx);
            self.selected_index = None;
            cx.notify();
        }
    }
}

impl Render for FileList {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let files = self.app_state.files.read(cx);

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
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .child(format!("ファイル ({} 件)", files.len()))
                    )
                    .child(
                        Button::new("remove-selected")
                            .label("削除")
                            .variant(ButtonVariant::Ghost)
                            .disabled(self.selected_index.is_none())
                            .on_click(cx.listener(|this, _, cx| {
                                this.remove_selected(cx);
                            }))
                    )
            )
            // ファイルリスト
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .overflow_y_scroll()
                    .children(
                        if files.is_empty() {
                            vec![
                                div()
                                    .size_full()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_color(rgb(0x6c7086))
                                    .child("ファイルをドラッグ＆ドロップまたは「ファイル追加」ボタンで追加")
                                    .into_any_element()
                            ]
                        } else {
                            files
                                .iter()
                                .enumerate()
                                .map(|(index, file)| {
                                    self.render_file_row(index, file, cx)
                                })
                                .collect()
                        }
                    )
            )
    }
}

impl FileList {
    /// ファイル行をレンダリング
    fn render_file_row(&self, index: usize, file: &FileEntry, cx: &mut ViewContext<Self>) -> AnyElement {
        let is_selected = self.selected_index == Some(index);
        let status_color = match file.status {
            FileStatus::Pending => rgb(0x6c7086),
            FileStatus::Processing => rgb(0x89b4fa),
            FileStatus::Completed => rgb(0xa6e3a1),
            FileStatus::Error(_) => rgb(0xf38ba8),
            FileStatus::Cancelled => rgb(0xfab387),
        };

        div()
            .w_full()
            .h(px(48.0))
            .px(px(16.0))
            .flex()
            .items_center()
            .gap(px(12.0))
            .bg(if is_selected { rgb(0x313244) } else { rgb(0x1e1e2e) })
            .hover(|s| s.bg(rgb(0x45475a)))
            .cursor_pointer()
            .on_click(cx.listener(move |this, _, cx| {
                this.selected_index = Some(index);
                cx.notify();
            }))
            // ステータスインジケーター
            .child(
                div()
                    .w(px(8.0))
                    .h(px(8.0))
                    .rounded_full()
                    .bg(status_color)
            )
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
                            .child(file.name.clone())
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x6c7086))
                            .truncate()
                            .child(file.path.to_string_lossy().to_string())
                    )
            )
            // サイズ
            .child(
                div()
                    .w(px(80.0))
                    .text_sm()
                    .text_color(rgb(0x6c7086))
                    .child(file.formatted_size())
            )
            // ステータス
            .child(
                div()
                    .w(px(80.0))
                    .text_sm()
                    .text_color(status_color)
                    .child(file.status.label())
            )
            // 進捗バー（処理中のみ表示）
            .when(file.status == FileStatus::Processing, |this| {
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
                                .w(relative(file.progress))
                        )
                )
            })
            .into_any_element()
    }
}
