//! Module for managing progress bars

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Constant for progress bar refresh rate
const PROGRESS_TICK_RATE_MS: u64 = 100;

/// Progress bar type
#[derive(Debug, Clone, Copy)]
pub enum ProgressType {
    /// Progress bar for download
    Download,
    /// Progress bar for audio processing
    Audio,
    /// Progress bar for splitting a track
    Track,
}

/// Create a progress bar according to the specified type
///
/// # Arguments
///
/// * `message` - Message to display
/// * `progress_type` - Type of progress bar
///
/// # Returns
///
/// A configured progress bar
///
/// # Examples
///
/// ```no_run
/// use youtube_chapter_splitter::progress::{create_progress, ProgressType};
///
/// let pb = create_progress("Downloading...", ProgressType::Download);
/// // ... do something ...
/// pb.finish_and_clear();
/// ```
pub fn create_progress(message: &str, progress_type: ProgressType) -> ProgressBar {
    let pb = ProgressBar::new_spinner();

    let template = match progress_type {
        ProgressType::Download => "{msg} {spinner:.green}",
        ProgressType::Audio => "{msg} {spinner:.cyan}",
        ProgressType::Track => "  {msg} {spinner:.yellow}",
    };

    pb.set_style(ProgressStyle::default_spinner().template(template).unwrap());
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(PROGRESS_TICK_RATE_MS));
    pb
}

/// Create a progress bar for download
///
/// # Arguments
///
/// * `message` - Message to display
///
/// # Returns
///
/// A configured progress bar for download
pub fn create_download_progress(message: &str) -> ProgressBar {
    create_progress(message, ProgressType::Download)
}

/// Create a progress bar for audio processing
///
/// # Arguments
///
/// * `message` - Message to display
///
/// # Returns
///
/// A configured progress bar for audio processing
pub fn create_audio_progress(message: &str) -> ProgressBar {
    create_progress(message, ProgressType::Audio)
}

/// Create a progress bar for track splitting
///
/// # Arguments
///
/// * `message` - Message to display
///
/// # Returns
///
/// A configured progress bar for splitting
pub fn create_track_progress(message: &str) -> ProgressBar {
    create_progress(message, ProgressType::Track)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_download_progress() {
        let pb = create_download_progress("Testing download");
        assert!(!pb.is_finished());
        pb.finish();
        assert!(pb.is_finished());
    }

    #[test]
    fn test_create_audio_progress() {
        let pb = create_audio_progress("Testing audio");
        assert!(!pb.is_finished());
        pb.finish();
        assert!(pb.is_finished());
    }

    #[test]
    fn test_create_track_progress() {
        let pb = create_track_progress("Testing track");
        assert!(!pb.is_finished());
        pb.finish();
        assert!(pb.is_finished());
    }

    #[test]
    fn test_create_progress_with_type() {
        let pb = create_progress("Testing", ProgressType::Download);
        assert!(!pb.is_finished());
        pb.finish();
        assert!(pb.is_finished());
    }
}
