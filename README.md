# YouTube Chapter Splitter (ytcs)

A powerful Rust application to download YouTube videos, extract audio to MP3, and automatically split them into individual tracks based on chapters.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

## ✨ Features

- 🎵 **Download YouTube audio** in high-quality MP3 format
- 📑 **Automatic chapter detection** from YouTube video metadata
- 🔇 **Silence detection** for videos without chapters
- ✂️ **Smart audio splitting** with proper ID3 metadata tags
- 🖥️ **Dual interface**: Command-line (CLI) and Graphical (GUI)
- ⚡ **Dependency checking** with automatic installation prompts
- 🎨 **Clean folder names** with intelligent formatting
- 📊 **Human-readable durations** (e.g., "5m 43s" instead of "343s")

## 🚀 Quick Start

### Prerequisites

The application will check for dependencies at startup and offer to install them:

- **yt-dlp**: `pip install yt-dlp`
- **ffmpeg**: 
  - Linux: `sudo apt install ffmpeg`
  - macOS: `brew install ffmpeg`
  - Windows: Download from [ffmpeg.org](https://ffmpeg.org/download.html)

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/youtube-chapter-splitter.git
cd youtube-chapter-splitter

# Build the project
cargo build --release

# The binaries will be in target/release/
# - ytcs (CLI)
# - ytcs-gui (GUI)
```

### Basic Usage

#### CLI

```bash
# Get video information
./target/release/ytcs info --url "https://www.youtube.com/watch?v=VIDEO_ID"

# Download and split by chapters
./target/release/ytcs download --url "https://www.youtube.com/watch?v=VIDEO_ID"

# Download with silence detection fallback
./target/release/ytcs download --url "URL" --detect-silence true

# Split an existing audio file
./target/release/ytcs split --input audio.mp3 --detect-silence
```

#### GUI

```bash
# Launch the graphical interface
./target/release/ytcs-gui
```

The GUI provides:
- Dependency status checking with install buttons
- Video information fetching
- Progress tracking
- Interactive configuration of silence detection parameters

## 📖 Detailed Usage

### Download Command

Download a YouTube video and split it into tracks:

```bash
ytcs download --url "https://www.youtube.com/watch?v=28vf7QxgCzA"
```

**Options:**
- `--url <URL>`: YouTube video URL (required)
- `--output <DIR>`: Output directory (default: `./output`)
- `--detect-silence <BOOL>`: Enable silence detection fallback (default: `true`)
- `--silence-threshold <DB>`: Silence threshold in dB (default: `-30`)
- `--min-silence-duration <SECONDS>`: Minimum silence duration (default: `2.0`)

**Example output:**
```
output/
└── Marigold - Oblivion Gate/
    ├── 01 - Oblivion Gate.mp3
    ├── 02 - Obsidian Throne.mp3
    ├── 03 - Crimson Citadel.mp3
    ├── 04 - Silver Spire.mp3
    └── 05 - Eternal Pyre.mp3
```

### Split Command

Split an existing audio file:

```bash
ytcs split --input album.mp3 --detect-silence --album "My Album"
```

**Options:**
- `--input <FILE>`: Input audio file (required)
- `--output <DIR>`: Output directory (default: `./output`)
- `--chapters <FILE>`: JSON file with chapter timestamps
- `--detect-silence`: Enable automatic silence detection
- `--silence-threshold <DB>`: Silence threshold in dB (default: `-30`)
- `--min-silence-duration <SECONDS>`: Minimum silence duration (default: `2.0`)
- `--album <NAME>`: Album name for metadata (default: `"Album"`)

### Info Command

Display video information without downloading:

```bash
ytcs info --url "https://www.youtube.com/watch?v=VIDEO_ID"
```

**Output:**
```
=== Video Information ===
Title: MARIGOLD - Oblivion Gate [Full Album]
ID: 28vf7QxgCzA
Duration: 29m 29s
Tracks: 5
Track list:
  1. Oblivion Gate [5m 54s]
  2. Obsidian Throne [5m 35s]
  3. Crimson Citadel [5m 47s]
  4. Silver Spire [6m 30s]
  5. Eternal Pyre [5m 43s]
```

### Install Command

Manually install missing dependencies:

```bash
ytcs install --tool yt-dlp
ytcs install --tool ffmpeg
```

## 🎯 Use Cases

### 1. Music Albums on YouTube

Perfect for downloading full albums uploaded as single videos with chapters:

```bash
ytcs download --url "https://www.youtube.com/watch?v=ALBUM_VIDEO_ID"
```

### 2. Podcasts and Interviews

Split long-form content into segments:

```bash
ytcs download --url "PODCAST_URL" --detect-silence true --min-silence-duration 3.0
```

### 3. DJ Mixes and Sets

Extract individual tracks from DJ sets:

```bash
ytcs download --url "MIX_URL" --silence-threshold -35
```

### 4. Local Audio Files

Process audio files you already have:

```bash
ytcs split --input recording.mp3 --detect-silence
```

## 🛠️ Advanced Features

### Folder Name Cleaning

The application automatically cleans folder names:
- Removes `[...]` and `(...)` content
- Replaces `_` with `-` between artist and album
- Capitalizes words properly

**Example:**
```
Input:  "MARIGOLD - Oblivion Gate [Full Album] (70s Psychedelic Blues Acid Rock)"
Output: "Marigold - Oblivion Gate"
```

### Metadata Tagging

Each MP3 file includes proper ID3 tags:
- **Title**: Track name
- **Track**: Track number / Total tracks (e.g., "1/5")
- **Album**: Video title or custom album name

### Silence Detection Algorithm

The silence detection uses ffmpeg's `silencedetect` filter:
1. Analyzes audio for silence periods
2. Uses the midpoint of each silence as split point
3. Configurable threshold and minimum duration
4. Automatically creates track boundaries

## 📁 Project Structure

```
youtube-chapter-splitter/
├── src/
│   ├── main.rs          # CLI application
│   ├── gui.rs           # GUI application
│   ├── lib.rs           # Library exports
│   ├── error.rs         # Error handling
│   ├── chapters.rs      # Chapter parsing and manipulation
│   ├── downloader.rs    # YouTube downloading with yt-dlp
│   ├── audio.rs         # Audio processing with ffmpeg
│   └── utils.rs         # Utility functions (formatting, cleaning)
├── examples/
│   ├── basic_usage.rs           # Programmatic usage example
│   └── chapters_example.json    # Sample chapters file
├── Cargo.toml           # Dependencies and configuration
├── README.md            # This file
├── LICENSE              # MIT License
└── .gitignore           # Git ignore rules
```

## 🧪 Testing

Run the test suite:

```bash
cargo test
```

Test with a real video:

```bash
cargo run --bin ytcs -- info --url "https://www.youtube.com/watch?v=28vf7QxgCzA"
```

## 📝 Chapter JSON Format

You can provide custom chapters via JSON:

```json
{
  "chapters": [
    {
      "title": "Track 1",
      "start_time": 0.0,
      "end_time": 180.0
    },
    {
      "title": "Track 2",
      "start_time": 180.0,
      "end_time": 360.0
    }
  ]
}
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/youtube-chapter-splitter.git
cd youtube-chapter-splitter

# Install dependencies
cargo build

# Run tests
cargo test

# Run with debug output
RUST_LOG=debug cargo run --bin ytcs -- info --url "URL"
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [yt-dlp](https://github.com/yt-dlp/yt-dlp) - YouTube video downloader
- [ffmpeg](https://ffmpeg.org/) - Audio processing
- [egui](https://github.com/emilk/egui) - Immediate mode GUI framework
- [clap](https://github.com/clap-rs/clap) - Command line argument parser

## 🐛 Known Issues

- Age-restricted videos may require authentication
- Some videos with DRM protection cannot be downloaded
- Download links are valid for 6 hours only

## 🔮 Future Improvements

- [ ] Parallel track processing
- [ ] Support for more audio formats (FLAC, OGG)
- [ ] Album artwork embedding
- [ ] Playlist batch processing
- [ ] Download progress bar in CLI
- [ ] Caching system for repeated downloads
- [ ] Custom output filename templates

## 📞 Support

If you encounter any issues or have questions:
- Open an issue on GitHub
- Check existing issues for solutions
- Refer to the documentation

---

**Made with ❤️ and Rust**
