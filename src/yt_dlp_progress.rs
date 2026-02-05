//! Module pour parser la progression de yt-dlp en temps réel.
//!
//! yt-dlp affiche la progression sur stderr avec ce format :
//! [download]  23.5MiB of  52.3MiB at  2.34MiB/s ETA 00:12
//! [download]  45.0% of  ~120.00MiB at  1.23MiB/s ETA 01:23

use crate::error::{Result, YtcsError};
use crate::ytdlp_helper;
use colored::Colorize;
use indicatif::ProgressBar;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// État de la progression du téléchargement
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// Pourcentage (0-100)
    pub percentage: f64,
    /// Taille téléchargée (ex: "23.5MiB")
    pub downloaded: String,
    /// Taille totale (ex: "52.3MiB")
    pub total: String,
    /// Vitesse (ex: "2.34MiB/s")
    pub speed: String,
    /// ETA (ex: "00:12")
    pub eta: String,
}

/// Callback pour rapporter la progression du téléchargement
pub trait ProgressCallback: Send + Sync {
    /// Appelé périodiquement pendant le téléchargement avec la progression actuelle
    fn on_progress(&self, progress: &DownloadProgress);

    /// Appelé quand le téléchargement commence
    fn on_start(&self) {
        // Default implementation does nothing
    }

    /// Appelé quand le téléchargement est terminé
    fn on_complete(&self) {
        // Default implementation does nothing
    }
}

/// Implémentation vide pour quand aucun callback n'est nécessaire
pub struct NoProgressCallback;

impl ProgressCallback for NoProgressCallback {
    fn on_progress(&self, _progress: &DownloadProgress) {
        // No-op
    }
}

/// Implémentation de callback qui stocke la progression dans un Arc<Mutex>
/// Utilisé pour partager la progression entre le thread de téléchargement et la TUI
pub struct SharedProgressCallback {
    /// La progression actuelle (partagée)
    pub progress: Arc<Mutex<Option<DownloadProgress>>>,
}

impl SharedProgressCallback {
    pub fn new() -> Self {
        Self {
            progress: Arc::new(Mutex::new(None)),
        }
    }

    /// Obtenir la progression actuelle
    pub fn get_progress(&self) -> Option<DownloadProgress> {
        self.progress.lock().ok().and_then(|p| p.clone())
    }

    /// Réinitialiser la progression
    pub fn reset(&self) {
        if let Ok(mut p) = self.progress.lock() {
            *p = None;
        }
    }
}

impl Default for SharedProgressCallback {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressCallback for SharedProgressCallback {
    fn on_progress(&self, progress: &DownloadProgress) {
        if let Ok(mut p) = self.progress.lock() {
            *p = Some(progress.clone());
        }
    }
}

/// Regex compilée pour extraire le pourcentage
static RE_PERCENTAGE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\d+\.\d+)%").unwrap());

/// Regex compilée pour extraire la vitesse
static RE_SPEED: Lazy<Regex> = Lazy::new(|| Regex::new(r"at\s+([\d.]+)(GiB|MiB|KiB)/s").unwrap());

/// Regex compilée pour extraire l'ETA
static RE_ETA: Lazy<Regex> = Lazy::new(|| Regex::new(r"ETA\s+(\d{2}:\d{2})").unwrap());

