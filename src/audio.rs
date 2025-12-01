//! Traitement audio et découpage par chapitres.
//!
//! Ce module gère le découpage des fichiers audio en pistes individuelles
//! et l'ajout de métadonnées ID3 avec pochettes d'album.

use crate::chapters::Chapter;
use crate::error::{Result, YtcsError};

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
static RE_SILENCE_START: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"silence_start: ([\d.]+)").unwrap());

static RE_SILENCE_END: Lazy<Regex> = Lazy::new(|| Regex::new(r"silence_end: ([\d.]+)").unwrap());

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
    cfg: &crate::config::Config,
) -> Result<Vec<PathBuf>> {
    std::fs::create_dir_all(output_dir)?;

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
        let filename_base = cfg.format_filename(track_number, &sanitized_title, artist, album);
        // Title Case: première lettre de chaque mot en majuscule
        let title_cased = filename_base
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<String>>()
            .join(" ");
        let output_filename = format!("{}.mp3", title_cased);
        let output_path = output_dir.join(&output_filename);

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

        let output = cmd
            .output()
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
    }

    Ok(output_files)
}

/// Découpe une seule piste audio
pub fn split_single_track(
    input_file: &Path,
    chapter: &Chapter,
    track_number: usize,
    total_tracks: usize,
    output_dir: &Path,
    artist: &str,
    album: &str,
    cover_data: Option<&[u8]>,
    cfg: &crate::config::Config,
) -> Result<PathBuf> {
    let sanitized_title = chapter.sanitize_title();
    let filename_base = cfg.format_filename(track_number, &sanitized_title, artist, album);
    // Title Case: première lettre de chaque mot en majuscule
    let title_cased = filename_base
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ");
    let output_filename = format!("{}.mp3", title_cased);
    let output_path = output_dir.join(&output_filename);

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
        .arg(format!("track={}/{}", track_number, total_tracks))
        .arg("-y")
        .arg(&output_path);

    let output = cmd
        .output()
        .map_err(|e| YtcsError::AudioError(format!("Failed to execute ffmpeg: {}", e)))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(YtcsError::AudioError(format!("ffmpeg failed: {}", error)));
    }

    // Ajouter la pochette d'album avec lofty si elle existe
    if let Some(cover) = cover_data {
        add_cover_to_file(&output_path, cover)?;
    }

    Ok(output_path)
}

/// Vérifie si les données sont au format WebP.
///
/// # Arguments
///
/// * `data` - Les données à vérifier
///
/// # Returns
///
/// `true` si c'est un WebP, `false` sinon
fn is_webp(data: &[u8]) -> bool {
    data.len() >= 12
        && data[0] == 0x52 && data[1] == 0x49 && data[2] == 0x46 && data[3] == 0x46  // "RIFF"
        && data[8] == 0x57 && data[9] == 0x45 && data[10] == 0x42 && data[11] == 0x50
    // "WEBP"
}

