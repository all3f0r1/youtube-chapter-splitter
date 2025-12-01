/// Tests de cas limites pour le module utils
#[cfg(test)]
mod utils_edge_cases_tests {
    use youtube_chapter_splitter::utils::{
        clean_folder_name, format_duration, format_duration_short, parse_artist_album,
        sanitize_title,
    };

    // Tests pour clean_folder_name

    #[test]
    fn test_clean_folder_name_empty() {
        let result = clean_folder_name("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_clean_folder_name_only_brackets() {
        let result = clean_folder_name("[test] (test)");
        assert_eq!(result, "");
    }

    #[test]
    fn test_clean_folder_name_unicode() {
        let result = clean_folder_name("Björk - Homogénic [Full Album]");
        assert_eq!(result, "Björk - Homogénic");
    }

    #[test]
    fn test_clean_folder_name_emoji() {
        let result = clean_folder_name("Artist 🎵 - Album 🎸");
        assert_eq!(result, "Artist 🎵 - Album 🎸");
    }

    #[test]
    fn test_clean_folder_name_multiple_spaces() {
        let result = clean_folder_name("Artist    -    Album");
        assert_eq!(result, "Artist - Album");
    }

    #[test]
    fn test_clean_folder_name_leading_trailing_spaces() {
        let result = clean_folder_name("   Artist - Album   ");
        assert_eq!(result, "Artist - Album");
    }

    #[test]
    fn test_clean_folder_name_only_dashes() {
        let result = clean_folder_name("---");
        assert_eq!(result, "");
    }

    #[test]
    fn test_clean_folder_name_mixed_case() {
        let result = clean_folder_name("aRtIsT - aLbUm");
        assert_eq!(result, "Artist - Album");
    }

    #[test]
    fn test_clean_folder_name_numbers() {
        let result = clean_folder_name("AC/DC - Back In Black [1980]");
        assert_eq!(result, "Ac-dc - Back In Black");
    }

    #[test]
    fn test_clean_folder_name_special_chars() {
        let result = clean_folder_name("Artist_Name | Album/Title");
        assert_eq!(result, "Artist-name - Album-title");
    }

    // Tests pour parse_artist_album

    #[test]
    fn test_parse_artist_album_no_separator() {
        let (artist, album) = parse_artist_album("Just A Title", "TestChannel");
        assert_eq!(artist, "Testchannel"); // clean_folder_name capitalise le nom
        assert_eq!(album, "Just A Title");
    }

    #[test]
    fn test_parse_artist_album_empty() {
        let (artist, album) = parse_artist_album("", "TestChannel");
        assert_eq!(artist, "Testchannel"); // clean_folder_name capitalise le nom
        assert_eq!(album, "");
    }

    #[test]
    fn test_parse_artist_album_only_artist() {
        let (artist, _album) = parse_artist_album("Artist -", "TestChannel");
        assert_eq!(artist, "Testchannel"); // clean_folder_name capitalise le nom
                                           // Avec un seul élément après split, ça retourne le titre nettoyé
    }

    #[test]
    fn test_parse_artist_album_multiple_separators() {
        let (artist, album) = parse_artist_album("Artist - Album - Extra", "TestChannel");
        assert_eq!(artist, "Artist");
        assert_eq!(album, "Album");
        // Le troisième élément est ignoré
    }

    #[test]
    fn test_parse_artist_album_pipe_separator() {
        let (artist, album) = parse_artist_album("Artist | Album", "TestChannel");
        assert_eq!(artist, "Artist");
        assert_eq!(album, "Album");
    }

    #[test]
    fn test_parse_artist_album_with_full_album_tag() {
        let (artist, album) = parse_artist_album("Artist - Album [FULL ALBUM]", "TestChannel");
        assert_eq!(artist, "Artist");
        assert_eq!(album, "Album");
    }

    // Tests pour sanitize_title

    #[test]
    fn test_sanitize_title_empty() {
        let result = sanitize_title("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_sanitize_title_only_number() {
        let result = sanitize_title("1");
        assert_eq!(result, "1");
    }

    #[test]
    fn test_sanitize_title_all_invalid_chars() {
        let result = sanitize_title("/:*?\"<>|");
        assert_eq!(result, "________");
    }

    #[test]
    fn test_sanitize_title_unicode() {
        let result = sanitize_title("Café Müller");
        assert_eq!(result, "Café Müller");
    }

    #[test]
    fn test_sanitize_title_mixed_prefixes() {
        let result = sanitize_title("01. Track 5: Song Name");
        // Devrait enlever "01. " mais pas "Track 5: "
        // ou enlever tout jusqu'à "Song Name"
        assert!(result.contains("Song Name"));
    }

    // Tests pour format_duration

    #[test]
    fn test_format_duration_zero() {
        assert_eq!(format_duration(0.0), "0m 00s");
    }

    #[test]
    fn test_format_duration_negative() {
        // Comportement avec valeur négative (non défini)
        let result = format_duration(-10.0);
        // Devrait gérer gracieusement ou retourner une erreur
        assert!(!result.is_empty());
    }

    #[test]
    fn test_format_duration_very_large() {
        // 100 heures
        let result = format_duration(360000.0);
        assert_eq!(result, "100h 00m 00s");
    }

    #[test]
    fn test_format_duration_fractional() {
        // 90.5 secondes
        assert_eq!(format_duration(90.5), "1m 30s");
    }

    #[test]
    fn test_format_duration_short_zero() {
        assert_eq!(format_duration_short(0.0), "0m 00s");
    }

    #[test]
    fn test_format_duration_short_over_hour() {
        // 3661 secondes = 1h 1m 1s, mais format_short n'affiche que minutes
        assert_eq!(format_duration_short(3661.0), "61m 01s");
    }

    #[test]
    fn test_parse_artist_album_with_em_dash() {
        let (artist, album) =
            parse_artist_album("Arcane Voyage – Third (FULL ALBUM)", "TestChannel");
        assert_eq!(artist, "Arcane Voyage");
        assert_eq!(album, "Third");
    }
}
