//! YouTube video chapter management.
//!
//! This module provides structures and functions to manipulate chapters
//! extracted from YouTube videos.

use crate::error::{Result, YtcsError};
use crate::utils;
use serde::{Deserialize, Serialize};

/// Represents a chapter of a YouTube video.
///
/// A chapter is defined by a title and a time range (start and end).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub title: String,
    pub start_time: f64,
    pub end_time: f64,
}

impl Chapter {
    /// Creates a new chapter.
    ///
    /// # Arguments
    ///
    /// * `title` - The chapter title
    /// * `start_time` - The start time in seconds (must be >= 0)
    /// * `end_time` - The end time in seconds (must be > start_time)
    ///
    /// # Panics
    ///
    /// Panics if start_time < 0 or if end_time <= start_time
    pub fn new(title: String, start_time: f64, end_time: f64) -> Self {
        assert!(
            start_time >= 0.0,
            "start_time must be >= 0, got {}",
            start_time
        );
        assert!(
            end_time > start_time,
            "end_time ({}) must be > start_time ({})",
            end_time,
            start_time
        );

        Self {
            title,
            start_time,
            end_time,
        }
    }

    /// Calculates the chapter duration in seconds.
    ///
    /// # Returns
    ///
    /// The chapter duration (end_time - start_time)
    pub fn duration(&self) -> f64 {
        self.end_time - self.start_time
    }

    /// Returns the title in a display-friendly format (Title Case).
    ///
    /// Converts all-uppercase titles to Title Case for better readability.
    /// Leaves mixed-case titles unchanged.
    ///
    /// # Returns
    ///
    /// The title formatted for display
    pub fn display_title(&self) -> String {
        // If the title is mostly uppercase, convert to title case
        if self.title.chars().filter(|c| c.is_uppercase()).count() > self.title.len() / 2 {
            to_title_case(&self.title)
        } else {
            self.title.clone()
        }
    }

    /// Cleans the chapter title for use as a filename.
    ///
    /// Delegates processing to [`utils::sanitize_title`].
    ///
    /// # Returns
    ///
    /// A cleaned title safe for use as a filename
    pub fn sanitize_title(&self) -> String {
        utils::sanitize_title(&self.title)
    }
}

/// Converts a string to title case.
///
/// Each word is capitalized (first letter uppercase, rest lowercase).
/// Small words like "a", "an", "the", "in", "on", "of", "and" are not capitalized
/// unless they are the first word.
fn to_title_case(s: &str) -> String {
    let small_words = [
        "a", "an", "the", "in", "on", "of", "and", "or", "but", "for", "nor", "to", "at", "by",
    ];

    s.split_whitespace()
        .enumerate()
        .map(|(i, word)| {
            let lower = word.to_lowercase();
            if i > 0 && small_words.contains(&lower.as_str()) {
                lower
            } else {
                let mut chars = lower.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Parses chapters from yt-dlp JSON output.
///
/// Extracts chapters from YouTube video JSON metadata.
///
/// # Arguments
///
/// * `json_str` - The JSON string containing video metadata
///
/// # Returns
///
/// A vector of extracted chapters, or an error if parsing fails
///
/// # Errors
///
/// Returns an error if:
/// - JSON is malformed
/// - The "chapters" field is missing
/// - start_time or end_time fields are invalid
pub fn parse_chapters_from_json(json_str: &str) -> Result<Vec<Chapter>> {
    let data: serde_json::Value = serde_json::from_str(json_str)?;

    let chapters_array = data["chapters"]
        .as_array()
        .ok_or_else(|| YtcsError::ChapterError("No chapters found".to_string()))?;

    let mut chapters = Vec::new();
    for (i, chapter) in chapters_array.iter().enumerate() {
        let title = chapter["title"]
            .as_str()
            .unwrap_or(&format!("Track {}", i + 1))
            .to_string();

        let start_time = chapter["start_time"]
            .as_f64()
            .ok_or_else(|| YtcsError::ChapterError("Invalid start_time".to_string()))?;

        let end_time = chapter["end_time"]
            .as_f64()
            .ok_or_else(|| YtcsError::ChapterError("Invalid end_time".to_string()))?;

        chapters.push(Chapter::new(title, start_time, end_time));
    }

    Ok(chapters)
}

/// Parses a timestamp in HH:MM:SS, MM:SS, or SS format.
///
/// # Arguments
///
/// * `timestamp` - The timestamp to parse (ex: "1:23:45", "5:30", "42")
///
/// # Returns
///
/// The number of seconds corresponding to the timestamp
///
/// # Errors
///
/// Returns an error if the timestamp format is invalid
pub fn parse_timestamp(timestamp: &str) -> Result<f64> {
    let parts: Vec<&str> = timestamp.split(':').collect();

    let seconds = match parts.len() {
        1 => parts[0]
            .parse::<f64>()
            .map_err(|_| YtcsError::ChapterError("Invalid timestamp format".to_string()))?,
        2 => {
            let minutes = parts[0]
                .parse::<f64>()
                .map_err(|_| YtcsError::ChapterError("Invalid minutes".to_string()))?;
            let seconds = parts[1]
                .parse::<f64>()
                .map_err(|_| YtcsError::ChapterError("Invalid seconds".to_string()))?;
            minutes * 60.0 + seconds
        }
        3 => {
            let hours = parts[0]
                .parse::<f64>()
                .map_err(|_| YtcsError::ChapterError("Invalid hours".to_string()))?;
            let minutes = parts[1]
                .parse::<f64>()
                .map_err(|_| YtcsError::ChapterError("Invalid minutes".to_string()))?;
            let seconds = parts[2]
                .parse::<f64>()
                .map_err(|_| YtcsError::ChapterError("Invalid seconds".to_string()))?;
            hours * 3600.0 + minutes * 60.0 + seconds
        }
        _ => {
            return Err(YtcsError::ChapterError(
                "Invalid timestamp format".to_string(),
            ));
        }
    };

    Ok(seconds)
}

/// Formats a number of seconds as HH:MM:SS or MM:SS timestamp.
///
/// # Arguments
///
/// * `seconds` - The number of seconds to format
///
/// # Returns
///
/// A formatted timestamp (HH:MM:SS if >= 1h, otherwise MM:SS)
pub fn format_timestamp(seconds: f64) -> String {
    let hours = (seconds / 3600.0).floor() as u32;
    let minutes = ((seconds % 3600.0) / 60.0).floor() as u32;
    let secs = (seconds % 60.0).floor() as u32;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{:02}:{:02}", minutes, secs)
    }
}
