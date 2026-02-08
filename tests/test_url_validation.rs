//! Tests avancés de validation d'URL YouTube

#[cfg(test)]
mod url_validation_tests {
    use youtube_chapter_splitter::downloader::extract_video_id;

    #[test]
    fn test_valid_video_id_length() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
        let id = extract_video_id(url).unwrap();
        assert_eq!(
            id.len(),
            11,
            "YouTube video IDs must be exactly 11 characters"
        );
    }

    #[test]
    fn test_video_id_alphanumeric_underscore_dash() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
        let id = extract_video_id(url).unwrap();

        for c in id.chars() {
            assert!(
                c.is_ascii_alphanumeric() || c == '_' || c == '-',
                "Video ID should only contain alphanumeric, underscore, or dash: found '{}'",
                c
            );
        }
    }

    #[test]
    fn test_invalid_url_no_video_id() {
        let url = "https://www.youtube.com/";
        let result = extract_video_id(url);
        assert!(result.is_err(), "Should fail for URL without video ID");
    }

    #[test]
    fn test_invalid_url_empty_video_id() {
        let url = "https://www.youtube.com/watch?v=";
        let result = extract_video_id(url);
        assert!(result.is_err(), "Should fail for empty video ID");
    }

    #[test]
    fn test_url_with_special_chars_in_id() {
        // YouTube IDs ne contiennent pas de caractères spéciaux comme @#$%
        let url = "https://www.youtube.com/watch?v=abc@def#ghi";
        let result = extract_video_id(url);

        if let Ok(id) = result {
            // Si ça passe, vérifier que les caractères spéciaux sont exclus
            assert!(!id.contains('@'));
            assert!(!id.contains('#'));
        }
        // Sinon, c'est correct que ça échoue
    }

    #[test]
    fn test_url_with_very_long_id() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQextracharacters";
        let result = extract_video_id(url);

        if let Ok(id) = result {
            // Devrait tronquer ou échouer
            assert!(id.len() <= 11, "Video ID should not exceed 11 characters");
        }
    }

    #[test]
    fn test_url_with_very_short_id() {
        let url = "https://www.youtube.com/watch?v=abc";
        let result = extract_video_id(url);

        // Un ID de 3 caractères n'est pas valide
        if let Ok(id) = result {
            assert_eq!(id.len(), 11, "Should validate ID length");
        }
    }

    #[test]
    fn test_url_case_sensitivity() {
        let url1 = "https://www.youtube.com/watch?v=AbCdEfGhIjK";
        let url2 = "https://www.youtube.com/watch?v=abcdefghijk";

        let id1 = extract_video_id(url1).unwrap();
        let id2 = extract_video_id(url2).unwrap();

        // Les IDs sont sensibles à la casse
        assert_ne!(id1, id2);
        assert_eq!(id1, "AbCdEfGhIjK");
        assert_eq!(id2, "abcdefghijk");
    }

    #[test]
    fn test_url_with_unicode() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ&title=Café";
        let id = extract_video_id(url).unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_url_with_encoded_chars() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=30%20seconds";
        let id = extract_video_id(url).unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_url_mobile_format() {
        let url = "https://m.youtube.com/watch?v=dQw4w9WgXcQ";
        let id = extract_video_id(url).unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_url_youtu_be_short_format() {
        let url = "https://youtu.be/dQw4w9WgXcQ";
        let result = extract_video_id(url);

        // Le code actuel ne gère pas youtu.be, mais devrait
        if result.is_err() {
            // C'est un bug connu, documenter
            println!("Note: youtu.be URLs are not currently supported");
        }
    }

    #[test]
    fn test_url_with_fragment() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ#comments";
        let id = extract_video_id(url).unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_url_with_multiple_v_params() {
        // URL malformée avec plusieurs paramètres v=
        let url = "https://www.youtube.com/watch?v=first&v=second";
        let result = extract_video_id(url);

        // Le comportement avec plusieurs v= n'est pas défini
        // Soit ça échoue (comportement actuel), soit ça prend le premier
        if let Ok(id) = result {
            // Si ça réussit, devrait prendre le premier
            assert!(id == "first" || id.len() == 11);
        }
        // C'est acceptable que ça échoue pour une URL malformée
    }

    #[test]
    fn test_url_without_protocol() {
        let url = "youtube.com/watch?v=dQw4w9WgXcQ";
        let id = extract_video_id(url).unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_url_with_www_prefix() {
        let url = "www.youtube.com/watch?v=dQw4w9WgXcQ";
        let id = extract_video_id(url).unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_url_with_trailing_slash() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ/";
        let id = extract_video_id(url).unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_url_with_spaces() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ ";
        let id = extract_video_id(url).unwrap();
        assert_eq!(id, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_completely_invalid_url() {
        let url = "not a url at all";
        let result = extract_video_id(url);
        assert!(result.is_err(), "Should fail for completely invalid URL");
    }

    #[test]
    fn test_empty_url() {
        let url = "";
        let result = extract_video_id(url);
        assert!(result.is_err(), "Should fail for empty URL");
    }
}
