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
/// If `initial_url` is provided, the TUI will start in download mode
/// with the URL pre-filled.
#[cfg(feature = "tui")]
pub fn run_tui(initial_url: Option<String>) -> Result<()> {
    let mut app = tui::App::new()?;
    if let Some(url) = initial_url {
        // Pre-fill the URL and start in download mode
        app.screen_data.input_url = url;
        app.current_screen = tui::app::Screen::Download;
    }
    app.run()
}
