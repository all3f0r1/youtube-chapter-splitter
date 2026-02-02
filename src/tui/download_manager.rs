//! Download manager for TUI
//!
//! Handles asynchronous download operations that can be polled from the TUI loop.

use crate::config::Config;
use crate::downloader;
use crate::error::Result;
use crate::{audio, utils, ytdlp_helper};

/// Download operation status
#[derive(Debug, Clone, PartialEq)]
pub enum DownloadStatus {
    /// Download is queued
    Pending,
    /// Download is in progress with percentage (0-100)
    InProgress { percent: u8, message: String },
    /// Download completed successfully
    Complete,
    /// Download failed
    Failed(String),
}

/// Download task
pub struct DownloadTask {
    pub url: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub status: DownloadStatus,
    pub result: Option<DownloadResult>,
}

impl DownloadTask {
    pub fn new(url: String) -> Self {
        Self {
            url,
            artist: None,
            album: None,
            status: DownloadStatus::Pending,
            result: None,
        }
    }

    pub fn with_metadata(mut self, artist: Option<String>, album: Option<String>) -> Self {
        self.artist = artist;
        self.album = album;
        self
    }
}

/// Result of a download operation
#[derive(Debug, Clone)]
pub struct DownloadResult {
    pub success: bool,
    pub tracks_count: usize,
    pub output_path: String,
    pub error: Option<String>,
}

/// Manages multiple download operations
pub struct DownloadManager {
    tasks: Vec<DownloadTask>,
    current_index: usize,
    config: Config,
    is_active: bool,
}

impl DownloadManager {
    pub fn new(config: Config) -> Self {
        Self {
            tasks: Vec::new(),
            current_index: 0,
            config,
            is_active: false,
        }
    }

    /// Add a single URL to download
    pub fn add_url(&mut self, url: String, artist: Option<String>, album: Option<String>) {
        self.tasks
            .push(DownloadTask::new(url).with_metadata(artist, album));
    }

    /// Add multiple URLs from a playlist
    pub fn add_playlist_urls(&mut self, urls: Vec<String>) {
        for url in urls {
            self.tasks.push(DownloadTask::new(url));
        }
    }

