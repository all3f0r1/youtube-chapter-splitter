use crate::chapters::{Chapter, parse_chapters_from_json};
use crate::error::{Result, YtcsError};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};

pub struct VideoInfo {
    pub title: String,
    pub duration: f64,
    pub chapters: Vec<Chapter>,
    pub video_id: String,
}

pub struct MissingDependency {
    pub tool: String,
    pub install_command: String,
}

pub fn check_dependencies() -> Result<()> {
    let mut missing = Vec::new();

    // Check yt-dlp
    if Command::new("yt-dlp").arg("--version").output().is_err() {
        missing.push(MissingDependency {
            tool: "yt-dlp".to_string(),
            install_command: "pip install yt-dlp".to_string(),
        });
    }

    // Check ffmpeg
    if Command::new("ffmpeg").arg("-version").output().is_err() {
        missing.push(MissingDependency {
            tool: "ffmpeg".to_string(),
            install_command: if cfg!(target_os = "linux") {
                "sudo apt install ffmpeg".to_string()
            } else if cfg!(target_os = "macos") {
                "brew install ffmpeg".to_string()
            } else {
                "Download from https://ffmpeg.org/download.html".to_string()
            },
        });
    }

    if !missing.is_empty() {
        let mut error_msg = String::from("Missing dependencies:\n");
        for dep in &missing {
            error_msg.push_str(&format!("  - {}: {}\n", dep.tool, dep.install_command));
        }
        return Err(YtcsError::MissingTool(error_msg));
    }

    Ok(())
}

pub fn install_dependency(tool: &str) -> Result<()> {
    let command = match tool {
        "yt-dlp" => "pip install yt-dlp",
        "ffmpeg" => {
            if cfg!(target_os = "linux") {
                "sudo apt install -y ffmpeg"
            } else if cfg!(target_os = "macos") {
                "brew install ffmpeg"
            } else {
                return Err(YtcsError::Other("Please install ffmpeg manually".to_string()));
            }
        }
        _ => return Err(YtcsError::Other(format!("Unknown tool: {}", tool))),
    };

    println!("Installing {}...", tool);
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(&["/C", command]).output()
    } else {
        Command::new("sh").args(&["-c", command]).output()
    };

    match output {
        Ok(out) if out.status.success() => {
            println!("✓ {} installed successfully", tool);
            Ok(())
        }
        Ok(out) => {
            let error = String::from_utf8_lossy(&out.stderr);
            Err(YtcsError::Other(format!("Failed to install {}: {}", tool, error)))
        }
        Err(e) => Err(YtcsError::Other(format!("Failed to run install command: {}", e))),
    }
}

pub fn extract_video_id(url: &str) -> Result<String> {
    let patterns = [
        r"(?:youtube\.com/watch\?v=|youtu\.be/)([a-zA-Z0-9_-]{11})",
    ];

    for pattern in &patterns {
        let re = regex::Regex::new(pattern)?;
        if let Some(caps) = re.captures(url) {
            if let Some(id) = caps.get(1) {
                return Ok(id.as_str().to_string());
            }
        }
    }

    Err(YtcsError::InvalidUrl(format!("Unable to extract video ID from: {}", url)))
}

