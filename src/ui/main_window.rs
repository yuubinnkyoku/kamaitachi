//! メインウィンドウ

use gpui::*;
use gpui::prelude::FluentBuilder;
use gpui_component::button::{Button, ButtonVariant, ButtonVariants};
use gpui_component::Disableable;

use crate::app::AppState;
use super::{FileList, SettingsPanel, ProgressView, AboutDialog};

/// メインウィンドウ
pub struct MainWindow {
    /// アプリケーション状態
    app_state: AppState,
    /// ファイルリスト
    file_list: Entity<FileList>,
    /// 設定パネル
    settings_panel: Entity<SettingsPanel>,
    /// 進捗ビュー
    progress_view: Entity<ProgressView>,
    /// Aboutダイアログ表示フラグ
    show_about: bool,
}

impl MainWindow {
    pub fn new(app_state: AppState, cx: &mut Context<Self>) -> Self {
        let file_list = cx.new(|cx| FileList::new(app_state.clone(), cx));
        let settings_panel = cx.new(|cx| SettingsPanel::new(app_state.clone(), cx));
        let progress_view = cx.new(|cx| ProgressView::new(app_state.clone(), cx));

        // FFmpegを検出
        Self::detect_ffmpeg(&app_state, cx);

        Self {
            app_state,
            file_list,
            settings_panel,
            progress_view,
            show_about: false,
        }
    }

    /// FFmpegを検出
    fn detect_ffmpeg(app_state: &AppState, cx: &mut Context<Self>) {
        use crate::ffmpeg::{FfmpegDetector, FfmpegDownloader};
        use log::{info, warn};

        // 既存のFFmpegを検出
        match FfmpegDetector::detect() {
            Ok(info) => {
                if FfmpegDetector::check_version_requirement(&info, 7) {
                    info!("Found FFmpeg {} at {:?}", info.version, info.ffmpeg_path);
                    app_state.ffmpeg_path.update(cx, |path, _| {
                        *path = Some(info.ffmpeg_path);
                    });
                } else {
                    warn!(
                        "FFmpeg {} found but version 7.0+ required",
                        info.version
                    );
                    // TODO: ダウンロードを促すダイアログを表示
                }
            }
            Err(e) => {
                warn!("FFmpeg not found: {}", e);
                // ダウンロード済みをチェック
                if let Ok(Some(path)) = FfmpegDownloader::is_downloaded() {
                    info!("Found downloaded FFmpeg at {:?}", path);
                    app_state.ffmpeg_path.update(cx, |ffmpeg_path, _| {
                        *ffmpeg_path = Some(path);
                    });
                } else {
                    // TODO: ダウンロードダイアログを表示
                    warn!("No FFmpeg available, download required");
                }
            }
        }
    }

    /// ファイル追加ダイアログを開く
    fn open_file_dialog(&mut self, cx: &mut Context<Self>) {
        let app_state = self.app_state.clone();

        cx.spawn(async move |this, cx| {
            let files = rfd::AsyncFileDialog::new()
                .add_filter("Video files", &["mp4", "mkv", "avi", "mov", "webm", "flv", "wmv", "m4v", "ts"])
                .set_title("ファイルを選択")
                .pick_files()
                .await;

            if let Some(files) = files {
                let paths: Vec<_> = files.into_iter().map(|f| f.path().to_path_buf()).collect();
                cx.update(|cx| {
                    app_state.add_files(paths, cx);
                }).ok();
                this.update(cx, |_, cx| cx.notify()).ok();
            }
        })
        .detach();
    }

    /// 出力フォルダを選択
    fn select_output_folder(&mut self, cx: &mut Context<Self>) {
        let app_state = self.app_state.clone();

        cx.spawn(async move |this, cx| {
            let folder = rfd::AsyncFileDialog::new()
                .set_title("出力フォルダを選択")
                .pick_folder()
                .await;

            if let Some(folder) = folder {
                let path = folder.path().to_path_buf();
                cx.update(|cx| {
                    app_state.transcode_settings.update(cx, |settings, _| {
                        settings.output_dir = Some(path);
                    });
                }).ok();
                this.update(cx, |_, cx| cx.notify()).ok();
            }
        })
        .detach();
    }

