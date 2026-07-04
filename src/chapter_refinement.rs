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
//!     1.2,  // minimum silence duration
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

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(YtcsError::AudioError(format!("ffmpeg failed: {}", error)));
    }

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

/// Minimum gap enforced between two adjacent refined boundaries, so that
/// `Chapter::new` (which panics on `end <= start`) never sees a degenerate range.
const MIN_BOUNDARY_GAP: f64 = 0.05;

/// Computes the `n + 1` cut points for `n` chapters: `boundaries[0]` is the
/// first chapter's declared start, `boundaries[n]` is the last chapter's
/// declared end (both fixed), and each `boundaries[i]` in between is the
/// single, shared cut between chapter `i - 1` and chapter `i`, snapped to the
/// nearest silence within `window` seconds of the declared cut (or left at
/// the declared position if none is found). Pulled out of
/// [`refine_chapters_with_silence`] so the boundary math can be unit-tested
/// without needing ffmpeg or a real audio file.
fn compute_refined_boundaries(
    chapters: &[Chapter],
    silences: &[SilencePoint],
    window: f64,
) -> Vec<f64> {
    let n = chapters.len();
    let first_start = chapters[0].start_time;
    let last_end = chapters[n - 1].end_time;

    // boundaries[0] and boundaries[n] are fixed; boundaries[1..n] are the interior
    // cuts, each computed exactly once from the declared start/end around it.
    let mut boundaries = vec![0.0_f64; n + 1];
    boundaries[0] = first_start;
    boundaries[n] = last_end;

    for i in 1..n {
        let target = (chapters[i - 1].end_time + chapters[i].start_time) / 2.0;
        boundaries[i] = find_nearest_silence(silences, target, window)
            .map(|s| s.position)
            .unwrap_or(target);
    }

    // Enforce strictly increasing boundaries without ever moving the fixed
    // first/last points: push interior collisions forward, then pull back
    // anything that got nudged past the fixed end.
    for i in 1..n {
        let min_allowed = boundaries[i - 1] + MIN_BOUNDARY_GAP;
        if boundaries[i] < min_allowed {
            boundaries[i] = min_allowed;
        }
    }
    for i in (1..n).rev() {
        let max_allowed = boundaries[i + 1] - MIN_BOUNDARY_GAP;
        if boundaries[i] > max_allowed {
            boundaries[i] = max_allowed;
        }
    }

    // The backward pass only pulls values down to satisfy the fixed last
    // boundary, so in principle it could re-introduce a collision with an
    // already-settled predecessor if the total span were too tight to fit
    // `n` gaps of MIN_BOUNDARY_GAP. Real chapters are always spaced by at
    // least ~1s (enforced upstream), so this can't trigger in practice; the
    // assert documents the invariant instead of silently producing chapters
    // with reversed/zero duration.
    debug_assert!(
        boundaries.windows(2).all(|w| w[1] > w[0]),
        "refined boundaries must be strictly increasing: {:?}",
        boundaries
    );

    boundaries
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
/// Each cut between two consecutive tracks is a single boundary, computed once and
/// shared by both tracks (track N's end == track N+1's start), so refinement can never
/// open a gap or an overlap. The very first start and very last end are never moved;
/// only the `n - 1` interior boundaries are snapped to the nearest silence in the window.
/// If no silence is found near a boundary, its original (declared) position is kept.
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

    let boundaries = compute_refined_boundaries(chapters, &silences, window);

    let refined: Vec<Chapter> = chapters
        .iter()
        .enumerate()
        .map(|(i, chapter)| {
            let final_start = boundaries[i];
            let final_end = boundaries[i + 1];
            log::debug!(
                "Chapter {}: start {:.2}s → {:.2}s (Δ{:.2}s), end {:.2}s → {:.2}s (Δ{:.2}s)",
                i + 1,
                chapter.start_time,
                final_start,
                final_start - chapter.start_time,
                chapter.end_time,
                final_end,
                (final_end - final_start) - chapter.duration()
            );
            Chapter::new(chapter.title.clone(), final_start, final_end)
        })
        .collect();

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

    /// Asserts every returned boundary set has no gaps and no overlaps: each
    /// chapter's end must equal the next chapter's start, and the first/last
    /// edges must match the originals exactly (they're never refined).
    fn assert_contiguous(chapters: &[Chapter], boundaries: &[f64]) {
        assert_eq!(boundaries.len(), chapters.len() + 1);
        assert_eq!(boundaries[0], chapters[0].start_time, "first start moved");
        assert_eq!(
            *boundaries.last().unwrap(),
            chapters.last().unwrap().end_time,
            "last end moved"
        );
        for w in boundaries.windows(2) {
            assert!(
                w[1] > w[0],
                "boundaries must be strictly increasing: {:?}",
                w
            );
        }
    }

    #[test]
    fn test_compute_refined_boundaries_no_gap_or_overlap() {
        // Regression for the original bug: a silence found close to chapter
        // 2's declared start (which precedes chapter 1's declared end) used
        // to only move `final_end` for chapter 1, leaving chapter 2's start
        // untouched — opening a gap/overlap between the two. Boundaries are
        // now computed once and shared, so this can no longer happen.
        let chapters = vec![
            Chapter::new("Track 1".to_string(), 0.0, 30.0),
            Chapter::new("Track 2".to_string(), 28.0, 60.0),
            Chapter::new("Track 3".to_string(), 60.5, 90.0),
        ];
        let silences = vec![
            SilencePoint::new(28.8, 29.2), // near the declared 28.0/30.0 split
            SilencePoint::new(60.2, 60.8), // near the declared 60.5 split
        ];

        let boundaries = compute_refined_boundaries(&chapters, &silences, 5.0);
        assert_contiguous(&chapters, &boundaries);
        // Both tracks around the first cut agree on exactly the same point.
        assert_eq!(boundaries[1], 29.0);
    }

    #[test]
    fn test_compute_refined_boundaries_keeps_declared_cut_when_no_silence_nearby() {
        let chapters = vec![
            Chapter::new("Track 1".to_string(), 0.0, 30.0),
            Chapter::new("Track 2".to_string(), 30.0, 60.0),
        ];
        // No silence anywhere near the 30.0s cut.
        let silences = vec![SilencePoint::new(200.0, 201.0)];

        let boundaries = compute_refined_boundaries(&chapters, &silences, 5.0);
        assert_contiguous(&chapters, &boundaries);
        assert_eq!(boundaries[1], 30.0);
    }

    #[test]
    fn test_compute_refined_boundaries_never_moves_first_start_or_last_end() {
        let chapters = vec![
            Chapter::new("Track 1".to_string(), 2.0, 30.0),
            Chapter::new("Track 2".to_string(), 30.0, 60.0),
            Chapter::new("Track 3".to_string(), 60.0, 90.0),
        ];
        // Silences suspiciously close to the very first start and very last
        // end must not clip the intro or trim the outro.
        let silences = vec![
            SilencePoint::new(0.4, 0.6),
            SilencePoint::new(29.8, 30.2),
            SilencePoint::new(59.8, 60.2),
            SilencePoint::new(89.0, 89.4),
        ];

        let boundaries = compute_refined_boundaries(&chapters, &silences, 5.0);
        assert_contiguous(&chapters, &boundaries);
    }

    #[test]
    fn test_compute_refined_boundaries_handles_many_close_declared_cuts() {
        // Several chapters declared back-to-back with barely any gap between
        // them (e.g. a sloppily authored description). Even if every interior
        // cut snaps to nearly the same silence, boundaries must stay strictly
        // increasing and the outer edges must stay fixed.
        let chapters = vec![
            Chapter::new("Track 1".to_string(), 0.0, 10.0),
            Chapter::new("Track 2".to_string(), 10.01, 10.02),
            Chapter::new("Track 3".to_string(), 10.03, 10.04),
            Chapter::new("Track 4".to_string(), 10.05, 20.0),
        ];
        let silences = vec![SilencePoint::new(9.99, 10.01)];

        let boundaries = compute_refined_boundaries(&chapters, &silences, 5.0);
        assert_contiguous(&chapters, &boundaries);
    }
}
