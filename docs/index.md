# YouTube Chapter Splitter - Documentation Index

## Project Overview

- **Type:** CLI Application (Monolith)
- **Language:** Rust 2021 (1.70+)
- **Binary:** `ytcs`
- **Version:** 0.14.5
- **Purpose:** Download YouTube videos, extract MP3 audio, split by chapters with metadata

## Quick Reference

| Item | Value |
|------|-------|
| **Tech Stack** | Rust, clap, ffmpeg, yt-dlp, lofty |
| **Entry Point** | `src/main.rs` → `ytcs` binary |
| **Architecture** | Sequential Pipeline (6-step process) |
| **External Deps** | yt-dlp, ffmpeg |
| **Config Location** | `~/.config/ytcs/config.toml` |

## Generated Documentation

### Core Documentation

- [Project Overview](./project-overview.md) - Purpose, features, and technology summary
- [Architecture](./architecture.md) - System design, data flow, and patterns
- [Source Tree Analysis](./source-tree-analysis.md) - Directory structure and module organization
- [Component Inventory](./component-inventory.md) - Module details and responsibilities
- [Development Guide](./development-guide.md) - Build, test, and contribution instructions

### Existing Documentation

- [README](../README.md) - User documentation and usage guide
- [CHANGELOG](../CHANGELOG.md) - Version history and changes
- [COOKIES_SETUP](../COOKIES_SETUP.md) - YouTube authentication setup
- [CLAUDE.md](../CLAUDE.md) - AI agent development instructions

## Getting Started

### For Users

```bash
# Install prerequisites
sudo apt install yt-dlp ffmpeg  # Ubuntu/Debian
brew install yt-dlp ffmpeg      # macOS

# Download and install binary
wget https://github.com/all3f0r1/youtube-chapter-splitter/releases/latest/download/ytcs-x86_64-unknown-linux-gnu.tar.gz
tar xzf ytcs-x86_64-unknown-linux-gnu.tar.gz
sudo mv ytcs /usr/local/bin/

# Download a video
ytcs "https://www.youtube.com/watch?v=..."
```

### For Developers

```bash
# Clone repository
git clone https://github.com/all3f0r1/youtube-chapter-splitter.git
cd youtube-chapter-splitter

# Build
cargo build --release

# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- "URL"
```

### For AI-Assisted Development

This documentation provides comprehensive context for:

1. **Brownfield PRD Creation** - Refer to `architecture.md` for system constraints
2. **Feature Implementation** - See `component-inventory.md` for module organization
3. **Code Understanding** - Check `source-tree-analysis.md` for file locations
4. **Testing Strategy** - Review `development-guide.md` for test patterns

## Architecture Summary

```
┌─────────────────────────────────────────────────────────┐
│                    6-STEP PIPELINE                      │
├─────────────────────────────────────────────────────────┤
│  1. Playlist Detection → Prompt user                   │
│  2. Video Info Fetch → yt-dlp metadata                │
│  3. Output Setup → Create directory                    │
│  4. Download Assets → Cover + audio                    │
│  5. Chapter Detection → 3-level fallback               │
│  6. Track Splitting → ffmpeg + ID3 tags                │
└─────────────────────────────────────────────────────────┘
```

## Module Organization

```
src/
├── main.rs                  # CLI entry point
├── downloader.rs            # yt-dlp integration
├── audio.rs                 # ffmpeg + lofty
├── chapters.rs              # Chapter structures
├── chapter_refinement.rs    # Silence detection
├── chapters_from_description.rs  # Description parsing
├── config.rs                # Configuration management
├── utils.rs                 # Utilities
├── temp_file.rs             # RAII temp files
├── playlist.rs              # Playlist handling
├── progress.rs              # Progress bars
├── ui.rs                    # Terminal UI
├── yt_dlp_progress.rs       # Download progress
├── ytdlp_error_parser.rs    # Error parsing
├── cookie_helper.rs         # YouTube auth
├── error.rs                 # Error types
└── lib.rs                   # Library exports
```

## Key Design Patterns

| Pattern | Implementation |
|---------|---------------|
| **RAII** | `TempFile` for automatic cleanup |
| **Fallback Chain** | Chapter detection (YouTube → description → silence) |
| **Format Selector** | 4-level download fallback |
| **Pipeline** | Sequential 6-step processing |

## External Dependencies

| Tool | Purpose | Integration |
|------|---------|-------------|
| yt-dlp | YouTube download | `downloader.rs` |
| ffmpeg | Audio processing | `audio.rs`, `chapter_refinement.rs` |

---

**Documentation Generated:** 2025-01-20
**Scan Level:** Deep
**Workflow:** document-project v1.2.0
