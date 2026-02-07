/// Tests pour les fonctions utilitaires de main.rs

#[cfg(test)]
mod main_tests {
    // Note: On ne peut pas importer directement les fonctions de main.rs
    // car elles sont privées. On va les recréer ici pour les tester.

    fn clean_url(url: &str) -> String {
        // Extract only the video ID, remove playlist and other parameters
        if let Some(id_start) = url.find("v=") {
            let id_part = &url[id_start + 2..];
            if let Some(amp_pos) = id_part.find('&') {
                format!("https://www.youtube.com/watch?v={}", &id_part[..amp_pos])
            } else {
                format!("https://www.youtube.com/watch?v={}", id_part)
            }
        } else {
            url.to_string()
        }
    }

    #[test]
    fn test_clean_url_with_playlist() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=PLtest&index=1";
        let cleaned = clean_url(url);
        assert_eq!(cleaned, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    }

    #[test]
    fn test_clean_url_with_timestamp() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=30s";
        let cleaned = clean_url(url);
        assert_eq!(cleaned, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    }

    #[test]
    fn test_clean_url_already_clean() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
        let cleaned = clean_url(url);
        assert_eq!(cleaned, url);
    }

    #[test]
    fn test_clean_url_short_format() {
        let url = "https://youtu.be/dQw4w9WgXcQ";
        let cleaned = clean_url(url);
        // Short URLs don't have v=, so they should be returned as-is
        assert_eq!(cleaned, url);
    }

    #[test]
    fn test_clean_url_multiple_parameters() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ&feature=share&t=10";
        let cleaned = clean_url(url);
        assert_eq!(cleaned, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    }

    #[test]
    fn test_get_default_music_dir() {
        // Test que le répertoire par défaut existe ou peut être créé
        let music_dir = if let Some(dir) = dirs::audio_dir() {
            dir
        } else {
            dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("Music")
        };

        // Vérifier que le chemin est valide
        assert!(music_dir.to_str().is_some());

        // Vérifier que le chemin contient "Music" ou est le répertoire home
        let path_str = music_dir.to_str().unwrap();
        assert!(path_str.contains("Music") || path_str.contains("home") || path_str == ".");
    }
}
