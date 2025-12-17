# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.14.5] - 2024-12-17

### Fixed
- **Track Prefix Parsing**: Fixed parsing of track prefixes with parenthesis format (e.g., "1) Dunes of Dread" → "Dunes Of Dread")
- **Emoji Separator**: Fixed artist/album parsing for titles using emoji separators (e.g., "Black Crown Crows 👑 When We Two Parted")
- **Title Case Normalization**: Applied Title Case to all chapter titles, fixing ALL CAPS titles (e.g., "VOICES BENEATH THE RAIN" → "Voices Beneath The Rain")

### Added
- **New Function**: Added `capitalize_words()` utility function for Title Case transformation
- **Emoji Regex**: Added `RE_EMOJI_SEPARATOR` regex to detect emoji separators in video titles
- **Tests**: Added 4 new tests for the bug fixes (track prefix, uppercase titles, emoji separator)

### Technical Details
- Updated `RE_TRACK_PREFIX` regex to also match `N)` format: `^\s*(?:Track\s+)?\d+\s*[-.:)]\s*`
- `sanitize_title()` now applies Title Case capitalization after removing track prefixes
- `parse_artist_album()` now tries emoji-based splitting before dash-based splitting
- All tests updated to reflect new Title Case behavior

## [0.14.4] - 2024-12-03

### Fixed
- **Artist Detection**: Fixed artist detection for titles with stuck dashes (e.g., "Mammoth- Solar Crown Of Fire")
- **Title Truncation**: Fixed display of titles with unclosed brackets/parentheses (e.g., "Heat At The Edge Of The Mirror (psychedel")
- Now correctly normalizes all dash variations ("Artist-Album", "Artist- Album", "Artist -Album") to "Artist - Album"
- Artist is now properly extracted from the title instead of falling back to channel name
- Unclosed brackets/parentheses at end of string are now properly removed

### Added
- **Tests**: Added 5 new tests (3 for stuck dash normalization, 2 for unclosed brackets)

### Technical Details
- Improved `parse_artist_album` function to normalize dashes before parsing
- Split by dash, trim each part, filter empty parts, then rejoin with " - "
- Handles all edge cases: stuck left, stuck right, stuck both sides
- Enhanced `RE_BRACKETS` regex to match unclosed brackets/parentheses at end of string: `\[.*$|\(.*$`

## [0.14.3] - 2024-12-03

### Changed
- **UI Improvement**: Changed "Audio downloaded" to "Audio downloading" during download, then "✓ Audio downloaded" when complete
- **Genre Tag Cleanup**: Automatically remove genre tags from folder names (e.g., "70s Psychedelic • Progressive Rock")
- **Minimalist Fallback Messages**: Replaced aggressive log warnings with subtle progress bar messages (e.g., "option 1/4 failed")

### Added
- **Tests**: Added 3 new tests for genre tag cleanup functionality

### Technical Details
- Added `RE_GENRE_TAGS` regex to detect and remove decade-based genre tags
- Changed `log::warn!` to `log::debug!` for format selector failures
- Progress bar now shows minimal fallback status during download attempts

## [0.14.2] - 2024-12-03

### Changed
- **Complete English Translation**: All French comments and documentation translated to English
- **Documentation Improvements**: Enhanced docstrings with better examples and explanations
- **README Updates**: Added new features (logging, RAII, multi-platform binaries) to Features section
- **Version badges**: Updated README version badges to 0.14.2

### Fixed
- **Clippy Warning**: Fixed `last_error` unused assignment warning in downloader by adding explicit type annotation

### Note
- GitHub Actions workflow for multi-platform builds (Linux, macOS Intel, macOS Apple Silicon, Windows) already exists and is fully functional
- All future code and documentation will be written in English

## [0.14.1] - 2024-12-03

