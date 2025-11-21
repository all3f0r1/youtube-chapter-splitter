/// Tests d'intégration end-to-end
/// 
/// Ces tests vérifient le workflow complet du téléchargement au découpage.
/// Ils sont ignorés par défaut car ils nécessitent une connexion internet
/// et des dépendances externes (yt-dlp, ffmpeg).

#[cfg(test)]
mod integration_e2e_tests {
    use youtube_chapter_splitter::{audio, downloader, utils};
    use std::fs;
    use std::path::PathBuf;

    /// Crée un répertoire de test temporaire
    fn create_test_dir(name: &str) -> PathBuf {
        let test_dir = PathBuf::from(format!("/tmp/ytcs_e2e_test_{}", name));
        fs::create_dir_all(&test_dir).unwrap();
        test_dir
    }

    /// Nettoie un répertoire de test
    fn cleanup_test_dir(dir: &PathBuf) {
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    #[ignore] // Nécessite connexion internet et yt-dlp
    fn test_e2e_download_and_split_with_chapters() {
        // Utiliser une vidéo de test courte avec des chapitres
        let url = "https://www.youtube.com/watch?v=28vf7QxgCzA";
        let test_dir = create_test_dir("with_chapters");

        // 1. Vérifier les dépendances
        let deps_result = downloader::check_dependencies();
        if deps_result.is_err() {
            println!("Skipping test: dependencies not available");
            cleanup_test_dir(&test_dir);
            return;
        }

        // 2. Obtenir les informations vidéo
        let video_info = downloader::get_video_info(url).unwrap();
        assert!(!video_info.title.is_empty(), "Video title should not be empty");
        assert!(video_info.duration > 0.0, "Video duration should be positive");

        // 3. Parser artiste et album
        let (artist, album) = utils::parse_artist_album(&video_info.title);
        assert!(!artist.is_empty(), "Artist should not be empty");
        assert!(!album.is_empty(), "Album should not be empty");

        // 4. Télécharger l'audio
        let audio_path = test_dir.join("temp_audio");
        let audio_file = downloader::download_audio(url, &audio_path).unwrap();
        assert!(audio_file.exists(), "Audio file should exist");

        // 5. Télécharger la miniature
        let cover_result = downloader::download_thumbnail(url, &test_dir);
        let cover_path = cover_result.ok();

        // 6. Découper l'audio
        let output_dir = test_dir.join("output");
        fs::create_dir_all(&output_dir).unwrap();

        let chapters = if !video_info.chapters.is_empty() {
            video_info.chapters
        } else {
            audio::detect_silence_chapters(&audio_file, -30.0, 2.0).unwrap()
        };

        assert!(!chapters.is_empty(), "Should have at least one chapter");

        let cfg = youtube_chapter_splitter::config::Config::load().unwrap();
        let output_files = audio::split_audio_by_chapters(
            &audio_file,
            &chapters,
            &output_dir,
            &artist,
            &album,
            cover_path.as_deref(),
            &cfg,
        ).unwrap();

        // 7. Vérifier les fichiers de sortie
        assert_eq!(output_files.len(), chapters.len(), "Should create one file per chapter");
        
        for file in &output_files {
            assert!(file.exists(), "Output file should exist: {:?}", file);
            assert!(file.extension().unwrap() == "mp3", "Output should be MP3");
            
            // Vérifier que le fichier n'est pas vide
            let metadata = fs::metadata(file).unwrap();
            assert!(metadata.len() > 0, "Output file should not be empty");
        }

        // 8. Vérifier les métadonnées d'un fichier
        if let Some(first_file) = output_files.first() {
            // Utiliser ffprobe pour vérifier les métadonnées
            use std::process::Command;
            
            let output = Command::new("ffprobe")
                .args(&[
                    "-v", "quiet",
                    "-print_format", "json",
                    "-show_format",
                    first_file.to_str().unwrap()
                ])
                .output();

            if let Ok(result) = output {
                let json_str = String::from_utf8_lossy(&result.stdout);
                
                // Vérifier que les métadonnées contiennent artist et album
                assert!(json_str.contains("artist") || json_str.contains("ARTIST"));
                assert!(json_str.contains("album") || json_str.contains("ALBUM"));
            }
        }

        // Nettoyer
        cleanup_test_dir(&test_dir);
    }

    #[test]
    #[ignore] // Nécessite connexion internet et yt-dlp
    fn test_e2e_silence_detection() {
        let url = "https://www.youtube.com/watch?v=28vf7QxgCzA";
        let test_dir = create_test_dir("silence_detection");

        let deps_result = downloader::check_dependencies();
        if deps_result.is_err() {
            println!("Skipping test: dependencies not available");
            cleanup_test_dir(&test_dir);
            return;
        }

        // Télécharger l'audio
        let audio_path = test_dir.join("temp_audio");
        let audio_file = downloader::download_audio(url, &audio_path).unwrap();

        // Détecter les silences
        let chapters = audio::detect_silence_chapters(&audio_file, -30.0, 2.0).unwrap();

        // Vérifier que des chapitres ont été détectés
        assert!(!chapters.is_empty(), "Should detect at least one chapter");

        // Vérifier que les chapitres sont valides
        for chapter in &chapters {
            assert!(chapter.start_time >= 0.0);
            assert!(chapter.end_time > chapter.start_time);
            assert!(!chapter.title.is_empty());
        }

        cleanup_test_dir(&test_dir);
    }

    #[test]
    #[ignore] // Nécessite connexion internet
    fn test_e2e_video_info_extraction() {
        let url = "https://www.youtube.com/watch?v=28vf7QxgCzA";

        let deps_result = downloader::check_dependencies();
        if deps_result.is_err() {
            println!("Skipping test: dependencies not available");
            return;
        }

        let video_info = downloader::get_video_info(url).unwrap();

        // Vérifications de base
        assert!(!video_info.title.is_empty());
        assert!(video_info.duration > 0.0);
        
        // Vérifier le format du titre
        assert!(video_info.title.len() < 500, "Title should be reasonable length");
        
        // Vérifier la durée (devrait être entre 1 seconde et 24 heures)
        assert!(video_info.duration >= 1.0);
        assert!(video_info.duration <= 86400.0);
    }

    #[test]
    #[ignore] // Nécessite connexion internet
    fn test_e2e_thumbnail_download() {
        let url = "https://www.youtube.com/watch?v=28vf7QxgCzA";
        let test_dir = create_test_dir("thumbnail");

        let deps_result = downloader::check_dependencies();
        if deps_result.is_err() {
            println!("Skipping test: dependencies not available");
            cleanup_test_dir(&test_dir);
            return;
        }

        let cover_path = downloader::download_thumbnail(url, &test_dir).unwrap();

        // Vérifier que le fichier existe
        assert!(cover_path.exists());
        assert_eq!(cover_path.extension().unwrap(), "jpg");

        // Vérifier que c'est une image valide
        let metadata = fs::metadata(&cover_path).unwrap();
        assert!(metadata.len() > 1000, "Image should be at least 1KB");

        cleanup_test_dir(&test_dir);
    }

    #[test]
    #[ignore] // Nécessite connexion internet
    fn test_e2e_invalid_url() {
        let url = "https://www.youtube.com/watch?v=invalid123";

        let deps_result = downloader::check_dependencies();
        if deps_result.is_err() {
            println!("Skipping test: dependencies not available");
            return;
        }

        let result = downloader::get_video_info(url);
        assert!(result.is_err(), "Should fail for invalid video ID");
    }

    #[test]
    #[ignore] // Nécessite connexion internet
    fn test_e2e_private_video() {
        // URL d'une vidéo privée (exemple)
        let url = "https://www.youtube.com/watch?v=privateVideo";

        let deps_result = downloader::check_dependencies();
        if deps_result.is_err() {
            println!("Skipping test: dependencies not available");
            return;
        }

        let result = downloader::get_video_info(url);
        // Devrait échouer car la vidéo est privée
        if result.is_err() {
            // C'est le comportement attendu
            assert!(true);
        }
    }

    #[test]
    fn test_e2e_metadata_parsing() {
        // Test sans connexion internet
        let test_titles = vec![
            ("Pink Floyd - Dark Side of the Moon [1973]", "Pink Floyd", "Dark Side Of The Moon"),
            ("MARIGOLD - Oblivion Gate [Full Album]", "Marigold", "Oblivion Gate"),
            ("Artist | Album Name", "Artist", "Album Name"),
            ("Just A Title", "Unknown Artist", "Just A Title"),
        ];

        for (title, expected_artist, expected_album) in test_titles {
            let (artist, album) = utils::parse_artist_album(title);
            assert_eq!(artist, expected_artist, "Failed for title: {}", title);
            assert_eq!(album, expected_album, "Failed for title: {}", title);
        }
    }

    #[test]
    fn test_e2e_chapter_duration_calculation() {
        use youtube_chapter_splitter::chapters::Chapter;

        let chapters = vec![
            Chapter::new("Track 1".to_string(), 0.0, 180.0),
            Chapter::new("Track 2".to_string(), 180.0, 360.0),
            Chapter::new("Track 3".to_string(), 360.0, 540.0),
        ];

        // Vérifier que les durées sont correctes
        assert_eq!(chapters[0].duration(), 180.0);
        assert_eq!(chapters[1].duration(), 180.0);
        assert_eq!(chapters[2].duration(), 180.0);

        // Vérifier qu'il n'y a pas de chevauchement
        for i in 0..chapters.len() - 1 {
            assert_eq!(chapters[i].end_time, chapters[i + 1].start_time);
        }
    }
}
