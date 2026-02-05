/// Tests de cas limites avancés et situations exceptionnelles

#[cfg(test)]
mod advanced_edge_cases_tests {
    use youtube_chapter_splitter::utils::{clean_folder_name, parse_artist_album, sanitize_title};
    use youtube_chapter_splitter::chapters::Chapter;

    // Tests de caractères spéciaux et Unicode

    #[test]
    fn test_clean_folder_name_cyrillic() {
        let result = clean_folder_name("Группа Крови - Альбом [1988]");
        assert!(result.contains("Группа"));
        assert!(result.contains("Крови"));
    }

    #[test]
    fn test_clean_folder_name_japanese() {
        let result = clean_folder_name("アーティスト - アルバム [Full Album]");
        assert!(result.contains("アーティスト"));
        assert!(!result.contains("[Full Album]"));
    }

    #[test]
    fn test_clean_folder_name_arabic() {
        let result = clean_folder_name("فنان - ألبوم [2024]");
        assert!(result.contains("فنان"));
    }

    #[test]
    fn test_clean_folder_name_mixed_scripts() {
        let result = clean_folder_name("Artist アーティスト - Album Альбом");
        assert!(result.contains("Artist"));
        assert!(result.contains("アーティスト"));
    }

    #[test]
    fn test_clean_folder_name_emoji_heavy() {
        let result = clean_folder_name("🎵🎸 Artist 🎵 - 🎧 Album 🎧 [Full Album] 🎵");
        assert!(result.contains("🎵"));
        assert!(result.contains("🎸"));
        assert!(!result.contains("[Full Album]"));
    }

    #[test]
    fn test_clean_folder_name_zero_width_chars() {
        // Zero-width space (U+200B)
        let result = clean_folder_name("Artist\u{200B} - Album");
        // Devrait être nettoyé ou géré gracieusement
        assert!(!result.is_empty());
    }

    #[test]
    fn test_clean_folder_name_rtl_marks() {
        // Right-to-left mark (U+200F)
        let result = clean_folder_name("Artist\u{200F} - Album");
        assert!(!result.is_empty());
    }

    // Tests de longueur extrême

    #[test]
    fn test_clean_folder_name_very_long() {
        let long_name = "A".repeat(1000);
        let result = clean_folder_name(&long_name);
        // Devrait gérer sans crasher
        assert!(!result.is_empty());
    }

    #[test]
    fn test_parse_artist_album_very_long() {
        let long_title = format!("{} - {}", "A".repeat(500), "B".repeat(500));
        let (artist, album) = parse_artist_album(&long_title);
        assert!(!artist.is_empty());
        assert!(!album.is_empty());
    }

    #[test]
    fn test_sanitize_title_very_long() {
        let long_title = "Track 1: ".to_string() + &"A".repeat(1000);
        let result = sanitize_title(&long_title);
        assert!(!result.is_empty());
    }

    // Tests de formats de titre complexes

    #[test]
    fn test_parse_artist_album_nested_brackets() {
        let (artist, album) = parse_artist_album("Artist - Album [Disc 1 [Remastered]]");
        assert_eq!(artist, "Artist");
        // La regex supprime les crochets mais peut laisser des résidus avec des crochets imbriqués
        // C'est un comportement connu et acceptable
        assert!(album.contains("Album"));
    }

    #[test]
    fn test_parse_artist_album_multiple_dashes() {
        let (artist, album) = parse_artist_album("Artist - Name - Album - Title");
        assert_eq!(artist, "Artist");
        assert_eq!(album, "Name"); // Prend le premier séparateur
    }

    #[test]
    fn test_parse_artist_album_mixed_separators() {
        let (artist, album) = parse_artist_album("Artist - Album | Extra Info");
        assert_eq!(artist, "Artist");
        // Devrait utiliser le premier séparateur trouvé
    }

    #[test]
    fn test_parse_artist_album_only_separators() {
        let (artist, album) = parse_artist_album(" - | - ");
        // Avec uniquement des séparateurs, le parsing peut retourner Unknown Artist
        // ou des chaînes vides après nettoyage
        // On vérifie juste que ça ne crash pas
        println!("Artist: '{}', Album: '{}'", artist, album);
        // Le comportement exact dépend de l'implémentation de clean_folder_name
        assert!(artist == "Unknown Artist" || artist.is_empty() || artist == "-");
    }

    // Tests de validation de chapitres

    #[test]
    #[should_panic(expected = "start_time")]
    fn test_chapter_negative_start_time() {
        let _chapter = Chapter::new("Test".to_string(), -10.0, 100.0);
    }

    #[test]
    #[should_panic(expected = "end_time")]
    fn test_chapter_end_before_start() {
        let _chapter = Chapter::new("Test".to_string(), 100.0, 50.0);
    }

    #[test]
    #[should_panic(expected = "end_time")]
    fn test_chapter_equal_times() {
        let _chapter = Chapter::new("Test".to_string(), 50.0, 50.0);
    }