    /// トランスコード開始
    fn start_transcode(&mut self, cx: &mut Context<Self>) {
        use crate::app::FileStatus;
        use crate::transcoder::{TranscodeJob, HwAccelDetector};
        use std::process::{Command, Stdio};
        use log::{info, error};

        // FFmpegパスを取得
        let ffmpeg_path = match self.app_state.ffmpeg_path.read(cx).clone() {
            Some(path) => path,
            None => {
                error!("FFmpeg not available");
                return;
            }
        };

        // ファイルがあるか確認
        let files = self.app_state.files.read(cx).clone();
        if files.is_empty() {
            info!("No files to transcode");
            return;
        }

        // 設定を取得
        let settings = self.app_state.transcode_settings.read(cx).clone();

        // 出力ディレクトリを決定（設定がなければ入力ファイルと同じディレクトリ）
        let output_dir = settings.output_dir.clone();
        let output_suffix = settings.output_suffix.clone();

        // HWアクセラレーションを解決
        let resolved_hwaccel = HwAccelDetector::resolve_auto(settings.hwaccel, Some(&ffmpeg_path));
        let mut resolved_settings = settings.clone();
        resolved_settings.hwaccel = resolved_hwaccel;

        let app_state = self.app_state.clone();

        info!("Starting transcode for {} files", files.len());

        // 非同期でトランスコード処理を実行
        cx.spawn(async move |this, cx| {
            for (index, file) in files.iter().enumerate() {
                // ファイルの状態を「処理中」に更新
                cx.update(|cx| {
                    app_state.files.update(cx, |files, _| {
                        if let Some(f) = files.get_mut(index) {
                            f.status = FileStatus::Processing;
                            f.progress = 0.0;
                        }
                    });
                }).ok();
                this.update(cx, |_, cx| cx.notify()).ok();

                // 出力パスを決定
                let out_dir = output_dir.clone().unwrap_or_else(|| {
                    file.path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| std::path::PathBuf::from("."))
                });
                let output_path = TranscodeJob::generate_output_path(
                    &file.path,
                    &out_dir,
                    &output_suffix,
                    &resolved_settings,
                );

                // ジョブを作成
                let job = TranscodeJob::new(
                    file.path.clone(),
                    output_path.clone(),
                    resolved_settings.clone(),
                );

                // 現在のジョブを設定
                cx.update(|cx| {
                    app_state.current_job.update(cx, |current, _| {
                        *current = Some(job.clone());
                    });
                }).ok();
                this.update(cx, |_, cx| cx.notify()).ok();

                // FFmpegコマンドを構築
                let args = job.build_ffmpeg_args();
                info!("Running FFmpeg: {:?} {:?}", ffmpeg_path, args);

                // FFmpegプロセスを別スレッドで実行（GPUIはTokioランタイムを使用しないため）
                let ffmpeg_path_clone = ffmpeg_path.clone();
                let result = smol::unblock(move || {
                    Command::new(&ffmpeg_path_clone)
                        .args(&args)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .output()
                }).await;

                match result {
                    Ok(output) => {
                        let final_status = if output.status.success() {
                            info!("Transcode completed: {:?}", output_path);
                            FileStatus::Completed
                        } else {
                            let error_msg = if !output.stderr.is_empty() {
                                String::from_utf8_lossy(&output.stderr).to_string()
                            } else {
                                format!("Exit code: {:?}", output.status.code())
                            };
                            error!("Transcode failed: {}", error_msg);
                            FileStatus::Error(error_msg)
                        };

                        // ファイルの状態を更新
                        cx.update(|cx| {
                            app_state.files.update(cx, |files, _| {
                                if let Some(f) = files.get_mut(index) {
                                    f.status = final_status;
                                    f.progress = 1.0;
                                }
                            });
                        }).ok();
                    }
                    Err(e) => {
                        error!("Failed to run FFmpeg: {}", e);
                        cx.update(|cx| {
                            app_state.files.update(cx, |files, _| {
                                if let Some(f) = files.get_mut(index) {
                                    f.status = FileStatus::Error(e.to_string());
                                }
                            });
                        }).ok();
                    }
                }

                this.update(cx, |_, cx| cx.notify()).ok();
            }

            // 完了後、現在のジョブをクリア
            cx.update(|cx| {
                app_state.current_job.update(cx, |current, _| {
                    *current = None;
                });
            }).ok();
            this.update(cx, |_, cx| cx.notify()).ok();

            info!("All transcoding completed");
        })
        .detach();
    }

    /// キューをクリア
    fn clear_queue(&mut self, cx: &mut Context<Self>) {
        self.app_state.files.update(cx, |files, _| files.clear());
        cx.notify();
    }

    /// Aboutダイアログを表示
    fn show_about(&mut self, cx: &mut Context<Self>) {
        self.show_about = true;
        cx.notify();
    }

    /// Aboutダイアログを閉じる
    fn hide_about(&mut self, cx: &mut Context<Self>) {
        self.show_about = false;
        cx.notify();
    }
}

