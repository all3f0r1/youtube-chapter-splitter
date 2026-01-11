# youtube-chapter-splitter

> **ytcs**: Download complete YouTube albums, cleanly split into MP3 tracks with metadata and cover art, all via a single command line.

[![Version](https://img.shields.io/badge/version-0.14.5-blue.svg)](https://github.com/all3f0r1/youtube-chapter-splitter/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

---

`youtube-chapter-splitter` (or `ytcs`) is a powerful and pragmatic CLI tool designed for one thing: archiving music from YouTube perfectly. It downloads the video, extracts audio to MP3, fetches the cover art, cleans titles, and splits the audio into pristine tracks based on chapters, all in a single command.

## Philosophy

- **Pragmatic**: No frills, just what matters.
- **Direct**: Clear info without detours.
- **Classy**: Elegant without being flashy.

```
ytcs v0.14.5

→ Paradox - Chemical Love Theory
  21m 47s • 5 tracks

Downloading the album...
  ✓ Cover downloaded
  ⏓ [==============================>] 100% | 45.2MiB | 2.34MiB/s | ETA: 00:00
  ✓ Audio downloaded

Refining chapter markers with silence detection...

Chapter refinement report:
      Title                           Original    → Refined    (Delta)
----------------------------------------------------------------------
1.   Light Years Apart                0.0s    →   0.0s      (—      )
2.   Event Horizon                   250.0s   → 249.8s     (-0.2s  )
3.   Orbit of Silence                294.5s   → 295.1s     (+0.6s  )
4.   Chemical Love Theory             348.0s   → 347.9s     (-0.1s  )
5.   Singularity Within               403.0s   → 403.0s     (—      )

  Average adjustment: 0.09s | Max adjustment: 0.60s

Splitting into the album...
  ✓ 01 - Light Years Apart
  ✓ 02 - Event Horizon
  ✓ 03 - Orbit of Silence
  ✓ 04 - Chemical Love Theory
  ✓ 05 - Singularity Within

✓ Done → /home/alex/Music/Paradox - Chemical Love Theory
```

## Features

- **MP3 Download**: High-quality audio (192 kbps by default).
- **Automatic Cover Art**: Album artwork embedded in MP3 metadata.
- **Chapter-based Splitting**: Automatic detection of YouTube chapters.
- **Description Parsing**: Detects chapters in video descriptions (multiple formats supported).
  - Standard format: `00:00 - Track Title`
  - Numbered format: `1 - Track Title (0:00)`
- **Silence Detection**: Fallback if the video has no chapters.
- **Chapter Refinement**: Adjusts chapter markers using silence detection for precise splits (enabled by default).
- **Real-time Progress**: Live download progress with percentage, speed, and ETA.
- **Smart Artist Detection**: Uses channel name if artist not in title.
- **Complete Metadata**: Title, artist, album, track number, cover art.
- **Persistent Configuration**: `config.toml` file for your preferences.
- **Customizable Formatting**: File names (`%n`, `%t`, `%a`, `%A`) and folders (`%a`, `%A`).
- **Smart Cleanup**: Removes `[Full Album]`, `(Official Audio)`, etc.
- **Playlist Support**: Interactive playlist handling.
- **Robust Download**: 4-level fallback system for maximum reliability.
- **Direct URL Support**: Use `ytcs <URL>` without the `download` command.
- **Dependency Verification**: `yt-dlp` and `ffmpeg` are checked at startup.
- **Structured Logging**: Debug mode with `RUST_LOG` environment variable.
- **RAII File Management**: Automatic cleanup of temporary files.
- **Multi-platform Binaries**: Pre-built binaries for Linux, macOS (Intel + Apple Silicon), and Windows.

## Installation

### Prerequisites

`ytcs` requires two external tools:

- **[yt-dlp](https://github.com/yt-dlp/yt-dlp)**: For downloading YouTube videos
- **[ffmpeg](https://ffmpeg.org/)**: For audio processing and splitting

**Quick install:**

```bash
# Linux/macOS (via package manager)
# Ubuntu/Debian
sudo apt install yt-dlp ffmpeg

# macOS (via Homebrew)
brew install yt-dlp ffmpeg

# Or via pip (cross-platform)
pip install yt-dlp

# Windows (via Chocolatey)
choco install yt-dlp ffmpeg
```

### 1. Pre-compiled Binaries (Recommended)

Download the latest release for your system from the [Releases page](https://github.com/all3f0r1/youtube-chapter-splitter/releases).

**Linux/macOS:**
```bash
# Download, extract, and install
wget https://github.com/all3f0r1/youtube-chapter-splitter/releases/latest/download/ytcs-x86_64-unknown-linux-gnu.tar.gz
tar xzf ytcs-x86_64-unknown-linux-gnu.tar.gz
sudo mv ytcs /usr/local/bin/

# Verify installation
ytcs --version
```

**Windows:**
1. Download `ytcs-x86_64-pc-windows-msvc.zip`.
2. Extract `ytcs.exe`.
3. Place it in a folder included in your `PATH`.

### 2. Via `cargo`

```bash
cargo install youtube_chapter_splitter
```

### 3. Build from Source

**Note for macOS users:** Pre-compiled Linux binaries won't work on macOS due to different architectures. You need to compile from source:

```bash
# Clone the repository
git clone https://github.com/all3f0r1/youtube-chapter-splitter.git
cd youtube-chapter-splitter

# Build in release mode
cargo build --release

# Install the binary
sudo cp target/release/ytcs /usr/local/bin/

# Verify installation
ytcs --version
```

**For Linux users**, you can also build from source using the same commands above.

## Usage

`ytcs` works with clear and direct commands.

### Download a Video

The simplest way to use ytcs:

```bash
# Direct URL (recommended)
ytcs "https://www.youtube.com/watch?v=..."

# Explicit download command (also works)
ytcs download "https://www.youtube.com/watch?v=..."
```

**Download options:**
- `-o, --output <DIR>`: Specify an output folder.
- `-a, --artist <ARTIST>`: Force the artist name.
- `-A, --album <ALBUM>`: Force the album name.
- `--no-cover`: Disable cover art download.
- `--refine-chapters`: Enable chapter refinement with silence detection (default: true).
- `--no-refine-chapters`: Disable chapter refinement.

**Examples:**

```bash
# Basic usage
ytcs "https://www.youtube.com/watch?v=dQw4w9WgXcQ"

# Custom output directory
ytcs -o ~/Downloads/Music "https://youtube.com/..."

# Override artist and album
ytcs -a "Pink Floyd" -A "The Wall" "https://youtube.com/..."

# Skip cover art
ytcs --no-cover "https://youtube.com/..."

# Disable chapter refinement (faster, less precise splits)
ytcs --no-refine-chapters "https://youtube.com/..."
```

### Debugging with Logs

`ytcs` includes structured logging for troubleshooting. Control log verbosity with the `RUST_LOG` environment variable:

```bash
# Show debug logs (very verbose, includes all operations)
RUST_LOG=debug ytcs "https://youtube.com/..."

# Show info logs (important events only)
RUST_LOG=info ytcs "https://youtube.com/..."

# Show warnings only (default)
ytcs "https://youtube.com/..."
```

**What's logged:**
- Download attempts and format selector fallbacks
- Chapter detection and parsing
- Audio splitting progress
- Temporary file creation and cleanup
- Error details for troubleshooting

**Save logs to file:**
```bash
RUST_LOG=debug ytcs "https://youtube.com/..." 2>&1 | tee debug.log
```

### Manage Configuration

`ytcs` uses a simple configuration file (`~/.config/ytcs/config.toml`).

```bash
# Show current configuration
ytcs config

# Modify a value
ytcs set audio_quality 128
ytcs set playlist_behavior video_only

# Reset to default configuration
ytcs reset
```

## Configuration

Customize `ytcs` according to your needs. Edit the `config.toml` file directly or use `ytcs set`.

| Key                  | Default                  | Description                                                                 |
|----------------------|--------------------------|-----------------------------------------------------------------------------|
| `default_output_dir` | `~/Music`                | Default output folder.                                                      |
| `download_cover`     | `true`                   | Download album cover art.                                                   |
| `filename_format`    | `"%n - %t"`              | File name format (`%n`: number, `%t`: title, `%a`: artist, `%A`: album).    |
| `directory_format`   | `"%a - %A"`              | Folder format (`%a`: artist, `%A`: album).                                  |
| `audio_quality`      | `192`                    | Audio quality in kbps (e.g., `128`, `192`, `320`).                          |
| `overwrite_existing` | `false`                  | Re-download and overwrite existing files.                                   |
| `max_retries`        | `3`                      | Number of retry attempts on download failure.                               |
| `create_playlist`    | `false`                  | Create a `.m3u` playlist file for YouTube playlists.                        |
| `playlist_behavior`  | `ask`                    | Behavior for playlist URLs: `ask`, `video_only`, `playlist_only`.           |

**Format placeholders:**

**File format:**
- `%n` - Track number (01, 02, ...)
- `%t` - Track title
- `%a` - Artist name
- `%A` - Album name

**Directory format:**
- `%a` - Artist name
- `%A` - Album name

## Robustness

`ytcs` uses a **4-level fallback system** for maximum download reliability:

1. **`bestaudio[ext=m4a]/bestaudio`** - Best quality M4A audio (preferred)
2. **`140`** - YouTube's standard M4A format (very reliable)
3. **`bestaudio`** - Generic best audio selector
4. **Auto-selection** - Let yt-dlp choose automatically (ultimate fallback)

This ensures downloads work even when YouTube's signature system has issues.

### Performance

- **80-90% faster downloads** by downloading audio directly in M4A format instead of full video
- Typical 20-minute album: ~25 MB instead of ~150 MB downloaded

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for the complete changelog.

### Recent Updates

**[0.14.5] - 2025-01-11**
- **Added:** Real-time download progress with percentage, speed, and ETA
- **Added:** Chapter refinement using silence detection for precise split points
- **Added:** `--refine-chapters` flag to enable/disable refinement (enabled by default)
- **Added:** Chapter refinement report showing original vs refined timestamps
- **Improved:** Download progress bar shows actual progress from yt-dlp output

**[0.14.4] - 2024-12-03**
- **Fixed:** Artist detection for titles with stuck dashes (e.g., "Mammoth- Solar Crown Of Fire")
- **Fixed:** Title display with unclosed brackets/parentheses (e.g., "Heat At The Edge Of The Mirror (psychedel")
- **Fixed:** Artist now properly extracted from title instead of falling back to channel name
- **Added:** 5 new tests (3 for stuck dash normalization, 2 for unclosed brackets)

**[0.14.3] - 2024-12-03**
- **Changed:** UI improvement - "Audio downloading" during download, then "✓ Audio downloaded" when complete
- **Changed:** Genre tag cleanup - automatically remove tags like "70s Psychedelic • Progressive Rock" from folder names
- **Changed:** Minimalist fallback messages - replaced aggressive warnings with subtle progress bar updates
- **Added:** 3 new tests for genre tag cleanup functionality

**[0.14.2] - 2024-12-03**
- **Changed:** Complete English translation of all French comments and documentation
- **Improved:** Documentation with enhanced docstrings and better examples
- **Updated:** README with latest features (logging, RAII, multi-platform binaries)
- **Fixed:** Clippy warning for unused assignment in downloader

**[0.14.1] - 2024-12-03**
- **Fixed:** macOS compatibility with build instructions
- **Changed:** Cargo.toml with comprehensive metadata for crates.io
- **Improved:** Documentation with English translations

**[0.14.0] - 2024-12-03**
- **Major Refactoring:** Reduced `process_single_url` from 240+ lines to ~60 lines
- **Added:** 6 modular helper functions for better code organization
- **Added:** 10 new unit tests for refactored functions

**[0.13.0] - 2024-12-03**
- **Added:** Extended logging throughout the pipeline
- **Added:** RAII cover file management with automatic cleanup
- **Improved:** Debugging section in README

**[0.12.0] - 2024-12-03**
- **Added:** Logging system with `log` and `env_logger`
- **Added:** RAII temp file management for automatic cleanup
- **Added:** Download timeout configuration

**[0.11.0] - 2024-12-02**
- **Improved:** Code quality with Clippy warnings fixed
- **Refactored:** `split_single_track` now uses `TrackSplitParams` struct (reduced from 9 to 1 parameter)
- **Refactored:** `progress.rs` module to eliminate code duplication
- **Improved:** Documentation with examples and better docstrings
- **Added:** Tests for progress bars
- **Changed:** Format selectors now use const array instead of vec
- **Updated:** README.md with current version and features

**[0.10.8] - 2024-12-02**
- **Fixed:** Add ultimate fallback with auto format selection
- Works even when all explicit format selectors fail

**[0.10.7] - 2024-12-02**
- **Fixed:** Implement fallback mechanism for format selection
- Better handling of yt-dlp signature extraction issues

**[0.10.6] - 2024-12-02**
- **Performance:** Download audio directly in M4A format (80-90% faster!)
- **UI:** Improved progress messages

**[0.10.5] - 2024-12-01**
- **Added:** Progress bars for downloads and processing
- **Added:** Direct URL support without `download` command
- **Added:** URL validation

**[0.10.4] - 2024-12-01**
- **Added:** Artist detection from channel name
- **Added:** Support for numbered track format in descriptions

---

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
