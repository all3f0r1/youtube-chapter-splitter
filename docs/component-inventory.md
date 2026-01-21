# Component Inventory

## Modules Overview

The application consists of 17 Rust modules organized by responsibility.

### Core Pipeline

| Module | LOC | Purpose | Dependencies |
|--------|-----|---------|--------------|
| `main.rs` | ~700 | CLI entry point, argument parsing, orchestration | clap, colored |
| `downloader.rs` | ~500 | yt-dlp integration, video info, audio download | ureq, serde_json |
| `audio.rs` | ~600 | ffmpeg splitting, MP3 encoding, ID3 tag embedding | lofty |
| `chapters.rs` | ~180 | Chapter struct, core operations | serde |
| `chapter_refinement.rs` | ~380 | Silence detection for precise chapter boundaries | ffmpeg |

### Chapter Detection

| Module | LOC | Purpose | Dependencies |
|--------|-----|---------|--------------|
| `chapters_from_description.rs` | ~290 | Parse timestamps from video descriptions | regex |
| `playlist.rs` | ~250 | Playlist URL detection and enumeration | - |

### Configuration & State

| Module | LOC | Purpose | Dependencies |
|--------|-----|---------|--------------|
| `config.rs` | ~420 | TOML config file management | toml, serde, dirs |
| `temp_file.rs` | ~130 | RAII wrapper for automatic cleanup | - |

### User Interface

| Module | LOC | Purpose | Dependencies |
|--------|-----|---------|--------------|
| `ui.rs` | ~130 | Terminal UI components | colored |
| `progress.rs` | ~110 | Progress bar utilities | indicatif |
| `yt_dlp_progress.rs` | ~350 | Real-time download progress parsing | regex |

### Utilities

| Module | LOC | Purpose | Dependencies |
|--------|-----|---------|--------------|
| `utils.rs` | ~530 | Title cleaning, formatting, string utilities | regex |
| `cookie_helper.rs` | ~95 | YouTube authentication via browser cookies | - |
| `ytdlp_error_parser.rs` | ~200 | Parse yt-dlp error messages | regex |
| `error.rs` | ~65 | Custom error types | thiserror |

## Data Structures

### Chapter

```rust
pub struct Chapter {
    pub start_time: f64,
    pub end_time: Option<f64>,
    pub title: String,
}
```

### VideoInfo

```rust
pub struct VideoInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub chapters: Vec<Chapter>,
    pub thumbnail_url: String,
    // ... additional fields
}
```

## External Integrations

| Tool | Purpose | Integration Point |
|------|---------|-------------------|
| yt-dlp | Video download, metadata | `downloader.rs` |
| ffmpeg | Audio splitting, silence detection | `audio.rs`, `chapter_refinement.rs` |
| lofty | MP3/ID3 tag writing | `audio.rs` |

## Reusable Patterns

### RAII TempFile Pattern

```rust
let temp = TempFile::new(&path);
// ... use the file ...
// File automatically deleted when `temp` goes out of scope
temp.keep(); // Optional: prevent deletion
```

### Chapter Detection Fallback

```
YouTube Chapters → Description Parsing → Silence Detection
     (1)                  (2)                    (3)
```

### Download Format Selector

4-level fallback for reliability:
1. `bestaudio[ext=m4a]/bestaudio`
2. `140`
3. `bestaudio`
4. Auto-selection
