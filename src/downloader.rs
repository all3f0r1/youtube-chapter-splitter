//! Téléchargement de vidéos YouTube et extraction de métadonnées.
//!
//! Ce module gère l'interaction avec `yt-dlp` pour télécharger les vidéos
//! et extraire leurs métadonnées (titre, durée, chapitres).

use crate::chapters::{parse_chapters_from_json, Chapter};
use crate::cookie_helper;
use crate::error::{Result, YtcsError};
use crate::ytdlp_error_parser;
use indicatif::{ProgressBar, ProgressStyle};

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Informations sur une vidéo YouTube.
#[derive(Debug, Clone)]
pub struct VideoInfo {
    pub title: String,
    pub duration: f64,
    pub chapters: Vec<Chapter>,
    pub video_id: String,
    pub thumbnail_url: String,
    pub uploader: String,
    pub description: String,
}

/// Informations sur une dépendance système manquante.
pub struct MissingDependency {
    pub tool: String,
    pub install_command: String,
}

/// Vérifie la présence des dépendances système requises.
///
/// # Returns
///
/// Ok si toutes les dépendances sont présentes, sinon une erreur avec les détails
///
/// # Errors
///
/// Retourne une erreur si `yt-dlp` ou `ffmpeg` sont manquants
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

/// Installe une dépendance système manquante.
///
/// # Arguments
///
/// * `tool` - Le nom de l'outil à installer ("yt-dlp" ou "ffmpeg")
///
/// # Errors
///
/// Retourne une erreur si l'installation échoue
pub fn install_dependency(tool: &str) -> Result<()> {
    let command = match tool {
        "yt-dlp" => "pip install yt-dlp",
        "ffmpeg" => {
            if cfg!(target_os = "linux") {
                "sudo apt install -y ffmpeg"
            } else if cfg!(target_os = "macos") {
                "brew install ffmpeg"
            } else {
                return Err(YtcsError::Other(
                    "Please install ffmpeg manually".to_string(),
                ));
            }
        }
        _ => return Err(YtcsError::Other(format!("Unknown tool: {}", tool))),
    };

    println!("Installing {}...", tool);
    let output = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", command]).output()
    } else {
        Command::new("sh").args(["-c", command]).output()
    };

    match output {
        Ok(out) if out.status.success() => {
            println!("✓ {} installed successfully", tool);
            Ok(())
        }
        Ok(out) => {
            let error = String::from_utf8_lossy(&out.stderr);
            Err(YtcsError::Other(format!(
                "Failed to install {}: {}",
                tool, error
            )))
        }
        Err(e) => Err(YtcsError::Other(format!(
            "Failed to run install command: {}",
            e
        ))),
    }
}

/// Extrait l'ID d'une vidéo YouTube depuis son URL.
///
/// # Arguments
///
/// * `url` - L'URL de la vidéo YouTube
///
/// # Returns
///
/// L'ID de la vidéo (11 caractères)
///
/// # Errors
///
/// Retourne une erreur si l'URL est invalide ou si l'ID ne peut pas être extrait
pub fn extract_video_id(url: &str) -> Result<String> {
    let patterns = [r"(?:youtube\.com/watch\?v=|youtu\.be/)([a-zA-Z0-9_-]{11})"];

    for pattern in &patterns {
        let re = regex::Regex::new(pattern)?;
        if let Some(caps) = re.captures(url) {
            if let Some(id) = caps.get(1) {
                return Ok(id.as_str().to_string());
            }
        }
    }

    Err(YtcsError::InvalidUrl(format!(
        "Unable to extract video ID from: {}",
        url
    )))
}

