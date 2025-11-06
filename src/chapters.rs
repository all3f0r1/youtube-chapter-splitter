use serde::{Deserialize, Serialize};
use crate::error::{Result, YtcsError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub title: String,
    pub start_time: f64,
    pub end_time: f64,
}

impl Chapter {
    pub fn new(title: String, start_time: f64, end_time: f64) -> Self {
        Self {
            title,
            start_time,
            end_time,
        }
    }

    pub fn duration(&self) -> f64 {
        self.end_time - self.start_time
    }

    pub fn sanitize_title(&self) -> String {
        // Retirer les numéros de piste au début (ex: "1 - ", "01. ", "Track 1: ")
        let title = regex::Regex::new(r"^\s*(?:Track\s+)?\d+\s*[-.:)]\s*")
            .unwrap()
            .replace(&self.title, "");
        
        // Remplacer les caractères invalides
        title
            .chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                _ => c,
            })
            .collect()
    }
}

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

pub fn parse_timestamp(timestamp: &str) -> Result<f64> {
    let parts: Vec<&str> = timestamp.split(':').collect();
    
    let seconds = match parts.len() {
        1 => parts[0].parse::<f64>()
            .map_err(|_| YtcsError::ChapterError("Invalid timestamp format".to_string()))?,
        2 => {
            let minutes = parts[0].parse::<f64>()
                .map_err(|_| YtcsError::ChapterError("Invalid minutes".to_string()))?;
            let seconds = parts[1].parse::<f64>()
                .map_err(|_| YtcsError::ChapterError("Invalid seconds".to_string()))?;
            minutes * 60.0 + seconds
        }
        3 => {
            let hours = parts[0].parse::<f64>()
                .map_err(|_| YtcsError::ChapterError("Invalid hours".to_string()))?;
            let minutes = parts[1].parse::<f64>()
                .map_err(|_| YtcsError::ChapterError("Invalid minutes".to_string()))?;
            let seconds = parts[2].parse::<f64>()
                .map_err(|_| YtcsError::ChapterError("Invalid seconds".to_string()))?;
            hours * 3600.0 + minutes * 60.0 + seconds
        }
        _ => return Err(YtcsError::ChapterError("Invalid timestamp format".to_string())),
    };

    Ok(seconds)
}

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
