//! kamaitachi - 鎌鼬
//!
//! HandBrake代替の高速トランスコーダー
//!
//! # ライセンス
//! GPL-3.0 (GPLビルドのFFmpegを使用するため)

mod app;
mod config;
mod ffmpeg;
mod transcoder;
mod ui;

use anyhow::Result;
use gpui::*;
use gpui_component::Root;
use log::info;

fn main() -> Result<()> {
    // ロガー初期化
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    info!("kamaitachi v{} starting...", env!("CARGO_PKG_VERSION"));

    // GPUIアプリケーション起動
    Application::new().run(|cx: &mut App| {
        // gpui-componentの初期化（テーマなどのグローバル設定に必要）
        gpui_component::init(cx);

        // アプリケーション状態を初期化
        let app_state = app::AppState::new(cx);

        // メインウィンドウを開く
        cx.open_window(
            WindowOptions {
                titlebar: Some(TitlebarOptions {
                    title: Some("kamaitachi - 鎌鼬".into()),
                    appears_transparent: false,
                    ..Default::default()
                }),
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: Point::default(),
                    size: Size {
                        width: px(1200.0),
                        height: px(800.0),
                    },
                })),
                ..Default::default()
            },
            |window, cx| {
                // メインウィンドウビューを作成
                let main_view = cx.new(|cx| ui::MainWindow::new(app_state, cx));
                // gpui-componentではRootでラップする必要がある
                cx.new(|cx| Root::new(main_view, window, cx))
            },
        )
        .expect("Failed to open window");
    });

    Ok(())
}
