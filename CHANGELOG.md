# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.15.0] - 2025-02-10

### Added
- `--artist` and `--album` CLI flags to manually override metadata
- Prompt for manual metadata entry when artist/album cannot be detected
- Progressive track display during splitting operation

### Changed
- **BREAKING**: UI completely redesigned with "Pragmatic • Direct • Classy" philosophy
  - Clean tree structure (▶ ├─ └─) for consistent visual hierarchy
  - Aligned labels (Duration, Tracks, Artist, Album) for readability
  - Bold section headers without excessive colors
  - Compact layout with minimal blank lines
- Video title display now shows "Artist - Album" instead of raw video title
- Metadata sources are now indicated: (detected), (user-forced), or (default)
- "Saved" messages now show file paths directly without extra spacing
- Download progress bar simplified to show only essential information

### Fixed
- Artwork path now correctly displayed on the same line as "Saved"
- Track numbers are now vertically aligned for better readability
- Removed redundant "100% Downloading audio..." message

### Removed
- Color-coded values (magenta/cyan/green/blue) for cleaner monochrome output
- "Audio downloaded" confirmation message after download completion
- Extra blank lines between sections

## [0.14.10] - 2025-02-09

### Fixed
- Improved artist parsing from video titles
- Enhanced track title display formatting
- Better CLI output styling and consistency

## [0.14.9] - 2025-02-08

### Fixed
- Improved CLI output format and styling

## [0.14.8] - 2025-02-07

### Added
- Real-time download progress bar

## [0.14.7] and earlier

See git history for earlier changes.
