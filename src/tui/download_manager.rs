//! Download manager for TUI
//!
//! Handles asynchronous download operations that can be polled from the TUI loop.

use crate::config::Config;
use crate::downloader;
use crate::error::{Result, YtcsError as Error};
use crate::yt_dlp_progress::DownloadProgress;
use crate::{audio, utils, ytdlp_helper};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;

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

/// Parameters needed to process a download task (cloned for background thread)
#[derive(Clone)]
struct ProcessTaskParams {
    pub url: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub config: Config,
    pub cookies_from_browser: Option<String>,
}

/// Manages multiple download operations
pub struct DownloadManager {
    tasks: Vec<DownloadTask>,
    current_index: usize,
    config: Config,
    is_active: bool,
    /// Shared progress for real-time updates
    shared_progress: Option<Arc<Mutex<Option<DownloadProgress>>>>,
    /// Handle to the current download thread
    download_handle: Option<thread::JoinHandle<Result<DownloadResult>>>,
    /// Current task index being processed
    processing_task_index: Option<usize>,
    /// Flag indicating if the current download is complete
    download_complete: Arc<AtomicBool>,
}

impl DownloadManager {
    pub fn new(config: Config) -> Self {
        Self {
            tasks: Vec::new(),
            current_index: 0,
            config,
            is_active: false,
            shared_progress: None,
            download_handle: None,
            processing_task_index: None,
            download_complete: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Set the shared progress for real-time updates
    pub fn set_shared_progress(&mut self, shared: Arc<Mutex<Option<DownloadProgress>>>) {
        self.shared_progress = Some(shared);
    }

    /// Get the current download progress (0-100) if available
    pub fn current_progress(&self) -> Option<u8> {
        if let Some(ref shared) = self.shared_progress {
            shared.lock().ok().and_then(|p| p.as_ref().map(|prog| prog.percentage as u8))
        } else {
            None
        }
    }

    /// Get the current progress message
    pub fn current_progress_message(&self) -> Option<String> {
        if let Some(ref shared) = self.shared_progress {
            shared.lock().ok().and_then(|p| p.as_ref().map(|prog| {
                if !prog.speed.is_empty() {
                    format!("{} | {} | ETA: {}", prog.downloaded, prog.speed, prog.eta)
                } else {
                    prog.downloaded.clone()
                }
            }))
        } else {
            None
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

    /// Check if a download is currently in progress (background thread running)
    pub fn is_downloading(&self) -> bool {
        self.download_handle.is_some()
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
        self.download_handle = None;
        self.processing_task_index = None;
        self.download_complete.store(false, Ordering::SeqCst);
    }

    /// Reset all tasks
    pub fn reset(&mut self) {
        self.tasks.clear();
        self.current_index = 0;
        self.is_active = false;
        self.download_handle = None;
        self.processing_task_index = None;
        self.download_complete.store(false, Ordering::SeqCst);
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

    /// Poll for download completion and start next if available
    ///
    /// This should be called from the TUI update loop.
    /// Returns true if downloads are still active, false if all complete.
    pub fn poll_downloads(&mut self) -> Result<bool> {
        if !self.is_active {
            return Ok(false);
        }

        // Check if current download is complete using the AtomicBool flag
        if self.download_complete.load(Ordering::SeqCst) {
            // Thread has completed, get the result
            if let Some(handle) = self.download_handle.take() {
                let result = handle.join().unwrap_or_else(|_| {
                    Err(Error::Other(
                        "Download thread panicked".to_string(),
                    ))
                });

                let task_idx = self.processing_task_index.unwrap();

                // Update task status based on result
                match &result {
                    Ok(success_result) => {
                        self.tasks[task_idx].status = DownloadStatus::Complete;
                        self.tasks[task_idx].result = Some(success_result.clone());
                    }
                    Err(e) => {
                        self.tasks[task_idx].status = DownloadStatus::Failed(e.to_string());
                        self.tasks[task_idx].result = Some(DownloadResult {
                            success: false,
                            tracks_count: 0,
                            output_path: String::new(),
                            error: Some(e.to_string()),
                        });
                    }
                }

                // Reset processing state
                self.processing_task_index = None;

                // Reset completion flag
                self.download_complete.store(false, Ordering::SeqCst);

                // Reset progress for next download
                if let Some(ref shared) = self.shared_progress {
                    if let Ok(mut p) = shared.lock() {
                        *p = None;
                    }
                }
            }
        } else if self.download_handle.is_some() {
            // Still downloading
            return Ok(true);
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

        // Start background download for this task
        self.current_index = idx;
        self.processing_task_index = Some(idx);

        // Mark as in progress
        self.tasks[idx].status = DownloadStatus::InProgress {
            percent: 0,
            message: "Starting...".to_string(),
        };

        // Clone what we need for the background thread
        let task = &self.tasks[idx];
        let params = ProcessTaskParams {
            url: task.url.clone(),
            artist: task.artist.clone(),
            album: task.album.clone(),
            config: self.config.clone(),
            cookies_from_browser: self.config.cookies_from_browser.clone(),
        };

        // Clone the shared progress for the background thread
        let shared_progress = self.shared_progress.as_ref().map(Arc::clone);
        let complete_flag = Arc::clone(&self.download_complete);

        // Reset flag before spawning
        complete_flag.store(false, Ordering::SeqCst);

        // Spawn background thread for download
        let handle = thread::spawn(move || {
            let result = process_single_task_background(params, shared_progress);
            complete_flag.store(true, Ordering::SeqCst);
            result
        });

        self.download_handle = Some(handle);

        Ok(true)
    }
}

/// Process a single download task in a background thread
fn process_single_task_background(
    params: ProcessTaskParams,
    shared_progress: Option<Arc<Mutex<Option<DownloadProgress>>>>,
) -> Result<DownloadResult> {
    use crate::yt_dlp_progress::DownloadProgress;

    let cookies = params.cookies_from_browser.as_deref();

    // Helper to update progress
    let update_progress = |percent: u8, message: &str| {
        if let Some(ref shared) = shared_progress {
            if let Ok(mut p) = shared.lock() {
                // Only update if we have meaningful progress data from yt-dlp
                // Otherwise use our manual progress
                if p.is_none() || p.as_ref().map_or(true, |prog| prog.percentage < 1.0) {
                    *p = Some(DownloadProgress {
                        percentage: percent as f64,
                        downloaded: message.to_string(),
                        total: String::new(),
                        speed: String::new(),
                        eta: String::new(),
                    });
                }
            }
        }
    };

    // Step 1: Update yt-dlp if needed
    update_progress(5, "Checking yt-dlp version...");
    if ytdlp_helper::should_check_for_update(params.config.ytdlp_update_interval_days)
        && params.config.ytdlp_auto_update
    {
        if let Some(info) = ytdlp_helper::get_ytdlp_version() {
            if info.is_outdated {
                update_progress(10, "Updating yt-dlp...");
                // Attempt update but don't fail if it doesn't work
                let _ = ytdlp_helper::update_ytdlp();
            }
        }
    }

    // Step 2: Fetch video info
    update_progress(15, "Fetching video info...");
    let video_info = downloader::get_video_info(&params.url, cookies)?;

    // Step 3: Determine artist and album
    let (artist, album) = if let (Some(a), Some(al)) = (&params.artist, &params.album) {
        (utils::clean_folder_name(a), utils::clean_folder_name(al))
    } else {
        utils::parse_artist_album(&video_info.title, &video_info.uploader)
    };

    // Step 4: Setup output directory
    update_progress(20, "Creating output directory...");
    let base_dir = params.config.get_output_dir();
    let dir_name = params.config.format_directory(&artist, &album);
    let output_dir = base_dir.join(&dir_name);
    std::fs::create_dir_all(&output_dir)?;

    // Step 5: Download audio
    update_progress(25, "Downloading audio...");
    let temp_audio_path = output_dir.join("temp_audio.mp3");
    let _temp_file = crate::temp_file::TempFile::new(&temp_audio_path);

    // Prepare shared progress reference
    let progress_shared = shared_progress.as_ref();

    // Download with progress (yt-dlp will update shared_progress directly)
    let audio_file = crate::download_audio_with_progress(
        &params.url,
        &temp_audio_path,
        cookies,
        None, // No CLI progress bar
        progress_shared,
    )?;

    // Step 6: Download cover if enabled
    update_progress(75, "Downloading cover art...");
    let cover_data = if params.config.download_cover {
        match downloader::download_thumbnail(&video_info.thumbnail_url, &output_dir) {
            Ok(cover_path) => {
                // Load image data into memory (matching CLI approach)
                match audio::load_cover_image(&cover_path) {
                    Ok(data) => data,
                    Err(e) => {
                        log::warn!("Failed to load cover image: {}", e);
                        None
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to download cover: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Step 7: Get chapters with fallback AND refinement
    update_progress(80, "Detecting chapters...");
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
            crate::refine_chapters_with_silence(&chapters, &audio_file, 5.0, -35.0, 1.0).ok()
        })
        .unwrap_or_else(|| {
            // Fallback to silence detection
            audio::detect_silence_chapters(&audio_file, -30.0, 2.0).unwrap_or_default()
        })
    };

    // Step 8: Split into tracks with cover data
    update_progress(90, "Splitting into tracks...");
    audio::split_audio_by_chapters_with_cover_data(
        &audio_file,
        &chapters,
        &output_dir,
        &artist,
        &album,
        cover_data.as_deref(),
        &params.config,
    )?;

    update_progress(100, "Complete!");

    Ok(DownloadResult {
        success: true,
        tracks_count: chapters.len(),
        output_path: output_dir.display().to_string(),
        error: None,
    })
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
