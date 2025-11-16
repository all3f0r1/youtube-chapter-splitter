# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
