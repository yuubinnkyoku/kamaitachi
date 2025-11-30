//! エンコード設定パネル

use gpui::*;
use gpui_component::button::{Button, ButtonVariant};
use gpui_component::input::TextInput;
use gpui_component::slider::Slider;

use crate::app::AppState;
use crate::transcoder::{
    AudioCodec, ContainerFormat, HwAccelType, TranscodeSettings, VideoCodec, VideoPreset,
    VideoResolution,
};

/// 設定パネル
pub struct SettingsPanel {
    /// アプリケーション状態
    app_state: AppState,
}

impl SettingsPanel {
    pub fn new(app_state: AppState, _cx: &mut ViewContext<Self>) -> Self {
        Self { app_state }
    }

    /// ドロップダウンを選択肢をレンダリング
    fn render_select<T: Clone + PartialEq + 'static>(
        &self,
        label: &str,
        current: T,
        options: &[(T, &str)],
        on_change: impl Fn(T, &mut ViewContext<Self>) + 'static,
        cx: &mut ViewContext<Self>,
    ) -> impl IntoElement {
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x6c7086))
                    .child(label.to_string())
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_wrap()
                    .gap(px(4.0))
                    .children(
                        options.iter().map(|(value, name)| {
                            let is_selected = *value == current;
                            let value_clone = value.clone();
                            
                            div()
                                .px(px(8.0))
                                .py(px(4.0))
                                .rounded(px(4.0))
                                .text_xs()
                                .cursor_pointer()
                                .bg(if is_selected { rgb(0x89b4fa) } else { rgb(0x313244) })
                                .text_color(if is_selected { rgb(0x1e1e2e) } else { rgb(0xcdd6f4) })
                                .hover(|s| if is_selected { s } else { s.bg(rgb(0x45475a)) })
                                .on_click(cx.listener(move |this, _, cx| {
                                    on_change(value_clone.clone(), cx);
                                    cx.notify();
                                }))
                                .child(name.to_string())
                        })
                    )
            )
    }
}

