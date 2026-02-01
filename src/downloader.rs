//! Téléchargement de vidéos YouTube et extraction de métadonnées.
//!
//! This module handles l'interaction avec `yt-dlp` pour télécharger les vidéos
//! et extraire leurs métadonnées (titre, durée, chapitres).

use crate::chapters::{Chapter, parse_chapters_from_json};
use crate::cookie_helper;
use crate::error::{Result, YtcsError};
use crate::ytdlp_error_parser;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Informations sur une vidéo YouTube.
///
/// Cette structure contient toutes les métadonnées nécessaires
/// pour télécharger et découper une vidéo en pistes audio.
///
/// # Examples
///
/// ```no_run
/// use youtube_chapter_splitter::downloader::get_video_info;
///
/// let video_info = get_video_info("https://youtube.com/watch?v=...", None)?;
/// println!("Title: {}", video_info.title);
/// println!("Duration: {} seconds", video_info.duration);
/// println!("Chapters: {}", video_info.chapters.len());
/// # Ok::<(), youtube_chapter_splitter::error::YtcsError>(())
/// ```
#[derive(Debug, Clone)]
pub struct VideoInfo {
    /// Titre de la vidéo (ex: "Artist - Album Name")
    pub title: String,

    /// Durée totale en secondes
    pub duration: f64,

    /// Liste des chapitres détectés dans la vidéo
    pub chapters: Vec<Chapter>,

    /// Identifiant unique de la vidéo (11 caractères)
    pub video_id: String,

    /// URL de la miniature (thumbnail)
    pub thumbnail_url: String,

    /// Nom de la chaîne qui a uploadé la vidéo
    pub uploader: String,

