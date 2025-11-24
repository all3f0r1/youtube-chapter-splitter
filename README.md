# YouTube Chapter Splitter

A simple and powerful Rust CLI tool to download YouTube videos, extract audio to MP3, and automatically split them into individual tracks based on chapters.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-0.9.3-blue.svg)](https://github.com/all3f0r1/youtube-chapter-splitter/releases)
[![CI](https://github.com/all3f0r1/youtube-chapter-splitter/workflows/CI/badge.svg)](https://github.com/all3f0r1/youtube-chapter-splitter/actions/workflows/ci.yml)

## ✨ Features

- 🎵 **Download YouTube audio** in high-quality MP3 format (192 kbps default).
- 🖼️ **Download album artwork** automatically with embedded cover art in MP3 tags.
- 📑 **Automatic chapter detection** from YouTube video metadata.
- 🔇 **Silence detection fallback** for videos without chapters.
- ✂️ **Smart audio splitting** with complete ID3 metadata tags (title, artist, album, track number, cover art).
- ⚙️ **Persistent configuration** with customizable defaults in a `config.toml` file.
- 📝 **Customizable filename format** with placeholders (`%n`, `%t`, `%a`, `%A`).
- 📁 **Customizable directory format** for organized music library (`%a`, `%A`).
- 🎨 **Clean folder names** with intelligent formatting (removes brackets, pipes, and capitalizes).
- 📊 **Progress bars** for download and splitting operations.
- 🎯 **Force artist/album names** with CLI options.
- ⚡ **Dependency checking** with automatic installation prompts.
- 🧹 **URL cleaning** - automatically removes playlist and extra parameters.
- 🎶 **Playlist support** with interactive prompts.
- 🔄 **Retry mechanism** for failed downloads.
- 덮어쓰기 **Overwrite option** for existing files.
- 🔠 **Title Case** final MP3 filenames (first letter of each word capitalized) for better readability.

## 🚀 Quick Start

### Prerequisites

The application will check for dependencies at startup and offer to install them:

- **yt-dlp**: `pip install yt-dlp`
- **ffmpeg**:
  - Linux: `sudo apt install ffmpeg`
  - macOS: `brew install ffmpeg`
  - Windows: Download from [ffmpeg.org](https://ffmpeg.org/download.html)

### Installation

#### Option 1: Download pre-built binaries (Easiest)

Download the latest release for your platform from the [Releases page](https://github.com/all3f0r1/youtube-chapter-splitter/releases).

**Linux/macOS:**
```bash
# Download and extract
wget https://github.com/all3f0r1/youtube-chapter-splitter/releases/latest/download/ytcs-x86_64-unknown-linux-gnu.tar.gz
tar xzf ytcs-x86_64-unknown-linux-gnu.tar.gz

# Install
sudo mv ytcs /usr/local/bin/

# Verify
ytcs --version
```

**Windows:**
1. Download `ytcs-x86_64-pc-windows-msvc.zip`
2. Extract `ytcs.exe`
3. Add the directory to your PATH or move to a directory in PATH

#### Option 2: From crates.io

```bash
cargo install youtube_chapter_splitter
```

The `ytcs` binary will be installed in `~/.cargo/bin/` (make sure it's in your PATH).

### Usage

**Simple syntax:**
```bash
ytcs "<YOUTUBE_URL>" [OPTIONS]
```

**Options:**
- `-o, --output <DIR>` - Output directory (overrides config)
- `-a, --artist <ARTIST>` - Force artist name (overrides auto-detection)
- `-A, --album <ALBUM>` - Force album name (overrides auto-detection)
- `--no-cover` - Skip downloading cover art

**Configuration commands:**
- `ytcs config` - Show current configuration
- `ytcs set <key> <value>` - Set a configuration value
- `ytcs reset` - Reset configuration to defaults

**Examples:**
```bash
# Download and split a YouTube video (saves to ~/Music)
ytcs "https://www.youtube.com/watch?v=28vf7QxgCzA"

# Download a playlist (will prompt for confirmation)
ytcs "https://www.youtube.com/playlist?list=..."

# Specify custom output directory
ytcs "https://www.youtube.com/watch?v=28vf7QxgCzA" --output ~/Downloads

# Force artist and album names
ytcs "https://www.youtube.com/watch?v=..." -a "Pink Floyd" -A "Dark Side of the Moon"
```

## ⚙️ Configuration

YouTube Chapter Splitter uses a persistent configuration file stored at:
- **Linux/macOS**: `~/.config/ytcs/config.toml`
- **Windows**: `%APPDATA%\ytcs\config.toml`

### Available Settings

| Setting              | Default                  | Description                                      |
|----------------------|--------------------------|--------------------------------------------------|
| `default_output_dir` | `~/Music`                | Default download directory                       |
| `download_cover`     | `true`                   | Download album artwork                           |
| `filename_format`    | `"%n - %t"`              | Filename format with placeholders                |
| `directory_format`   | `"%a - %A"`              | Directory format with placeholders               |
| `audio_quality`      | `192`                    | MP3 quality (128 or 192 kbps)                    |
| `overwrite_existing` | `false`                  | Overwrite existing files                         |
| `max_retries`        | `3`                      | Retries on download failure                      |
| `create_playlist`    | `false`                  | Create .m3u playlist file for playlists          |
| `playlist_behavior`  | `ask`                    | `ask`, `video_only`, or `playlist_only`          |

### Format Placeholders

**Filename format**:
- `%n` - Track number (01, 02, etc.)
- `%t` - Track title
- `%a` - Artist name
- `%A` - Album name

**Directory format**:
- `%a` - Artist name
- `%A` - Album name

## 📝 Changelog

### [0.9.3] - 2025-11-24
- **Fixed:** Configuration parsing error for users upgrading from older versions.
- **Added:** Serde default values for all configuration fields to ensure backward compatibility.
- **Technical:** Old config files missing `playlist_behavior` field now work correctly.

### [0.9.2] - 2025-11-24
- **Changed:** MP3 filenames now use Title Case (first letter of each word capitalized) instead of full uppercase for better readability.
- **Example:** `01 - Oblivion Gate.mp3` instead of `01 - OBLIVION GATE.MP3`

### [0.9.1] - 2025-11-24
- **Added:** Final MP3 filenames are now fully capitalized for better compatibility.
- **Fixed:** Corrected all `clippy` warnings for improved code quality.
- **Changed:** Refactored `main.rs` to reduce code duplication between single video and playlist video downloads.
- **Removed:** Deleted obsolete changelogs, examples, and backup files from the repository.
- **Docs:** Completely rewrote the README for clarity, and consolidated the changelog.

### [0.9.0] - (Previous Version)
- Major internal refactoring and UI improvements.

### [0.3.2] - 2024-11-16
- **Fixed:** Simplified cover art logic to always use external `cover.jpg`.

### [0.3.1] - 2024-11-16
- **Fixed:** "Stream map '1:v' matches no streams" error when audio has no embedded cover art.

### [0.3.0] - 2024-11-16
- **Fixed:** Cover art is now properly embedded in ALL tracks, not just the first one.

### [0.2.x] - 2024-11-10
- **Added:** Cover art download, `--artist` and `--album` options, cross-platform default directory, and automatic name cleaning.
- **Changed:** Replaced `reqwest` with `ureq` to reduce binary size.

### [0.1.0] - 2024-11-09
- **Added:** Initial release with core features: YouTube download, MP3 conversion, chapter splitting, and metadata tagging.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- [yt-dlp](https://github.com/yt-dlp/yt-dlp) for YouTube downloading
- [FFmpeg](https://ffmpeg.org/) for audio processing
- [lofty](https://github.com/Serial-ATA/lofty-rs) for ID3 tag handling
