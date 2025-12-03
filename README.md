# youtube-chapter-splitter

> **ytcs**: Download complete YouTube albums, cleanly split into MP3 tracks with metadata and cover art, all via a single command line.

[![Version](https://img.shields.io/badge/version-0.13.0-blue.svg)](https://github.com/all3f0r1/youtube-chapter-splitter/releases) 
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT) 
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

---

`youtube-chapter-splitter` (or `ytcs`) is a powerful and pragmatic CLI tool designed for one thing: archiving music from YouTube perfectly. It downloads the video, extracts audio to MP3, fetches the cover art, cleans titles, and splits the audio into pristine tracks based on chapters, all in a single command.

## Philosophy

- **Pragmatic**: No frills, just what matters.
- **Direct**: Clear info without detours.
- **Classy**: Elegant without being flashy.

```
ytcs v0.13.0

Fetching video information...
→ Paradox - Chemical Love Theory
  21m 47s • 5 tracks

Downloading the album...

  ✓ Cover downloaded
  ✓ Audio downloaded

Splitting into the album...

  ✓ 01 - Paradox - Light Years Apart (4m 10s)
  ✓ 02 - Paradox - Event Horizon (4m 54s)
  ✓ 03 - Paradox - Orbit of Silence (3m 10s)
  ✓ 04 - Paradox - Chemical Love Theory (4m 03s)
  ✓ 05 - Paradox - Singularity Within (5m 30s)

✓ Done → /home/alex/Musique/Paradox - Chemical Love Theory
```

## Features

- **MP3 Download**: High-quality audio (192 kbps by default).
- **Automatic Cover Art**: Album artwork embedded in MP3 metadata.
- **Chapter-based Splitting**: Automatic detection of YouTube chapters.
- **Description Parsing**: Detects chapters in video descriptions (multiple formats supported).
  - Standard format: `00:00 - Track Title`
  - Numbered format: `1 - Track Title (0:00)`
- **Silence Detection**: Fallback if the video has no chapters.
- **Smart Artist Detection**: Uses channel name if artist not in title.
- **Complete Metadata**: Title, artist, album, track number, cover art.
- **Persistent Configuration**: `config.toml` file for your preferences.
- **Customizable Formatting**: File names (`%n`, `%t`, `%a`, `%A`) and folders (`%a`, `%A`).
- **Smart Cleanup**: Removes `[Full Album]`, `(Official Audio)`, etc.
- **Playlist Support**: Interactive playlist handling.
- **Robust Download**: 4-level fallback system for maximum reliability.
- **Progress Bars**: Real-time progress indicators for downloads and processing.
- **Direct URL Support**: Use `ytcs <URL>` without the `download` command.
- **Dependency Verification**: `yt-dlp` and `ffmpeg` are checked at startup.

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
