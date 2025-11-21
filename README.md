# YouTube Chapter Splitter

A simple and powerful Rust CLI tool to download YouTube videos, extract audio to MP3, and automatically split them into individual tracks based on chapters.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-0.8.2-blue.svg)](https://github.com/all3f0r1/youtube-chapter-splitter/releases)

## ✨ Features

- 🎵 **Download YouTube audio** in high-quality MP3 format (192 kbps)
- 🖼️ **Download album artwork** automatically with embedded cover art in MP3 tags
- 📑 **Automatic chapter detection** from YouTube video metadata
- 🔇 **Silence detection fallback** for videos without chapters
- ✂️ **Smart audio splitting** with complete ID3 metadata tags (title, artist, album, track number, cover art)
- ⚙️ **Persistent configuration** with customizable defaults (NEW in v0.8.1)
- 📝 **Customizable filename format** with placeholders (%n, %t, %a, %A)
- 📁 **Customizable directory format** for organized music library
- 🎨 **Clean folder names** with intelligent formatting (removes brackets, pipes, capitalizes)
- 📊 **Progress bars** for download and splitting operations
- 🎯 **Force artist/album names** with CLI options
- ⚡ **Dependency checking** with automatic installation prompts
- 🧹 **URL cleaning** - automatically removes playlist and extra parameters
- 🪶 **Lightweight binary** (8.5 MB) with minimal dependencies
- 🎶 **Playlist support** (NEW in v0.8.2)
- 🔄 **Retry mechanism** for failed downloads (NEW in v0.8.2)
- 덮어쓰기 **Overwrite option** for existing files (NEW in v0.8.2)

## 🚀 Quick Start

### Prerequisites

The application will check for dependencies at startup and offer to install them:

- **yt-dlp**: `pip install yt-dlp`
- **ffmpeg**: 
  - Linux: `sudo apt install ffmpeg`
  - macOS: `brew install ffmpeg`
  - Windows: Download from [ffmpeg.org](https://ffmpeg.org/download.html)

### Installation

#### Option 1: From crates.io (Recommended)

```bash
cargo install youtube_chapter_splitter
```

The `ytcs` binary will be installed in `~/.cargo/bin/` (make sure it's in your PATH).

#### Option 2: From source

```bash
# Clone the repository
git clone https://github.com/all3f0r1/youtube-chapter-splitter.git
cd youtube-chapter-splitter

# Build and install
cargo install --path .
```

### Usage

**Simple syntax:**

```bash
ytcs <YOUTUBE_URL> [OPTIONS]
```

**Options:**
- `-o, --output <DIR>` - Output directory (overrides config)
- `-a, --artist <ARTIST>` - Force artist name (overrides auto-detection)
- `-A, --album <ALBUM>` - Force album name (overrides auto-detection)
- `--no-cover` - Skip downloading cover art
- `--playlist` - Force playlist download

**Configuration commands:**
- `ytcs config` - Show current configuration
- `ytcs set <key> <value>` - Set a configuration value
- `ytcs reset` - Reset configuration to defaults

**Examples:**

```bash
# Download and split a YouTube video (saves to ~/Music)
ytcs "https://www.youtube.com/watch?v=28vf7QxgCzA"

# Download a playlist
ytcs "https://www.youtube.com/playlist?list=..."

# Specify custom output directory
ytcs "https://www.youtube.com/watch?v=28vf7QxgCzA" --output ~/Downloads

# Force artist and album names
ytcs "https://www.youtube.com/watch?v=..." -a "Pink Floyd" -A "Dark Side of the Moon"

# Skip cover art download
ytcs "https://www.youtube.com/watch?v=..." --no-cover

# Configure default output directory
ytcs set default_output_dir "~/Downloads/Music"

# Customize filename format
ytcs set filename_format "%n %t"

# Show current configuration
ytcs config
```

**Important:** Always put URLs in quotes to avoid shell interpretation of special characters:
```bash
ytcs "URL"  # ✅ Correct
ytcs URL    # ❌ May cause issues with & characters
```

## ⚙️ Configuration

YouTube Chapter Splitter uses a persistent configuration file stored at:
- **Linux/macOS**: `~/.config/ytcs/config.toml`
- **Windows**: `%APPDATA%\ytcs\config.toml`

### Available Settings

| Setting | Default | Description |
|---|---|---|
| `default_output_dir` | `~/Music` | Default download directory |
| `download_cover` | `true` | Download album artwork |
| `filename_format` | `"%n - %t"` | Filename format with placeholders |
| `directory_format` | `"%a - %A"` | Directory format with placeholders |
| `audio_quality` | `192` | MP3 quality (128 or 192 kbps) |
| `overwrite_existing` | `false` | Overwrite existing files |
| `max_retries` | `3` | Retries on download failure |
| `create_playlist` | `false` | Create .m3u playlist file |

### Format Placeholders

**Filename format**:
- `%n` - Track number (01, 02, etc.)
- `%t` - Track title
- `%a` - Artist name
- `%A` - Album name

**Directory format**:
- `%a` - Artist name
- `%A` - Album name

### Configuration Examples

```bash
# Organize by artist, then album
ytcs set directory_format "%a/%A"
# Result: ~/Music/Marigold/Oblivion Gate/

# Simple filenames without track numbers
ytcs set filename_format "%t"
# Result: Oblivion Gate.mp3

# Include artist in filename
ytcs set filename_format "%a - %t"
# Result: Marigold - Oblivion Gate.mp3

# Disable cover art download by default
ytcs set download_cover false
```

See [CONFIGURATION_GUIDE.md](./CONFIGURATION_GUIDE.md) for detailed configuration documentation.

## 📊 Example Output

```
=== YouTube Chapter Splitter ===

Fetching video information...
Title: Marigold - Oblivion Gate
Duration: 29m 29s
Tracks found: 5

Downloading album artwork...
✓ Artwork saved: /home/user/Music/Marigold - Oblivion Gate/cover.jpg

⠋ Downloading audio from YouTube...
✓ Audio downloaded: /home/user/Music/Marigold - Oblivion Gate/temp_audio.mp3

Using YouTube tracks

Tracks to create:
  1. Oblivion Gate [5m 54s]
  2. Obsidian Throne [5m 35s]
  3. Crimson Citadel [5m 47s]
  4. Silver Spire [6m 30s]
  5. Eternal Pyre [5m 43s]

⠋ [████████████████████████████████████████] 5/5 Track 5: Eternal Pyre
✓ Splitting completed successfully!

✓ Processing completed successfully!
Files created: 5
Directory: /home/user/Music/Marigold - Oblivion Gate
```

## 🧪 Testing

The project includes a comprehensive test suite with 172 tests covering:

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_clean_folder_name
```

## 📝 Changelog

### v0.8.2 (2024)
- 🎶 Added playlist support
- 🔄 Added retry mechanism for failed downloads
- 덮어쓰기 Added overwrite option for existing files
- 🎧 Added audio quality option (128/192 kbps)

### v0.8.1 (2024)
- ⚙️ Added persistent configuration system with TOML
- 📝 Customizable filename and directory formats
- 📊 Progress bars for download and splitting operations
- 🎨 Improved UX with colored output
- 📚 Comprehensive configuration documentation

### v0.8.0 (2024)
- 🗑️ Removed TUI interface (simplified to CLI only)
- 🧹 Cleaned up dependencies
- ⚡ Improved compilation time (-62%)
- 📦 Reduced binary size to 7.8 MB

### v0.7.0 (2024)
- 🖥️ Added TUI (Text User Interface) with ratatui
- 🎯 Interactive chapter selection
- ✏️ Metadata editing in TUI

### v0.6.0 (2024)
- 🧪 Added 139 comprehensive tests (85% coverage)
- 🎯 Performance benchmarks with Criterion
- 🌍 Unicode and emoji support in metadata
- ✅ Validation and edge case handling

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- [yt-dlp](https://github.com/yt-dlp/yt-dlp) for YouTube downloading
- [FFmpeg](https://ffmpeg.org/) for audio processing
- [lofty](https://github.com/Serial-ATA/lofty-rs) for ID3 tag handling
- [clap](https://github.com/clap-rs/clap) for CLI parsing
- [indicatif](https://github.com/console-rs/indicatif) for progress bars
