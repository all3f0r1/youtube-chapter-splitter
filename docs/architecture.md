# Architecture

## System Overview

`ytcs` is a Rust CLI application that implements a **pipeline architecture** for downloading and splitting YouTube videos by chapters.

## Architecture Pattern

**Sequential Pipeline** - Data flows through a series of discrete stages:

```
┌─────────────────────────────────────────────────────────────────┐
│                         MAIN PIPELINE                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  1. PLAYLIST DETECTION                                           │
│     └─→ Check if URL is playlist → Prompt user                  │
│                                                                   │
│  2. VIDEO INFO FETCH                                             │
│     └─→ yt-dlp --dump-json → Parse metadata                      │
│                                                                   │
│  3. OUTPUT DIRECTORY SETUP                                       │
│     └─→ Create folder based on artist/album                      │
│                                                                   │
│  4. DOWNLOAD ASSETS                                              │
│     ├─→ Download thumbnail (WebP → JPEG conversion)              │
│     └─→ Download audio (4-level format fallback)                 │
│                                                                   │
│  5. CHAPTER DETECTION (3-level fallback)                         │
│     ├─→ Level 1: YouTube chapters metadata                       │
│     ├─→ Level 2: Description parsing (regex)                      │
│     └─→ Level 3: Silence detection (ffmpeg silencedetect)        │
│                                                                   │
│  6. TRACK SPLITTING                                               │
│     ├─→ Chapter refinement (optional, silence-based)             │
│     ├─→ ffmpeg split by timestamps                               │
│     └─→ lofty ID3 tag + cover art embedding                      │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

## Technology Stack

| Component | Technology | Purpose |
|-----------|-----------|---------|
| **Language** | Rust 2021 | Core application logic |
| **CLI Framework** | clap 4.5 | Argument parsing |
| **Audio Processing** | ffmpeg (external) | Splitting, encoding, silence detection |
| **Metadata Writing** | lofty 0.22 | MP3/ID3 tags, cover art |
| **Video Download** | yt-dlp (external) | YouTube extraction |
| **Progress Display** | indicatif 0.17 | Terminal progress bars |
| **Config** | toml 0.8 | Configuration files |
| **Error Handling** | thiserror 1.0 | Custom error types |

## Module Architecture

```
┌───────────────────────────────────────────────────────────────┐
│                         main.rs                               │
│                    (CLI Entry Point)                          │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │  process_single_url(url)                               │  │
│  │    └─→ 6-step pipeline orchestration                   │  │
│  └─────────────────────────────────────────────────────────┘  │
└────────────┬──────────────────────────────────────────────────┘
             │
    ┌────────┴────────┐
    │                 │
    ▼                 ▼
┌─────────┐      ┌────────────┐
│ downloader │   │    config   │
│  (yt-dlp)  │   │   (TOML)    │
└─────┬─────┘      └────────────┘
      │
      ▼
┌───────────────────────────────────────┐
│        chapters module                │
│  ┌─────────────────────────────────┐  │
│  │  get_chapters_with_fallback()  │  │
│  │    1. YouTube metadata          │  │
│  │    2. Description parsing       │  │
│  │    3. Silence detection         │  │
│  └─────────────────────────────────┘  │
└───────┬───────────────┬────────────────┘
        │               │
        ▼               ▼
  ┌──────────┐    ┌──────────────────┐
  │   audio  │    │chapter_refinement│
  │  ffmpeg  │    │  ffmpeg silence  │
  └────┬─────┘    └──────────────────┘
       │
       ▼
  ┌────────────────────────────────┐
  │           utils                │
  │  - Title cleaning              │
  │  - Format strings              │
  │  - File operations             │
  └────────────────────────────────┘
```

## Data Flow

```
YouTube URL
     │
     ▼
┌─────────────────┐
│  downloader.rs  │ ──► VideoInfo { title, artist, album, chapters, thumbnail }
└─────────────────┘
     │
     ▼
┌─────────────────────────────┐
│ chapters_from_description   │ ──► Vec<Chapter> (if fallback needed)
└─────────────────────────────┘
     │
     ▼
┌──────────────────────────┐
│   chapter_refinement.rs   │ ──► Vec<Chapter> (adjusted timestamps)
└──────────────────────────┘
     │
     ▼
┌────────────────────────────────────────┐
│            audio.rs                    │
│  ┌──────────────────────────────────┐  │
│  │  1. ffmpeg split                 │  │
│  │  2. lofty metadata embedding     │  │
│  │  3. Cover art attachment         │  │
│  └──────────────────────────────────┘  │
└────────────────────────────────────────┘
     │
     ▼
MP3 Files with metadata
```

## Key Design Patterns

### RAII for Temporary Files

```rust
// temp_file.rs
pub struct TempFile {
    path: PathBuf,
    keep_on_drop: bool,
}

impl Drop for TempFile {
    fn drop(&mut self) {
        if !self.keep_on_drop {
            let _ = fs::remove_file(&self.path);
        }
    }
}
```

### Fallback Chain for Chapter Detection

```rust
// main.rs
fn get_chapters_with_fallback(...) -> Vec<Chapter> {
    // 1. Try YouTube chapters
    if let Some(chapters) = video_info.chapters {
        return chapters;
    }
    // 2. Parse from description
    if let Ok(chapters) = parse_chapters_from_description(&desc) {
        return chapters;
    }
    // 3. Detect silence points
    detect_silence_chapters(&audio_file)
}
```

### 4-Level Download Fallback

```rust
// downloader.rs
const FORMAT_SELECTORS: &[&str] = &[
    "bestaudio[ext=m4a]/bestaudio",  // Best M4A
    "140",                            // Standard M4A
    "bestaudio",                      // Generic best
    "",                               // Auto-select
];
```

## Error Handling

```rust
// error.rs
pub enum YtcsError {
    DownloadFailed(String),
    AudioProcessingFailed(String),
    ChapterDetectionFailed,
    ConfigError(String),
    IoError(String),
    // ... with thiserror derive
}
```

## Configuration Architecture

```
~/.config/ytcs/config.toml
         │
         ▼
   config::Config
         │
    ┌────┴────┐
    │         │
    ▼         ▼
  CLI args  Config file  ──► Merged Config
```

## Testing Architecture

```
tests/
├── Unit Tests           → Test individual functions
├── Integration Tests    → Test module interactions
└── E2E Tests           → Test complete workflows
```

## External Dependencies

| Tool | Purpose | Called From |
|------|---------|-------------|
| yt-dlp | Video/audio download | `downloader.rs` |
| ffmpeg | Splitting, silence detection | `audio.rs`, `chapter_refinement.rs` |

Both are verified at startup and cached paths used throughout execution.
