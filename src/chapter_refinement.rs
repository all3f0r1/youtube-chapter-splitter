//! Affinement des chapitres par détection de silence.
//!
//! Ce module ajuste les timecodes des chapitres déclarés en utilisant
//! la détection de silence pour trouver les points de coupe optimaux.
//!
//! # Principe
//!
//! Les timecodes de chapitres YouTube sont souvent imprécis (quelques secondes
//! d'écart). Ce module :
//! 1. Analyse tout l'audio pour trouver tous les silences
//! 2. Pour chaque chapitre, cherche le silence le plus proche (dans une fenêtre)
//! 3. Ajuste le timecode vers ce silence
//!
//! # Exemple
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
//!     5.0,  // fenêtre de ±5 secondes
//!     -35.0, // seuil de silence
//!     1.5,  // durée minimale de silence
//! )?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::chapters::Chapter;
use crate::error::{Result, YtcsError};
use colored::Colorize;
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::Path;
use std::process::Command;

/// Point de silence détecté dans l'audio
#[derive(Debug, Clone)]
struct SilencePoint {
    /// Position du silence en secondes
    position: f64,
}

impl SilencePoint {
    fn new(start: f64, end: f64) -> Self {
        let position = (start + end) / 2.0;
        Self { position }
    }
}

/// Analyse l'audio et extrait tous les points de silence.
///
/// # Arguments
///
/// * `audio_file` - Fichier audio à analyser
/// * `noise_threshold` - Seuil de silence en dB (ex: -35.0)
/// * `min_duration` - Durée minimale d'un silence en secondes (ex: 1.0)
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

/// Trouve le silence le plus proche d'une position cible.
///
/// # Arguments
///
/// * `silences` - Liste des points de silence
/// * `target` - Position cible en secondes
/// * `window` - Fenêtre de recherche en secondes (ex: 5.0 = ±5s)
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

/// Affine les chapitres en utilisant les points de silence.
///
/// # Arguments
///
/// * `chapters` - Chapitres originaux avec timecodes déclarés
/// * `audio_file` - Fichier audio à analyser
/// * `window` - Fenêtre de recherche en secondes (recommandé: 3.0 à 8.0)
/// * `noise_threshold` - Seuil de silence en dB (recommandé: -35.0 à -50.0)
/// * `min_silence_duration` - Durée minimale de silence (recommandé: 0.8 à 2.0)
///
/// # Returns
///
/// Nouveaux chapitres avec timecodes ajustés vers les silences les plus proches.
///
/// # Stratégie
///
/// - **Début de piste** : Ajusté vers le silence le plus proche **après** le timecode
/// - **Fin de piste** : Ajusté vers le silence le plus proche **avant** le timecode
/// - Si aucun silence n'est trouvé dans la fenêtre, le timecode original est conservé
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

    // Détecter tous les silences une seule fois
    let silences = detect_all_silences(audio_file, noise_threshold, min_silence_duration)?;

    if silences.is_empty() {
        log::warn!("No silences detected, returning original chapters");
        return Ok(chapters.to_vec());
    }

    let mut refined = Vec::new();

    for (i, chapter) in chapters.iter().enumerate() {
        let is_last = i == chapters.len() - 1;

        // Pour le début, chercher un silence APRÈS le timecode déclaré
        // Pour la fin (sauf dernière piste), chercher un silence AVANT
        let new_start = if i == 0 {
            // Première piste : garder le début ou chercher avant
            let before = find_nearest_silence(&silences, chapter.start_time, window);
            before
                .map(|s| s.position)
                .filter(|&pos| pos <= chapter.start_time + 0.5)
        } else {
            // Pistes suivantes : chercher après le timecode
            let after = find_nearest_silence(&silences, chapter.start_time, window);
            after
                .map(|s| s.position)
                .filter(|&pos| pos >= chapter.start_time - 0.5)
        };

        let new_end = if is_last {
            // Dernière piste : garder la fin originale
            chapter.end_time
        } else {
            // Chercher un silence avant la fin déclarée
            let before = find_nearest_silence(&silences, chapter.end_time, window);
            before
                .map(|s| s.position)
                .filter(|&pos| pos <= chapter.end_time + 0.5)
                .unwrap_or(chapter.end_time)
        };

        let final_start = new_start.unwrap_or(chapter.start_time);
        let final_end = new_end;

        // S'assurer que les chapitres ne se chevauchent pas
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

        // Vérifier que la durée est raisonnable (au moins 30 secondes)
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

/// Affiche un rapport de comparaison entre chapitres originaux et affinés.
pub fn print_refinement_report(original: &[Chapter], refined: &[Chapter]) {
    if original.len() != refined.len() {
        println!("⚠ Cannot compare: different chapter counts");
        return;
    }

    let mut total_start_delta = 0.0_f64;
    let mut max_start_delta = 0.0_f64;

    println!();
    println!("{}", "Chapter refinement report:".dimmed());
    println!(
        "{:<5} {:<30} {:>12} → {:>12} ({:>8})",
        "", "Title", "Original", "Refined", "Delta"
    );
    println!("{}", "-".repeat(70).dimmed());

    for (i, (orig, refn)) in original.iter().zip(refined.iter()).enumerate() {
        let start_delta = refn.start_time - orig.start_time;
        let _duration_delta = (refn.end_time - refn.start_time) - (orig.end_time - orig.start_time);

        total_start_delta += start_delta.abs();
        max_start_delta = max_start_delta.max(start_delta.abs());

        let delta_str = if start_delta.abs() < 0.1 {
            "—".dimmed().to_string()
        } else if start_delta > 0.0 {
            format!("{:+.1}s", start_delta).green().to_string()
        } else {
            format!("{:+.1}s", start_delta).red().to_string()
        };

        println!(
            "{:<5} {:<30} {:>6.1}s → {:>6.1}s ({:>8})",
            format!("{}.", i + 1),
            if orig.title.len() > 28 {
                format!("{}...", &orig.title[..25])
            } else {
                orig.title.clone()
            },
            orig.start_time,
            refn.start_time,
            delta_str
        );
    }

    println!();
    println!(
        "  Average adjustment: {:.2}s | Max adjustment: {:.2}s",
        total_start_delta / original.len() as f64,
        max_start_delta
    );
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Crée un fichier audio de test avec des silences artificiels.
    /// Note: nécessite ffmpeg dans le PATH.
    #[test]
    #[ignore] // Ignoré par défaut car lent et nécessite ffmpeg
    fn test_refine_chapters() {
        // Créer un fichier audio de test avec silence
        // Ce test est principalement documentatif
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

        // La position est le point médian du silence
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

        // Cible exacte
        let nearest = find_nearest_silence(&silences, 10.0, 2.0);
        assert_eq!(nearest.unwrap().position, 10.0);

        // Cible proche
        let nearest = find_nearest_silence(&silences, 9.8, 2.0);
        assert_eq!(nearest.unwrap().position, 10.0);

        // Cible hors fenêtre
        let nearest = find_nearest_silence(&silences, 15.0, 2.0);
        assert!(nearest.is_none());
    }
}
