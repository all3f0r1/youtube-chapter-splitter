use youtube_chapter_splitter::{audio, downloader, utils};
use std::path::PathBuf;

fn main() -> youtube_chapter_splitter::Result<()> {
    // Vérifier les dépendances
    downloader::check_dependencies()?;

    // URL de la vidéo YouTube
    let url = "https://www.youtube.com/watch?v=28vf7QxgCzA";

    // Obtenir les informations
    println!("Récupération des informations...");
    let video_info = downloader::get_video_info(url)?;
    println!("Titre: {}", video_info.title);
    println!("Chapitres: {}", video_info.chapters.len());

    // Parser artiste et album
    let (artist, album) = utils::parse_artist_album(&video_info.title);
    println!("Artiste: {}", artist);
    println!("Album: {}", album);

    // Télécharger l'audio
    let output_path = PathBuf::from("./temp_audio");
    println!("Téléchargement...");
    let audio_file = downloader::download_audio(url, &output_path)?;

    // Télécharger la miniature
    let output_dir = PathBuf::from("./output");
    std::fs::create_dir_all(&output_dir)?;
    
    let cover_path = match downloader::download_thumbnail(url, &output_dir) {
        Ok(path) => {
            println!("Miniature téléchargée: {:?}", path);
            Some(path)
        }
        Err(e) => {
            println!("Impossible de télécharger la miniature: {}", e);
            None
        }
    };

    // Si des chapitres existent, les utiliser
    let chapters = if !video_info.chapters.is_empty() {
        video_info.chapters
    } else {
        // Sinon, détecter les silences
        println!("Détection des silences...");
        audio::detect_silence_chapters(&audio_file, -30.0, 2.0)?
    };

    // Découper l'audio
    let cfg = youtube_chapter_splitter::config::Config::load()?;
    audio::split_audio_by_chapters(
        &audio_file,
        &chapters,
        &output_dir,
        &artist,
        &album,
        cover_path.as_deref(),
        &cfg,
    )?;

    // Nettoyer
    std::fs::remove_file(&audio_file).ok();

    println!("Terminé!");
    Ok(())
}
