# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
