use youtube_chapter_splitter::{audio, downloader};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> youtube_chapter_splitter::Result<()> {
    // Vérifier les dépendances
    downloader::check_dependencies()?;

    // URL de la vidéo YouTube
    let url = "https://www.youtube.com/watch?v=28vf7QxgCzA";

    // Obtenir les informations
    println!("Récupération des informations...");
    let video_info = downloader::get_video_info(url)?;
    println!("Titre: {}", video_info.title);
    println!("Chapitres: {}", video_info.chapters.len());

    // Télécharger l'audio
    let output_path = PathBuf::from("./temp_audio");
    println!("Téléchargement...");
    let audio_file = downloader::download_audio(url, &output_path)?;

    // Si des chapitres existent, les utiliser
    if !video_info.chapters.is_empty() {
        let output_dir = PathBuf::from("./output");
        audio::split_audio_by_chapters(
            &audio_file,
            &video_info.chapters,
            &output_dir,
            &video_info.title,
        )?;
    } else {
        // Sinon, détecter les silences
        println!("Détection des silences...");
        let chapters = audio::detect_silence_chapters(&audio_file, -30.0, 2.0)?;
        
        let output_dir = PathBuf::from("./output");
        audio::split_audio_by_chapters(
            &audio_file,
            &chapters,
            &output_dir,
            &video_info.title,
        )?;
    }

    // Nettoyer
    std::fs::remove_file(&audio_file).ok();

    println!("Terminé!");
    Ok(())
}