### Fixed
- **macOS compatibility**: Added build instructions for macOS users (Linux binaries don't work on macOS)
- Pre-compiled binaries are architecture-specific and won't run cross-platform

### Changed
- **Cargo.toml**: Added comprehensive metadata for crates.io publication
  - Added `homepage`, `documentation`, `rust-version` fields
  - Added `exclude` list to reduce package size
  - Improved description for better discoverability
- **Documentation**: Translated UI module comments to English
- **README**: Added "Build from Source" section with macOS-specific instructions
- **Cleanup**: Removed obsolete RELEASE_NOTES_*.md files (changelog is the single source of truth)

### Added
- Build instructions for macOS users in README
- Comprehensive Cargo.toml metadata for crates.io

### Note
- Full translation of all code comments to English will be completed in v0.15.0
- All future code and documentation will be written in English

## [0.14.0] - 2024-12-03

### Changed
- **MAJOR REFACTORING**: Refactored `process_single_url` from 240+ lines to ~60 lines
- Extracted 6 modular helper functions:
  - `handle_playlist_detection`: Gestion de la détection de playlist
  - `fetch_and_display_video_info`: Récupération et affichage des infos vidéo
  - `setup_output_directory`: Configuration du répertoire de sortie
  - `download_cover_and_audio`: Téléchargement cover + audio
  - `get_chapters_with_fallback`: Récupération des chapitres avec fallback
  - `split_into_tracks`: Découpage en pistes
- Improved code maintainability and readability (75% complexity reduction)
- Added 2 new structures: `VideoContext` and `DownloadedAssets`

### Added
- 10 new unit tests for refactored helper functions
- Documentation for all helper functions
- Test coverage for modular architecture

### Fixed
- Improved code organization and separation of concerns
- Better error handling in modular functions

## [0.13.0] - 2024-12-03

### Added
- **Extended logging**: Added structured logs in `audio.rs` and `chapters_from_description.rs`
  - Log chapter splitting progress and details
  - Log chapter parsing attempts and results
  - Track number and title logged for each split operation
- **RAII cover file management**: Cover files now use `TempFile` for automatic cleanup
  - Cover files are automatically deleted unless `--no-cover` is not used
  - Consistent with audio file management from v0.12.0
- **Debugging section in README**: New documentation section explaining logging usage
  - Examples for different log levels (debug, info, warn)
  - Instructions for saving logs to file
  - List of what gets logged

### Improved
- **Better observability**: More detailed logs throughout the pipeline
- **Consistent resource management**: All temporary files now use RAII pattern
- **Documentation**: README updated with debugging instructions

### Technical Details
- Added `log::info!` and `log::debug!` calls in audio splitting functions
- Added `log::info!` and `log::warn!` in chapter parsing
- Refactored cover file handling to use `TempFile` with `.keep()` method
- Updated README with logging examples and version badges

## [0.12.0] - 2024-12-03

### Added
- **Logging system**: Integrated `log` and `env_logger` for structured logging
  - Use `RUST_LOG=debug ytcs <url>` for debug logs
  - Use `RUST_LOG=info ytcs <url>` for info logs
  - Default level is `warn` for minimal output
- **RAII temp file management**: New `TempFile` struct for automatic cleanup of temporary files
  - Temporary audio files are now automatically deleted when out of scope
  - Prevents leftover files in case of errors or interruptions
- **Download timeout configuration**: Added `download_timeout` config option (default: 300s)
  - Can be configured via `ytcs set download_timeout <seconds>`
  - Set to 0 to disable timeout

### Improved
- **Better debugging**: Log messages throughout download and processing pipeline
- **Resource management**: Automatic cleanup of temporary files using RAII pattern
- **Error tracking**: Detailed logs for format selector fallbacks and failures

### Technical Details
- Added `log` (0.4) and `env_logger` (0.11) dependencies
- Created `temp_file` module with `TempFile` struct implementing `Drop` trait
- Added logging in `download_audio` for format selector attempts and results
- Removed manual `fs::remove_file` calls in favor of RAII cleanup

## [0.11.0] - 2024-12-03

### Improved
- **Code Quality**: Fixed Clippy warnings and improved code structure
- **Refactored**: `split_single_track` now uses `TrackSplitParams` struct (reduced from 9 to 1 parameter)
- **Refactored**: `progress.rs` module to eliminate code duplication with generic `create_progress` function
- **Documentation**: Added comprehensive examples and better docstrings for public APIs
- **Documentation**: Updated README.md with current version (0.11.0), features, and examples

### Added
- **Tests**: 7 new tests for numbered chapter format (`1 - Title (MM:SS)`)
- **Tests**: Unit tests for progress bar creation
- **Documentation**: Examples in docstrings for `download_audio`, `VideoInfo`, and progress functions
- **Documentation**: Detailed field documentation for `VideoInfo` struct

### Changed
- Format selectors now use const array instead of vec for better performance
- Progress bar tick rate extracted as constant (100ms)
- `ProgressType` enum added for better type safety

## [0.10.8] - 2024-12-02

### Fixed
- **Ultimate fallback:** Add a final fallback that doesn't specify any format, letting yt-dlp choose automatically
- Fixes cases where all explicit format selectors fail due to severe nsig extraction or SABR streaming issues
- Now tries 4 strategies: `bestaudio[ext=m4a]/bestaudio` → `140` → `bestaudio` → **no format (auto)**

### Technical Details
- When all explicit format selectors fail with "Requested format is not available", the downloader now tries one last time without specifying any format
- This allows yt-dlp to use its internal logic to select the best available format, bypassing format selection issues entirely
- The auto-selection fallback works even when YouTube's signature system is completely broken

## [0.10.7] - 2024-12-02

### Fixed
- **Download robustness:** Implement fallback mechanism for format selection to handle yt-dlp signature extraction issues
- Try multiple format selectors in order: `bestaudio[ext=m4a]/bestaudio` → `140` → `bestaudio`
- Improved error messages showing which format selector failed and why
- Better handling of YouTube SABR streaming and nsig extraction warnings

### Technical Details
- When the preferred format selector fails (e.g., due to signature issues), the downloader now automatically tries alternative formats
- Format 140 is YouTube's standard M4A audio format and works reliably even when signature extraction fails
- The generic `bestaudio` selector serves as a final fallback for edge cases

## [0.10.6] - 2024-12-02

### Changed
- **Performance:** Download audio directly in M4A format (`bestaudio[ext=m4a]`) instead of full video, significantly reducing download time
- **UI:** Changed "Downloading video" to "Downloading the album..."
- **UI:** Changed "Making an album out of the video" to "Audio downloaded" with progress bar
- **UI:** Changed "Making the album..." to "Splitting into the album..."

### Technical Details
- yt-dlp now uses `-f "bestaudio[ext=m4a]/bestaudio"` to download only the audio stream (typically format 140 on YouTube)
- FFmpeg still converts M4A to MP3 as before, but the initial download is much faster
- This change reduces bandwidth usage and speeds up the entire process

## [0.10.5] - 2024-12-01

### Added
- Progress bars for video download, audio conversion, and track splitting
- Direct URL support: `ytcs <URL>` now works without the `download` subcommand
- YouTube URL validation with clear error messages
- Track-by-track progress display with formatted names during splitting

### Changed
- Video title in UI now matches the output folder name format for consistency
- Track count display now shows "? tracks → silence detection mode" when using silence detection
- Track count display shows "checking description..." when checking video description for chapters
- "Downloading video" section now shows progress bar inline
- "Making an album out of the video" replaces "Audio downloaded" with progress bar
- "Making the album..." replaces "Making an album" for clarity
- Removed "Splitting tracks..." message, replaced with individual track progress
- Each track now shows formatted name (e.g., "01 - Artist - Title") with progress bar
- UI version updated to 0.10.5

### Fixed
- Improved user experience with real-time progress feedback
- Better error handling for invalid URLs
- Consistent formatting between UI display and file output

## [0.10.4] - 2024-11-30

### Added
- Fallback to channel name for artist detection when artist is not found in video title
- Support for track listing format in video descriptions: `1 - Title (0:00)`
- New test cases for channel name fallback and track number format

### Changed
- `parse_artist_album` function now accepts channel name as second parameter
- Artist detection now uses channel name (e.g., "HasvAlner") when title doesn't contain artist-album separator
- Chapter detection from description now supports track number format: `N - Track Title (MM:SS)`

### Fixed
- Videos without artist in title now use channel name instead of "Unknown Artist"
- Better support for music channels that use track listing format in descriptions

## [0.10.3] - 2024-11-27

### Added
- Automatic cookie extraction from browser via `cookies_from_browser` configuration option
- Support for extracting cookies from Chrome, Firefox, Safari, Edge, Chromium, Brave, Opera, and Vivaldi
- Intelligent error message parsing for yt-dlp errors with actionable suggestions
- User-friendly error messages for authentication, age-restriction, geo-blocking, and network issues
- New `ytdlp_error_parser` module for transforming technical errors into helpful messages
- New `cookie_helper` module for unified cookie management

### Changed
- yt-dlp errors now show clear, actionable messages instead of raw technical output
- Cookie authentication now prioritizes browser extraction over file-based cookies
- Error messages now include specific suggestions based on the type of error and current configuration

### Fixed
- Better error handling for member-only and private videos
- Improved error messages when cookies are expired or missing
- More helpful guidance for users encountering authentication issues

## [0.10.2] - 2024-11-27

### Added
- YouTube authentication support via cookies file for member-only and private videos
- Automatic detection and use of cookies file at `~/.config/ytcs/cookies.txt`
- New documentation file `COOKIES_SETUP.md` with setup instructions
- Chapter detection from video description as fallback before silence detection
- Support for multiple timestamp formats in description: `[HH:MM:SS]`, `HH:MM:SS`, `MM:SS`
- New module `chapters_from_description` for parsing chapters from video descriptions
- Section headers to distinguish "Downloading video" from "Making an album" phases

### Changed
- Cover art is now always downloaded for embedding in MP3 files
- Cover art file is automatically deleted after processing if `download_cover` config is false
- Removed redundant "Playlist detected!" and "Downloading video only" messages when `playlist_behavior` is set to `video_only`
- Removed verbose "Audio downloaded" message showing temporary file path
- Chapter detection now follows a 3-step fallback: 1) YouTube metadata, 2) Video description, 3) Silence detection
- Improved output messages to clearly separate download and conversion phases