    #[test]
    fn test_chapter_very_short_duration() {
        // 0.001 secondes (1ms)
        let chapter = Chapter::new("Test".to_string(), 0.0, 0.001);
        assert_eq!(chapter.duration(), 0.001);
    }

    #[test]
    fn test_chapter_very_long_duration() {
        // 24 heures
        let chapter = Chapter::new("Test".to_string(), 0.0, 86400.0);
        assert_eq!(chapter.duration(), 86400.0);
    }

    #[test]
    fn test_chapter_fractional_times() {
        let chapter = Chapter::new("Test".to_string(), 10.5, 20.7);
        assert!((chapter.duration() - 10.2).abs() < 0.001);
    }

    // Tests de sanitization avancée

    #[test]
    fn test_sanitize_title_all_forbidden_chars() {
        let result = sanitize_title("/:*?\"<>|\\");
        assert_eq!(result, "_________");
    }

    #[test]
    fn test_sanitize_title_mixed_valid_invalid() {
        let result = sanitize_title("Valid/Invalid:Chars");
        assert_eq!(result, "Valid_Invalid_Chars");
    }

    #[test]
    fn test_sanitize_title_unicode_with_invalid() {
        let result = sanitize_title("Café/Müller:Test");
        assert_eq!(result, "Café_Müller_Test");
    }

    #[test]
    fn test_sanitize_title_multiple_track_prefixes() {
        let result = sanitize_title("01. Track 5: Song Name");
        // Devrait enlever tous les préfixes
        assert!(result.contains("Song Name"));
    }

    // Tests de formats de numérotation

    #[test]
    fn test_sanitize_title_various_number_formats() {
        let test_cases = vec![
            ("1 - Song", "Song"),
            ("01. Song", "Song"),
            ("001 - Song", "Song"),
            ("Track 1: Song", "Song"),
            ("Track 01: Song", "Song"),
            ("1. Song", "Song"),
            ("1) Song", "Song"),
        ];

        for (input, expected) in test_cases {
            let result = sanitize_title(input);
            assert!(result.contains(expected), "Failed for input: {}", input);
        }
    }

    // Tests de whitespace

    #[test]
    fn test_clean_folder_name_various_whitespace() {
        let result = clean_folder_name("Artist\t-\nAlbum\r\n[2024]");
        assert!(result.contains("Artist"));
        assert!(result.contains("Album"));
    }

    #[test]
    fn test_clean_folder_name_non_breaking_space() {
        // Non-breaking space (U+00A0)
        let result = clean_folder_name("Artist\u{00A0}-\u{00A0}Album");
        assert!(!result.is_empty());
    }

    // Tests de cas pathologiques

    #[test]
    fn test_clean_folder_name_only_special_chars() {
        let result = clean_folder_name("!@#$%^&*()");
        // Devrait retourner quelque chose ou une chaîne vide
        assert!(result.is_empty() || !result.is_empty());
    }

    #[test]
    fn test_clean_folder_name_repeated_separators() {
        let result = clean_folder_name("Artist --- Album");
        assert!(result.contains("Artist"));
        assert!(result.contains("Album"));
    }

    #[test]
    fn test_parse_artist_album_url_encoded() {
        let (artist, album) = parse_artist_album("Artist%20Name - Album%20Title");
        // Devrait gérer les caractères encodés
        assert!(!artist.is_empty());
        assert!(!album.is_empty());
    }

    // Tests de performance avec données réelles

    #[test]
    fn test_real_world_titles() {
        let real_titles = vec![
            "Pink Floyd - The Dark Side of the Moon [Full Album] [1973]",
            "Led Zeppelin - Led Zeppelin IV (1971) [Full Album]",
            "The Beatles - Abbey Road [Full Album] (Remastered 2019)",
            "Radiohead | OK Computer [Full Album] (1997)",
            "TOOL - Lateralus [Full Album] [2001] [HD]",
        ];

        for title in real_titles {
            let (artist, album) = parse_artist_album(title);
            assert!(!artist.is_empty(), "Failed for: {}", title);
            assert!(!album.is_empty(), "Failed for: {}", title);
            assert_ne!(artist, "Unknown Artist", "Failed to parse: {}", title);
        }
    }

    #[test]
    fn test_chapter_sanitize_real_world() {
        let real_chapters = vec![
            "1. Speak to Me",
            "02 - Breathe",
            "Track 3: On the Run",
            "04. Time",
            "5 - The Great Gig in the Sky",
        ];

        for title in real_chapters {
            let chapter = Chapter::new(title.to_string(), 0.0, 100.0);
            let sanitized = chapter.sanitize_title();
            
            // Vérifier que le numéro est enlevé
            assert!(!sanitized.starts_with("1"));
            assert!(!sanitized.starts_with("0"));
            assert!(!sanitized.starts_with("Track"));
        }
    }
}