    /// Get the number of pending tasks
    pub fn pending_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| matches!(t.status, DownloadStatus::Pending))
            .count()
    }

    /// Get the number of completed tasks
    pub fn completed_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| matches!(t.status, DownloadStatus::Complete))
            .count()
    }

    /// Get the number of failed tasks
    pub fn failed_count(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| matches!(t.status, DownloadStatus::Failed(_)))
            .count()
    }

    /// Get all tasks
    pub fn tasks(&self) -> &[DownloadTask] {
        &self.tasks
    }

    /// Check if currently downloading
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Get the current task index
    pub fn current_index(&self) -> usize {
        self.current_index
    }

    /// Start processing downloads
    pub fn start(&mut self) {
        self.is_active = true;
        self.current_index = 0;
    }

    /// Stop processing downloads
    pub fn stop(&mut self) {
        self.is_active = false;
    }

    /// Reset all tasks
    pub fn reset(&mut self) {
        self.tasks.clear();
        self.current_index = 0;
        self.is_active = false;
    }

    /// Get overall progress (0-100)
    pub fn overall_percent(&self) -> u8 {
        if self.tasks.is_empty() {
            return 0;
        }

        let total = self.tasks.len() as u32;
        let completed = self.completed_count() as u32;
        let failed = self.failed_count() as u32;

        ((completed + failed) * 100 / total) as u8
    }

    /// Process the next pending download
    ///
    /// This should be called from the TUI update loop.
    /// Returns true if a download was started/completed.
    pub fn process_next(&mut self) -> Result<bool> {
        if !self.is_active {
            return Ok(false);
        }

        // Find next pending task
        let next_idx = self
            .tasks
            .iter()
            .position(|t| matches!(t.status, DownloadStatus::Pending));

        let Some(idx) = next_idx else {
            // No more pending tasks - we're done
            self.is_active = false;
            return Ok(false);
        };

        self.current_index = idx;

        // Process this task
        let task = &self.tasks[idx];
        let result = self.process_single_task(task);

        // Update task status
        match &result {
            Ok(success_result) => {
                self.tasks[idx].status = DownloadStatus::Complete;
                self.tasks[idx].result = Some(success_result.clone());
            }
            Err(e) => {
                self.tasks[idx].status = DownloadStatus::Failed(e.to_string());
                self.tasks[idx].result = Some(DownloadResult {
                    success: false,
                    tracks_count: 0,
                    output_path: String::new(),
                    error: Some(e.to_string()),
                });
            }
        }

        Ok(true)
    }

    /// Process a single download task
    fn process_single_task(&self, task: &DownloadTask) -> Result<DownloadResult> {
        let cookies = self.config.cookies_from_browser.as_deref();

        // Update yt-dlp if needed (matching CLI behavior)
        if ytdlp_helper::should_check_for_update(self.config.ytdlp_update_interval_days)
            && self.config.ytdlp_auto_update
        {
            if let Some(info) = ytdlp_helper::get_ytdlp_version() {
                if info.is_outdated {
                    // Attempt update but don't fail if it doesn't work
                    let _ = ytdlp_helper::update_ytdlp();
                }
            }
        }

        // Fetch video info
        let video_info = downloader::get_video_info(&task.url, cookies)?;

        // Determine artist and album
        let (artist, album) = if let (Some(a), Some(al)) = (&task.artist, &task.album) {
            (utils::clean_folder_name(a), utils::clean_folder_name(al))
        } else {
            utils::parse_artist_album(&video_info.title, &video_info.uploader)
        };

        // Setup output directory
        let base_dir = self.config.get_output_dir();
        let dir_name = self.config.format_directory(&artist, &album);
        let output_dir = base_dir.join(&dir_name);
        std::fs::create_dir_all(&output_dir)?;

        // Download audio (using temp file for cleanup)
        let temp_audio_path = output_dir.join("temp_audio.mp3");
        let _temp_file = crate::temp_file::TempFile::new(&temp_audio_path);

        // Download with progress (using the existing function)
        let audio_file = crate::download_audio_with_progress(
            &task.url,
            &temp_audio_path,
            cookies,
            None, // Progress bar will use default indicatif
        )?;

        // Download cover if enabled
        let cover_path = if self.config.download_cover {
            match downloader::download_thumbnail(&video_info.thumbnail_url, &output_dir) {
                Ok(_) => Some(output_dir.join("cover.jpg")),
                Err(_) => None,
            }
        } else {
            None
        };

        // Get chapters with fallback AND refinement (matching CLI behavior)
        let chapters = if !video_info.chapters.is_empty() {
            let original = video_info.chapters.clone();
            // Always refine chapters like CLI does
            crate::refine_chapters_with_silence(
                &original,
                &audio_file,
                5.0,   // window of ±5 seconds
                -35.0, // silence threshold in dB
                1.0,   // minimum silence duration in seconds
            )
            .unwrap_or(original)
        } else {
            // Try description parsing first
            crate::chapters_from_description::parse_chapters_from_description(
                &video_info.description,
                video_info.duration,
            )
            .ok()
            .and_then(|chapters| {
                // Refine chapters from description too
                crate::refine_chapters_with_silence(
                    &chapters,
                    &audio_file,
                    5.0,
                    -35.0,
                    1.0,
                )
                .ok()
            })
            .unwrap_or_else(|| {
                // Fallback to silence detection
                audio::detect_silence_chapters(&audio_file, -30.0, 2.0).unwrap_or_default()
            })
        };

        // Split into tracks
        audio::split_audio_by_chapters(
            &audio_file,
            &chapters,
            &output_dir,
            &artist,
            &album,
            cover_path.as_deref(),
            &self.config,
        )?;

        Ok(DownloadResult {
            success: true,
            tracks_count: chapters.len(),
            output_path: output_dir.display().to_string(),
            error: None,
        })
    }
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