    /// Description complète de la vidéo
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
/// Returns an error if `yt-dlp` or `ffmpeg` are missing
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
/// Returns an error if installation fails
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
/// Returns an error if the URL is invalid or if the ID cannot be extracted
pub fn extract_video_id(url: &str) -> Result<String> {
    let patterns = [r"(?:youtube\.com/watch\?v=|youtu\.be/)([a-zA-Z0-9_-]{11})"];

    for pattern in &patterns {
        let re = regex::Regex::new(pattern)?;
        if let Some(caps) = re.captures(url)
            && let Some(id) = caps.get(1)
        {
            return Ok(id.as_str().to_string());
        }
    }

    Err(YtcsError::InvalidUrl(format!(
        "Unable to extract video ID from: {}",
        url
    )))
}

/// Check if an error is likely caused by expired/invalid cookies.
fn is_cookie_related_error(stderr: &str) -> bool {
    let error_lower = stderr.to_lowercase();
    // HTTP 403 Forbidden, 401 Unauthorized, or explicit cookie/auth errors
    error_lower.contains("http error 403")
        || error_lower.contains("http error 401")
        || error_lower.contains("forbidden")
        || error_lower.contains(" unauthorized")
        || error_lower.contains("invalid cookies")
        || error_lower.contains("cookies have expired")
}

/// Try to get video info, automatically retrying without cookies if cookie-related error occurs.
fn get_video_info_impl(
    url: &str,
    cookies_from_browser: Option<&str>,
    with_cookies: bool,
) -> Result<VideoInfo> {
    let mut cmd = Command::new("yt-dlp");
    cmd.arg("--dump-json").arg("--no-playlist");

    if with_cookies {
        cookie_helper::add_cookie_args(&mut cmd, cookies_from_browser);
    }

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

/// Retrieves information from a YouTube video.
///
/// Uses `yt-dlp` to extract video metadata. Automatically falls back to no cookies
/// if cookies are expired/invalid (HTTP 403/401 errors).
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
/// Returns an error if yt-dlp échoue ou si les métadonnées sont invalides
pub fn get_video_info(url: &str, cookies_from_browser: Option<&str>) -> Result<VideoInfo> {
    // Check if we have cookies available
    let has_cookies = cookie_helper::cookies_available(cookies_from_browser);

    if !has_cookies {
        return get_video_info_impl(url, cookies_from_browser, false);
    }

    // Try with cookies first
    match get_video_info_impl(url, cookies_from_browser, true) {
        Ok(info) => Ok(info),
        Err(YtcsError::DownloadError(e)) if is_cookie_related_error(&e) => {
            // Cookie-related error, retry without cookies
            eprintln!(
                "{}",
                "WARNING: Cookies failed (expired/invalid). Retrying without cookies...".yellow()
            );
            get_video_info_impl(url, cookies_from_browser, false)
        }
        Err(e) => Err(e),
    }
}

/// Internal implementation for audio download with specific cookie setting.
fn download_audio_impl(
    url: &str,
    output_path: &Path,
    cookies_from_browser: Option<&str>,
    pb: Option<ProgressBar>,
    with_cookies: bool,
) -> Result<PathBuf> {
    log::info!(
        "Starting audio download from: {} (with_cookies: {})",
        url,
        with_cookies
    );
    log::debug!("Output path: {:?}", output_path);

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

    const FORMAT_SELECTORS: &[Option<&str>] = &[
        Some("bestaudio[ext=m4a]/bestaudio"),
        Some("140"),
        Some("bestaudio"),
        None,
    ];

    #[allow(unused_assignments)]
    let mut last_error: Option<String> = None;

    for (i, format) in FORMAT_SELECTORS.iter().enumerate() {
        log::debug!("Trying format selector #{}: {:?}", i + 1, format);
        let mut cmd = Command::new("yt-dlp");

        if let Some(fmt) = format {
            cmd.arg("-f").arg(fmt);
        }

        cmd.arg("-x")
            .arg("--audio-format")
            .arg("mp3")
            .arg("--audio-quality")
            .arg("0")
            .arg("-o")
            .arg(output_path.to_str().unwrap())
            .arg("--no-playlist");

        if with_cookies {
            cookie_helper::add_cookie_args(&mut cmd, cookies_from_browser);
        }

        let output = cmd
            .arg(url)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| YtcsError::DownloadError(format!("Download failed: {}", e)))?;

        if output.status.success() {
            log::info!("Audio download successful with format selector #{}", i + 1);
            progress_bar.finish_and_clear();
            break;
        } else {
            let error_msg = String::from_utf8_lossy(&output.stderr).to_string();
            log::debug!(
                "Format selector #{} failed: {}",
                i + 1,
                error_msg.lines().next().unwrap_or("Unknown error")
            );

            if i < FORMAT_SELECTORS.len() - 1 {
                progress_bar.set_message(format!(
                    "Audio downloading (option {}/{} failed)",
                    i + 1,
                    FORMAT_SELECTORS.len()
                ));
            }

            last_error = Some(error_msg);
            if i < FORMAT_SELECTORS.len() - 1 {
                continue;
            } else {
                progress_bar.finish_and_clear();
                return Err(YtcsError::DownloadError(format!(
                    "yt-dlp failed with all format selectors. Last error: {}",
                    last_error.unwrap_or_else(|| "Unknown error".to_string())
                )));
            }
        }
    }

    let mut final_path = output_path.to_path_buf();
    final_path.set_extension("mp3");

    Ok(final_path)
}

/// Télécharge l'audio d'une vidéo YouTube en format MP3.
///
/// This function uses `yt-dlp` avec une stratégie de fallback à 4 niveaux
/// pour maximiser la fiabilité du téléchargement :
/// 1. `bestaudio[ext=m4a]/bestaudio` - Audio M4A de meilleure qualité (préféré)
/// 2. `140` - Format M4A standard de YouTube (très fiable)
/// 3. `bestaudio` - Sélecteur audio générique
/// 4. Auto-sélection - Laisse yt-dlp choisir automatiquement
///
/// Automatically falls back to no cookies if cookies are expired/invalid (HTTP 403/401 errors).
///
/// # Arguments
///
/// * `url` - L'URL de la vidéo YouTube
/// * `output_path` - Le chemin de sortie (sans extension, .mp3 sera ajouté automatiquement)
/// * `cookies_from_browser` - Optionnel : navigateur pour extraire les cookies (ex: "firefox", "chrome")
/// * `pb` - Optionnel : barre de progression personnalisée
///
/// # Returns
///
/// Le chemin du fichier MP3 téléchargé
///
/// # Errors
///
/// Returns an error if :
/// - yt-dlp n'est pas installé
/// - Tous les sélecteurs de format échouent
/// - Le téléchargement est interrompu
///
/// # Examples
///
/// ```no_run
/// use youtube_chapter_splitter::downloader::download_audio;
/// use std::path::Path;
///
/// let audio_file = download_audio(
///     "https://youtube.com/watch?v=dQw4w9WgXcQ",
///     Path::new("/tmp/audio"),
///     None,
///     None,
/// )?;
/// println!("Downloaded to: {:?}", audio_file);
/// # Ok::<(), youtube_chapter_splitter::error::YtcsError>(())
/// ```
pub fn download_audio(
    url: &str,
    output_path: &Path,
    cookies_from_browser: Option<&str>,
    pb: Option<ProgressBar>,
) -> Result<PathBuf> {
    let has_cookies = cookie_helper::cookies_available(cookies_from_browser);

    if !has_cookies {
        return download_audio_impl(url, output_path, cookies_from_browser, pb, false);
    }

    // Try with cookies first
    match download_audio_impl(url, output_path, cookies_from_browser, pb.clone(), true) {
        Ok(path) => Ok(path),
        Err(YtcsError::DownloadError(e)) if is_cookie_related_error(&e) => {
            // Cookie-related error, retry without cookies
            eprintln!(
                "{}",
                "WARNING: Cookies failed (expired/invalid). Retrying without cookies...".yellow()
            );
            download_audio_impl(url, output_path, cookies_from_browser, pb, false)
        }
        Err(e) => Err(e),
    }
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
/// Returns an error if aucune miniature n'a pu être téléchargée
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

#[cfg(test)]
mod tests {
    use super::*;

    // Test extract_video_id with various URL formats
    #[test]
    fn test_extract_video_id_youtube_com() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
        let result = extract_video_id(url);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_youtu_be() {
        let url = "https://youtu.be/dQw4w9WgXcQ";
        let result = extract_video_id(url);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_with_parameters() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=10s";
        let result = extract_video_id(url);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_short() {
        let url = "https://youtu.be/9bZkp7q19f0";
        let result = extract_video_id(url);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "9bZkp7q19f0");
    }

    #[test]
    fn test_extract_video_id_underscores_and_hyphens() {
        // YouTube IDs are exactly 11 characters
        let url = "https://www.youtube.com/watch?v=aB-c_d01234";
        let result = extract_video_id(url);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "aB-c_d01234");
    }

    #[test]
    fn test_extract_video_id_invalid_url() {
        let url = "https://example.com/watch?v=invalid";
        let result = extract_video_id(url);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_video_id_empty_string() {
        let result = extract_video_id("");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_video_id_too_short_id() {
        let url = "https://youtu.be/short";
        let result = extract_video_id(url);
        assert!(result.is_err());
    }

    // Test is_cookie_related_error function
    #[test]
    fn test_is_cookie_error_403() {
        let stderr = "HTTP Error 403: Forbidden";
        assert!(is_cookie_related_error(stderr));
    }

    #[test]
    fn test_is_cookie_error_401() {
        let stderr = "HTTP Error 401: Unauthorized";
        assert!(is_cookie_related_error(stderr));
    }

    #[test]
    fn test_is_cookie_error_forbidden() {
        let stderr = "Access forbidden for this video";
        assert!(is_cookie_related_error(stderr));
    }

    #[test]
    fn test_is_cookie_error_unauthorized() {
        let stderr = "This video is unauthorized";
        assert!(is_cookie_related_error(stderr));
    }

    #[test]
    fn test_is_cookie_error_expired_cookies() {
        let stderr = "cookies have expired, please update";
        assert!(is_cookie_related_error(stderr));
    }

    #[test]
    fn test_is_cookie_error_invalid_cookies() {
        let stderr = "invalid cookies detected";
        assert!(is_cookie_related_error(stderr));
    }

    #[test]
    fn test_is_cookie_error_case_insensitive() {
        let stderr = "HTTP ERROR 403: FORBIDDEN";
        assert!(is_cookie_related_error(stderr));
    }

    #[test]
    fn test_is_not_cookie_error_404() {
        let stderr = "HTTP Error 404: Not Found";
        assert!(!is_cookie_related_error(stderr));
    }

    #[test]
    fn test_is_not_cookie_error_network() {
        let stderr = "Network connection failed";
        assert!(!is_cookie_related_error(stderr));
    }

    // Test VideoInfo struct
    #[test]
    fn test_video_info_creation() {
        let info = VideoInfo {
            title: "Test Video".to_string(),
            duration: 120.0,
            chapters: vec![],
            video_id: "abc123".to_string(),
            thumbnail_url: "https://example.com/thumb.jpg".to_string(),
            uploader: "Test Channel".to_string(),
            description: "Test description".to_string(),
        };
        assert_eq!(info.title, "Test Video");
        assert_eq!(info.duration, 120.0);
        assert_eq!(info.video_id, "abc123");
    }

    // Test MissingDependency struct
    #[test]
    fn test_missing_dependency_creation() {
        let dep = MissingDependency {
            tool: "ffmpeg".to_string(),
            install_command: "sudo apt install ffmpeg".to_string(),
        };
        assert_eq!(dep.tool, "ffmpeg");
        assert!(dep.install_command.contains("install"));
    }

    // Integration test: check_dependencies with actual system
    #[test]
    fn test_check_dependencies_returns_result() {
        // This test just verifies the function runs without panicking
        // It may fail if dependencies are missing, which is expected
        let result = check_dependencies();
        // We don't assert success/failure since it depends on the test environment
        let _ = result;
    }

    #[test]
    fn test_extract_video_id_mixed_case() {
        // YouTube IDs are case-sensitive and exactly 11 characters
        let url = "https://youtu.be/AbCdEf12345";
        let result = extract_video_id(url);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "AbCdEf12345");
    }

    #[test]
    fn test_extract_video_id_with_playlist_param() {
        // Should still work even with list parameter
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=PLxyz";
        let result = extract_video_id(url);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_mobile_url() {
        let url = "https://m.youtube.com/watch?v=dQw4w9WgXcQ";
        let result = extract_video_id(url);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_www_less() {
        let url = "https://youtube.com/watch?v=dQw4w9WgXcQ";
        let result = extract_video_id(url);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "dQw4w9WgXcQ");
    }
}
