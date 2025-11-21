/// Tests pour les fonctions du module audio
#[cfg(test)]
mod audio_function_tests {
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;
    use youtube_chapter_splitter::audio::get_audio_duration;
    use youtube_chapter_splitter::chapters::Chapter;

    /// Crée un fichier audio de test
    fn create_test_audio(
        path: &PathBuf,
        duration_secs: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("ffmpeg")
            .args([
                "-f",
                "lavfi",
                "-i",
                "anullsrc=r=44100:cl=mono",
                "-t",
                &duration_secs.to_string(),
                "-c:a",
                "libmp3lame",
                "-q:a",
                "2",
                "-y",
                path.to_str().unwrap(),
            ])
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to create test audio: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        Ok(())
    }

    #[test]
    #[ignore] // Ignorer par défaut car nécessite ffmpeg
    fn test_get_audio_duration() {
        let test_dir = PathBuf::from("/tmp/ytcs_audio_test");
        fs::create_dir_all(&test_dir).unwrap();

        let test_file = test_dir.join("test_5s.mp3");
        create_test_audio(&test_file, 5).unwrap();

        let duration = get_audio_duration(&test_file).unwrap();

        // La durée devrait être proche de 5 secondes (avec une marge d'erreur)
        assert!((4.9..=5.1).contains(&duration), "Duration was {}", duration);

        fs::remove_dir_all(&test_dir).ok();
    }

    #[test]
    fn test_chapter_duration_calculation() {
        let chapter = Chapter::new("Test".to_string(), 10.0, 70.0);
        assert_eq!(chapter.duration(), 60.0);
    }

    #[test]
    #[should_panic(expected = "end_time")]
    fn test_chapter_zero_duration() {
        // Ce test vérifie que Chapter::new panique si end_time == start_time
        let _chapter = Chapter::new("Test".to_string(), 10.0, 10.0);
    }

    #[test]
    fn test_chapter_sanitize_title_removes_prefix() {
        let chapter = Chapter::new("1 - Song Name".to_string(), 0.0, 100.0);
        let sanitized = chapter.sanitize_title();
        assert_eq!(sanitized, "Song Name");
    }

    #[test]
    fn test_chapter_sanitize_title_replaces_invalid_chars() {
        let chapter = Chapter::new("Song/Name:Test".to_string(), 0.0, 100.0);
        let sanitized = chapter.sanitize_title();
        assert_eq!(sanitized, "Song_Name_Test");
    }

    #[test]
    fn test_chapter_sanitize_title_track_prefix() {
        let chapter = Chapter::new("Track 5: Another Song".to_string(), 0.0, 100.0);
        let sanitized = chapter.sanitize_title();
        assert_eq!(sanitized, "Another Song");
    }

    #[test]
    fn test_multiple_chapters_no_overlap() {
        let chapters = [
            Chapter::new("Track 1".to_string(), 0.0, 100.0),
            Chapter::new("Track 2".to_string(), 100.0, 200.0),
            Chapter::new("Track 3".to_string(), 200.0, 300.0),
        ];

        for i in 0..chapters.len() - 1 {
            assert_eq!(chapters[i].end_time, chapters[i + 1].start_time);
        }
    }
}
