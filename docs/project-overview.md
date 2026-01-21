# Project Overview

## YouTube Chapter Splitter (ytcs)

**Type:** CLI Application
**Language:** Rust 2021 (1.70+)
**Binary:** `ytcs`
**Version:** 0.14.5

## Purpose

`ytcs` is a command-line tool that downloads YouTube videos, extracts audio as MP3, and splits it into individual tracks based on chapters. It automatically embeds metadata (artist, album, track numbers) and cover art.

## Core Features

- **MP3 Download**: High-quality audio (192 kbps default)
- **Chapter Detection**: YouTube chapters, description parsing, or silence-based fallback
- **Chapter Refinement**: Silence detection for precise split points (±5 second windows)
- **Metadata Embedding**: Complete ID3 tags with cover art
- **Progress Tracking**: Real-time download progress with percentage, speed, and ETA
- **Persistent Configuration**: `~/.config/ytcs/config.toml`
- **Playlist Support**: Interactive playlist handling

## Technology Stack

| Category | Technology | Version | Purpose |
|----------|-----------|---------|---------|
| **Language** | Rust | 2021 edition (1.70+) | Core application |
| **CLI Framework** | clap | 4.5 | Argument parsing |
| **Audio Processing** | ffmpeg (external) | - | Splitting, format conversion |
| **Silence Detection** | ffmpeg silencedetect | - | Chapter refinement |
| **MP3/ID3 Tags** | lofty | 0.22 | Metadata embedding |
| **Video Download** | yt-dlp (external) | - | YouTube extraction |
| **Progress Bars** | indicatif | 0.17 | Terminal progress |
| **Config Parsing** | toml | 0.8 | Configuration files |
| **Serialization** | serde | 1.0 | JSON/TOML handling |
| **Regex** | regex | 1.10 | Chapter parsing |
| **Error Handling** | thiserror | 1.0 | Custom error types |
| **HTTP Client** | ureq | 2.10 | Simple HTTP requests |
| **Path Management** | dirs | 5.0 | Config directory location |

## Architecture Pattern

**Pipeline Architecture** - Sequential 6-step process:

1. **Playlist Detection** → Prompt user for behavior
2. **Video Info Fetch** → Get metadata via yt-dlp
3. **Output Directory Setup** → Create target folder
4. **Download Assets** → Cover + audio via yt-dlp
5. **Chapter Detection** → 3-level fallback: YouTube → description → silence
6. **Track Splitting** → ffmpeg split + lofty metadata

## Repository Structure

- **Type:** Monolith (single cohesive codebase)
- **Source:** 17 Rust modules in `src/`
- **Tests:** 16 test files in `tests/`
- **Benchmarks:** Criterion in `benches/`

## Design Philosophy

- **Pragmatic**: No frills, just what matters
- **Direct**: Clear info without detours
- **Classy**: Elegant without being flashy

## External Dependencies

The project requires two external tools:

1. **yt-dlp** - For downloading YouTube videos/audio
2. **ffmpeg** - For audio processing and splitting

## Documentation Links

- [Architecture](./architecture.md) - System design and module organization
- [Source Tree](./source-tree-analysis.md) - Directory structure
- [Component Inventory](./component-inventory.md) - Module details
- [Development Guide](./development-guide.md) - Build and test instructions
