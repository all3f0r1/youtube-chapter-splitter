# YouTube Chapter Splitter (ytcs)

A simple and powerful Rust CLI tool to download YouTube videos, extract audio as **MP3, Opus, or M4A**, and automatically split them into individual tracks based on chapters.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-0.15.12-blue.svg)](https://github.com/all3f0r1/youtube-chapter-splitter/releases)

## ✨ Features

- 🎵 **Download YouTube audio** as MP3, Opus, or M4A at configurable bitrate (`audio_format` + `audio_quality` in config)
- 🖼️ **Download album artwork** automatically with embedded cover art in MP3 tags
- 📑 **Chapter detection** — YouTube JSON chapters, then timestamps in the video description, then silence detection
- 🎯 **Silence refinement** — on by default (`refine_chapters`); tunable window / dB / min-silence in config; `--refine-chapters` forces it on for a run if you turned it off in config
- ✂️ **Smart audio splitting** with complete ID3 metadata tags (title, artist, album, track number, cover art)
- 🎨 **Clean folder names** with intelligent formatting (removes brackets, pipes, capitalizes)
- 📁 **Smart default output** to ~/Music directory (cross-platform)
- 🎯 **Force artist/album names** with CLI options
- 📋 **Playlist URLs** — `playlist_behavior` in config: single video (strip `list=`), full playlist, or ask each time; optional `playlist_prefix_index` for `01-`… folder prefixes
- 📝 **`.m3u` playlist** — optional `create_playlist` in config writes `playlist.m3u` after splitting
- 🔁 **`overwrite_existing`** — config option controls replacing existing track files
- ⚡ **Dependency checking** with automatic installation prompts
- 🧹 **Canonical watch URLs** — `youtu.be` and `watch?v=` are normalized via the video ID
- 🪶 **Lightweight binary** (6.3 MB) with minimal dependencies

## 🚀 Quick Start

### Prerequisites

The application checks the three external tools below at startup and offers to install the missing ones (behavior controlled by the `dependency_auto_install` config key: `prompt` / `always` / `never`).

- **yt-dlp** — YouTube metadata + download backend.
  ```bash
  pip install yt-dlp
  ```

- **ffmpeg** — audio extraction, splitting, and silence detection.
  - Linux: `sudo apt install ffmpeg`
  - macOS: `brew install ffmpeg`
  - Windows: download from [ffmpeg.org](https://ffmpeg.org/download.html)

- **deno** — JavaScript runtime required by yt-dlp to solve YouTube's `n` challenge. Without deno, yt-dlp only exposes image formats and audio download fails with *"Requested format is not available"*.
  - Linux / macOS: `curl -fsSL https://deno.land/install.sh | sh`, then add `~/.deno/bin` to `PATH`:
    ```bash
    echo 'export PATH="$HOME/.deno/bin:$PATH"' >> ~/.bashrc && source ~/.bashrc
    ```
  - Windows: `irm https://deno.land/install.ps1 | iex`

On first run `ytcs` also asks yt-dlp to fetch the **EJS challenge-solver script** from GitHub (passed via `--remote-components ejs:github`). No manual step is needed; the script is cached by yt-dlp after the first successful run.

> **Authentication (optional but often required).** YouTube frequently gates requests coming from CLI clients with *"Sign in to confirm you're not a bot"* / HTTP 429. To pass a signed-in session:
> - run `ytcs config` and set **Cookies from browser** to `chrome`, `firefox`, `brave`, … (for LibreWolf or a custom profile: `firefox:/path/to/profile`), **or**
> - export cookies to `~/.config/ytcs/cookies.txt` (e.g. with the *cookies.txt* browser extension).

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

**Download and split:**

```bash
ytcs <YOUTUBE_URL> [OPTIONS]
```

**Configuration (interactive wizard; Enter keeps each current value):**

```bash
ytcs config
ytcs config --show    # print ~/.config/ytcs/config.toml values and exit
```

Settings are stored in `~/.config/ytcs/config.toml` (or `$XDG_CONFIG_HOME/ytcs/config.toml`). The wizard is created on first `ytcs config` or first download.

**Options (download command):**
- `-o, --output <DIR>` - Output directory (overrides `default_output_dir` in config)
- `-a, --artist <ARTIST>` - Force artist name (overrides auto-detection)
- `-A, --album <ALBUM>` - Force album name (overrides auto-detection)
- `--refine-chapters` - Force silence-based chapter refinement for this run (default in config is on; set `refine_chapters = false` to skip the extra ffmpeg pass)
- `--dry-run` - Show target output folder and chapter plan only (no download or split)
- `-q`, `--quiet` - Suppress tree/progress output (still prints each album output path on its own line)
- `--no-cover` - Skip thumbnail download for this run (overrides `download_cover`)
- `--skip-download` - Use existing `temp_audio.<ext>` in the album folder if non-empty instead of yt-dlp
- `--non-interactive` - Never read from stdin; fail instead of prompting for a playlist choice, missing artist/album, dependency install, or a yt-dlp update (see [Exit codes](#exit-codes))

**Examples:**

```bash
# Download and split a YouTube video (saves to ~/Music)
ytcs "https://www.youtube.com/watch?v=28vf7QxgCzA"

# Specify custom output directory
ytcs "https://www.youtube.com/watch?v=28vf7QxgCzA" --output ~/Downloads

# Force artist and album names
ytcs "https://www.youtube.com/watch?v=..." -a "Pink Floyd" -A "Dark Side of the Moon"

# With default playlist_behavior (video_only), only the current video is used even if list= is present
ytcs "https://www.youtube.com/watch?v=28vf7QxgCzA&list=RD28vf7QxgCzA&start_radio=1"
```

Use `ytcs config` and set **playlist behavior** to `playlist_only` (or `ask`) to download every entry from a playlist URL. Plain `youtube.com/playlist?list=…` links require `playlist_only` or answering **y** when prompted.

**Important:** Always put URLs in quotes to avoid shell interpretation of special characters:
```bash
ytcs "URL"  # ✅ Correct
ytcs URL    # ❌ May cause issues with & characters
```

### Exit codes

For scripting/CI, `ytcs` uses distinct exit codes:
- `0` — success.
- `1` — any other error (download failure, ffmpeg error, invalid config, …).
- `2` — the run needed interactive input but `--non-interactive` was set (ambiguous playlist URL with `playlist_behavior = ask`, undetectable artist/album, a missing dependency with `dependency_auto_install = prompt`, or a yt-dlp update prompt). The error message says which config setting or flag to change.

## 📊 Example Output

```
ytcs v0.15.12

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

### Configuration file

Run `ytcs config` to set output folder, MP3 bitrate (128/192/320), cover download, filename/folder templates (`%n`, `%t`, `%a`, `%A`), cookies browser, download timeouts, retries, dependency install behavior, and yt-dlp auto-update options.

Without a custom `default_output_dir`, albums go to the system Music folder (`~/Music` on Linux/macOS, `%USERPROFILE%\Music` on Windows). The `-o` flag still overrides that for a single run.

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

Two separate silence-detection passes exist:
- **Fallback track detection** (no YouTube chapters and no usable description timestamps): fixed -30 dB threshold, 2.0s minimum duration, via ffmpeg's `silencedetect` filter.
- **Boundary refinement** (`refine_chapters` in config, on by default when chapters/description timestamps *are* available): snaps each cut to the nearest detected silence within `refine_silence_window` seconds, using `refine_noise_db` / `refine_min_silence` as the detection thresholds.

## 📁 Project Structure

```
youtube-chapter-splitter/
├── src/
│   ├── main.rs                       # CLI application (arg parsing, orchestration)
│   ├── lib.rs                        # Library exports
│   ├── error.rs                      # YtcsError, MissingToolsError
│   ├── error_handler.rs              # Centralized user-facing error reporting
│   ├── config.rs                     # config.toml load/save/validate + wizard
│   ├── chapters.rs                   # Chapter struct, JSON chapter parsing
│   ├── chapters_from_description.rs  # Chapter timestamps parsed from descriptions
│   ├── chapter_refinement.rs         # Silence-based chapter boundary refinement
│   ├── downloader.rs                 # yt-dlp metadata/download, thumbnail fetch
│   ├── audio.rs                      # ffmpeg splitting, ID3 tagging, silence detection
│   ├── playlist.rs                   # Playlist URL detection and expansion
│   ├── cookie_helper.rs              # Browser-cookie authentication
│   ├── temp_file.rs                  # RAII temporary-file cleanup
│   ├── progress.rs                   # Progress bar utilities
│   ├── yt_dlp_progress.rs            # Real-time yt-dlp download progress parsing
│   ├── yt_dlp_update.rs              # yt-dlp update helpers
│   ├── ytdlp_helper.rs               # yt-dlp version check / auto-update
│   ├── ytdlp_error_parser.rs         # Friendly yt-dlp error messages
│   ├── dependency/                   # Dependency detection and installation
│   ├── ui.rs                         # Terminal output (tree view, progress, prompts)
│   └── utils.rs                      # Formatting, filename/title sanitization
├── tests/                            # Integration tests (one file per concern)
├── examples/
│   ├── basic_usage.rs                # Programmatic usage example
│   └── chapters_example.json         # Sample chapters file
├── .github/workflows/                # CI (test/fmt/clippy/build) and release
├── Cargo.toml                        # Dependencies and configuration
├── CHANGELOG.md                      # Version history
├── README.md                         # This file
├── LICENSE                           # MIT License
└── .gitignore                        # Git ignore rules
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
A: Yes. Set **playlist behavior** in `ytcs config` to `playlist_only` to always expand a playlist URL into every video, or `ask` to be prompted each time (with `--non-interactive`, `ask` fails instead of prompting — pick `playlist_only` or `video_only` for scripted use). The default, `video_only`, downloads just the current video and strips the `list=` parameter. `playlist_prefix_index` prefixes each album folder with `01-`, `02-`, … to avoid name clashes across a batch.

**Q: What if a video has no chapters?**  
A: The tool tries chapter timestamps in the video description next, then falls back to silence detection to identify track boundaries.

**Q: Can I customize silence detection parameters?**  
A: Yes, via `ytcs config`: `refine_silence_window`, `refine_noise_db`, and `refine_min_silence` control the silence-refinement pass (defaults: ±5s window, -35 dB, 1.2s minimum). The initial fallback detection (when there are no chapters or description timestamps at all) uses fixed -30 dB / 2.0s.

**Q: How do I avoid the [1], [2] background job messages?**  
A: Always put the URL in quotes: `ytcs "URL"` instead of `ytcs URL`. The `&` character in URLs is interpreted by the shell as a background job operator.

**Q: Does the cover art appear in my music player?**  
A: Yes! The album artwork is automatically embedded in each MP3 file using the `lofty` Rust library. It works with iTunes, VLC, foobar2000, Windows Media Player, and most mobile music apps.

## 📈 Changelog

See [CHANGELOG.md](CHANGELOG.md) for the full, per-version history — the highlights below are only a snapshot and will drift out of date, so treat CHANGELOG.md as the source of truth.

- **v0.15.12**: Chapter-refinement boundaries can no longer gap/overlap, `--skip-download` no longer deletes the reused file, per-track splitting is atomic (validated + temp-file + rename), and a new `--non-interactive` flag with dedicated exit codes.
- **v0.15.11**: Hardened thumbnail download (retry on transient 5xx, response-size cap, no sensitive URLs/paths in user-facing errors).
- **v0.15.10 / v0.15.9**: `deno` is now a checked, auto-installable dependency; yt-dlp's EJS `n`-challenge solver is fetched automatically (required for YouTube audio downloads).
- **v0.15.7**: Configurable output format (`mp3` / `opus` / `m4a`), silence-refinement tuning, `playlist_prefix_index`, `--dry-run` / `--quiet` / `--no-cover` / `--skip-download`.
- **v0.15.5**: Interactive `ytcs config` wizard, tree-style CLI output.

## 📞 Support

If you encounter any issues or have questions:
- Open an issue on [GitHub](https://github.com/all3f0r1/youtube-chapter-splitter/issues)
- Check existing issues for solutions

---

**Made with ❤️ and Rust**
