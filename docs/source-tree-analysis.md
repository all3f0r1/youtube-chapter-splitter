# Source Tree Analysis

## Project Root

```
youtube-chapter-splitter/
├── src/                    # Main source code (Rust)
│   ├── main.rs            # Entry point - CLI parser and main logic
│   ├── lib.rs             # Library exports and module declarations
│   ├── audio.rs           # Audio processing: splitting, MP3 encoding, ID3 tags
│   ├── chapter_refinement.rs    # Silence detection for precise chapter boundaries
│   ├── chapters_from_description.rs  # Parse chapters from video descriptions
│   ├── chapters.rs        # Chapter struct and core operations
│   ├── config.rs          # Configuration file management (TOML)
│   ├── cookie_helper.rs   # YouTube authentication via browser cookies
│   ├── downloader.rs      # yt-dlp integration for video/audio download
│   ├── error.rs           # Error types with thiserror
│   ├── playlist.rs        # Playlist detection and handling
│   ├── progress.rs        # Progress bar utilities (indicatif)
│   ├── temp_file.rs       # RAII wrapper for automatic temp file cleanup
│   ├── ui.rs              # Terminal UI components
│   ├── utils.rs           # Utility functions (title cleaning, formatting)
│   ├── ytdlp_error_parser.rs   # Parse yt-dlp error messages
│   └── yt_dlp_progress.rs # Real-time download progress parsing
├── tests/                 # Integration and unit tests
│   ├── test_audio_functions.rs
│   ├── test_audio_tags.rs
│   ├── test_chapters_numbered_format.rs
│   ├── test_chapters.rs
│   ├── test_config_options.rs
│   ├── test_downloader_functions.rs
│   ├── test_downloader.rs
│   ├── test_edge_cases_advanced.rs
│   ├── test_error.rs
│   ├── test_integration_e2e.rs
│   ├── test_main.rs
│   ├── test_playlist.rs
│   ├── test_refactored_helpers.rs
│   ├── test_stdin_handling.rs
│   ├── test_url_validation.rs
│   └── test_utils_edge_cases.rs
├── benches/               # Performance benchmarks
│   └── performance_benchmarks.rs
├── Cargo.toml             # Rust project manifest
├── Cargo.lock             # Dependency lock file
├── README.md              # User documentation
├── CHANGELOG.md           # Version history
├── CLAUDE.md              # AI agent instructions
└── COOKIES_SETUP.md       # Cookie authentication guide
```

## Critical Folders

| Folder | Purpose |
|--------|---------|
| `src/` | All application source code |
| `tests/` | Test suite (16 test files) |
| `benches/` | Criterion benchmarks |

## Entry Points

| File | Purpose |
|------|---------|
| `src/main.rs` | CLI entry point, binary: `ytcs` |
| `src/lib.rs` | Library entry point, crate: `youtube_chapter_splitter` |

## Module Dependencies

```
main.rs
├── clap (CLI parsing)
├── colored (terminal colors)
└── youtube_chapter_splitter (lib)
    ├── audio
    │   ├── lofty (MP3/ID3)
    │   └── ffmpeg (via command)
    ├── chapter_refinement
    │   └── ffmpeg silencedetect
    ├── chapters_from_description
    │   └── regex (timestamp parsing)
    ├── config
    │   └── toml (config parsing)
    ├── downloader
    │   └── yt-dlp (external)
    ├── playlist
    └── utils
```

## External Dependencies

- **yt-dlp**: YouTube video download (external binary)
- **ffmpeg**: Audio processing and silence detection (external binary)
- **lofty**: MP3/ID3 tag writing
- **clap**: CLI argument parsing
- **indicatif**: Progress bars
- **regex**: Chapter parsing from descriptions
- **thiserror**: Error handling
- **serde/toml**: Configuration serialization