/// Parse une ligne de progression de yt-dlp.
///
/// # Exemples de formats reconnus
/// - `[download]  23.5MiB of  52.3MiB at  2.34MiB/s ETA 00:12`
/// - `[download]  45.0% of  ~120.00MiB at  1.23MiB/s ETA 01:23`
/// - `[download]   3.0% at  450.00KiB/s ETA 02:34`
fn parse_download_line(line: &str) -> Option<DownloadProgress> {
    // Vérifier que c'est une ligne de download
    if !line.contains("[download]") || line.contains("100%") {
        return None;
    }

    // Extraire le pourcentage
    let percentage = if let Some(caps) = RE_PERCENTAGE.captures(line) {
        caps.get(1)?.as_str().parse().ok()?
    } else {
        // Format sans pourcentage explicite, calculer depuis les tailles
        // Parser les valeurs avec les unités
        let re_with_unit = Regex::new(r"(\d+\.\d+)(GiB|MiB|KiB)").ok()?;
        let sizes: Vec<_> = re_with_unit.captures_iter(line).collect();

        if sizes.len() >= 2 {
            let v1 = sizes[0].get(1)?.as_str().parse::<f64>().ok()?;
            let u1 = sizes[0].get(2)?.as_str();
            let v2 = sizes[1].get(1)?.as_str().parse::<f64>().ok()?;
            let u2 = sizes[1].get(2)?.as_str();

            let to_mb = |unit: &str| -> f64 {
                match unit {
                    "GiB" => 1024.0,
                    "MiB" => 1.0,
                    "KiB" => 1.0 / 1024.0,
                    _ => 1.0,
                }
            };

            ((v1 * to_mb(u1)) / (v2 * to_mb(u2)) * 100.0).min(99.9)
        } else {
            return None;
        }
    };

    // Extraire la vitesse
    let speed = RE_SPEED
        .captures(line)
        .and_then(|caps| {
            Some(format!(
                "{} {}/s",
                caps.get(1)?.as_str(),
                caps.get(2)?.as_str()
            ))
        })
        .unwrap_or_default();

    // Extraire l'ETA
    let eta = RE_ETA
        .captures(line)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
        .unwrap_or_default();

    // Extraire les tailles (réutiliser les captures si disponibles)
    let (downloaded, total) = if RE_PERCENTAGE.is_match(line) {
        // Format avec pourcentage, extraire les tailles directement
        let re_full = Regex::new(r"(\d+\.\d+)(GiB|MiB|KiB)").ok()?;
        let sizes: Vec<_> = re_full.captures_iter(line).collect();
        if sizes.len() >= 2 {
            (
                sizes[0].get(1)?.as_str().to_string(),
                sizes[1].get(1)?.as_str().to_string(),
            )
        } else {
            ("?".to_string(), "?".to_string())
        }
    } else {
        // Format sans pourcentage, réutiliser les captures du calcul
        let re_with_unit = Regex::new(r"(\d+\.\d+)(GiB|MiB|KiB)").ok()?;
        let sizes: Vec<_> = re_with_unit.captures_iter(line).collect();
        if sizes.len() >= 2 {
            (
                sizes[0].get(1)?.as_str().to_string(),
                sizes[1].get(1)?.as_str().to_string(),
            )
        } else {
            ("?".to_string(), "?".to_string())
        }
    };

    Some(DownloadProgress {
        percentage: percentage.min(99.9),
        downloaded,
        total,
        speed,
        eta,
    })
}

/// Télécharge l'audio avec une barre de progression en temps réel.
///
/// Cette fonction lit stderr de yt-dlp en continu pour afficher la progression.
/// En cas d'erreur, propose de mettre à jour yt-dlp si nécessaire.
///
/// # Arguments
///
/// * `url` - URL YouTube
/// * `output_path` - Chemin de sortie (sans extension)
/// * `cookies_from_browser` - Navigateur pour les cookies
/// * `pb` - ProgressBar optionnelle (créée automatiquement si None)
/// * `progress_shared` - Progress partagée pour TUI (Arc<Mutex<Option<DownloadProgress>>>)
pub fn download_audio_with_progress(
    url: &str,
    output_path: &std::path::Path,
    cookies_from_browser: Option<&str>,
    pb: Option<ProgressBar>,
    progress_shared: Option<&Arc<Mutex<Option<DownloadProgress>>>>,
) -> Result<std::path::PathBuf> {
    use indicatif::ProgressStyle;

    let mut progress_bar = pb.unwrap_or_else(|| {
        let pb = ProgressBar::new(100);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {percent}% {msg}")
                .unwrap()
                .progress_chars("=> "),
        );
        pb.set_message("Downloading audio...");
        pb.enable_steady_tick(Duration::from_millis(100));
        pb
    });

    // Reset progress at start
    if let Some(shared) = progress_shared {
        if let Ok(mut p) = shared.lock() {
            *p = None;
        }
    }

    // Try downloading, with auto-update retry on failure
    let mut attempts = 0;
    const MAX_ATTEMPTS: usize = 2; // Initial try + one retry after update

    loop {
        attempts += 1;

        let result = download_audio_with_progress_impl(
            url,
            output_path,
            cookies_from_browser,
            &progress_bar,
            progress_shared,
        );

        match result {
            Ok(path) => {
                progress_bar.finish_with_message("  ✓ Audio downloaded");
                println!();
                return Ok(path);
            }
            Err(YtcsError::DownloadError(ref e)) if attempts < MAX_ATTEMPTS => {
                // Check if error is due to outdated yt-dlp
                if ytdlp_helper::is_outdated_error(e) {
                    progress_bar.abandon();

                    println!();
                    println!(
                        "{}",
                        "Download failed - this may be due to an outdated yt-dlp version."
                            .red()
                            .bold()
                    );

                    match ytdlp_helper::prompt_and_update_ytdlp() {
                        Ok(true) => {
                            // Update successful, retry download
                            println!();
                            println!("{}", "Retrying download with updated yt-dlp...".cyan());
                            println!();

                            // Recreate progress bar after abandon
                            let new_pb = ProgressBar::new(100);
                            new_pb.set_style(
                                ProgressStyle::default_bar()
                                    .template(
                                        "{spinner:.green} [{bar:40.cyan/blue}] {percent}% {msg}",
                                    )
                                    .unwrap()
                                    .progress_chars("=> "),
                            );
                            new_pb.set_message("Downloading audio...");
                            new_pb.enable_steady_tick(Duration::from_millis(100));

                            // Swap the progress bar reference
                            drop(std::mem::replace(&mut progress_bar, new_pb));

                            continue;
                        }
                        Ok(false) => {
                            // User declined update
                            return Err(YtcsError::DownloadError(e.clone()));
                        }
                        Err(update_err) => {
                            // Update failed
                            println!();
                            println!(
                                "{}",
                                format!("Update failed: {}. Original error: {}", update_err, e)
                                    .red()
                            );
                            return Err(YtcsError::DownloadError(e.clone()));
                        }
                    }
                } else {
                    // Not an outdated version error, return the original error
                    return Err(YtcsError::DownloadError(e.clone()));
                }
            }
            Err(e) => {
                progress_bar.abandon();
                return Err(e);
            }
        }
    }
}

