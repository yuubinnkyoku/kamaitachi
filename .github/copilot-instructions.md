# Copilot Instructions for kamaitachi

## Project Overview

**kamaitachi** (鎌鼬) is a high-speed video transcoder application built as a HandBrake alternative. It features:
- Hardware-accelerated encoding (NVENC, QSV, AMF)
- Modern UI using GPUI framework
- FFmpeg-based transcoding backend

## Technology Stack

- **Language**: Rust (Edition 2021)
- **UI Framework**: GPUI + gpui-component
- **Video Processing**: FFmpeg (external binary)
- **Async Runtime**: smol (GPUI-compatible)
- **License**: GPL-3.0

## Project Structure

```
src/
├── main.rs          # Application entry point
├── app.rs           # Application state management
├── config/          # Configuration and settings
├── ffmpeg/          # FFmpeg detection and management
├── transcoder/      # Transcoding engine and job management
└── ui/              # UI components (GPUI-based)
```

## Code Style Guidelines

- **Comments**: Write code comments in Japanese (日本語でコメントを書く)
- **Documentation**: Module-level documentation should use `//!` style
- **Error Handling**: Use `anyhow::Result` for error propagation
- **Logging**: Use the `log` crate macros (`log::info!`, `log::debug!`, etc.)

## Building and Testing

```bash
# Build the project
cargo build

# Build for release
cargo build --release

# Run the application
cargo run --release

# Check code without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy
```

## Key Dependencies

- `gpui` - UI framework from Zed editor
- `gpui-component` - Additional UI components
- `rfd` - File dialogs
- `serde` / `serde_json` - Serialization
- `reqwest` - HTTP client for FFmpeg downloads
- `zip` - Archive extraction

## Hardware Acceleration Support

When working with transcoding code, consider these encoder mappings:
- NVIDIA: `h264_nvenc`, `hevc_nvenc`, `av1_nvenc`
- Intel QSV: `h264_qsv`, `hevc_qsv`, `av1_qsv`, `vp9_qsv`
- AMD AMF: `h264_amf`, `hevc_amf`, `av1_amf`

## Supported Formats

**Input**: mp4, mkv, avi, mov, webm, flv, wmv, m4v, ts

**Output Containers**: MP4, MKV

**Video Codecs**: H.264, H.265 (HEVC), VP9, AV1

**Audio Codecs**: AAC, MP3, FLAC, Copy (passthrough)

## FFmpeg Integration

The application uses external FFmpeg binaries. Detection logic is in `src/ffmpeg/detector.rs`. The application can:
- Auto-detect system FFmpeg
- Download FFmpeg if not found
- Use custom FFmpeg path via `FFMPEG_DIR` environment variable

## UI Development Notes

- All UI components are in `src/ui/`
- Use `gpui-component` for common UI elements
- Wrap views with `Root` component for proper theming
- Window creation happens in `main.rs`

## Thread Safety

Progress tracking uses atomic operations for thread-safe updates:
- `Arc<AtomicU32>` for shared progress values
- Use `Ordering::Relaxed` for progress updates
- See `CurrentProgress` struct in `app.rs`
