# Development Guide

## Prerequisites

- **Rust**: 1.70 or later
- **yt-dlp**: For YouTube downloads
- **ffmpeg**: For audio processing

## Installation

```bash
# Clone repository
git clone https://github.com/all3f0r1/youtube-chapter-splitter.git
cd youtube-chapter-splitter

# Build in release mode
cargo build --release

# Binary location: target/release/ytcs
```

## Development Commands

### Build

```bash
# Debug build (faster compilation)
cargo build

# Release build (optimized)
cargo build --release
```

### Run

```bash
# Run CLI (after building)
cargo run -- --help

# Run with arguments
cargo run -- "https://www.youtube.com/watch?v=..."
```

### Test

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in a specific file
cargo test --test test_chapters
```

### Lint & Format

```bash
# Lint with Clippy
cargo clippy

# Format code
cargo fmt

# Check without formatting
cargo fmt --check
```

### Benchmarks

```bash
# Run benchmarks (requires nightly Rust or stable with feature)
cargo bench
```

## Project Structure

```
src/
├── main.rs              # Entry point, CLI
├── lib.rs               # Library exports
├── audio.rs             # Audio processing
├── chapters.rs          # Chapter structures
├── chapter_refinement.rs # Silence detection
├── chapters_from_description.rs # Description parsing
├── config.rs            # Configuration
├── downloader.rs        # yt-dlp integration
├── error.rs             # Error types
├── playlist.rs          # Playlist handling
├── progress.rs          # Progress bars
├── temp_file.rs         # RAII temp files
├── ui.rs                # Terminal UI
├── utils.rs             # Utilities
├── ytdlp_error_parser.rs # Error parsing
└── yt_dlp_progress.rs   # Download progress

tests/                   # Integration/unit tests
benches/                 # Criterion benchmarks
```

## Configuration File

Location: `~/.config/ytcs/config.toml`

```toml
default_output_dir = "~/Music"
download_cover = true
filename_format = "%n - %t"
directory_format = "%a - %A"
audio_quality = 192
overwrite_existing = false
max_retries = 3
create_playlist = false
playlist_behavior = "ask"
```

## Debugging

```bash
# Enable debug logging
RUST_LOG=debug cargo run -- "URL"

# Show info logs
RUST_LOG=info cargo run -- "URL"

# Save logs to file
RUST_LOG=debug cargo run -- "URL" 2>&1 | tee debug.log
```

## Testing Strategy

| Test File | Coverage |
|-----------|----------|
| `test_chapters.rs` | Chapter struct operations |
| `test_audio_functions.rs` | Audio processing utilities |
| `test_audio_tags.rs` | ID3 tag handling |
| `test_downloader.rs` | yt-dlp integration |
| `test_integration_e2e.rs` | End-to-end workflows |
| `test_edge_cases_advanced.rs` | Edge cases and corner cases |
| `test_utils_edge_cases.rs` | Title cleaning edge cases |
| `test_refactored_helpers.rs` | Refactored helper functions |

## Adding Features

1. **New CLI argument**: Add to `Cli` struct in `main.rs`
2. **New config option**: Add to `Config` struct in `config.rs`
3. **New chapter source**: Extend `get_chapters_with_fallback`
4. **New metadata field**: Update `VideoInfo` and tag writing in `audio.rs`

## Code Conventions

- **Error handling**: Use `Result<T>` and `YtcsError`
- **Temp files**: Always use `TempFile` RAII wrapper
- **Progress**: Use `indicatif` for long operations
- **Logging**: Use `log::{debug,info,warn,error}` macros
- **Tests**: Write unit tests in `tests/`, name with `test_` prefix

## Release Checklist

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Run `cargo test --all`
4. Run `cargo clippy -- -D warnings`
5. Build release binaries for all platforms
6. Create GitHub release
7. Upload binaries
