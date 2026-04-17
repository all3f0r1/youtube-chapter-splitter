# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.15.9] - 2026-04-17

### Added
- `deno` is now a checked (and auto-installable) dependency alongside `yt-dlp` and `ffmpeg`. YouTube requires a JS runtime to solve the `n` challenge; without it yt-dlp only returns image formats and audio download fails. `install_dependency("deno")` runs the official installer on Linux/macOS and prints a PATH hint for `~/.deno/bin`.
- `ytdlp_error_parser` detects "n challenge solving failed" / missing JS runtime / "Only images are available" errors and suggests installing deno.

### Changed
- `MissingToolsError` gained a `missing_deno` field (breaking change for direct struct construction; all in-tree tests and call sites updated).

## [0.15.8] - 2026-04-17

### Fixed
- `get_video_info` now threads `cookies_from_browser` through to `yt-dlp`, so the metadata fetch step honors the cookies config (previously only the audio download did).
- Friendlier error output: top-level errors are rendered with `Display` (one-line red message) instead of `Debug` (`Error: DownloadError(…)`).

### Added
- `ytdlp_error_parser` recognizes YouTube's bot-detection ("Sign in to confirm you're not a bot") and HTTP 429 rate-limit responses, and prints actionable cookie-setup guidance (including how to target LibreWolf / custom Firefox profiles).

## [0.15.7] - 2026-04-06

### Added
- Config `audio_format`: **mp3** (default), **opus**, or **m4a** for yt-dlp extraction and ffmpeg per-track encoding; `audio_quality` kbps applies to all three.
- Config `refine_silence_window`, `refine_noise_db`, `refine_min_silence` for silence-based chapter refinement (wizard + `ytcs config --show`).
- Config `playlist_prefix_index`: prefix album folders with `01-`, `02-`, … when processing multiple playlist videos.
- CLI: `--dry-run` (output path + chapter plan without download), `-q` / `--quiet` (minimal UI; still prints output folder path per album), `--no-cover`, `--skip-download` (reuse non-empty `temp_audio.<ext>` in the album folder).
- `VideoInfo`: `upload_date`, `genre` (from categories), `webpage_url`; embedded in split tracks as `date`, `genre`, `comment` via ffmpeg when present.
- `utils::upload_date_to_id3_date` (YYYYMMDD → YYYY-MM-DD).
- `log::info` lines for playlist resolution and refinement; `ui::set_output_quiet` / `is_output_quiet`.

### Changed
- Temporary download path is `temp_audio.<ext>` matching `audio_format`.
- `YtdlpDownloadOpts` includes `audio_format`.
- Default `refine_chapters` is **on**; default `refine_min_silence` is **1.2** s (was 1.5 s).

## [0.15.6] - 2026-03-29

### Added
- `sanitize_filesystem_chars` function to replace invalid characters (`/`, `\`, `:`, `*`, `?`, `"`, `<`, `>`, `|`) in filenames and directory names with safe equivalents

### Changed
- `Config::format_filename` and `Config::format_directory` now apply filesystem character sanitization to ensure safe output on all platforms

### Fixed
- Filenames and directory names derived from video titles no longer contain characters that are invalid on Windows/Linux/macOS filesystems

## [0.15.5] - 2026-03-27

### Added
- `ytcs config` interactive wizard: walks every setting; **Enter** keeps the current value
- `ytcs config --show` (`-s`) prints the config file path and all values without prompts
- Persistent settings are applied to downloads: output base directory, `directory_format` / `filename_format`, `download_cover`, `cookies_from_browser`, MP3 bitrate (128/192/320), `max_retries`, `download_timeout`, `dependency_auto_install`, `ytdlp_auto_update`, etc.
- `YtdlpDownloadOpts` and `impl From<&Config>` for yt-dlp invocation
- `Config::format_filename_with_template` for track filenames from the configured template

### Changed
- `main` loads `~/.config/ytcs/config.toml` (created on first run); CLI `-o` overrides only `default_output_dir` for that run
- yt-dlp uses config-driven `--audio-quality`, `--retries`, `--socket-timeout` (skipped when timeout is 0)
- Dependency install behavior respects `dependency_auto_install` (`prompt` / `always` / `never`)

### Fixed
- Artist/album parsing: strip trailing unbracketed ` - Full Album` and ` - Full Album - …` promo tails from titles (e.g. genre suffixes after “Full Album”)
- Splitting output: removed per-track indicatif bars that overlapped the tree-style track list

### Removed
- Unused string-based config helpers (`set_config`, `reset_config`, former `show_config`) in favor of the wizard and `print_config_summary`

### Documentation
- README, CLAUDE.md, and COOKIES_SETUP.md updated for `ytcs config`; error hints in `ytdlp_error_parser` no longer reference a non-existent `ytcs set` command

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