impl Render for MainWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let has_files = self.app_state.files.read(cx).len() > 0;

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0x1e1e2e))
            .text_color(rgb(0xcdd6f4))
            // ツールバー
            .child(
                div()
                    .w_full()
                    .h(px(56.0))
                    .px(px(16.0))
                    .flex()
                    .items_center()
                    .justify_between()
                    .bg(rgb(0x181825))
                    .border_b_1()
                    .border_color(rgb(0x313244))
                    // 左側: ファイル操作ボタン
                    .child(
                        div()
                            .flex()
                            .gap(px(8.0))
                            .child(
                                Button::new("add-files")
                                    .label("ファイル追加")
                                    .with_variant(ButtonVariant::Primary)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.open_file_dialog(cx);
                                    }))
                            )
                            .child(
                                Button::new("clear-queue")
                                    .label("クリア")
                                    .with_variant(ButtonVariant::Ghost)
                                    .disabled(!has_files)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.clear_queue(cx);
                                    }))
                            )
                    )
                    // 中央: タイトル
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .text_xl()
                                    .font_weight(FontWeight::BOLD)
                                    .child("kamaitachi")
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x6c7086))
                                    .child("鎌鼬")
                            )
                    )
                    // 右側: 開始・About
                    .child(
                        div()
                            .flex()
                            .gap(px(8.0))
                            .child(
                                Button::new("start")
                                    .label("変換開始")
                                    .with_variant(ButtonVariant::Primary)
                                    .disabled(!has_files)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.start_transcode(cx);
                                    }))
                            )
                            .child(
                                Button::new("about")
                                    .label("About")
                                    .with_variant(ButtonVariant::Ghost)
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.show_about(cx);
                                    }))
                            )
                    )
            )
            // メインコンテンツ
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .flex()
                    .overflow_hidden()
                    // 左側: ファイルリスト
                    .child(
                        div()
                            .flex_1()
                            .h_full()
                            .flex()
                            .flex_col()
                            .border_r_1()
                            .border_color(rgb(0x313244))
                            .child(self.file_list.clone())
                    )
                    // 右側: 設定パネル
                    .child(
                        div()
                            .w(px(360.0))
                            .h_full()
                            .flex()
                            .flex_col()
                            .child(self.settings_panel.clone())
                    )
            )
            // ステータスバー / 進捗
            .child(
                div()
                    .w_full()
                    .border_t_1()
                    .border_color(rgb(0x313244))
                    .child(self.progress_view.clone())
            )
            // Aboutダイアログ（モーダル）
            .when(self.show_about, |this| {
                this.child(
                    div()
                        .absolute()
                        .inset_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(rgba(0x00000080))
                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _, cx| {
                            this.hide_about(cx);
                        }))
                        .child(
                            div()
                                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                    // ダイアログ内のクリックは伝播させない
                                    cx.stop_propagation();
                                })
                                .child(AboutDialog::render_content(cx.listener(|this, _, _, cx| {
                                    this.hide_about(cx);
                                })))
                        )
                )
            })
    }
}
