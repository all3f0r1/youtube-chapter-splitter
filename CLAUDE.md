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

# Run benchmarks
cargo bench
```

## Architecture Overview

This is a Rust CLI tool (`ytcs`) that downloads YouTube videos, extracts audio as MP3, detects chapters, and splits into individual tracks with metadata.

### Core Pipeline (main.rs)

The main entry point orchestrates a 6-step process for each URL:

1. **Playlist detection** (`handle_playlist_detection`) - Detects playlists and prompts user
2. **Video info fetch** (`fetch_and_display_video_info`) - Gets metadata via yt-dlp
3. **Output directory setup** (`setup_output_directory`) - Creates target folder
4. **Download assets** (`download_cover_and_audio`) - Downloads thumbnail and audio
5. **Chapter detection** (`get_chapters_with_fallback`) - 3-step fallback: YouTube chapters → description parsing → silence detection
6. **Track splitting** (`split_into_tracks`) - Uses ffmpeg to split and add metadata

The `process_single_url` function was refactored from 240+ lines to ~60 lines using helper functions with clear names like `VideoContext` and `DownloadedAssets` structs for grouping related data.

### Key Modules

- **`downloader.rs`** - Interacts with `yt-dlp` for video metadata and audio download. Uses a 4-level format selector fallback for reliability: `bestaudio[ext=m4a]/bestaudio` → `140` → `bestaudio` → auto.
- **`audio.rs`** - Uses `ffmpeg` for splitting audio and `lofty` for ID3 metadata/cover art embedding. Handles WebP→JPEG conversion for thumbnails.
- **`chapters.rs`** - Core `Chapter` struct with `start_time`, `end_time`, `title`. Parses JSON chapters from yt-dlp.
- **`chapters_from_description.rs`** - Parses chapter timestamps from video descriptions (multiple formats: "00:00 - Title", "1. Title (0:00)")
- **`chapter_refinement.rs`** - Adjusts chapter markers using silence detection for precise split points. Uses ffmpeg's silencedetect to find optimal boundaries within ±5 second windows.
- **`config.rs`** - TOML-based persistent config at `~/.config/ytcs/config.toml`. Supports format strings like `%n` (track number), `%t` (title), `%a` (artist), `%A` (album).
- **`error.rs`** - `YtcsError` enum covering all error types with `thiserror`.
- **`temp_file.rs`** - RAII wrapper for automatic cleanup of temporary files.
- **`cookie_helper.rs`** - YouTube authentication via browser cookies (for member-only/private videos).
- **`playlist.rs`** - Playlist URL detection and video enumeration.
- **`ui.rs`** - Minimalist TUI with "Pragmatic • Direct • Classy" design philosophy.
- **`progress.rs`** - Progress bar utilities using `indicatif`.
- **`yt_dlp_progress.rs`** - Real-time download progress parsing from yt-dlp stderr output.
- **`ytdlp_error_parser.rs`** - Parses yt-dlp error messages for user-friendly reporting.

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

### Testing

Tests are in `tests/` directory. Test fixtures in `tests/fixtures/`. Key test files:
- `test_chapters.rs` - Chapter parsing
- `test_downloader.rs` - Download functions
- `test_integration_e2e.rs` - End-to-end tests
- `test_main.rs` - CLI argument handling

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
