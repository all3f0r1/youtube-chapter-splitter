//! Chapter refinement by silence detection.
//!
//! This module adjusts declared chapter timecodes using
//! silence detection to find optimal split points.
//!
//! # Principle
//!
//! YouTube chapter timecodes are often imprecise (a few seconds off).
//! This module:
//! 1. Analyzes all audio to find all silences
//! 2. For each chapter, finds the nearest silence (within a window)
//! 3. Adjusts the timecode towards that silence
//!
//! # Example
//!
//! ```no_run
//! # use youtube_chapter_splitter::chapter_refinement::refine_chapters_with_silence;
//! # use youtube_chapter_splitter::chapters::Chapter;
//! # let chapters = vec![
//! #     Chapter::new("Track 1".to_string(), 0.0, 30.0),
//! #     Chapter::new("Track 2".to_string(), 30.0, 60.0),
//! # ];
//! # let audio_file = std::path::PathBuf::from("/tmp/audio.mp3");
//! let refined = refine_chapters_with_silence(
//!     &chapters,
//!     &audio_file,
//!     5.0,  // window of ±5 seconds
//!     -35.0, // silence threshold
//!     1.5,  // minimum silence duration
//! )?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::chapters::Chapter;
use crate::error::{Result, YtcsError};
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;
use std::process::Command;

/// Detected silence point in the audio
#[derive(Debug, Clone)]
struct SilencePoint {
    /// Position of the silence in seconds
    position: f64,
}

impl SilencePoint {
    fn new(start: f64, end: f64) -> Self {
        let position = (start + end) / 2.0;
        Self { position }
    }
}

/// Analyzes the audio and extracts all silence points.
///
/// # Arguments
///
/// * `audio_file` - Audio file to analyze
/// * `noise_threshold` - Silence threshold in dB (ex: -35.0)
/// * `min_duration` - Minimum duration of a silence in seconds (ex: 1.0)
fn detect_all_silences(
    audio_file: &Path,
    noise_threshold: f64,
    min_duration: f64,
) -> Result<Vec<SilencePoint>> {
    log::info!(
        "Detecting silences (threshold: {} dB, min: {}s)",
        noise_threshold,
        min_duration
    );

    let output = Command::new("ffmpeg")
        .arg("-i")
        .arg(audio_file)
        .arg("-af")
        .arg(format!(
            "silencedetect=noise={}dB:d={}",
            noise_threshold, min_duration
        ))
        .arg("-f")
        .arg("null")
        .arg("-")
        .output()
        .map_err(|e| YtcsError::AudioError(format!("Failed to run ffmpeg: {}", e)))?;

    let stderr = String::from_utf8_lossy(&output.stderr);

    static RE_SILENCE_START: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"silence_start: ([\d.]+)").unwrap());
    static RE_SILENCE_END: Lazy<Regex> =
        Lazy::new(|| Regex::new(r"silence_end: ([\d.]+)").unwrap());

    let mut silence_points = Vec::new();
    let mut current_start: Option<f64> = None;

    for line in stderr.lines() {
        if let Some(caps) = RE_SILENCE_START.captures(line) {
            current_start = caps.get(1).and_then(|m| m.as_str().parse().ok());
        } else if let Some(caps) = RE_SILENCE_END.captures(line) {
            if let Some(start) = current_start
                && let Some(end_caps) = caps.get(1)
                && let Ok(end) = end_caps.as_str().parse::<f64>()
            {
                silence_points.push(SilencePoint::new(start, end));
            }
            current_start = None;
        }
    }

    log::info!("Found {} silence points", silence_points.len());
    Ok(silence_points)
}