### Fixed
- Member-only YouTube videos can now be downloaded with proper authentication
- Cleaner console output when downloading videos from playlists with video_only mode
- Videos without YouTube chapters but with timestamps in description are now properly split

## [0.3.2] - 2024-11-16

### Fixed
- Simplified cover art logic to always use external cover.jpg (YouTube thumbnail) when available
- Removed complex embedded cover detection that was causing issues
- Changed stream mapping from `1:v` to `1:0` to properly reference the image file

### Changed
- Completely simplified the cover art embedding approach
- Now uses straightforward logic: if cover.jpg exists, use it for all tracks
- Removed `check_for_video_stream()` function and all embedded cover extraction logic
- Uses `-map 0:a -map 1:0` for reliable image stream mapping

## [0.3.1] - 2024-11-16

### Fixed
- Fixed "Stream map '1:v' matches no streams" error when audio file has no embedded cover art
- Added automatic detection of embedded cover art using ffprobe before attempting to map video stream
- Improved cover art handling logic with three scenarios:
  1. Audio has embedded cover → use dual input mapping from audio file
  2. Audio has no embedded cover but external cover provided → use external cover image
  3. No cover at all → skip cover art embedding

### Changed
- Added `check_for_video_stream()` function to detect presence of video streams in audio files
- Refactored cover art mapping logic to be conditional based on stream detection