/// Implementation of the download function with progress tracking.
fn download_audio_with_progress_impl(
    url: &str,
    output_path: &std::path::Path,
    cookies_from_browser: Option<&str>,
    progress_bar: &ProgressBar,
    progress_shared: Option<&Arc<Mutex<Option<DownloadProgress>>>>,
) -> Result<std::path::PathBuf> {
    // Fallback strategy pour les formats
    const FORMAT_SELECTORS: &[Option<&str>] = &[
        Some("bestaudio[ext=m4a]/bestaudio"),
        Some("140"),
        Some("bestaudio"),
        None,
    ];

    let mut last_error = None;

    for (attempt, format) in FORMAT_SELECTORS.iter().enumerate() {
        log::debug!("Trying format selector #{}: {:?}", attempt + 1, format);

        let result = try_download_with_format(
            url,
            output_path,
            cookies_from_browser,
            *format,
            progress_bar,
            progress_shared,
        );

        match result {
            Ok(path) => {
                return Ok(path);
            }
            Err(e) => {
                log::debug!("Format selector #{} failed: {}", attempt + 1, e);
                last_error = Some(e);

                progress_bar.set_length(100);
                progress_bar.set_position(0);

                if attempt < FORMAT_SELECTORS.len() - 1 {
                    progress_bar.set_message(format!(
                        "Retrying ({}/{} failed)...",
                        attempt + 1,
                        FORMAT_SELECTORS.len()
                    ));
                }
            }
        }
    }

    // Return the last error with full stderr content
    if let Some(err) = last_error {
        Err(err)
    } else {
        Err(YtcsError::DownloadError(
            "All format selectors failed".to_string(),
        ))
    }
}