/// Finds the silence point closest to a target position.
///
/// # Arguments
///
/// * `silences` - List of silence points
/// * `target` - Target position in seconds
/// * `window` - Search window in seconds (ex: 5.0 = ±5s)
fn find_nearest_silence(
    silences: &[SilencePoint],
    target: f64,
    window: f64,
) -> Option<&SilencePoint> {
    silences
        .iter()
        .filter(|s| (s.position - target).abs() <= window)
        .min_by(|a, b| {
            let dist_a = (a.position - target).abs();
            let dist_b = (b.position - target).abs();
            dist_a
                .partial_cmp(&dist_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

/// Refines chapters using silence points.
///
/// # Arguments
///
/// * `chapters` - Original chapters with declared timecodes
/// * `audio_file` - Audio file to analyze
/// * `window` - Search window in seconds (recommended: 3.0 to 8.0)
/// * `noise_threshold` - Silence threshold in dB (recommended: -35.0 to -50.0)
/// * `min_silence_duration` - Minimum silence duration (recommended: 0.8 to 2.0)
///
/// # Returns
///
/// New chapters with timecodes adjusted towards the nearest silences.
///
/// # Strategy
///
/// - **Track start**: Adjusted towards the nearest silence **after** the timecode
/// - **Track end**: Adjusted towards the nearest silence **before** the timecode
/// - If no silence is found within the window, the original timecode is kept
pub fn refine_chapters_with_silence(
    chapters: &[Chapter],
    audio_file: &Path,
    window: f64,
    noise_threshold: f64,
    min_silence_duration: f64,
) -> Result<Vec<Chapter>> {
    if chapters.is_empty() {
        return Ok(Vec::new());
    }

    // Detect all silences once
    let silences = detect_all_silences(audio_file, noise_threshold, min_silence_duration)?;

    if silences.is_empty() {
        log::warn!("No silences detected, returning original chapters");
        return Ok(chapters.to_vec());
    }

    let mut refined = Vec::new();

    for (i, chapter) in chapters.iter().enumerate() {
        let is_last = i == chapters.len() - 1;

        // For start, look for a silence AFTER the declared timecode
        // For end (except last track), look for a silence BEFORE
        let new_start = if i == 0 {
            // First track: keep the beginning or look before
            let before = find_nearest_silence(&silences, chapter.start_time, window);
            before
                .map(|s| s.position)
                .filter(|&pos| pos <= chapter.start_time + 0.5)
        } else {
            // Following tracks: look for a silence after the timecode
            let after = find_nearest_silence(&silences, chapter.start_time, window);
            after
                .map(|s| s.position)
                .filter(|&pos| pos >= chapter.start_time - 0.5)
        };

        let new_end = if is_last {
            // Last track: keep the original end
            chapter.end_time
        } else {
            // Look for a silence before the declared end
            let before = find_nearest_silence(&silences, chapter.end_time, window);
            before
                .map(|s| s.position)
                .filter(|&pos| pos <= chapter.end_time + 0.5)
                .unwrap_or(chapter.end_time)
        };

        let final_start = new_start.unwrap_or(chapter.start_time);
        let final_end = new_end;

        // Ensure chapters don't overlap
        let previous_end = refined.last().map(|c: &Chapter| c.end_time);
        let final_end = if let Some(prev_end) = previous_end {
            if final_start < prev_end {
                prev_end + 0.1
            } else {
                final_end
            }
        } else {
            final_end
        };

        // Check that duration is reasonable (at least 30 seconds)
        let final_end = final_end.max(final_start + 30.0);

        let duration_delta = (final_end - final_start) - chapter.duration();
        let start_delta = final_start - chapter.start_time;

        log::debug!(
            "Chapter {}: start {:.2}s → {:.2}s (Δ{:.2}s), end {:.2}s → {:.2}s (Δ{:.2}s)",
            i + 1,
            chapter.start_time,
            final_start,
            start_delta,
            chapter.end_time,
            final_end,
            duration_delta
        );

        refined.push(Chapter::new(chapter.title.clone(), final_start, final_end));
    }

    Ok(refined)
}

/// Prints a comparison report between original and refined chapters.
pub fn print_refinement_report(original: &[Chapter], refined: &[Chapter]) {
    if original.len() != refined.len() {
        println!("⚠ Cannot compare: different chapter counts");
        return;
    }

    let mut total_delta = 0.0_f64;
    let mut max_delta = 0.0_f64;
    let mut adjusted_count = 0;

    for (orig, refn) in original.iter().zip(refined.iter()) {
        let delta = refn.start_time - orig.start_time;
        if delta.abs() >= 0.1 {
            adjusted_count += 1;
            total_delta += delta.abs();
            max_delta = max_delta.max(delta.abs());
        }
    }

    if adjusted_count > 0 {
        println!(
            "  Adjusted {}/{} chapters (avg: {:.1}s, max: {:.1}s)",
            adjusted_count,
            original.len(),
            total_delta / adjusted_count as f64,
            max_delta
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a test audio file with artificial silences.
    /// Note: requires ffmpeg in PATH.
    #[test]
    #[ignore] // Ignored by default because it's slow and requires ffmpeg
    fn test_refine_chapters() {
        // Create a test audio file with silence
        // This test is mainly documentary
        let test_audio = std::path::PathBuf::from("/tmp/test_silence.mp3");

        let chapters = vec![
            Chapter::new("Track 1".to_string(), 0.0, 30.0),
            Chapter::new("Track 2".to_string(), 30.5, 60.0),
            Chapter::new("Track 3".to_string(), 60.5, 90.0),
        ];

        if test_audio.exists() {
            let result = refine_chapters_with_silence(&chapters, &test_audio, 5.0, -35.0, 1.0);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_silence_point_position() {
        let s1 = SilencePoint::new(10.0, 11.0);
        let s2 = SilencePoint::new(10.0, 14.0);

        // Position is the midpoint of the silence
        assert_eq!(s1.position, 10.5); // (10 + 11) / 2
        assert_eq!(s2.position, 12.0); // (10 + 14) / 2
    }

    #[test]
    fn test_find_nearest_silence() {
        let silences = vec![
            SilencePoint::new(0.0, 1.0),   // position: 0.5
            SilencePoint::new(9.5, 10.5),  // position: 10.0
            SilencePoint::new(19.0, 20.0), // position: 19.5
        ];

        // Exact target
        let nearest = find_nearest_silence(&silences, 10.0, 2.0);
        assert_eq!(nearest.unwrap().position, 10.0);

        // Close target
        let nearest = find_nearest_silence(&silences, 9.8, 2.0);
        assert_eq!(nearest.unwrap().position, 10.0);

        // Target outside window
        let nearest = find_nearest_silence(&silences, 15.0, 2.0);
        assert!(nearest.is_none());
    }
}
