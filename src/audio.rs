//! Audio processing and chapter-based splitting.
//!
//! This module handles splitting audio files into individual tracks
//! and adding ID3 metadata with album cover art.

use crate::chapters::Chapter;
use crate::error::{Result, YtcsError};
use indicatif::{ProgressBar, ProgressStyle};
use lofty::config::WriteOptions;
use lofty::picture::{Picture, PictureType};
use lofty::prelude::*;
use lofty::probe::Probe;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

// Regex compiled once at startup
static RE_SILENCE_START: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"silence_start: ([\d.]+)").unwrap());

static RE_SILENCE_END: Lazy<Regex> = Lazy::new(|| Regex::new(r"silence_end: ([\d.]+)").unwrap());

/// Callback type for track progress during splitting
pub type TrackProgressCallback =
    fn(track_number: usize, total_tracks: usize, title: &str, duration: &str);

/// Splits an audio file into individual tracks based on chapters.
///
/// This function uses `ffmpeg` to split the audio and `lofty` to add
/// ID3 metadata and album cover art.
///
/// # Arguments
///
/// * `input_file` - The source audio file
/// * `chapters` - The chapters defining the split points
/// * `output_dir` - The output directory for tracks
/// * `artist` - The artist name
/// * `album` - The album name
/// * `cover_path` - Optional path to the cover image
/// * `progress_callback` - Optional callback for track-by-track progress
///
/// # Returns
///
/// A vector containing the paths of created files
///
/// # Errors
///
/// Returns an error if splitting or adding metadata fails
pub fn split_audio_by_chapters(
    input_file: &Path,
    chapters: &[Chapter],
    output_dir: &Path,
    artist: &str,
    album: &str,
    cover_path: Option<&Path>,
    progress_callback: Option<TrackProgressCallback>,
) -> Result<Vec<PathBuf>> {
    std::fs::create_dir_all(output_dir)?;

    // Create a progress bar
    let pb = ProgressBar::new(chapters.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{bar:40.cyan/blue}]")
            .unwrap()
            .progress_chars("#>-"),
    );

    // Load cover image once if it exists
    let cover_data = if let Some(cover) = cover_path {
        load_cover_image(cover)?
    } else {
        None
    };

    let mut output_files = Vec::new();

    for (index, chapter) in chapters.iter().enumerate() {
        let track_number = index + 1;
        let sanitized_title = chapter.sanitize_title();
        let output_filename = format!("{:02} - {}.mp3", track_number, sanitized_title);
        let output_path = output_dir.join(&output_filename);

        let duration = chapter.duration();

        // Split audio with ffmpeg (without cover art)
        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-i")
            .arg(input_file)
            .arg("-ss")
            .arg(chapter.start_time.to_string())
            .arg("-t")
            .arg(duration.to_string())
            .arg("-c:a")
            .arg("libmp3lame")
            .arg("-q:a")
            .arg("0")
            .arg("-metadata")
            .arg(format!("title={}", chapter.title))
            .arg("-metadata")
            .arg(format!("artist={}", artist))
            .arg("-metadata")
            .arg(format!("album={}", album))
            .arg("-metadata")
            .arg(format!("track={}/{}", track_number, chapters.len()))
            .arg("-y")
            .arg(&output_path);

        let output = cmd
            .output()
            .map_err(|e| YtcsError::AudioError(format!("Failed to execute ffmpeg: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(YtcsError::AudioError(format!("ffmpeg failed: {}", error)));
        }

        // Add album cover art with lofty if it exists
        if let Some(ref cover) = cover_data {
            add_cover_to_file(&output_path, cover)?;
        }

        output_files.push(output_path);

        // Call progress callback if provided
        if let Some(callback) = progress_callback {
            let duration_str = format!(
                "{}m {:02}s",
                (duration / 60.0).floor() as u32,
                (duration % 60.0).floor() as u32
            );
            callback(
                track_number,
                chapters.len(),
                &chapter.display_title(),
                &duration_str,
            );
        }

        pb.inc(1);
    }

    pb.finish();
    Ok(output_files)
}

/// Loads a cover image from a file.
///
/// # Arguments
///
/// * `cover_path` - Path to the image file
///
/// # Returns
///
/// The image data as a byte vector
fn load_cover_image(cover_path: &Path) -> Result<Option<Vec<u8>>> {
    let mut file = File::open(cover_path)
        .map_err(|e| YtcsError::AudioError(format!("Failed to open cover image: {}", e)))?;

    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .map_err(|e| YtcsError::AudioError(format!("Failed to read cover image: {}", e)))?;

    Ok(Some(data))
}

/// Adds album cover art to an audio file using lofty.
///
/// # Arguments
///
/// * `audio_path` - Path to the audio file
/// * `cover_data` - Cover image data
fn add_cover_to_file(audio_path: &Path, cover_data: &[u8]) -> Result<()> {
    // Load the audio file
    let mut tagged_file = Probe::open(audio_path)
        .map_err(|e| YtcsError::AudioError(format!("Failed to open audio file: {}", e)))?
        .guess_file_type()
        .map_err(|e| YtcsError::AudioError(format!("Failed to guess file type: {}", e)))?
        .read()
        .map_err(|e| YtcsError::AudioError(format!("Failed to read audio file: {}", e)))?;

    // Create Picture object from data
    let mut cover_reader = cover_data;
    let mut picture = Picture::from_reader(&mut cover_reader)
        .map_err(|e| YtcsError::AudioError(format!("Failed to create picture: {}", e)))?;

    // Set type and description
    picture.set_pic_type(PictureType::CoverFront);
    picture.set_description(Some("Album Cover".to_string()));

    // Get or create primary tag
    let tag = match tagged_file.primary_tag_mut() {
        Some(primary_tag) => primary_tag,
        None => {
            let tag_type = tagged_file.primary_tag_type();
            tagged_file.insert_tag(lofty::tag::Tag::new(tag_type));
            tagged_file.primary_tag_mut().unwrap()
        }
    };

    // Add image to tag
    tag.push_picture(picture);

    // Save changes with tagged_file.save_to() to preserve all tags
    // Note: save_to() preserves all existing metadata, unlike save_to_path()
    tagged_file
        .save_to_path(audio_path, WriteOptions::default())
        .map_err(|e| YtcsError::AudioError(format!("Failed to save tags: {}", e)))?;

    Ok(())
}

/// Automatically detects chapters by analyzing silence periods.
///
/// Uses `ffmpeg` with the `silencedetect` filter to identify potential
/// split points in the audio.
///
/// # Arguments
///
/// * `input_file` - The audio file to analyze
/// * `silence_threshold` - Silence threshold in dB (ex: -30.0)
/// * `min_silence_duration` - Minimum silence duration in seconds (ex: 2.0)
///
/// # Returns
///
/// A vector of automatically detected chapters
///
/// # Errors
///
/// Returns an error if no silence is detected or if ffmpeg fails
pub fn detect_silence_chapters(
    input_file: &Path,
    silence_threshold: f64,
    min_silence_duration: f64,
) -> Result<Vec<Chapter>> {
    println!("Detecting silence to identify tracks...");

    let output = Command::new("ffmpeg")
        .arg("-i")
        .arg(input_file)
        .arg("-af")
        .arg(format!(
            "silencedetect=noise={}dB:d={}",
            silence_threshold, min_silence_duration
        ))
        .arg("-f")
        .arg("null")
        .arg("-")
        .output()
        .map_err(|e| YtcsError::AudioError(format!("Failed to execute ffmpeg: {}", e)))?;

    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut silence_periods = Vec::new();
    let mut current_start: Option<f64> = None;

    for line in stderr.lines() {
        if let Some(caps) = RE_SILENCE_START.captures(line) {
            if let Some(start_str) = caps.get(1) {
                current_start = start_str.as_str().parse::<f64>().ok();
            }
        } else if let Some(caps) = RE_SILENCE_END.captures(line)
            && let (Some(start), Some(end_str)) = (current_start, caps.get(1))
        {
            if let Ok(end) = end_str.as_str().parse::<f64>() {
                let mid_point = (start + end) / 2.0;
                silence_periods.push(mid_point);
            }
            current_start = None;
        }
    }

    if silence_periods.is_empty() {
        return Err(YtcsError::ChapterError(
            "No silence detected. Try adjusting the parameters.".to_string(),
        ));
    }

    // Get total duration
    let duration = get_audio_duration(input_file)?;

    let mut chapters = Vec::new();
    let mut start_time = 0.0;

    for (i, &split_point) in silence_periods.iter().enumerate() {
        chapters.push(Chapter::new(
            format!("Track {}", i + 1),
            start_time,
            split_point,
        ));
        start_time = split_point;
    }

    // Last track
    chapters.push(Chapter::new(
        format!("Track {}", chapters.len() + 1),
        start_time,
        duration,
    ));

    println!("✓ {} tracks detected", chapters.len());
    Ok(chapters)
}

/// Gets the total duration of an audio file.
///
/// Uses `ffprobe` to extract the file duration.
///
/// # Arguments
///
/// * `input_file` - The audio file to analyze
///
/// # Returns
///
/// The duration in seconds
///
/// # Errors
///
/// Returns an error if ffprobe fails or if duration is invalid
pub fn get_audio_duration(input_file: &Path) -> Result<f64> {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-show_entries")
        .arg("format=duration")
        .arg("-of")
        .arg("default=noprint_wrappers=1:nokey=1")
        .arg(input_file)
        .output()
        .map_err(|e| YtcsError::AudioError(format!("Failed to execute ffprobe: {}", e)))?;

    if !output.status.success() {
        return Err(YtcsError::AudioError("Unable to get duration".to_string()));
    }

    let duration_str = String::from_utf8_lossy(&output.stdout);
    duration_str
        .trim()
        .parse::<f64>()
        .map_err(|_| YtcsError::AudioError("Invalid duration format".to_string()))
}
