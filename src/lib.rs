//! YouTube Chapter Splitter - Library for downloading and splitting YouTube videos
//!
//! This library provides tools for:
//! - Downloading YouTube videos and extracting audio to MP3
//! - Parsing chapters from YouTube metadata
//! - Splitting audio into individual tracks based on chapters
//! - Adding complete ID3 metadata and album art
//!
//! # Example Usage
//!
//! ```no_run
//! use youtube_chapter_splitter::{downloader, audio, config, Result};
//! use std::path::PathBuf;
//!
//! fn main() -> Result<()> {
//!     let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
//!
//!     // Fetch video information
//!     let video_info = downloader::get_video_info(url, None)?;
//!
//!     // Download audio
//!     let output_path = PathBuf::from("temp_audio");
//!     let audio_file = downloader::download_audio(url, &output_path, None, None)?;
//!
//!     // Split by chapters
//!     let output_dir = PathBuf::from("output");
//!     let cfg = config::Config::default();
//!     audio::split_audio_by_chapters(
//!         &audio_file,
//!         &video_info.chapters,
//!         &output_dir,
//!         "Artist Name",
//!         "Album Name",
//!         None,
//!         &cfg,
//!     )?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Modules
//!
//! - [`error`] - Custom error handling
//! - [`chapters`] - Chapter structures and functions
//! - [`downloader`] - Video and metadata downloading
//! - [`audio`] - Audio processing and splitting
//! - [`utils`] - Utility functions (formatting, cleaning)
//! - [`dependency`] - Dependency detection and installation
//! - [`tui`] - Terminal User Interface

pub mod audio;
pub mod chapter_refinement;
pub mod chapters;
pub mod chapters_from_description;
pub mod config;
pub mod cookie_helper;
pub mod dependency;
pub mod downloader;
pub mod error;
pub mod error_handler;
pub mod playlist;
pub mod progress;
pub mod temp_file;
pub mod ui;
pub mod utils;
pub mod yt_dlp_progress;
pub mod yt_dlp_update;
pub mod ytdlp_error_parser;
pub mod ytdlp_helper;

// TUI module (optional, behind feature flag for now)
#[cfg(feature = "tui")]
pub mod tui;

pub use chapter_refinement::{print_refinement_report, refine_chapters_with_silence};
pub use chapters::Chapter;
pub use downloader::VideoInfo;
pub use error::{Result, YtcsError};
pub use yt_dlp_progress::download_audio_with_progress;

/// Entry point for the interactive TUI
///
/// If `initial_url` is provided, the TUI will:
/// 1. Fetch video metadata (title, uploader)
/// 2. Parse artist/album from metadata
/// 3. Auto-start the download immediately (like the old CLI behavior)
///
/// If no URL provided, starts on welcome screen.
#[cfg(feature = "tui")]
pub fn run_tui(initial_url: Option<String>) -> Result<()> {
    use crate::{downloader, utils};

    let mut app = tui::App::new()?;

    if let Some(url) = initial_url {
        // Check if this is a playlist URL
        if crate::playlist::is_playlist_url(&url).is_some() {
            // For playlists, go to playlist screen
            app.screen_data.input_url = url.clone();
            app.current_screen = tui::app::Screen::Playlist;
            app.playlist_screen.load_from_url(&url, &app.config);
        } else {
            // For single videos, fetch metadata and auto-start download
            let cookies_from_browser = app.config.cookies_from_browser.as_deref();
            match downloader::get_video_info(&url, cookies_from_browser) {
                Ok(video_info) => {
                    // Parse artist and album from title/uploader
                    let (artist, album) =
                        utils::parse_artist_album(&video_info.title, &video_info.uploader);

                    // Add to download manager and start immediately
                    app.download_manager.add_url(
                        url.clone(),
                        if artist.is_empty() {
                            None
                        } else {
                            Some(artist)
                        },
                        if album.is_empty() { None } else { Some(album) },
                    );
                    app.download_manager.start();

                    // Navigate directly to progress screen (auto-start like CLI mode)
                    app.current_screen = tui::app::Screen::Progress;
                }
                Err(e) => {
                    // If metadata fetch fails, go to download screen with URL pre-filled
                    log::warn!("Could not fetch metadata: {}", e);
                    app.screen_data.input_url = url;
                    app.current_screen = tui::app::Screen::Download;
                }
            }
        }
    }

    app.run()
}