/// Tente le téléchargement avec un sélecteur de format spécifique.
fn try_download_with_format(
    url: &str,
    output_path: &std::path::Path,
    cookies_from_browser: Option<&str>,
    format_selector: Option<&str>,
    progress_bar: &ProgressBar,
    progress_shared: Option<&Arc<Mutex<Option<DownloadProgress>>>>,
) -> Result<std::path::PathBuf> {
    let mut cmd = Command::new("yt-dlp");

    if let Some(fmt) = format_selector {
        cmd.arg("-f").arg(fmt);
    }

    cmd.arg("-x")
        .arg("--audio-format")
        .arg("mp3")
        .arg("--audio-quality")
        .arg("0")
        .arg("-o")
        .arg(output_path.to_str().unwrap())
        .arg("--no-playlist")
        .arg(url)
        .stdout(Stdio::null())
        .stderr(Stdio::piped());

    // Add cookie arguments
    crate::cookie_helper::add_cookie_args(&mut cmd, cookies_from_browser);

    let mut child = cmd
        .spawn()
        .map_err(|e| YtcsError::DownloadError(format!("Failed to spawn yt-dlp: {}", e)))?;

    // Prendre ownership du stderr
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| YtcsError::DownloadError("No stderr stream".to_string()))?;

    // Shared buffer to capture stderr for error reporting
    let stderr_buffer = Arc::new(Mutex::new(String::new()));

    // Thread pour lire stderr en continu et mettre à jour la progress bar
    let pb = progress_bar.clone();
    let stderr_clone = Arc::clone(&stderr_buffer);
    // Clone the shared progress Arc for the thread
    let progress_shared_clone = progress_shared.map(Arc::clone);

    let handle = thread::spawn(move || {
        let mut stderr = stderr;
        let mut buffer = [0; 8192];
        let mut partial_line = String::new();
        let mut last_percentage = 0.0;

        loop {
            match stderr.read(&mut buffer) {
                Ok(0) => break, // EOF
                Ok(n) => {
                    let chunk = String::from_utf8_lossy(&buffer[..n]);
                    // Also store in buffer for error reporting
                    if let Ok(mut buf) = stderr_clone.lock() {
                        buf.push_str(&chunk);
                    }

                    for c in chunk.chars() {
                        if c == '\n' || c == '\r' {
                            if !partial_line.is_empty() {
                                if let Some(progress) = parse_download_line(&partial_line)
                                    && progress.percentage - last_percentage >= 0.5
                                {
                                    pb.set_length(100);
                                    pb.set_position(progress.percentage as u64);
                                    let msg = if !progress.speed.is_empty() {
                                        format!(
                                            "{} | {} | ETA: {}",
                                            progress.downloaded, progress.speed, progress.eta
                                        )
                                    } else {
                                        progress.downloaded.clone()
                                    };
                                    pb.set_message(msg);
                                    last_percentage = progress.percentage;

                                    // Update shared progress for TUI
                                    if let Some(ref shared) = progress_shared_clone {
                                        if let Ok(mut p) = shared.lock() {
                                            *p = Some(progress.clone());
                                        }
                                    }
                                }
                                log::trace!("{}", partial_line);
                                partial_line.clear();
                            }
                        } else {
                            partial_line.push(c);
                        }
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Attendre la fin du processus
    let status = child
        .wait()
        .map_err(|e| YtcsError::DownloadError(format!("Failed to wait: {}", e)))?;

    handle.join().ok();

    if !status.success() {
        // Get the stderr content for better error reporting
        let stderr_content = stderr_buffer.lock().unwrap_or_else(|e| e.into_inner());

        // Extract meaningful error message from stderr
        let error_msg = extract_error_message(&stderr_content);

        return Err(YtcsError::DownloadError(error_msg));
    }

    let mut final_path = output_path.to_path_buf();
    final_path.set_extension("mp3");
    Ok(final_path)
}

/// Extracts a meaningful error message from yt-dlp stderr.
fn extract_error_message(stderr: &str) -> String {
    // Look for specific error patterns
    for line in stderr.lines() {
        let line_lower = line.to_lowercase();

        // HTTP errors
        if line_lower.contains("http error") {
            if let Some(rest) = line.strip_prefix("ERROR: ") {
                return rest.to_string();
            }
            return line.trim().to_string();
        }

        // Generic ERROR: lines
        if line.starts_with("ERROR: ") {
            return line.trim().to_string();
        }
    }

    // If no specific error found, check for version warning
    if stderr.to_lowercase().contains("older than 90 days") {
        return "yt-dlp is outdated (older than 90 days). YouTube may be blocking downloads."
            .to_string();
    }

    // Fallback to a generic message with a hint
    if stderr.len() > 200 {
        // Truncate very long stderr
        format!(
            "yt-dlp failed. Last error: {}...",
            &stderr[stderr.len().saturating_sub(200)..].trim()
        )
    } else {
        format!("yt-dlp failed: {}", stderr.trim())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_percentage_line() {
        let line = "[download]  45.0% of  120.00MiB at  2.34MiB/s ETA 00:12";
        let progress = parse_download_line(line).unwrap();
        assert!((progress.percentage - 45.0).abs() < 0.1);
        assert_eq!(progress.eta, "00:12");
    }

    #[test]
    fn test_parse_size_line() {
        let line = "[download]  23.5MiB of  52.3MiB at  2.34MiB/s ETA 00:12";
        let progress = parse_download_line(line).unwrap();
        // 23.5 / 52.3 ≈ 44.9%
        assert!((progress.percentage - 44.9).abs() < 1.0);
        assert_eq!(progress.downloaded, "23.5");
        assert_eq!(progress.total, "52.3");
    }

    #[test]
    fn test_parse_gib_line() {
        let line = "[download]  1.2GiB of  2.4GiB at  5.6MiB/s ETA 03:45";
        let progress = parse_download_line(line).unwrap();
        assert!((progress.percentage - 50.0).abs() < 1.0);
        assert_eq!(progress.eta, "03:45");
    }

    #[test]
    fn test_ignore_non_download_lines() {
        assert!(parse_download_line("[info] Downloading video").is_none());
        assert!(parse_download_line("[debug] Some debug info").is_none());
    }

    #[test]
    fn test_extract_error_message() {
        let stderr = "WARNING: Some warning\nERROR: unable to download video data: HTTP Error 403: Forbidden\n";
        let msg = extract_error_message(stderr);
        assert!(msg.contains("403"));
        assert!(msg.contains("Forbidden"));
    }

    #[test]
    fn test_extract_error_message_outdated() {
        // No ERROR line, so version warning should be detected
        let stderr = "WARNING: Your yt-dlp version is older than 90 days\nSome other output\n";
        let msg = extract_error_message(stderr);
        assert!(msg.contains("outdated"));
    }
}
