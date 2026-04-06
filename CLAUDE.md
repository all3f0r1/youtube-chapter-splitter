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
cargo run -- "https://www.youtube.com/watch?v=VIDEO_ID"
./target/release/ytcs <URL>
./target/release/ytcs config

# Lint with Clippy
cargo clippy

# Format code
cargo fmt
```

## Architecture Overview

This is a Rust CLI tool (`ytcs`) that downloads YouTube videos, extracts audio as MP3, detects chapters, and splits into individual tracks with metadata.

### Core Pipeline (main.rs)

The main entry point orchestrates the download and processing (`ytcs <URL>`) or configuration (`ytcs config` / `ytcs config --show`):

1. **Dependency check** - Verifies yt-dlp and ffmpeg are installed (behavior from `dependency_auto_install` in config). Missing tools are reported as `YtcsError::MissingTools` (`MissingToolsError` with `missing_ytdlp` / `missing_ffmpeg` flags).
2. **URL resolution** - `playlist::is_playlist_url` + `playlist_behavior` (`VideoOnly` / `Ask` / `PlaylistOnly`): single canonical `watch?v=` URL or a list of URLs from `get_playlist_info`. Plain playlist-only pages need `PlaylistOnly` or user confirmation when `Ask`.
3. **Video info fetch** - Gets metadata via yt-dlp (`VideoInfo` includes optional `description` for description-based chapters)
4. **Output directory setup** - Creates target folder using `directory_format` and config output base (CLI `-o` overrides)
5. **Download assets** - Thumbnail if `download_cover` (unless `--no-cover`); audio via yt-dlp using `audio_format` (mp3/opus/m4a), `audio_quality`, cookies, timeouts. Optional `--skip-download` reuses `temp_audio.<ext>`.
6. **Chapter detection** - YouTube JSON chapters → `chapters_from_description` (if at least 2 markers) → `detect_silence_chapters`. If chapters came from JSON or description, `chapter_refinement::refine_chapters_with_silence` runs by default (`refine_chapters` true); disable in config or rely on silence-only path skipping refinement. Uses `refine_silence_window`, `refine_noise_db`, `refine_min_silence` (defaults ±5s, -35 dB, 1.2s).
7. **Track splitting** - ffmpeg encodes per `audio_format` and `audio_quality`; embeds date/genre/comment when `VideoInfo` provides them. Honors `overwrite_existing`. If `create_playlist`, writes `playlist.m3u` via `audio::write_m3u_playlist`. CLI: `--dry-run`, `-q`/`--quiet` via `ui::set_output_quiet`.

### Key Modules

- **`downloader.rs`** - Interacts with `yt-dlp` for video metadata and audio download. Includes `extract_video_id()` function for parsing YouTube URLs.
- **`audio.rs`** - Uses `ffmpeg` for splitting audio and `lofty` for ID3 metadata/cover art embedding. Handles WebP→JPEG conversion for thumbnails.
- **`chapters.rs`** - Core `Chapter` struct with `start_time`, `end_time`, `title`. Parses JSON chapters from yt-dlp.
- **`chapters_from_description.rs`** - Parses chapter timestamps from video descriptions (multiple formats: "00:00 - Title", "1. Title (0:00)")
- **`chapter_refinement.rs`** - Adjusts chapter markers using silence detection; window/threshold/duration come from config (defaults ±5s, -35 dB, 1.2s min silence).
- **`config.rs`** - TOML config at `~/.config/ytcs/config.toml`, edited via `ytcs config` (interactive wizard) or manually. `print_config_summary` / `run_interactive_config_wizard`. Format strings `%n`, `%t`, `%a`, `%A`.
- **`error.rs`** - `YtcsError` enum with `thiserror`; structured `MissingToolsError` for dependency install flows.
- **`temp_file.rs`** - RAII wrapper for automatic cleanup of temporary files.
- **`cookie_helper.rs`** - YouTube authentication via browser cookies (for member-only/private videos).
- **`playlist.rs`** - `is_playlist_url`, `get_playlist_info`, `remove_playlist_param`; behavior driven by config `playlist_behavior`.
- **`progress.rs`** - Progress bar utilities using `indicatif`.
- **`yt_dlp_progress.rs`** - Real-time download progress parsing from yt-dlp stderr output.
- **`ytdlp_error_parser.rs`** - Parses yt-dlp error messages for user-friendly reporting.
- **`ytdlp_helper.rs`** - Version management and auto-update for yt-dlp.
- **`yt_dlp_update.rs`** - Auto-update functionality.
- **`dependency/`** - Dependency detection and installation modules.
- **`error_handler.rs`** - Centralized error handling for user-facing error messages.
- **`ui.rs`** - Minimalist CLI output; `set_output_quiet` / `is_output_quiet` for `-q` mode.

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

Config file: `~/.config/ytcs/config.toml` (created on first `Config::load()` or first `ytcs config`).

Edit with **`ytcs config`** (prompts for each field; Enter keeps current). **`ytcs config --show`** prints the file path and all values.

Key settings include: `default_output_dir`, `filename_format`, `directory_format`, `audio_quality` (128/192/320), `audio_format` (mp3/opus/m4a), `download_cover`, `overwrite_existing`, `create_playlist`, `refine_chapters`, `refine_silence_window`, `refine_noise_db`, `refine_min_silence`, `playlist_prefix_index`, `playlist_behavior`, `cookies_from_browser`, `download_timeout`, `max_retries`, `dependency_auto_install`, `ytdlp_auto_update`.

### Debugging

Enable debug logging:
```bash
RUST_LOG=debug ytcs <URL>
```

### Binary Name

The binary is named `ytcs` (not `youtube-chapter-splitter`). This is defined in `Cargo.toml`.
