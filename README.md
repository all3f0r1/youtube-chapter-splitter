# YouTube Chapter Splitter (ytcs)

A simple and powerful Rust CLI tool to download YouTube videos, extract audio to MP3, and automatically split them into individual tracks based on chapters.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

## ✨ Features

- 🎵 **Download YouTube audio** in high-quality MP3 format
- 🖼️ **Download album artwork** automatically (cover.jpg)
- 📑 **Automatic chapter detection** from YouTube video metadata
- 🔇 **Silence detection fallback** for videos without chapters
- ✂️ **Smart audio splitting** with proper ID3 metadata tags
- 🎨 **Clean folder names** with intelligent formatting
- ⚡ **Dependency checking** with automatic installation prompts
- 🧹 **URL cleaning** - automatically removes playlist and extra parameters

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
git clone https://github.com/all3f0r1/youtube-chapter-splitter.git
cd youtube-chapter-splitter

# Build the project
cargo build --release

# The binary will be in target/release/ytcs
```

### Usage

**Simple syntax:**

```bash
ytcs <YOUTUBE_URL> [--output <DIR>]
```

**Examples:**

```bash
# Download and split a YouTube video
ytcs "https://www.youtube.com/watch?v=28vf7QxgCzA"

# Specify output directory
ytcs "https://www.youtube.com/watch?v=28vf7QxgCzA" --output ~/Music

# URL cleaning works automatically (removes &list=, &start_radio=, etc.)
ytcs "https://www.youtube.com/watch?v=28vf7QxgCzA&list=RD28vf7QxgCzA&start_radio=1"
```

## 📊 Example Output

```
=== YouTube Chapter Splitter ===

Fetching video information...
Title: MARIGOLD - Oblivion Gate [Full Album] (70s Psychedelic Blues Acid Rock)
Duration: 29m 29s
Tracks found: 5

Downloading album artwork...
✓ Artwork saved: output/Marigold - Oblivion Gate/cover.jpg

Downloading audio...
✓ Audio downloaded: output/Marigold - Oblivion Gate/temp_audio.mp3

Using YouTube tracks

Tracks to create:
  1. Oblivion Gate [5m 54s]
  2. Obsidian Throne [5m 35s]
  3. Crimson Citadel [5m 47s]
  4. Silver Spire [6m 30s]
  5. Eternal Pyre [5m 43s]

Splitting audio into 5 tracks...
  Track 1/5: Oblivion Gate
  Track 2/5: Obsidian Throne
  Track 3/5: Crimson Citadel
  Track 4/5: Silver Spire
  Track 5/5: Eternal Pyre
✓ Splitting completed successfully!

✓ Processing completed successfully!
Files created: 5
Directory: output/Marigold - Oblivion Gate
```

**Result:**
```
output/Marigold - Oblivion Gate/
├── cover.jpg
├── 01 - Oblivion Gate.mp3
├── 02 - Obsidian Throne.mp3
├── 03 - Crimson Citadel.mp3
├── 04 - Silver Spire.mp3
└── 05 - Eternal Pyre.mp3
```

## 🎯 How It Works

1. **URL Cleaning**: Removes playlist parameters and extra query strings
2. **Video Info**: Fetches video metadata including title, duration, and chapters
3. **Artwork Download**: Downloads the highest quality thumbnail as `cover.jpg`
4. **Audio Download**: Extracts audio in MP3 format using yt-dlp
5. **Track Detection**: Uses YouTube chapters or falls back to silence detection
6. **Audio Splitting**: Splits audio using ffmpeg with proper metadata
7. **Cleanup**: Removes temporary files and organizes output

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
- **Album**: Video title (cleaned)

### Silence Detection

If no chapters are found, the tool automatically detects silence periods:
- Threshold: -30 dB
- Minimum duration: 2.0 seconds
- Uses ffmpeg's `silencedetect` filter

## 📁 Project Structure

```
youtube-chapter-splitter/
├── src/
│   ├── main.rs          # CLI application
│   ├── lib.rs           # Library exports
│   ├── error.rs         # Error handling
│   ├── chapters.rs      # Chapter parsing and manipulation
│   ├── downloader.rs    # YouTube downloading and thumbnail
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

Test with a real video:

```bash
cargo run --release -- "https://www.youtube.com/watch?v=28vf7QxgCzA"
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/all3f0r1/youtube-chapter-splitter.git
cd youtube-chapter-splitter

# Install dependencies
cargo build

# Run tests
cargo test

# Run with debug output
RUST_LOG=debug cargo run -- "URL"
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [yt-dlp](https://github.com/yt-dlp/yt-dlp) - YouTube video downloader
- [ffmpeg](https://ffmpeg.org/) - Audio processing
- [clap](https://github.com/clap-rs/clap) - Command line argument parser
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP client for thumbnail download

## 🐛 Known Issues

- Age-restricted videos may require authentication
- Some videos with DRM protection cannot be downloaded
- Download links are valid for 6 hours only

## ❓ FAQ

**Q: Why not use a pure Rust solution instead of ffmpeg?**  
A: After extensive research, there is no viable pure Rust alternative to ffmpeg for MP3 encoding and audio manipulation. Libraries like Symphonia only support decoding, not encoding. ffmpeg remains the industry standard.

**Q: Can I use this for playlists?**  
A: Currently, the tool processes one video at a time. Playlist support may be added in the future.

**Q: What if a video has no chapters?**  
A: The tool automatically falls back to silence detection to identify track boundaries.

**Q: Can I customize silence detection parameters?**  
A: Currently, the parameters are fixed (-30 dB threshold, 2.0s minimum duration). Custom parameters may be added in future versions.

## 📞 Support

If you encounter any issues or have questions:
- Open an issue on [GitHub](https://github.com/all3f0r1/youtube-chapter-splitter/issues)
- Check existing issues for solutions

---

**Made with ❤️ and Rust**
