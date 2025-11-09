use crate::chapters::Chapter;
use crate::error::{Result, YtcsError};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn split_audio_by_chapters(
    input_file: &Path,
    chapters: &[Chapter],
    output_dir: &Path,
    artist: &str,
    album: &str,
    cover_path: Option<&Path>,
) -> Result<Vec<PathBuf>> {
    println!("Splitting audio into {} tracks...", chapters.len());
    
    std::fs::create_dir_all(output_dir)?;
    
    let mut output_files = Vec::new();

    for (index, chapter) in chapters.iter().enumerate() {
        let track_number = index + 1;
        let sanitized_title = chapter.sanitize_title();
        let output_filename = format!("{:02} - {}.mp3", track_number, sanitized_title);
        let output_path = output_dir.join(&output_filename);

        println!("  Track {}/{}: {}", track_number, chapters.len(), chapter.title);

        let duration = chapter.duration();
        
        let mut cmd = Command::new("ffmpeg");
        
        // Add inputs: audio first, then cover if available
        cmd.arg("-i").arg(input_file);
        if let Some(cover) = cover_path {
            cmd.arg("-i").arg(cover);
        }
        
        // Seek and duration
        cmd.arg("-ss")
            .arg(chapter.start_time.to_string())
            .arg("-t")
            .arg(duration.to_string());
        
        // Map streams
        if cover_path.is_some() {
            cmd.arg("-map").arg("0:a")  // Audio from first input
               .arg("-map").arg("1:v");  // Video (image) from second input
        }
        
        // Audio encoding
        cmd.arg("-c:a")
            .arg("libmp3lame")
            .arg("-q:a")
            .arg("0");
        
        // Cover art encoding (simplified for Android compatibility)
        if cover_path.is_some() {
            cmd.arg("-c:v").arg("copy")  // Copy image without re-encoding
               .arg("-disposition:v").arg("attached_pic");  // Mark as attached picture
            // Note: Removed -metadata:s:v flags for better Android compatibility
            // Android doesn't always recognize stream-specific metadata
        }
        
        cmd.arg("-metadata")
            .arg(format!("title={}", chapter.title))
            .arg("-metadata")
            .arg(format!("artist={}", artist))
            .arg("-metadata")
            .arg(format!("album={}", album))
            .arg("-metadata")
            .arg(format!("track={}/{}", track_number, chapters.len()))
            .arg("-y")
            .arg(&output_path);
        
        let output = cmd.output()
            .map_err(|e| YtcsError::AudioError(format!("Failed to execute ffmpeg: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(YtcsError::AudioError(format!("ffmpeg failed: {}", error)));
        }

        output_files.push(output_path);
    }

    println!("✓ Splitting completed successfully!");
    Ok(output_files)
}

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
    
    let silence_start_re = regex::Regex::new(r"silence_start: ([\d.]+)")?;
    let silence_end_re = regex::Regex::new(r"silence_end: ([\d.]+)")?;

    let mut silence_periods = Vec::new();
    let mut current_start: Option<f64> = None;

    for line in stderr.lines() {
        if let Some(caps) = silence_start_re.captures(line) {
            if let Some(start_str) = caps.get(1) {
                current_start = start_str.as_str().parse::<f64>().ok();
            }
        } else if let Some(caps) = silence_end_re.captures(line) {
            if let (Some(start), Some(end_str)) = (current_start, caps.get(1)) {
                if let Ok(end) = end_str.as_str().parse::<f64>() {
                    let mid_point = (start + end) / 2.0;
                    silence_periods.push(mid_point);
                }
                current_start = None;
            }
        }
    }

    if silence_periods.is_empty() {
        return Err(YtcsError::ChapterError(
            "No silence detected. Try adjusting the parameters.".to_string()
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
