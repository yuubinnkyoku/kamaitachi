//! エンコード設定パネル

use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonVariant, ButtonVariants};

use crate::app::AppState;
use crate::transcoder::{
    AudioCodec, ContainerFormat, HwAccelType, VideoCodec, VideoPreset, VideoResolution,
};

/// 設定パネル
pub struct SettingsPanel {
    /// アプリケーション状態
    app_state: AppState,
}

impl SettingsPanel {
    pub fn new(app_state: AppState, _cx: &mut Context<Self>) -> Self {
        Self { app_state }
    }

    /// すべてのファイルの予測サイズを更新
    fn update_estimated_sizes(app_state: &AppState, cx: &mut Context<Self>) {
        let settings = app_state.transcode_settings.read(cx).clone();
        app_state.files.update(cx, |files, _| {
            for file in files.iter_mut() {
                file.update_estimated_size(&settings);
            }
        });
    }

    /// コンテナ形式ボタンをレンダリング
    fn render_container_select(
        &self,
        current: ContainerFormat,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let app_state = self.app_state.clone();
        let options = [(ContainerFormat::Mp4, "MP4"), (ContainerFormat::Mkv, "MKV")];

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x6c7086))
                    .child("コンテナ形式"),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_wrap()
                    .gap(px(4.0))
                    .children(options.iter().map(|(value, name)| {
                        let is_selected = *value == current;
                        let value_clone = *value;
                        let app_state_clone = app_state.clone();

                        div()
                            .id(SharedString::from(format!("container-{}", name)))
                            .px(px(8.0))
                            .py(px(4.0))
                            .rounded(px(4.0))
                            .text_xs()
                            .cursor_pointer()
                            .bg(if is_selected {
                                rgb(0x89b4fa)
                            } else {
                                rgb(0x313244)
                            })
                            .text_color(if is_selected {
                                rgb(0x1e1e2e)
                            } else {
                                rgb(0xcdd6f4)
                            })
                            .hover(|s| if is_selected { s } else { s.bg(rgb(0x45475a)) })
                            .on_click(cx.listener(move |_this, _, _, cx| {
                                app_state_clone
                                    .transcode_settings
                                    .update(cx, |settings, _| {
                                        settings.container = value_clone;
                                    });
                                cx.notify();
                            }))
                            .child(name.to_string())
                    })),
            )
    }

    /// ビデオコーデックボタンをレンダリング
    fn render_video_codec_select(
        &self,
        current: VideoCodec,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let app_state = self.app_state.clone();
        let options = [
            (VideoCodec::H264, "H.264"),
            (VideoCodec::H265, "H.265"),
            (VideoCodec::Vp9, "VP9"),
            (VideoCodec::Av1, "AV1"),
        ];

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x6c7086))
                    .child("ビデオコーデック"),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_wrap()
                    .gap(px(4.0))
                    .children(options.iter().map(|(value, name)| {
                        let is_selected = *value == current;
                        let value_clone = *value;
                        let app_state_clone = app_state.clone();

                        div()
                            .id(SharedString::from(format!("video-codec-{}", name)))
                            .px(px(8.0))
                            .py(px(4.0))
                            .rounded(px(4.0))
                            .text_xs()
                            .cursor_pointer()
                            .bg(if is_selected {
                                rgb(0x89b4fa)
                            } else {
                                rgb(0x313244)
                            })
                            .text_color(if is_selected {
                                rgb(0x1e1e2e)
                            } else {
                                rgb(0xcdd6f4)
                            })
                            .hover(|s| if is_selected { s } else { s.bg(rgb(0x45475a)) })
                            .on_click(cx.listener(move |_this, _, _, cx| {
                                app_state_clone
                                    .transcode_settings
                                    .update(cx, |settings, _| {
                                        settings.video_codec = value_clone;
                                    });
                                cx.notify();
                            }))
                            .child(name.to_string())
                    })),
            )
    }

    /// 解像度ボタンをレンダリング
    fn render_resolution_select(
        &self,
        current: VideoResolution,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let app_state = self.app_state.clone();
        let options = [
            (VideoResolution::Original, "元の解像度"),
            (VideoResolution::Uhd4K, "4K"),
            (VideoResolution::Fhd1080, "1080p"),
            (VideoResolution::Hd720, "720p"),
            (VideoResolution::Sd480, "480p"),
        ];

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .child(div().text_xs().text_color(rgb(0x6c7086)).child("解像度"))
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_wrap()
                    .gap(px(4.0))
                    .children(options.iter().map(|(value, name)| {
                        let is_selected = *value == current;
                        let value_clone = *value;
                        let app_state_clone = app_state.clone();

                        div()
                            .id(SharedString::from(format!("resolution-{}", name)))
                            .px(px(8.0))
                            .py(px(4.0))
                            .rounded(px(4.0))
                            .text_xs()
                            .cursor_pointer()
                            .bg(if is_selected {
                                rgb(0x89b4fa)
                            } else {
                                rgb(0x313244)
                            })
                            .text_color(if is_selected {
                                rgb(0x1e1e2e)
                            } else {
                                rgb(0xcdd6f4)
                            })
                            .hover(|s| if is_selected { s } else { s.bg(rgb(0x45475a)) })
                            .on_click(cx.listener(move |_this, _, _, cx| {
                                app_state_clone
                                    .transcode_settings
                                    .update(cx, |settings, _| {
                                        settings.resolution = value_clone;
                                    });
                                // 予測サイズを更新
                                Self::update_estimated_sizes(&app_state_clone, cx);
                                cx.notify();
                            }))
                            .child(name.to_string())
                    })),
            )
    }

    /// プリセットボタンをレンダリング
    fn render_preset_select(
        &self,
        current: VideoPreset,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let app_state = self.app_state.clone();
        let options = [
            (VideoPreset::Ultrafast, "最速"),
            (VideoPreset::Fast, "高速"),
            (VideoPreset::Medium, "標準"),
            (VideoPreset::Slow, "高品質"),
            (VideoPreset::Veryslow, "最高品質"),
        ];

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x6c7086))
                    .child("プリセット"),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_wrap()
                    .gap(px(4.0))
                    .children(options.iter().map(|(value, name)| {
                        let is_selected = *value == current;
                        let value_clone = *value;
                        let app_state_clone = app_state.clone();

                        div()
                            .id(SharedString::from(format!("preset-{}", name)))
                            .px(px(8.0))
                            .py(px(4.0))
                            .rounded(px(4.0))
                            .text_xs()
                            .cursor_pointer()
                            .bg(if is_selected {
                                rgb(0x89b4fa)
                            } else {
                                rgb(0x313244)
                            })
                            .text_color(if is_selected {
                                rgb(0x1e1e2e)
                            } else {
                                rgb(0xcdd6f4)
                            })
                            .hover(|s| if is_selected { s } else { s.bg(rgb(0x45475a)) })
                            .on_click(cx.listener(move |_this, _, _, cx| {
                                app_state_clone
                                    .transcode_settings
                                    .update(cx, |settings, _| {
                                        settings.preset = value_clone;
                                    });
                                cx.notify();
                            }))
                            .child(name.to_string())
                    })),
            )
    }

    /// HWアクセラレーションボタンをレンダリング
    fn render_hwaccel_select(
        &self,
        current: HwAccelType,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let app_state = self.app_state.clone();
        let options = [
            (HwAccelType::Auto, "自動"),
            (HwAccelType::Nvenc, "NVIDIA"),
            (HwAccelType::Qsv, "Intel"),
            (HwAccelType::Amf, "AMD"),
            (HwAccelType::Software, "ソフトウェア"),
        ];

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x6c7086))
                    .child("HWアクセラレーション"),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_wrap()
                    .gap(px(4.0))
                    .children(options.iter().map(|(value, name)| {
                        let is_selected = *value == current;
                        let value_clone = *value;
                        let app_state_clone = app_state.clone();

                        div()
                            .id(SharedString::from(format!("hwaccel-{}", name)))
                            .px(px(8.0))
                            .py(px(4.0))
                            .rounded(px(4.0))
                            .text_xs()
                            .cursor_pointer()
                            .bg(if is_selected {
                                rgb(0x89b4fa)
                            } else {
                                rgb(0x313244)
                            })
                            .text_color(if is_selected {
                                rgb(0x1e1e2e)
                            } else {
                                rgb(0xcdd6f4)
                            })
                            .hover(|s| if is_selected { s } else { s.bg(rgb(0x45475a)) })
                            .on_click(cx.listener(move |_this, _, _, cx| {
                                app_state_clone
                                    .transcode_settings
                                    .update(cx, |settings, _| {
                                        settings.hwaccel = value_clone;
                                    });
                                cx.notify();
                            }))
                            .child(name.to_string())
                    })),
            )
    }

    /// オーディオコーデックボタンをレンダリング
    fn render_audio_codec_select(
        &self,
        current: AudioCodec,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let app_state = self.app_state.clone();
        let options = [
            (AudioCodec::Aac, "AAC"),
            (AudioCodec::Mp3, "MP3"),
            (AudioCodec::Flac, "FLAC"),
            (AudioCodec::Copy, "コピー"),
        ];

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x6c7086))
                    .child("オーディオコーデック"),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_wrap()
                    .gap(px(4.0))
                    .children(options.iter().map(|(value, name)| {
                        let is_selected = *value == current;
                        let value_clone = *value;
                        let app_state_clone = app_state.clone();

                        div()
                            .id(SharedString::from(format!("audio-codec-{}", name)))
                            .px(px(8.0))
                            .py(px(4.0))
                            .rounded(px(4.0))
                            .text_xs()
                            .cursor_pointer()
                            .bg(if is_selected {
                                rgb(0x89b4fa)
                            } else {
                                rgb(0x313244)
                            })
                            .text_color(if is_selected {
                                rgb(0x1e1e2e)
                            } else {
                                rgb(0xcdd6f4)
                            })
                            .hover(|s| if is_selected { s } else { s.bg(rgb(0x45475a)) })
                            .on_click(cx.listener(move |_this, _, _, cx| {
                                app_state_clone
                                    .transcode_settings
                                    .update(cx, |settings, _| {
                                        settings.audio_codec = value_clone;
                                    });
                                cx.notify();
                            }))
                            .child(name.to_string())
                    })),
            )
    }

    /// CRF選択ボタンをレンダリング
    fn render_crf_select(
        &self,
        current: u8,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let app_state = self.app_state.clone();
        // CRFの選択肢（数値が低いほど高品質）
        let options = [
            (18u8, "最高"),
            (20u8, "高"),
            (23u8, "標準"),
            (26u8, "中"),
            (28u8, "低"),
            (32u8, "最低"),
        ];

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x6c7086))
                    .child(format!("品質 (CRF: {})", current)),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_wrap()
                    .gap(px(4.0))
                    .children(options.iter().map(|(value, name)| {
                        let is_selected = *value == current;
                        let value_clone = *value;
                        let app_state_clone = app_state.clone();

                        div()
                            .id(SharedString::from(format!("crf-{}", value)))
                            .px(px(8.0))
                            .py(px(4.0))
                            .rounded(px(4.0))
                            .text_xs()
                            .cursor_pointer()
                            .bg(if is_selected {
                                rgb(0x89b4fa)
                            } else {
                                rgb(0x313244)
                            })
                            .text_color(if is_selected {
                                rgb(0x1e1e2e)
                            } else {
                                rgb(0xcdd6f4)
                            })
                            .hover(|s| if is_selected { s } else { s.bg(rgb(0x45475a)) })
                            .on_click(cx.listener(move |_this, _, _, cx| {
                                app_state_clone
                                    .transcode_settings
                                    .update(cx, |settings, _| {
                                        settings.crf = value_clone;
                                    });
                                // 予測サイズを更新
                                Self::update_estimated_sizes(&app_state_clone, cx);
                                cx.notify();
                            }))
                            .child(name.to_string())
                    })),
            )
    }

    /// オーディオビットレートボタンをレンダリング
    fn render_audio_bitrate_select(
        &self,
        current: u32,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let app_state = self.app_state.clone();
        let options = [
            (128u32, "128 kbps"),
            (192u32, "192 kbps"),
            (256u32, "256 kbps"),
            (320u32, "320 kbps"),
        ];

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x6c7086))
                    .child("オーディオビットレート"),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_wrap()
                    .gap(px(4.0))
                    .children(options.iter().map(|(value, name)| {
                        let is_selected = *value == current;
                        let value_clone = *value;
                        let app_state_clone = app_state.clone();

                        div()
                            .id(SharedString::from(format!("audio-bitrate-{}", value)))
                            .px(px(8.0))
                            .py(px(4.0))
                            .rounded(px(4.0))
                            .text_xs()
                            .cursor_pointer()
                            .bg(if is_selected {
                                rgb(0x89b4fa)
                            } else {
                                rgb(0x313244)
                            })
                            .text_color(if is_selected {
                                rgb(0x1e1e2e)
                            } else {
                                rgb(0xcdd6f4)
                            })
                            .hover(|s| if is_selected { s } else { s.bg(rgb(0x45475a)) })
                            .on_click(cx.listener(move |_this, _, _, cx| {
                                app_state_clone
                                    .transcode_settings
                                    .update(cx, |settings, _| {
                                        settings.audio_bitrate = value_clone;
                                    });
                                cx.notify();
                            }))
                            .child(name.to_string())
                    })),
            )
    }
}