pub fn get_video_info(url: &str) -> Result<VideoInfo> {
    let output = Command::new("yt-dlp")
        .arg("--dump-json")
        .arg("--no-playlist")
        .arg(url)
        .output()
        .map_err(|e| YtcsError::DownloadError(format!("Failed to execute yt-dlp: {}", e)))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(YtcsError::DownloadError(format!("yt-dlp failed: {}", error)));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let data: serde_json::Value = serde_json::from_str(&json_str)?;

    let title = data["title"]
        .as_str()
        .unwrap_or("Untitled Video")
        .to_string();

    let duration = data["duration"]
        .as_f64()
        .unwrap_or(0.0);

    let video_id = data["id"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let chapters = if let Some(chapters_array) = data["chapters"].as_array() {
        if !chapters_array.is_empty() {
            parse_chapters_from_json(&json_str).unwrap_or_else(|_| Vec::new())
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    Ok(VideoInfo {
        title,
        duration,
        chapters,
        video_id,
    })
}

pub fn download_audio(url: &str, output_path: &PathBuf) -> Result<PathBuf> {
    println!("Downloading audio from YouTube...");
    
    // Créer une barre de progression
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {percent}% {msg}")
            .unwrap()
            .progress_chars("#>-")
    );
    
    let mut child = Command::new("yt-dlp")
        .arg("-x")
        .arg("--audio-format")
        .arg("mp3")
        .arg("--audio-quality")
        .arg("0")
        .arg("-o")
        .arg(output_path.to_str().unwrap())
        .arg("--no-playlist")
        .arg("--newline")
        .arg(url)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| YtcsError::DownloadError(format!("Download failed: {}", e)))?;

    // Lire la sortie pour extraire la progression
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                // Chercher les lignes de progression de yt-dlp
                if line.contains("[download]") {
                    if let Some(percent_str) = line.split_whitespace()
                        .find(|s| s.ends_with('%'))
                        .and_then(|s| s.trim_end_matches('%').parse::<u64>().ok()) {
                        pb.set_position(percent_str);
                    }
                    pb.set_message(line.clone());
                }
            }
        }
    }

    let output = child.wait_with_output()
        .map_err(|e| YtcsError::DownloadError(format!("Failed to wait for yt-dlp: {}", e)))?;

    pb.finish_with_message("Download complete");

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(YtcsError::DownloadError(format!("yt-dlp failed: {}", error)));
    }

    // yt-dlp adds .mp3 automatically
    let mut final_path = output_path.clone();
    final_path.set_extension("mp3");

    if !final_path.exists() {
        return Err(YtcsError::DownloadError("Audio file was not created".to_string()));
    }

    Ok(final_path)
}


/// Télécharge la miniature d'une vidéo YouTube.
///
/// Tente de télécharger la miniature en plusieurs qualités (maxres, hq, mq)
/// avec timeout et retry automatique.
///
/// # Arguments
///
/// * `url` - L'URL de la vidéo YouTube
/// * `output_dir` - Le répertoire de sortie pour la miniature
///
/// # Returns
///
/// Le chemin du fichier de miniature téléchargé
///
/// # Errors
///
/// Retourne une erreur si aucune miniature n'a pu être téléchargée
pub fn download_thumbnail(url: &str, output_dir: &std::path::Path) -> Result<std::path::PathBuf> {
    // Get video ID
    let video_id = extract_video_id(url)?;
    
    // YouTube thumbnail URLs (try different qualities)
    let thumbnail_urls = vec![
        format!("https://img.youtube.com/vi/{}/maxresdefault.jpg", video_id),
        format!("https://img.youtube.com/vi/{}/hqdefault.jpg", video_id),
        format!("https://img.youtube.com/vi/{}/mqdefault.jpg", video_id),
    ];
    
    let output_path = output_dir.join("cover.jpg");
    
    // Créer un agent avec timeout
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(30))
        .build();
    
    // Try each thumbnail URL with retry
    for thumb_url in thumbnail_urls {
        // Retry jusqu'à 3 fois
        for attempt in 1..=3 {
            match agent.get(&thumb_url).call() {
                Ok(response) if response.status() == 200 => {
                    let mut reader = response.into_reader();
                    let mut bytes = Vec::new();
                    std::io::Read::read_to_end(&mut reader, &mut bytes)
                        .map_err(|e| YtcsError::DownloadError(format!("Failed to read thumbnail: {}", e)))?;
                    
                    std::fs::write(&output_path, bytes)?;
                    return Ok(output_path);
                }
                Err(e) if attempt < 3 => {
                    eprintln!("Attempt {}/3 failed for {}: {}", attempt, thumb_url, e);
                    std::thread::sleep(std::time::Duration::from_secs(1));
                    continue;
                }
                _ => break,
            }
        }
    }
    
    Err(YtcsError::DownloadError("Could not download thumbnail from any source".to_string()))
}
