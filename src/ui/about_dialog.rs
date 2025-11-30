//! ライセンス情報ダイアログ

use gpui::*;
use gpui_component::button::{Button, ButtonVariant, ButtonVariants};

/// Aboutダイアログ
pub struct AboutDialog;

impl AboutDialog {
    /// ダイアログ内容をレンダリング
    pub fn render_content<F: Fn(&ClickEvent, &mut Window, &mut App) + 'static>(
        on_close: F,
    ) -> impl IntoElement {
        div()
            .w(px(500.0))
            .max_h(px(600.0))
            .rounded(px(8.0))
            .bg(rgb(0x1e1e2e))
            .border_1()
            .border_color(rgb(0x313244))
            .overflow_hidden()
            .flex()
            .flex_col()
            // ヘッダー
            .child(
                div()
                    .w_full()
                    .p(px(16.0))
                    .flex()
                    .items_center()
                    .justify_between()
                    .border_b_1()
                    .border_color(rgb(0x313244))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(FontWeight::BOLD)
                            .child("About kamaitachi"),
                    )
                    .child(
                        Button::new("close")
                            .label("✕")
                            .with_variant(ButtonVariant::Ghost)
                            .on_click(on_close),
                    ),
            )
            // コンテンツ
            .child(
                div()
                    .flex_1()
                    .p(px(16.0))
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .gap(px(16.0))
                    // タイトル
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(FontWeight::BOLD)
                                    .child("kamaitachi - 鎌鼬"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x6c7086))
                                    .child(format!("Version {}", env!("CARGO_PKG_VERSION"))),
                            ),
                    )
                    // 説明
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xa6adc8))
                            .child("HandBrake代替の高速トランスコーダー"),
                    )
                    // 区切り線
                    .child(div().w_full().h(px(1.0)).bg(rgb(0x313244)))
                    // ライセンス
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child("ライセンス"),
                            )
                            .child(
                                div()
                                    .p(px(12.0))
                                    .rounded(px(4.0))
                                    .bg(rgb(0x181825))
                                    .text_xs()
                                    .font_family("monospace")
                                    .child(LICENSE_TEXT),
                            ),
                    )
                    // 使用ライブラリ
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child("使用ライブラリ"),
                            )
                            .child(div().flex().flex_col().gap(px(4.0)).children(
                                LIBRARIES.iter().map(|(name, license)| {
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(div().text_sm().child(name.to_string()))
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x6c7086))
                                                .child(license.to_string()),
                                        )
                                }),
                            )),
                    )
                    // リンク
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child("リンク"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(4.0))
                                    .child(Self::render_link("FFmpeg", "https://ffmpeg.org/"))
                                    .child(Self::render_link(
                                        "FFmpegソースコード",
                                        "https://ffmpeg.org/download.html",
                                    ))
                                    .child(Self::render_link(
                                        "GitHub",
                                        "https://github.com/yuubinnkyoku/kamaitachi",
                                    )),
                            ),
                    ),
            )
    }

    fn render_link(name: &str, url: &str) -> impl IntoElement {
        div()
            .flex()
            .items_center()
            .justify_between()
            .child(div().text_sm().child(name.to_string()))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x89b4fa))
                    .child(url.to_string()),
            )
    }
}

const LICENSE_TEXT: &str = r#"kamaitachi - 鎌鼬
Copyright (C) 2025 yuubinnkyoku
Licensed under GPL-3.0

This software uses FFmpeg (https://ffmpeg.org/)
licensed under the GPL v2 or later.
FFmpeg source code: https://ffmpeg.org/download.html"#;

const LIBRARIES: &[(&str, &str)] = &[
    ("GPUI", "Apache-2.0"),
    ("gpui-component", "Apache-2.0"),
    ("ez-ffmpeg", "MIT/Apache-2.0/MPL-2.0"),
    ("smol", "Apache-2.0/MIT"),
    ("anyhow", "MIT/Apache-2.0"),
    ("serde", "MIT/Apache-2.0"),
    ("reqwest", "MIT/Apache-2.0"),
    ("rfd", "MIT"),
];
