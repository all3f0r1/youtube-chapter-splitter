//! Comprehensive tests for filename sanitization

#[cfg(test)]
mod filename_sanitization_tests {
    use youtube_chapter_splitter::utils::sanitize_title;

    // =========================================================================
    // Invalid character replacement tests
    // =========================================================================

    #[test]
    fn test_sanitize_forward_slash() {
        // Note: to_title_case treats underscore as part of word, so "Song_Name" -> "Song_name"
        assert_eq!(sanitize_title("Song/Name"), "Song_name");
    }

    #[test]
    fn test_sanitize_backslash() {
        assert_eq!(sanitize_title("Song\\Name"), "Song_name");
    }

    #[test]
    fn test_sanitize_colon() {
        assert_eq!(sanitize_title("Song:Name"), "Song_name");
    }

    #[test]
    fn test_sanitize_asterisk() {
        assert_eq!(sanitize_title("Song*Name"), "Song_name");
    }

    #[test]
    fn test_sanitize_question_mark() {
        assert_eq!(sanitize_title("Song?Name"), "Song_name");
    }

    #[test]
    fn test_sanitize_double_quote() {
        assert_eq!(sanitize_title("Song\"Name"), "Song_name");
    }

    #[test]
    fn test_sanitize_less_than() {
        assert_eq!(sanitize_title("Song<Name"), "Song_name");
    }

    #[test]
    fn test_sanitize_greater_than() {
        assert_eq!(sanitize_title("Song>Name"), "Song_name");
    }

    #[test]
    fn test_sanitize_pipe() {
        assert_eq!(sanitize_title("Song|Name"), "Song_name");
    }

    #[test]
    fn test_sanitize_all_invalid_chars() {
        assert_eq!(sanitize_title("/:*?\"<>|\\"), "_________");
    }

    #[test]
    fn test_sanitize_consecutive_invalid_chars() {
        assert_eq!(sanitize_title("Song///Name"), "Song___name");
        assert_eq!(sanitize_title("Test:::Title"), "Test___title");
    }

    // =========================================================================
    // Unicode preservation tests
    // =========================================================================

    #[test]
    fn test_sanitize_unicode_acute() {
        assert_eq!(sanitize_title("Café Deluxe"), "Café Deluxe");
    }

    #[test]
    fn test_sanitize_unicode_umlaut() {
        assert_eq!(sanitize_title("Björk"), "Björk");
    }

    #[test]
    fn test_sanitize_unicode_tilde() {
        assert_eq!(sanitize_title("El Niño"), "El Niño");
    }

    #[test]
    fn test_sanitize_unicode_cjk() {
        assert_eq!(sanitize_title("日本語 Title"), "日本語 Title");
    }

    #[test]
    fn test_sanitize_unicode_greek() {
        assert_eq!(sanitize_title("Ελληνικά"), "Ελληνικά");
    }

    // =========================================================================
    // Track prefix removal tests
    // =========================================================================

    #[test]
    fn test_sanitize_prefix_with_dash() {
        assert_eq!(sanitize_title("1 - Song Name"), "Song Name");
    }

    #[test]
    fn test_sanitize_prefix_with_dot() {
        assert_eq!(sanitize_title("01. Song Name"), "Song Name");
    }

    #[test]
    fn test_sanitize_prefix_track_colon() {
        assert_eq!(sanitize_title("Track 5: Song Name"), "Song Name");
    }

    #[test]
    fn test_sanitize_prefix_number_only() {
        assert_eq!(sanitize_title("1 the unholy"), "The Unholy");
    }

    #[test]
    fn test_sanitize_prefix_two_digit() {
        assert_eq!(sanitize_title("12 song title"), "Song Title");
    }

    // =========================================================================
    // Title case capitalization tests
    // =========================================================================

    #[test]
    fn test_sanitize_all_caps() {
        assert_eq!(sanitize_title("THE UNHOLY"), "The Unholy");
    }

    #[test]
    fn test_sanitize_all_lowercase() {
        assert_eq!(sanitize_title("café deluxe"), "Café Deluxe");
    }

    #[test]
    fn test_sanitize_mixed_case() {
        assert_eq!(sanitize_title("sOnG NaMe"), "Song Name");
    }

    // =========================================================================
    // Edge case tests
    // =========================================================================

    #[test]
    fn test_sanitize_empty() {
        assert_eq!(sanitize_title(""), "");
    }

    #[test]
    fn test_sanitize_whitespace_only() {
        assert_eq!(sanitize_title("   "), "");
    }

    #[test]
    fn test_sanitize_only_number() {
        assert_eq!(sanitize_title("1"), "1");
    }

    #[test]
    fn test_sanitize_only_dashes() {
        assert_eq!(sanitize_title("---"), "---");
    }

    #[test]
    fn test_sanitize_long_title() {
        let long_title = "a".repeat(300);
        let result = sanitize_title(&long_title);
        assert_eq!(result.len(), 300);
    }

    // =========================================================================
    // Mixed scenario tests
    // =========================================================================

    #[test]
    fn test_sanitize_mixed_prefix_caps_invalid() {
        assert_eq!(
            sanitize_title("01. CAFÉ/DELUXE: SPECIAL"),
            "Café_deluxe_ Special"
        );
    }

    #[test]
    fn test_sanitize_complex_title() {
        assert_eq!(
            sanitize_title("Track 3: THE/ANSWER:REVEALED"),
            "The_answer_revealed"
        );
    }
}