## [0.3.0] - 2024-11-16

### Fixed
- **CRITICAL FIX**: Cover art is now properly embedded in ALL tracks, not just the first one
- Implemented the correct ffmpeg approach using dual input mapping (`-i audio.mp3 -i audio.mp3 -map 0:a -map 1:v`)
- This ensures the cover art stream is correctly mapped for each individual track during splitting

### Changed
- Completely rewrote the cover art embedding logic in `src/audio.rs`
- Now uses the audio file twice as input: once for audio extraction, once for cover art extraction
- Added automatic cover art extraction from source audio if no external cover is provided
- Improved stream metadata with proper title and comment fields for cover art

### Technical Details
The previous approach failed because when using `-ss` (seek) with a single input, ffmpeg would only map the cover art stream for the first track. The new approach uses the same audio file as two separate inputs, allowing ffmpeg to properly map both the audio stream and the cover art stream for each split, regardless of the seek position.

## [0.2.7] - 2024-11-16

### Changed
- Attempted fix by moving `-ss` and `-t` after input mapping (unsuccessful)

## [0.2.6] - 2024-11-10

### Changed
- Removed `-metadata:s:v` flags for better Android compatibility
- Simplified cover art metadata approach

### Fixed
- Attempted to fix cover art embedding issue (unsuccessful - fixed in 0.3.0)

## [0.2.5] - 2024-11-10

### Added
- Added `-disposition:v attached_pic` flag for cover art
- Improved cover art handling

### Fixed
- Attempted to fix cover art embedding issue (unsuccessful - fixed in 0.3.0)

## [0.2.0] - 2024-11-10

### Added
- Cover art download and embedding support
- Automatic cover art from YouTube thumbnails
- `--artist` and `--album` CLI options to override auto-detection
- Cross-platform default output directory (`~/Music`)
- Automatic name cleaning: remove brackets, parentheses, replace pipes with hyphens

### Changed
- Replaced `reqwest` with `ureq` to reduce binary size (from ~8MB to 6.3MB)
- Improved metadata handling

## [0.1.0] - 2024-11-09

### Added
- Initial release
- YouTube video download via yt-dlp
- MP3 conversion with ffmpeg
- Chapter-based audio splitting
- Automatic chapter detection from YouTube metadata
- ID3v2.3 metadata tagging
- Cross-platform support (Linux, macOS, Windows)
