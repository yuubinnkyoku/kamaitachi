//! FFmpegパス設定のためのビルドスクリプト
//!
//! 環境変数 FFMPEG_DIR が設定されている場合、そのパスを使用します。
//! 設定されていない場合は、アプリケーションデータディレクトリを使用します。

use std::env;
use std::path::PathBuf;

fn main() {
    // 再コンパイルトリガー
    println!("cargo:rerun-if-env-changed=FFMPEG_DIR");

    // FFmpegパスの設定
    if let Ok(ffmpeg_dir) = env::var("FFMPEG_DIR") {
        println!("cargo:warning=Using FFMPEG_DIR: {}", ffmpeg_dir);

        // FFmpegのbinディレクトリをPATHに追加
        let bin_path = PathBuf::from(&ffmpeg_dir).join("bin");
        if bin_path.exists() {
            println!(
                "cargo:rustc-env=FFMPEG_BIN_PATH={}",
                bin_path.to_string_lossy()
            );
        } else {
            println!(
                "cargo:rustc-env=FFMPEG_BIN_PATH={}",
                ffmpeg_dir
            );
        }
    }

    // Windows用のリンク設定
    #[cfg(target_os = "windows")]
    {
        // 静的リンク用の設定（ez-ffmpegのstaticフィーチャー使用時）
        println!("cargo:rustc-link-search=native=C:/Windows/System32");
    }
}
