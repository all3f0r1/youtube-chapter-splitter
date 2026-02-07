//! YouTube Chapter Splitter - Library for downloading and splitting YouTube videos.
//!
//! This library provides tools for:
//! - Downloading YouTube videos and extracting audio to MP3
//! - Parsing chapters from YouTube metadata
//! - Splitting audio into individual tracks based on chapters
//! - Adding complete ID3 metadata and album cover art
//!
//! # Example Usage
//!
//! ```no_run
//! use youtube_chapter_splitter::{downloader, audio, Result};
//! use std::path::PathBuf;
//!
//! fn main() -> Result<()> {
//!     let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
//!
//!     // Fetch video information
//!     let video_info = downloader::get_video_info(url)?;
//!
//!     // Download audio
//!     let output_path = PathBuf::from("temp_audio");
//!     let audio_file = downloader::download_audio(url, &output_path)?;
//!
//!     // Split by chapters
//!     let output_dir = PathBuf::from("output");
//!     audio::split_audio_by_chapters(
//!         &audio_file,
//!         &video_info.chapters,
//!         &output_dir,
//!         "Artist Name",
//!         "Album Name",
//!         None,
//!     )?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Modules
//!
//! - [`error`] - Custom error handling
//! - [`chapters`] - Chapter structures and parsing
//! - [`downloader`] - Video downloading and metadata
//! - [`audio`] - Audio processing and splitting
//! - [`utils`] - Utility functions (formatting, cleaning)
//! - [`config`] - Configuration management
//! - [`playlist`] - Playlist detection and handling

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

pub use chapters::Chapter;
pub use config::Config;
pub use downloader::VideoInfo;
pub use error::{Result, YtcsError};
