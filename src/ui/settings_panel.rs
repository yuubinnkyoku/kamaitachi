//! エンコード設定パネル

use gpui::prelude::*;
use gpui::{InteractiveElement, *};
use gpui_component::button::{Button, ButtonVariant, ButtonVariants};

use crate::app::AppState;
use crate::transcoder::{
    AmfQuality, AmfUsage, AqMode, AudioCodec, ContainerFormat, HwAccelType, NvencBRefMode,
    NvencMultipass, NvencTune, RateControlMode, TranscodeSettings, VideoCodec, VideoPreset,
    VideoResolution, X264Profile, X264Tune,
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
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |_this, _, _, cx| {
                                    app_state_clone
                                        .transcode_settings
                                        .update(cx, |settings, _| {
                                            settings.container = value_clone;
                                        });
                                    // 予測サイズを更新
                                    Self::update_estimated_sizes(&app_state_clone, cx);
                                    cx.notify();
                                }),
                            )
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
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |_this, _, _, cx| {
                                    app_state_clone
                                        .transcode_settings
                                        .update(cx, |settings, _| {
                                            settings.video_codec = value_clone;
                                        });
                                    // 予測サイズを更新
                                    Self::update_estimated_sizes(&app_state_clone, cx);
                                    cx.notify();
                                }),
                            )
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
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |_this, _, _, cx| {
                                    app_state_clone
                                        .transcode_settings
                                        .update(cx, |settings, _| {
                                            settings.resolution = value_clone;
                                        });
                                    // 予測サイズを更新
                                    Self::update_estimated_sizes(&app_state_clone, cx);
                                    cx.notify();
                                }),
                            )
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
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |_this, _, _, cx| {
                                    app_state_clone
                                        .transcode_settings
                                        .update(cx, |settings, _| {
                                            settings.preset = value_clone;
                                        });
                                    // 予測サイズを更新
                                    Self::update_estimated_sizes(&app_state_clone, cx);
                                    cx.notify();
                                }),
                            )
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
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |_this, _, _, cx| {
                                    app_state_clone
                                        .transcode_settings
                                        .update(cx, |settings, _| {
                                            settings.hwaccel = value_clone;
                                        });
                                    // 予測サイズを更新
                                    Self::update_estimated_sizes(&app_state_clone, cx);
                                    cx.notify();
                                }),
                            )
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
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |_this, _, _, cx| {
                                    app_state_clone
                                        .transcode_settings
                                        .update(cx, |settings, _| {
                                            settings.audio_codec = value_clone;
                                        });
                                    // 予測サイズを更新
                                    Self::update_estimated_sizes(&app_state_clone, cx);
                                    cx.notify();
                                }),
                            )
                            .child(name.to_string())
                    })),
            )
    }

    /// CRF選択ボタンをレンダリング
    fn render_crf_select(&self, current: u8, cx: &mut Context<Self>) -> impl IntoElement {
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
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |_this, _, _, cx| {
                                    app_state_clone
                                        .transcode_settings
                                        .update(cx, |settings, _| {
                                            settings.crf = value_clone;
                                        });
                                    // 予測サイズを更新
                                    Self::update_estimated_sizes(&app_state_clone, cx);
                                    cx.notify();
                                }),
                            )
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
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |_this, _, _, cx| {
                                    app_state_clone
                                        .transcode_settings
                                        .update(cx, |settings, _| {
                                            settings.audio_bitrate = value_clone;
                                        });
                                    // 予測サイズを更新
                                    Self::update_estimated_sizes(&app_state_clone, cx);
                                    cx.notify();
                                }),
                            )
                            .child(name.to_string())
                    })),
            )
    }

    /// レートコントロールモードボタンをレンダリング
    fn render_rate_control_select(
        &self,
        current: RateControlMode,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let app_state = self.app_state.clone();
        let options = [
            (RateControlMode::Crf, "CRF (固定品質)"),
            (RateControlMode::Cbr, "CBR (固定レート)"),
            (RateControlMode::Vbr, "VBR (可変レート)"),
            (RateControlMode::Cqp, "CQP (固定QP)"),
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
                    .child("レートコントロールモード"),
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
                            .id(SharedString::from(format!("rate-control-{}", name)))
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
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |_this, _, _, cx| {
                                    app_state_clone
                                        .transcode_settings
                                        .update(cx, |settings, _| {
                                            settings.rate_control = value_clone;
                                        });
                                    // 予測サイズを更新
                                    Self::update_estimated_sizes(&app_state_clone, cx);
                                    cx.notify();
                                }),
                            )
                            .child(name.to_string())
                    })),
            )
    }

    /// ビットレート選択ボタンをレンダリング
    fn render_bitrate_select(
        &self,
        current: u32,
        label: &str,
        id_prefix: &str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let app_state = self.app_state.clone();
        let options = [
            (1000, "1 Mbps"),
            (2500, "2.5 Mbps"),
            (5000, "5 Mbps"),
            (8000, "8 Mbps"),
            (12000, "12 Mbps"),
            (20000, "20 Mbps"),
        ];

        let id_prefix_string = id_prefix.to_string();

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(4.0))
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x6c7086))
                    .child(format!("{} ({} kbps)", label, current)),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_wrap()
                    .gap(px(4.0))
                    .children(options.iter().map(move |(value, name)| {
                        let is_selected = *value == current;
                        let value_clone = *value;
                        let app_state_clone = app_state.clone();
                        let id_prefix_clone = id_prefix_string.clone();

                        div()
                            .id(SharedString::from(format!("{}-{}", id_prefix_clone, value)))
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
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |_this, _, _, cx| {
                                    app_state_clone
                                        .transcode_settings
                                        .update(cx, |settings, _| {
                                            if id_prefix_clone == "target-bitrate" {
                                                settings.target_bitrate = value_clone;
                                            } else if id_prefix_clone == "max-bitrate" {
                                                settings.max_bitrate = value_clone;
                                            }
                                        });
                                    // 予測サイズを更新
                                    Self::update_estimated_sizes(&app_state_clone, cx);
                                    cx.notify();
                                }),
                            )
                            .child(name.to_string())
                    })),
            )
    }

    /// 最大ビットレート選択ボタンをレンダリング
    fn render_max_bitrate_select(&self, current: u32, cx: &mut Context<Self>) -> impl IntoElement {
        self.render_bitrate_select(current, "最大ビットレート", "max-bitrate", cx)
    }

    /// Bフレーム数選択ボタンをレンダリング
    fn render_bframes_select(&self, current: u8, cx: &mut Context<Self>) -> impl IntoElement {
        let app_state = self.app_state.clone();
        let options = [
            (0u8, "0 (なし)"),
            (1u8, "1"),
            (2u8, "2"),
            (3u8, "3 (標準)"),
            (4u8, "4"),
            (5u8, "5"),
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
                    .child(format!("Bフレーム数: {}", current)),
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

                        gpui::div()
                            .id(SharedString::from(format!("bframes-{}", value)))
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
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |_this, _, _, cx| {
                                    app_state_clone
                                        .transcode_settings
                                        .update(cx, |settings, _| {
                                            settings.bframes = value_clone;
                                        });
                                    // 予測サイズを更新
                                    Self::update_estimated_sizes(&app_state_clone, cx);
                                    cx.notify();
                                }),
                            )
                            .child(name.to_string())
                    })),
            )
    }

    /// 参照フレーム数選択ボタンをレンダリング
    fn render_ref_frames_select(&self, current: u8, cx: &mut Context<Self>) -> impl IntoElement {
        let app_state = self.app_state.clone();
        let options = [
            (1u8, "1"),
            (2u8, "2"),
            (3u8, "3"),
            (4u8, "4 (標準)"),
            (5u8, "5"),
            (6u8, "6"),
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
                    .child(format!("参照フレーム数: {}", current)),
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
                            .id(SharedString::from(format!("ref-frames-{}", value)))
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
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |_this, _, _, cx| {
                                    app_state_clone
                                        .transcode_settings
                                        .update(cx, |settings, _| {
                                            settings.ref_frames = value_clone;
                                        });
                                    // 予測サイズを更新
                                    Self::update_estimated_sizes(&app_state_clone, cx);
                                    cx.notify();
                                }),
                            )
                            .child(name.to_string())
                    })),
            )
    }

    /// GOPサイズ選択ボタンをレンダリング
    fn render_gop_select(&self, current: u32, cx: &mut Context<Self>) -> impl IntoElement {
        let app_state = self.app_state.clone();
        let options = [
            (30u32, "30 (1秒)"),
            (60u32, "60 (2秒)"),
            (120u32, "120 (4秒)"),
            (250u32, "250 (標準)"),
            (300u32, "300 (10秒)"),
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
                    .child(format!("GOPサイズ (キーフレーム間隔): {}", current)),
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
                            .id(SharedString::from(format!("gop-{}", value)))
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
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |_this, _, _, cx| {
                                    app_state_clone
                                        .transcode_settings
                                        .update(cx, |settings, _| {
                                            settings.gop_size = value_clone;
                                        });
                                    // 予測サイズを更新
                                    Self::update_estimated_sizes(&app_state_clone, cx);
                                    cx.notify();
                                }),
                            )
                            .child(name.to_string())
                    })),
            )
    }

    /// ルックアヘッド選択ボタンをレンダリング
    fn render_lookahead_select(&self, current: u8, cx: &mut Context<Self>) -> impl IntoElement {
        let app_state = self.app_state.clone();
        let options = [
            (0u8, "0 (なし)"),
            (10u8, "10"),
            (20u8, "20 (標準)"),
            (30u8, "30"),
            (40u8, "40"),
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
                    .child(format!("ルックアヘッド (先行読み込み): {}", current)),
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
                            .id(SharedString::from(format!("lookahead-{}", value)))
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
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |_this, _, _, cx| {
                                    app_state_clone
                                        .transcode_settings
                                        .update(cx, |settings, _| {
                                            settings.lookahead = value_clone;
                                        });
                                    // 予測サイズを更新
                                    Self::update_estimated_sizes(&app_state_clone, cx);
                                    cx.notify();
                                }),
                            )
                            .child(name.to_string())
                    })),
            )
    }

    /// NVENC設定をレンダリング
    fn render_nvenc_settings(
        &self,
        settings: &TranscodeSettings,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let app_state = self.app_state.clone();

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(0xa6adc8))
                    .child("NVENC設定"),
            )
            // プリセット (Tune)
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
                            .child("チューニング"),
                    )
                    .child(div().w_full().flex().flex_wrap().gap(px(4.0)).children(
                        NvencTune::all().iter().map(|value| {
                            let is_selected = *value == settings.nvenc_tune;
                            let value_clone = *value;
                            let app_state_clone = app_state.clone();

                            div()
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
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |_this, _, _, cx| {
                                        app_state_clone.transcode_settings.update(cx, |s, _| {
                                            s.nvenc_tune = value_clone;
                                        });
                                        cx.notify();
                                    }),
                                )
                                .child(value.display_name())
                        }),
                    )),
            )
            // マルチパス
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
                            .child("マルチパス"),
                    )
                    .child(div().w_full().flex().flex_wrap().gap(px(4.0)).children(
                        NvencMultipass::all().iter().map(|value| {
                            let is_selected = *value == settings.nvenc_multipass;
                            let value_clone = *value;
                            let app_state_clone = app_state.clone();

                            div()
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
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |_this, _, _, cx| {
                                        app_state_clone.transcode_settings.update(cx, |s, _| {
                                            s.nvenc_multipass = value_clone;
                                        });
                                        cx.notify();
                                    }),
                                )
                                .child(value.display_name())
                        }),
                    )),
            )
    }

    /// QSV設定をレンダリング
    fn render_qsv_settings(
        &self,
        _settings: &TranscodeSettings,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        // QSV設定の実装（現在はプレースホルダー）
        div().w_full().child(
            div()
                .text_xs()
                .text_color(rgb(0x6c7086))
                .child("QSV設定 (標準設定を使用)"),
        )
    }

    /// AMF設定をレンダリング
    fn render_amf_settings(
        &self,
        settings: &TranscodeSettings,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let app_state = self.app_state.clone();

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(0xa6adc8))
                    .child("AMF設定"),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(div().text_xs().text_color(rgb(0x6c7086)).child("使用法"))
                    .child(div().w_full().flex().flex_wrap().gap(px(4.0)).children(
                        AmfUsage::all().iter().map(|value| {
                            let is_selected = *value == settings.amf_usage;
                            let value_clone = *value;
                            let app_state_clone = app_state.clone();

                            div()
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
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |_this, _, _, cx| {
                                        app_state_clone.transcode_settings.update(cx, |s, _| {
                                            s.amf_usage = value_clone;
                                        });
                                        cx.notify();
                                    }),
                                )
                                .child(value.display_name())
                        }),
                    )),
            )
    }

    /// ソフトウェアエンコード設定をレンダリング
    fn render_software_settings(
        &self,
        settings: &TranscodeSettings,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let app_state = self.app_state.clone();

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(rgb(0xa6adc8))
                    .child("ソフトウェアエンコード設定"),
            )
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
                            .child("チューニング"),
                    )
                    .child(div().w_full().flex().flex_wrap().gap(px(4.0)).children(
                        X264Tune::all().iter().map(|value| {
                            let is_selected = *value == settings.x264_tune;
                            let value_clone = *value;
                            let app_state_clone = app_state.clone();

                            div()
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
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |_this, _, _, cx| {
                                        app_state_clone.transcode_settings.update(cx, |s, _| {
                                            s.x264_tune = value_clone;
                                        });
                                        cx.notify();
                                    }),
                                )
                                .child(value.display_name())
                        }),
                    )),
            )
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
                            .child("プロファイル"),
                    )
                    .child(div().w_full().flex().flex_wrap().gap(px(4.0)).children(
                        X264Profile::all().iter().map(|value| {
                            let is_selected = *value == settings.x264_profile;
                            let value_clone = *value;
                            let app_state_clone = app_state.clone();

                            div()
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
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |_this, _, _, cx| {
                                        app_state_clone.transcode_settings.update(cx, |s, _| {
                                            s.x264_profile = value_clone;
                                        });
                                        cx.notify();
                                    }),
                                )
                                .child(value.display_name())
                        }),
                    )),
            )
    }

    /// VP9設定をレンダリング
    fn render_vp9_settings(
        &self,
        _settings: &TranscodeSettings,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div().w_full().child(
            div()
                .text_xs()
                .text_color(rgb(0x6c7086))
                .child("VP9設定 (標準設定を使用)"),
        )
    }

    /// AV1設定をレンダリング
    fn render_av1_settings(
        &self,
        _settings: &TranscodeSettings,
        _cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div().w_full().child(
            div()
                .text_xs()
                .text_color(rgb(0x6c7086))
                .child("AV1設定 (標準設定を使用)"),
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
                    .id("settings-content")
                    .flex_1()
                    .w_full()
                    .p(px(16.0))
                    .overflow_y_scroll()
                    .flex()
                    .flex_col()
                    .gap(px(16.0))
                    // 出力形式
                    .child(self.render_container_select(settings.container, cx))
                    // ビデオコーデック
                    .child(self.render_video_codec_select(settings.video_codec, cx))
                    // 解像度
                    .child(self.render_resolution_select(settings.resolution, cx))
                    // HWアクセラレーション
                    .child(self.render_hwaccel_select(settings.hwaccel, cx))
                    // セクション区切り
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(0xa6adc8))
                                    .child("レートコントロール"),
                            )
                            .child(div().w_full().h(px(1.0)).bg(rgb(0x313244))),
                    )
                    // レートコントロールモード
                    .child(self.render_rate_control_select(settings.rate_control, cx))
                    // 品質 (CRF/QP) - CRFまたはCQPモードの時のみ
                    .when(
                        settings.rate_control == RateControlMode::Crf
                            || settings.rate_control == RateControlMode::Cqp,
                        |this| this.child(self.render_crf_select(settings.crf, cx)),
                    )
                    // ターゲットビットレート - CBR/VBRモードの時
                    .when(
                        settings.rate_control == RateControlMode::Cbr
                            || settings.rate_control == RateControlMode::Vbr,
                        |this| {
                            this.child(self.render_bitrate_select(
                                settings.target_bitrate,
                                "ターゲットビットレート",
                                "target-bitrate",
                                cx,
                            ))
                        },
                    )
                    // 最大ビットレート - VBRモードの時のみ
                    .when(settings.rate_control == RateControlMode::Vbr, |this| {
                        this.child(self.render_max_bitrate_select(settings.max_bitrate, cx))
                    })
                    // プリセット
                    .child(self.render_preset_select(settings.preset, cx))
                    // セクション区切り - エンコーダー詳細設定
                    .child(
                        div()
                            .w_full()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(rgb(0xa6adc8))
                                    .child("エンコーダー詳細設定"),
                            )
                            .child(div().w_full().h(px(1.0)).bg(rgb(0x313244))),
                    )
                    // 共通フレーム設定
                    .child(self.render_bframes_select(settings.bframes, cx))
                    .child(self.render_ref_frames_select(settings.ref_frames, cx))
                    .child(self.render_gop_select(settings.gop_size, cx))
                    .child(self.render_lookahead_select(settings.lookahead, cx))
                    // エンコーダー固有設定
                    .when(
                        settings.hwaccel == HwAccelType::Nvenc
                            || settings.hwaccel == HwAccelType::Auto,
                        |this| this.child(self.render_nvenc_settings(&settings, cx)),
                    )
                    .when(settings.hwaccel == HwAccelType::Qsv, |this| {
                        this.child(self.render_qsv_settings(&settings, cx))
                    })
                    .when(settings.hwaccel == HwAccelType::Amf, |this| {
                        this.child(self.render_amf_settings(&settings, cx))
                    })
                    .when(settings.hwaccel == HwAccelType::Software, |this| {
                        this.child(self.render_software_settings(&settings, cx))
                    })
                    // VP9固有設定
                    .when(settings.video_codec == VideoCodec::Vp9, |this| {
                        this.child(self.render_vp9_settings(&settings, cx))
                    })
                    // AV1固有設定
                    .when(settings.video_codec == VideoCodec::Av1, |this| {
                        this.child(self.render_av1_settings(&settings, cx))
                    })
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
