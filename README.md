# YouTube Chapter Splitter (ytcs)

A simple and powerful Rust CLI tool to download YouTube videos, extract audio to MP3, and automatically split them into individual tracks based on chapters.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-0.15.0-blue.svg)](https://github.com/all3f0r1/youtube-chapter-splitter/releases)

## ✨ Features

- 🎵 **Download YouTube audio** in high-quality MP3 format (192 kbps)
- 🖼️ **Download album artwork** automatically with embedded cover art in MP3 tags
- 📑 **Automatic chapter detection** from YouTube video metadata
- 🔇 **Silence detection fallback** for videos without chapters
- ✂️ **Smart audio splitting** with complete ID3 metadata tags (title, artist, album, track number, cover art)
- 🎨 **Clean folder names** with intelligent formatting (removes brackets, pipes, capitalizes)
- 📁 **Smart default output** to ~/Music directory (cross-platform)
- 🎯 **Force artist/album names** with CLI options
- ⚡ **Dependency checking** with automatic installation prompts
- 🧹 **URL cleaning** - automatically removes playlist and extra parameters
- 🪶 **Lightweight binary** (6.3 MB) with minimal dependencies

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
- `-o, --output <DIR>` - Output directory (default: ~/Music)
- `-a, --artist <ARTIST>` - Force artist name (overrides auto-detection)
- `-A, --album <ALBUM>` - Force album name (overrides auto-detection)

**Examples:**

```bash
# Download and split a YouTube video (saves to ~/Music)
ytcs "https://www.youtube.com/watch?v=28vf7QxgCzA"

# Specify custom output directory
ytcs "https://www.youtube.com/watch?v=28vf7QxgCzA" --output ~/Downloads

# Force artist and album names
ytcs "https://www.youtube.com/watch?v=..." -a "Pink Floyd" -A "Dark Side of the Moon"

# URL cleaning works automatically (removes &list=, &start_radio=, etc.)
ytcs "https://www.youtube.com/watch?v=28vf7QxgCzA&list=RD28vf7QxgCzA&start_radio=1"
```

**Important:** Always put URLs in quotes to avoid shell interpretation of special characters:
```bash
ytcs "URL"  # ✅ Correct
ytcs URL    # ❌ May cause issues with & characters
```

## 📊 Example Output

```
ytcs v0.15.0

▶ Fetching video information
▶ Marigold - Oblivion Gate
  ├─ Duration   29m 29s
  ├─ Tracks     5
  ├─ Artist     Marigold (detected)
  └─ Album      Oblivion Gate (detected)

▶ Downloading artwork
  └─ Saved ~/Music/Marigold - Oblivion Gate/cover.jpg

▶ Downloading audio
[========================================] Downloading audio...
  └─ Saved ~/Music/Marigold - Oblivion Gate/temp_audio.mp3

▶ Splitting into 5 tracks
  ├─ 1  Oblivion Gate [5m 54s]
  ├─ 2  Obsidian Throne [5m 35s]
  ├─ 3  Crimson Citadel [5m 47s]
  ├─ 4  Silver Spire [6m 30s]
  └─ 5  Eternal Pyre [5m 43s]

✓ ~/Music/Marigold - Oblivion Gate
```

**Result:**
```
~/Music/Marigold - Oblivion Gate/
├── cover.jpg
├── 01. Oblivion Gate.mp3
├── 02. Obsidian Throne.mp3
├── 03. Crimson Citadel.mp3
├── 04. Silver Spire.mp3
└── 05. Eternal Pyre.mp3
```

## 🎯 How It Works

1. **URL Cleaning**: Removes playlist parameters and extra query strings
2. **Video Info**: Fetches video metadata including title, duration, and chapters
3. **Artwork Download**: Downloads the highest quality thumbnail as `cover.jpg`
4. **Audio Download**: Extracts audio in MP3 format using yt-dlp
5. **Track Detection**: Uses YouTube chapters or falls back to silence detection
6. **Audio Splitting**: Splits audio using ffmpeg with proper metadata and embedded cover art
7. **Cleanup**: Removes temporary files and organizes output

## 🛠️ Advanced Features

### Default Output Directory

The application automatically saves to your system's Music directory:

- **Linux**: `~/Music` (e.g., `/home/username/Music`)
- **macOS**: `~/Music` (e.g., `/Users/username/Music`)
- **Windows**: `%USERPROFILE%\Music` (e.g., `C:\Users\username\Music`)

You can override this with the `-o` flag.

### Folder Name Cleaning

The application automatically cleans folder names:
- Removes `[...]` and `(...)` content and everything after `[FULL ALBUM]`
- Replaces `_`, `|`, and `/` with `-`
- Capitalizes words properly
- Removes duplicate track numbers from chapter titles

**Examples:**
```
Input:  "MARIGOLD - Oblivion Gate [Full Album] (70s Psychedelic Blues Acid Rock)"
Output: "Marigold - Oblivion Gate"

Input:  "PURPLE DREAMS - WANDERING SHADOWS (FULL ALBUM) | 70s Progressive/Psychedelic Rock"
Output: "Purple Dreams - Wandering Shadows"
```

### Complete ID3 Metadata Tagging

Each MP3 file includes comprehensive ID3v2.3 tags:
- **Title**: Track name (e.g., "Oblivion Gate")
- **Artist**: Auto-detected or forced (e.g., "Marigold")
- **Album**: Auto-detected or forced (e.g., "Oblivion Gate")
- **Track**: Track number / Total tracks (e.g., "1/5")
- **Cover Art**: ✅ Embedded album artwork (if downloaded)

**Music players like iTunes, VLC, foobar2000, and mobile apps will display the album artwork automatically!**

### Force Artist and Album Names

Override automatic detection when video titles are non-standard:

```bash
# Auto-detection from title
ytcs "URL"

# Force both artist and album
ytcs "URL" --artist "Pink Floyd" --album "The Dark Side of the Moon"

# Force only artist (album auto-detected)
ytcs "URL" -a "Led Zeppelin"

# Force only album (artist auto-detected)
ytcs "URL" -A "Houses of the Holy"
```

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
- [ureq](https://github.com/algesten/ureq) - Lightweight HTTP client for thumbnail download
- [dirs](https://github.com/dirs-dev/dirs-rs) - Cross-platform directory detection

## 🐛 Known Issues

- Age-restricted videos may require authentication
- Some videos with DRM protection cannot be downloaded
- Download links are valid for 6 hours only

## ❓ FAQ

**Q: Where are my files saved?**  
A: By default, files are saved to your Music directory (`~/Music` on Linux/macOS, `%USERPROFILE%\Music` on Windows). You can change this with the `-o` flag.

**Q: Why not use a pure Rust solution instead of ffmpeg?**  
A: We use ffmpeg for audio encoding and splitting (industry standard), but we use the Rust library `lofty` for adding metadata and album artwork. This hybrid approach gives us the best of both worlds: ffmpeg's robust audio processing and Rust's safe, efficient metadata handling.

**Q: Can I use this for playlists?**  
A: Currently, the tool processes one video at a time. Playlist support may be added in the future.

**Q: What if a video has no chapters?**  
A: The tool automatically falls back to silence detection to identify track boundaries.

**Q: Can I customize silence detection parameters?**  
A: Currently, the parameters are fixed (-30 dB threshold, 2.0s minimum duration). Custom parameters may be added in future versions.

**Q: How do I avoid the [1], [2] background job messages?**  
A: Always put the URL in quotes: `ytcs "URL"` instead of `ytcs URL`. The `&` character in URLs is interpreted by the shell as a background job operator.

**Q: Does the cover art appear in my music player?**  
A: Yes! The album artwork is automatically embedded in each MP3 file using the `lofty` Rust library. It works with iTunes, VLC, foobar2000, Windows Media Player, and most mobile music apps.

## 📈 Changelog

See [CHANGELOG.md](CHANGELOG.md) for detailed version history.

### v0.15.0 (Latest)
- **UI Redesign**: Clean tree-style interface with "Pragmatic • Direct • Classy" philosophy
- **Metadata Prompts**: Interactive prompts when artist/album cannot be detected
- **Progressive Feedback**: Real-time track display during splitting
- **Simplified Output**: Removed excessive colors, bold section headers only
- **Better Alignment**: Vertically aligned track numbers and durations

## 📞 Support

If you encounter any issues or have questions:
- Open an issue on [GitHub](https://github.com/all3f0r1/youtube-chapter-splitter/issues)
- Check existing issues for solutions

---

**Made with ❤️ and Rust**