impl Render for SettingsPanel {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let settings = self.app_state.transcode_settings.read(cx).clone();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0x181825))
            // ヘッダー
            .child(
                div()
                    .w_full()
                    .h(px(40.0))
                    .px(px(16.0))
                    .flex()
                    .items_center()
                    .border_b_1()
                    .border_color(rgb(0x313244))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .child("エンコード設定")
                    )
            )
            // 設定項目
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .p(px(16.0))
                    .overflow_y_scroll()
                    .flex()
                    .flex_col()
                    .gap(px(16.0))
                    // 出力形式
                    .child(self.render_select(
                        "コンテナ形式",
                        settings.container,
                        &[
                            (ContainerFormat::Mp4, "MP4"),
                            (ContainerFormat::Mkv, "MKV"),
                        ],
                        |value, cx| {
                            // TODO: 更新
                        },
                        cx,
                    ))
                    // ビデオコーデック
                    .child(self.render_select(
                        "ビデオコーデック",
                        settings.video_codec,
                        &[
                            (VideoCodec::H264, "H.264"),
                            (VideoCodec::H265, "H.265"),
                            (VideoCodec::Vp9, "VP9"),
                            (VideoCodec::Av1, "AV1"),
                        ],
                        |value, cx| {
                            // TODO: 更新
                        },
                        cx,
                    ))
                    // 解像度
                    .child(self.render_select(
                        "解像度",
                        settings.resolution,
                        &[
                            (VideoResolution::Original, "元の解像度"),
                            (VideoResolution::Uhd4K, "4K"),
                            (VideoResolution::Fhd1080, "1080p"),
                            (VideoResolution::Hd720, "720p"),
                            (VideoResolution::Sd480, "480p"),
                        ],
                        |value, cx| {
                            // TODO: 更新
                        },
                        cx,
                    ))
                    // 品質 (CRF)
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x6c7086))
                                    .child(format!("品質 (CRF: {})", settings.crf))
                            )
                            .child(
                                div()
                                    .w_full()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.0))
                                    .child(div().text_xs().child("高品質"))
                                    .child(
                                        div()
                                            .flex_1()
                                            .h(px(4.0))
                                            .rounded(px(2.0))
                                            .bg(rgb(0x313244))
                                            .child(
                                                div()
                                                    .h_full()
                                                    .rounded(px(2.0))
                                                    .bg(rgb(0x89b4fa))
                                                    .w(relative(settings.crf as f32 / 51.0))
                                            )
                                    )
                                    .child(div().text_xs().child("低品質"))
                            )
                    )
                    // プリセット
                    .child(self.render_select(
                        "プリセット",
                        settings.preset,
                        &[
                            (VideoPreset::Ultrafast, "最速"),
                            (VideoPreset::Fast, "高速"),
                            (VideoPreset::Medium, "標準"),
                            (VideoPreset::Slow, "高品質"),
                            (VideoPreset::Veryslow, "最高品質"),
                        ],
                        |value, cx| {
                            // TODO: 更新
                        },
                        cx,
                    ))
                    // HWアクセラレーション
                    .child(self.render_select(
                        "HWアクセラレーション",
                        settings.hwaccel,
                        &[
                            (HwAccelType::Auto, "自動"),
                            (HwAccelType::Nvenc, "NVIDIA"),
                            (HwAccelType::Qsv, "Intel"),
                            (HwAccelType::Amf, "AMD"),
                            (HwAccelType::Software, "ソフトウェア"),
                        ],
                        |value, cx| {
                            // TODO: 更新
                        },
                        cx,
                    ))
                    // セクション区切り
                    .child(
                        div()
                            .w_full()
                            .h(px(1.0))
                            .bg(rgb(0x313244))
                    )
                    // オーディオコーデック
                    .child(self.render_select(
                        "オーディオコーデック",
                        settings.audio_codec,
                        &[
                            (AudioCodec::Aac, "AAC"),
                            (AudioCodec::Mp3, "MP3"),
                            (AudioCodec::Flac, "FLAC"),
                            (AudioCodec::Copy, "コピー"),
                        ],
                        |value, cx| {
                            // TODO: 更新
                        },
                        cx,
                    ))
                    // オーディオビットレート
                    .when(settings.audio_codec != AudioCodec::Copy && settings.audio_codec != AudioCodec::Flac, |this| {
                        this.child(self.render_select(
                            "オーディオビットレート",
                            settings.audio_bitrate,
                            &[
                                (128, "128 kbps"),
                                (192, "192 kbps"),
                                (256, "256 kbps"),
                                (320, "320 kbps"),
                            ],
                            |value, cx| {
                                // TODO: 更新
                            },
                            cx,
                        ))
                    })
                    // セクション区切り
                    .child(
                        div()
                            .w_full()
                            .h(px(1.0))
                            .bg(rgb(0x313244))
                    )
                    // 出力先
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x6c7086))
                                    .child("出力先フォルダ")
                            )
                            .child(
                                div()
                                    .w_full()
                                    .flex()
                                    .items_center()
                                    .gap(px(8.0))
                                    .child(
                                        div()
                                            .flex_1()
                                            .px(px(8.0))
                                            .py(px(6.0))
                                            .rounded(px(4.0))
                                            .bg(rgb(0x313244))
                                            .text_sm()
                                            .truncate()
                                            .child(
                                                settings.output_dir
                                                    .as_ref()
                                                    .map(|p| p.to_string_lossy().to_string())
                                                    .unwrap_or_else(|| "入力ファイルと同じ場所".to_string())
                                            )
                                    )
                                    .child(
                                        Button::new("select-output")
                                            .label("選択")
                                            .variant(ButtonVariant::Ghost)
                                    )
                            )
                    )
                    // 出力サフィックス
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x6c7086))
                                    .child("出力ファイル名サフィックス")
                            )
                            .child(
                                div()
                                    .w_full()
                                    .px(px(8.0))
                                    .py(px(6.0))
                                    .rounded(px(4.0))
                                    .bg(rgb(0x313244))
                                    .text_sm()
                                    .child(settings.output_suffix.clone())
                            )
                    )
            )
    }
}
