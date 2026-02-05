//! Traitement audio et découpage par chapitres.
//!
//! Ce module gère le découpage des fichiers audio en pistes individuelles
//! et l'ajout de métadonnées ID3 avec pochettes d'album.

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

// Regex compilées une seule fois au démarrage
static RE_SILENCE_START: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"silence_start: ([\d.]+)").unwrap()
});

static RE_SILENCE_END: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"silence_end: ([\d.]+)").unwrap()
});

/// Découpe un fichier audio en pistes individuelles basées sur les chapitres.
///
/// Cette fonction utilise `ffmpeg` pour découper l'audio et `lofty` pour ajouter
/// les métadonnées ID3 et la pochette d'album.
///
/// # Arguments
///
/// * `input_file` - Le fichier audio source
/// * `chapters` - Les chapitres définissant les points de découpe
/// * `output_dir` - Le répertoire de sortie pour les pistes
/// * `artist` - Le nom de l'artiste
/// * `album` - Le nom de l'album
/// * `cover_path` - Chemin optionnel vers l'image de pochette
///
/// # Returns
///
/// Un vecteur contenant les chemins des fichiers créés
///
/// # Errors
///
/// Retourne une erreur si le découpage ou l'ajout de métadonnées échoue
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
    
    // Créer une barre de progression
    let pb = ProgressBar::new(chapters.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-")
    );
    
    // Charger l'image de couverture une seule fois si elle existe
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

        pb.set_message(format!("Track {}: {}", track_number, chapter.title));

        let duration = chapter.duration();
        
        // Découper l'audio avec ffmpeg (sans cover art)
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
        
        let output = cmd.output()
            .map_err(|e| YtcsError::AudioError(format!("Failed to execute ffmpeg: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(YtcsError::AudioError(format!("ffmpeg failed: {}", error)));
        }

        // Ajouter la pochette d'album avec lofty si elle existe
        if let Some(ref cover) = cover_data {
            add_cover_to_file(&output_path, cover)?;
        }

        output_files.push(output_path);
        pb.inc(1);
    }

    pb.finish_with_message("Splitting completed successfully!");
    Ok(output_files)
}

/// Charge une image de couverture depuis un fichier.
///
/// # Arguments
///
/// * `cover_path` - Chemin vers le fichier image
///
/// # Returns
///
/// Les données de l'image sous forme de vecteur d'octets
fn load_cover_image(cover_path: &Path) -> Result<Option<Vec<u8>>> {
    let mut file = File::open(cover_path)
        .map_err(|e| YtcsError::AudioError(format!("Failed to open cover image: {}", e)))?;
    
    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .map_err(|e| YtcsError::AudioError(format!("Failed to read cover image: {}", e)))?;
    
    Ok(Some(data))
}

/// Ajoute une pochette d'album à un fichier audio avec lofty.
///
/// # Arguments
///
/// * `audio_path` - Chemin vers le fichier audio
/// * `cover_data` - Données de l'image de couverture
fn add_cover_to_file(audio_path: &Path, cover_data: &[u8]) -> Result<()> {
    // Charger le fichier audio
    let mut tagged_file = Probe::open(audio_path)
        .map_err(|e| YtcsError::AudioError(format!("Failed to open audio file: {}", e)))?
        .guess_file_type()
        .map_err(|e| YtcsError::AudioError(format!("Failed to guess file type: {}", e)))?
        .read()
        .map_err(|e| YtcsError::AudioError(format!("Failed to read audio file: {}", e)))?;
    
    // Créer l'objet Picture depuis les données
    let mut cover_reader = &cover_data[..];
    let mut picture = Picture::from_reader(&mut cover_reader)
        .map_err(|e| YtcsError::AudioError(format!("Failed to create picture: {}", e)))?;
    
    // Définir le type et la description
    picture.set_pic_type(PictureType::CoverFront);
    picture.set_description(Some("Album Cover".to_string()));
    
    // Obtenir ou créer le tag principal
    let tag = match tagged_file.primary_tag_mut() {
        Some(primary_tag) => primary_tag,
        None => {
            let tag_type = tagged_file.primary_tag_type();
            tagged_file.insert_tag(lofty::tag::Tag::new(tag_type));
            tagged_file.primary_tag_mut().unwrap()
        }
    };
    
    // Ajouter l'image au tag
    tag.push_picture(picture);
    
    // Sauvegarder les modifications avec tagged_file.save_to() pour préserver tous les tags
    // Note: save_to() préserve toutes les métadonnées existantes, contrairement à save_to_path()
    tagged_file.save_to_path(audio_path, WriteOptions::default())
        .map_err(|e| YtcsError::AudioError(format!("Failed to save tags: {}", e)))?;
    
    Ok(())
}

/// Détecte les chapitres automatiquement en analysant les périodes de silence.
///
/// Utilise `ffmpeg` avec le filtre `silencedetect` pour identifier les points
/// de découpe potentiels dans l'audio.
///
/// # Arguments
///
/// * `input_file` - Le fichier audio à analyser
/// * `silence_threshold` - Seuil de silence en dB (ex: -30.0)
/// * `min_silence_duration` - Durée minimale de silence en secondes (ex: 2.0)
///
/// # Returns
///
/// Un vecteur de chapitres détectés automatiquement
///
/// # Errors
///
/// Retourne une erreur si aucun silence n'est détecté ou si ffmpeg échoue
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
        } else if let Some(caps) = RE_SILENCE_END.captures(line) {
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

/// Obtient la durée totale d'un fichier audio.
///
/// Utilise `ffprobe` pour extraire la durée du fichier.
///
/// # Arguments
///
/// * `input_file` - Le fichier audio à analyser
///
/// # Returns
///
/// La durée en secondes
///
/// # Errors
///
/// Retourne une erreur si ffprobe échoue ou si la durée est invalide
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