impl Render for SettingsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
                            .child("エンコード設定"),
                    ),
            )
            // 設定項目
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .p(px(16.0))
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .gap(px(16.0))
                    // 出力形式
                    .child(self.render_container_select(settings.container, cx))
                    // ビデオコーデック
                    .child(self.render_video_codec_select(settings.video_codec, cx))
                    // 解像度
                    .child(self.render_resolution_select(settings.resolution, cx))
                    // 品質 (CRF)
                    .child(self.render_crf_select(settings.crf, cx))
                    // プリセット
                    .child(self.render_preset_select(settings.preset, cx))
                    // HWアクセラレーション
                    .child(self.render_hwaccel_select(settings.hwaccel, cx))
                    // セクション区切り
                    .child(div().w_full().h(px(1.0)).bg(rgb(0x313244)))
                    // オーディオコーデック
                    .child(self.render_audio_codec_select(settings.audio_codec, cx))
                    // オーディオビットレート
                    .when(
                        settings.audio_codec != AudioCodec::Copy
                            && settings.audio_codec != AudioCodec::Flac,
                        |this| {
                            this.child(self.render_audio_bitrate_select(settings.audio_bitrate, cx))
                        },
                    )
                    // セクション区切り
                    .child(div().w_full().h(px(1.0)).bg(rgb(0x313244)))
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
                                    .child("出力先フォルダ"),
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
                                                settings
                                                    .output_dir
                                                    .as_ref()
                                                    .map(|p| p.to_string_lossy().to_string())
                                                    .unwrap_or_else(|| {
                                                        "入力ファイルと同じ場所".to_string()
                                                    }),
                                            ),
                                    )
                                    .child(
                                        Button::new("select-output")
                                            .label("選択")
                                            .with_variant(ButtonVariant::Ghost),
                                    ),
                            ),
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
                                    .child("出力ファイル名サフィックス"),
                            )
                            .child(
                                div()
                                    .w_full()
                                    .px(px(8.0))
                                    .py(px(6.0))
                                    .rounded(px(4.0))
                                    .bg(rgb(0x313244))
                                    .text_sm()
                                    .child(settings.output_suffix.clone()),
                            ),
                    ),
            )
    }
}
