# youtube-chapter-splitter

> **ytcs**: Download complete YouTube albums, cleanly split into MP3 tracks with metadata and cover art.

[![Version](https://img.shields.io/badge/version-0.15.1-blue.svg)](https://github.com/all3f0r1/youtube-chapter-splitter/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

---

`youtube-chapter-splitter` (or `ytcs`) is a CLI tool designed for one thing: archiving music from YouTube perfectly. It downloads the video, extracts audio to MP3, fetches the cover art, cleans titles, and splits the audio into pristine tracks based on chapters.

## Features

- **Interactive TUI**: Beautiful terminal UI (default mode) for guided downloads
- **Quick CLI**: Direct URL download with `ytcs --cli <URL>`
- **MP3 Download**: High-quality audio (192 kbps by default)
- **Automatic Cover Art**: Album artwork embedded in MP3 metadata
- **Smart Chapter Detection**: YouTube chapters → description parsing → silence detection fallback
- **Chapter Refinement**: Adjusts markers using silence detection for precise splits
- **Playlist Support**: Download entire playlists with interactive selection
- **Complete Metadata**: Title, artist, album, track number, cover art
- **Persistent Configuration**: `config.toml` file for your preferences
- **Auto-Dependency Check**: Detects missing yt-dlp/ffmpeg and prompts to install

## Installation

### Prerequisites

`ytcs` requires:

- **[yt-dlp](https://github.com/yt-dlp/yt-dlp)** - YouTube downloader
- **[ffmpeg](https://ffmpeg.org/)** - Audio processing

```bash
# Ubuntu/Debian
sudo apt install yt-dlp ffmpeg

# macOS
brew install yt-dlp ffmpeg

# Windows (chocolatey)
choco install yt-dlp ffmpeg
```

### Pre-compiled Binaries (Recommended)

Download from [Releases](https://github.com/all3f0r1/youtube-chapter-splitter/releases).

```bash
# Linux
wget https://github.com/all3f0r1/youtube-chapter-splitter/releases/latest/download/ytcs-x86_64-unknown-linux-gnu.tar.gz
tar xzf ytcs-x86_64-unknown-linux-gnu.tar.gz
sudo mv ytcs /usr/local/bin/

# macOS (Intel)
brew install ytcs
```

### Via cargo

```bash
cargo install youtube_chapter_splitter
```

### Build from Source

```bash
git clone https://github.com/all3f0r1/youtube-chapter-splitter.git
cd youtube-chapter-splitter
cargo build --release
sudo cp target/release/ytcs /usr/local/bin/
```

## Usage

### Interactive Mode (Default)

Launch the TUI:

```bash
ytcs
```

**Keyboard shortcuts:**
- `Enter` / `D` - Download from URL
- `S` - Settings
- `H` - Help
- `Q` - Quit
- `Esc` - Go back

### CLI Mode

Download directly:

```bash
# Single video
ytcs --cli "https://www.youtube.com/watch?v=..."

# Custom output
ytcs --cli -o ~/Downloads/Music "URL"

# Override artist/album
ytcs --cli -a "Artist" -A "Album" "URL"
```

**Options:**
| Option | Description |
|--------|-------------|
| `-o, --output <DIR>` | Output directory |
| `-a, --artist <ARTIST>` | Force artist name |
| `-A, --album <ALBUM>` | Force album name |
| `--no-cover` | Skip cover art |
| `--no-refine-chapters` | Disable silence refinement |

### With URL (TUI pre-filled)

```bash
ytcs "https://youtube.com/..."
# Launches TUI with URL ready to download
```

## Configuration

Config file: `~/.config/ytcs/config.toml`

```bash
# View config
ytcs config

# Set value
ytcs set audio_quality 320
ytcs set default_output_dir ~/Music

# Reset to defaults
ytcs reset
```

| Key | Default | Description |
|-----|---------|-------------|
| `default_output_dir` | `~/Music` | Output folder |
| `download_cover` | `true` | Download cover art |
| `filename_format` | `"%n - %t"` | File format |
| `directory_format` | `"%a - %A"` | Folder format |
| `audio_quality` | `192` | MP3 quality (kbps) |
| `playlist_behavior` | `ask` | `ask` / `video_only` / `playlist_only` |
| `cookies_from_browser` | - | Browser for cookies |

**Format placeholders:**
- `%n` - Track number
- `%t` - Track title
- `%a` - Artist
- `%A` - Album

## Debugging

```bash
# Verbose logging
RUST_LOG=debug ytcs "URL"

# Save logs
RUST_LOG=debug ytcs "URL" 2>&1 | tee debug.log
```

## Changelog

See [CHANGELOG.md](CHANGELOG.md).

**Recent:**

- **0.15.1** - TUI by default, URL pre-fill, reduced verbosity
- **0.15.0** - Interactive TUI mode, dependency auto-detection
- **0.14.7** - Windows forbidden character filtering

## License

MIT