/// Convertit une image WebP en JPEG en utilisant ffmpeg.
///
/// # Arguments
///
/// * `webp_path` - Chemin vers le fichier WebP
///
/// # Returns
///
/// Les données de l'image JPEG
fn convert_webp_to_jpeg(webp_path: &Path) -> Result<Vec<u8>> {
    // Créer un fichier temporaire pour le JPEG
    let temp_dir = std::env::temp_dir();
    let temp_jpeg = temp_dir.join(format!("cover_{}.jpg", std::process::id()));

    // Convertir avec ffmpeg
    let output = Command::new("ffmpeg")
        .arg("-i")
        .arg(webp_path)
        .arg("-y") // Overwrite output file
        .arg("-q:v")
        .arg("2") // Haute qualité JPEG (1-31, 2 = très haute qualité)
        .arg(&temp_jpeg)
        .output()
        .map_err(|e| {
            YtcsError::AudioError(format!("Failed to run ffmpeg for WebP conversion: {}", e))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(YtcsError::AudioError(format!(
            "ffmpeg failed to convert WebP to JPEG: {}",
            stderr
        )));
    }

    // Lire le fichier JPEG converti
    let mut jpeg_file = File::open(&temp_jpeg)
        .map_err(|e| YtcsError::AudioError(format!("Failed to open converted JPEG: {}", e)))?;

    let mut jpeg_data = Vec::new();
    jpeg_file
        .read_to_end(&mut jpeg_data)
        .map_err(|e| YtcsError::AudioError(format!("Failed to read converted JPEG: {}", e)))?;

    // Nettoyer le fichier temporaire
    let _ = std::fs::remove_file(&temp_jpeg);

    Ok(jpeg_data)
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
pub fn load_cover_image(cover_path: &Path) -> Result<Option<Vec<u8>>> {
    // Vérifier si le fichier existe
    if !cover_path.exists() {
        return Ok(None);
    }

    let mut file = File::open(cover_path)
        .map_err(|e| YtcsError::AudioError(format!("Failed to open cover image: {}", e)))?;

    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .map_err(|e| YtcsError::AudioError(format!("Failed to read cover image: {}", e)))?;

    // Détecter si c'est un WebP et le convertir en JPEG si nécessaire
    if is_webp(&data) {
        eprintln!("Warning: Cover image is WebP format. Converting to JPEG...");
        data = convert_webp_to_jpeg(cover_path)?;
    }

    Ok(Some(data))
}

/// Détecte le type MIME d'une image basé sur ses magic bytes.
///
/// # Arguments
///
/// * `data` - Les données de l'image
///
/// # Returns
///
/// Le type MIME détecté, ou None si non reconnu
fn detect_image_mime_type(data: &[u8]) -> Option<lofty::picture::MimeType> {
    use lofty::picture::MimeType;

    if data.len() < 12 {
        return None;
    }

    // JPEG: FF D8 FF
    if data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
        return Some(MimeType::Jpeg);
    }

    // PNG: 89 50 4E 47 0D 0A 1A 0A
    if data[0] == 0x89 && data[1] == 0x50 && data[2] == 0x4E && data[3] == 0x47 {
        return Some(MimeType::Png);
    }

    // GIF: 47 49 46 38 (GIF8)
    if data[0] == 0x47 && data[1] == 0x49 && data[2] == 0x46 && data[3] == 0x38 {
        return Some(MimeType::Gif);
    }

    // BMP: 42 4D (BM)
    if data[0] == 0x42 && data[1] == 0x4D {
        return Some(MimeType::Bmp);
    }

    // WEBP: 52 49 46 46 ... 57 45 42 50 (RIFF...WEBP)
    // Note: WEBP n'est pas dans l'enum MimeType de lofty, donc on ne le supporte pas pour l'instant
    // if data.len() >= 12 && data[0] == 0x52 && data[1] == 0x49 && data[2] == 0x46 && data[3] == 0x46
    //     && data[8] == 0x57 && data[9] == 0x45 && data[10] == 0x42 && data[11] == 0x50 {
    //     return Some(MimeType::from("image/webp"));
    // }

    None
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
    let mut cover_reader = cover_data;
    let mut picture = match Picture::from_reader(&mut cover_reader) {
        Ok(pic) => pic,
        Err(e) => {
            // Si la lecture automatique échoue, essayer de créer manuellement avec le MIME type
            eprintln!(
                "Warning: Failed to auto-detect image format: {}. Trying manual creation...",
                e
            );

            // Détecter le MIME type basé sur les magic bytes
            let mime_type = detect_image_mime_type(cover_data)
                .ok_or_else(|| YtcsError::AudioError(
                    "Failed to detect image format. The cover image may be corrupted or in an unsupported format.".to_string()
                ))?;

            // Créer la picture manuellement
            Picture::new_unchecked(
                PictureType::CoverFront,
                Some(mime_type),
                None,
                cover_data.to_vec(),
            )
        }
    };

    // Définir le type et la description (seulement si pas déjà défini)
    if picture.pic_type() != PictureType::CoverFront {
        picture.set_pic_type(PictureType::CoverFront);
    }
    if picture.description().is_none() {
        picture.set_description(Some("Album Cover".to_string()));
    }

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
    tagged_file
        .save_to_path(audio_path, WriteOptions::default())
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