/// Récupère les informations d'une vidéo YouTube.
///
/// Utilise `yt-dlp` pour extraire les métadonnées de la vidéo.
///
/// # Arguments
///
/// * `url` - L'URL de la vidéo YouTube
///
/// # Returns
///
/// Les informations de la vidéo (titre, durée, chapitres, ID)
///
/// # Errors
///
/// Retourne une erreur si yt-dlp échoue ou si les métadonnées sont invalides
pub fn get_video_info(url: &str, cookies_from_browser: Option<&str>) -> Result<VideoInfo> {
    let mut cmd = Command::new("yt-dlp");
    cmd.arg("--dump-json").arg("--no-playlist");

    // Add cookie arguments
    cookie_helper::add_cookie_args(&mut cmd, cookies_from_browser);

    let output = cmd
        .arg(url)
        .output()
        .map_err(|e| YtcsError::DownloadError(format!("Failed to execute yt-dlp: {}", e)))?;

    if !output.status.success() {
        let raw_error = String::from_utf8_lossy(&output.stderr);
        let (error_msg, suggestion) =
            ytdlp_error_parser::parse_ytdlp_error(&raw_error, cookies_from_browser);

        let full_error = if let Some(sug) = suggestion {
            format!("{}\n\n{}", error_msg, sug)
        } else {
            error_msg
        };

        return Err(YtcsError::DownloadError(full_error));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let data: serde_json::Value = serde_json::from_str(&json_str)?;

    let title = data["title"]
        .as_str()
        .unwrap_or("Untitled Video")
        .to_string();

    let duration = data["duration"].as_f64().unwrap_or(0.0);

    let video_id = data["id"].as_str().unwrap_or("").to_string();

    let thumbnail_url = data["thumbnail"].as_str().unwrap_or("").to_string();

    let uploader = data["uploader"].as_str().unwrap_or("Unknown").to_string();

    let description = data["description"].as_str().unwrap_or("").to_string();

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
        thumbnail_url,
        uploader,
        description,
    })
}

/// Télécharge l'audio d'une vidéo YouTube en format MP3.
///
/// Utilise `yt-dlp` avec une barre de progression pour télécharger et convertir l'audio.
///
/// # Arguments
///
/// * `url` - L'URL de la vidéo YouTube
/// * `output_path` - Le chemin de sortie (sans extension)
///
/// # Returns
///
/// Le chemin du fichier MP3 téléchargé
///
/// # Errors
///
/// Retourne une erreur si le téléchargement échoue
pub fn download_audio(
    url: &str,
    output_path: &Path,
    cookies_from_browser: Option<&str>,
    pb: Option<ProgressBar>,
) -> Result<PathBuf> {
    let progress_bar = pb.unwrap_or_else(|| {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message("Downloading audio from YouTube...");
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        pb
    });

    // Try multiple format selectors with fallback
    let format_selectors = vec![
        "bestaudio[ext=m4a]/bestaudio",
        "140",       // YouTube M4A audio format
        "bestaudio", // Generic best audio
    ];

    let mut last_error = String::new();

    for (i, format) in format_selectors.iter().enumerate() {
        let mut cmd = Command::new("yt-dlp");
        cmd.arg("-f")
            .arg(format)
            .arg("-x")
            .arg("--audio-format")
            .arg("mp3")
            .arg("--audio-quality")
            .arg("0")
            .arg("-o")
            .arg(output_path.to_str().unwrap())
            .arg("--no-playlist");

        // Add cookie arguments
        cookie_helper::add_cookie_args(&mut cmd, cookies_from_browser);

        let output = cmd
            .arg(url)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| YtcsError::DownloadError(format!("Download failed: {}", e)))?;

        if output.status.success() {
            progress_bar.finish_and_clear();
            break;
        } else {
            last_error = String::from_utf8_lossy(&output.stderr).to_string();
            // If this is not the last format, try the next one
            if i < format_selectors.len() - 1 {
                continue;
            } else {
                // All formats failed, return error
                progress_bar.finish_and_clear();
                return Err(YtcsError::DownloadError(format!(
                    "yt-dlp failed with all format selectors. Last error: {}",
                    last_error
                )));
            }
        }
    }

    // yt-dlp adds .mp3 automatically
    let mut final_path = output_path.to_path_buf();
    final_path.set_extension("mp3");

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
    // Si l'URL est déjà une URL d'image, l'utiliser directement
    let thumbnail_urls = if url.contains("ytimg.com") || url.contains("img.youtube.com") {
        vec![url.to_string()]
    } else {
        // Sinon, extraire le video ID et construire les URLs
        let video_id = extract_video_id(url)?;
        vec![
            format!("https://img.youtube.com/vi/{}/maxresdefault.jpg", video_id),
            format!("https://img.youtube.com/vi/{}/hqdefault.jpg", video_id),
            format!("https://img.youtube.com/vi/{}/mqdefault.jpg", video_id),
        ]
    };

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
                    std::io::Read::read_to_end(&mut reader, &mut bytes).map_err(|e| {
                        YtcsError::DownloadError(format!("Failed to read thumbnail: {}", e))
                    })?;

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

    Err(YtcsError::DownloadError(
        "Could not download thumbnail from any source".to_string(),
    ))
}
