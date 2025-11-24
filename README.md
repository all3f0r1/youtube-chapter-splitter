# youtube-chapter-splitter

> **ytcs**: Download complete YouTube albums, cleanly split into MP3 tracks with metadata and cover art, all via a single command line.

[![Version](https://img.shields.io/badge/version-0.10.1-blue.svg)](https://github.com/all3f0r1/youtube-chapter-splitter/releases) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT) [![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/) [![CI](https://github.com/all3f0r1/youtube-chapter-splitter/workflows/CI/badge.svg)](https://github.com/all3f0r1/youtube-chapter-splitter/actions/workflows/ci.yml)

---

`youtube-chapter-splitter` (or `ytcs`) is a powerful and pragmatic CLI tool designed for one thing: archiving music from YouTube perfectly. It downloads the video, extracts audio to MP3, fetches the cover art, cleans titles, and splits the audio into pristine tracks based on chapters, all in a single command.

## Philosophy

- **Pragmatic**: No frills, just what matters.
- **Direct**: Clear info without detours.
- **Classy**: Elegant without being flashy.

```
ytcs v0.10.1

→ Marigold - Oblivion Gate
  29m 29s • 5 tracks

  ✓ Cover downloaded
  ✓ Audio downloaded

  Splitting tracks...
  ✓ 01 Oblivion Gate (5m 54s)
  ✓ 02 Obsidian Throne (5m 35s)
  ✓ 03 Crimson Citadel (5m 47s)
  ✓ 04 Silver Spire (6m 30s)
  ✓ 05 Eternal Pyre (5m 43s)

✓ Done → ~/Music/Marigold - Oblivion Gate
```

## Features

- **MP3 Download**: High-quality audio (192 kbps by default).
- **Automatic Cover Art**: Album artwork embedded in MP3 metadata.
- **Chapter-based Splitting**: Automatic detection of YouTube chapters.
- **Silence Detection**: Fallback if the video has no chapters.
- **Complete Metadata**: Title, artist, album, track number, cover art.
- **Persistent Configuration**: `config.toml` file for your preferences.
- **Customizable Formatting**: File names (`%n`, `%t`) and folders (`%a`, `%A`).
- **Smart Cleanup**: Removes `[Full Album]`, `(Official Audio)`, etc.
- **Playlist Support**: Interactive playlist handling.
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

The default command is `download`. You can omit it for quick usage.

```bash
# Full syntax
ytcs download "https://www.youtube.com/watch?v=..."

# Quick syntax (recommended)
ytcs "https://www.youtube.com/watch?v=..."
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

## Changelog

### [0.10.1] - 2025-11-24
- **Fixed:** CLI subcommands (`config`, `set`, `reset`) now work correctly.
- **Technical:** Refactored CLI structure to properly handle subcommands with `clap`.

### [0.10.0] - 2025-11-24
- **Changed:** Complete UI redesign - minimal, clean, and pragmatic.
- **Removed:** Unnecessary borders, boxes, and visual noise.
- **Removed:** Emojis from main output (kept only status checkmarks).
- **Improved:** Auto-clean video titles (removes `[Full Album]`, `[Official Audio]`, etc.).
- **Improved:** More direct and readable output format.
- **Philosophy:** Pragmatic • Direct • Classy

### [0.9.3] - 2025-11-24
- **Fixed:** Configuration parsing error for users upgrading from older versions.
- **Added:** Serde default values for all configuration fields to ensure backward compatibility.
- **Technical:** Old config files missing `playlist_behavior` field now work correctly.

### [0.9.2] - 2025-11-24
- **Changed:** MP3 filename capitalization from UPPERCASE to Title Case for better readability.
- **Example:** `01 - Oblivion Gate.mp3` instead of `01 - OBLIVION GATE.MP3`.

### [0.9.1] - 2025-11-24
- **Added:** Complete code audit with all clippy warnings fixed.
- **Removed:** Obsolete documentation files and examples.
- **Improved:** Code quality and maintainability.
- **Changed:** Capitalized MP3 filenames (reverted in 0.9.2).

### [0.9.0] - 2025-11-24
- **Added:** Playlist support with configurable behavior.
- **Added:** Configuration management commands (`config`, `set`, `reset`).
- **Added:** Customizable file and directory naming formats.
- **Improved:** Better error handling and user feedback.

---

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
