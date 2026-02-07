# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Development Commands

```bash
# Build release binary
cargo build --release

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_name

# Run the CLI (after building)
cargo run -- --help
./target/release/ytcs <URL>

# Lint with Clippy
cargo clippy

# Format code
cargo fmt
```

## Architecture Overview

This is a Rust CLI tool (`ytcs`) that downloads YouTube videos, extracts audio as MP3, detects chapters, and splits into individual tracks with metadata.

### Core Pipeline (main.rs)

The main entry point orchestrates the download and processing:

1. **Dependency check** - Verifies yt-dlp and ffmpeg are installed
2. **URL cleaning** - Extracts video ID, removes playlist parameters
3. **Video info fetch** - Gets metadata via yt-dlp
4. **Output directory setup** - Creates target folder based on artist/album
5. **Download assets** - Downloads thumbnail and audio
6. **Chapter detection** - 3-step fallback: YouTube chapters → description parsing → silence detection
7. **Track splitting** - Uses ffmpeg to split and add metadata

### Key Modules

- **`downloader.rs`** - Interacts with `yt-dlp` for video metadata and audio download. Includes `extract_video_id()` function for parsing YouTube URLs.
- **`audio.rs`** - Uses `ffmpeg` for splitting audio and `lofty` for ID3 metadata/cover art embedding. Handles WebP→JPEG conversion for thumbnails.
- **`chapters.rs`** - Core `Chapter` struct with `start_time`, `end_time`, `title`. Parses JSON chapters from yt-dlp.
- **`chapters_from_description.rs`** - Parses chapter timestamps from video descriptions (multiple formats: "00:00 - Title", "1. Title (0:00)")
- **`chapter_refinement.rs`** - Adjusts chapter markers using silence detection for precise split points. Uses ffmpeg's silencedetect to find optimal boundaries within ±5 second windows.
- **`config.rs`** - TOML-based persistent config at `~/.config/ytcs/config.toml`. Supports format strings like `%n` (track number), `%t` (title), `%a` (artist), `%A` (album).
- **`error.rs`** - `YtcsError` enum covering all error types with `thiserror`.
- **`temp_file.rs`** - RAII wrapper for automatic cleanup of temporary files.
- **`cookie_helper.rs`** - YouTube authentication via browser cookies (for member-only/private videos).
- **`playlist.rs`** - Playlist URL detection and video enumeration. Note: Currently defaults to "video only" mode.
- **`progress.rs`** - Progress bar utilities using `indicatif`.
- **`yt_dlp_progress.rs`** - Real-time download progress parsing from yt-dlp stderr output.
- **`ytdlp_error_parser.rs`** - Parses yt-dlp error messages for user-friendly reporting.
- **`ytdlp_helper.rs`** - Version management and auto-update for yt-dlp.
- **`yt_dlp_update.rs`** - Auto-update functionality.
- **`dependency/`** - Dependency detection and installation modules.
- **`error_handler.rs`** - Centralized error handling for user-facing error messages.
- **`ui.rs`** - Minimalist CLI output utilities with "Pragmatic • Direct • Classy" design philosophy.

### RAII Pattern for Temporary Files

`temp_file.rs` implements a RAII (Resource Acquisition Is Initialization) wrapper for temporary files:

```rust
let temp = TempFile::new(&path);
// ... use the file ...
// File is automatically deleted when `temp` goes out of scope
temp.keep(); // Optional: prevent deletion
```

This pattern is used throughout the codebase for audio files and cover art.

### External Dependencies

- **yt-dlp** - YouTube video metadata and download
- **ffmpeg** - Audio splitting and format conversion
- **lofty** - MP3 metadata/cover art embedding
- **clap** - CLI argument parsing
- **regex** - Chapter/description parsing
- **indicatif** - Progress bars
- **colored** - Terminal color output
- **toml** - Configuration file parsing
- **shellexpand** - Shell path expansion (e.g., ~)
- **time** - Date/time utilities
- **log** - Logging facade
- **thiserror** - Error handling
- **once_cell** - Lazy static initialization

### Testing

Tests are in `tests/` directory. Key test files:
- `test_chapters.rs` - Chapter parsing
- `test_downloader.rs` - Download functions
- `test_integration_e2e.rs` - End-to-end tests
- `test_playlist.rs` - Playlist handling

### Configuration

Config file location: `~/.config/ytcs/config.toml`

Key settings:
- `default_output_dir` - Default: `~/Music`
- `filename_format` - Default: `"%n - %t"` (track number - title)
- `directory_format` - Default: `"%a - %A"` (artist - album)
- `download_cover` - Default: `true`
- `cookies_from_browser` - Browser for auto cookie extraction (chrome, firefox, etc.)

### Debugging

Enable debug logging:
```bash
RUST_LOG=debug ytcs <URL>
```

### Binary Name

The binary is named `ytcs` (not `youtube-chapter-splitter`). This is defined in `Cargo.toml`.
