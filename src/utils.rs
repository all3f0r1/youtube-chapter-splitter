use once_cell::sync::Lazy;
use regex::Regex;

// Regex compiled once at startup
static RE_FULL_ALBUM: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\s*[\[(]full\s+album[\])].*$").unwrap());

static RE_FULL_ALBUM_UNBRACKETED: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\s*-\s*full\s+album\s*$").unwrap());

static RE_BRACKETS: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[.*?\]|\(.*?\)").unwrap());

static RE_SPACES: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

static RE_TRACK_PREFIX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(?:Track\s+)?\d+\s*[-.:)]?\s+").unwrap());

/// Converts a string to title case (first letter of each word capitalized).
///
/// # Arguments
///
/// * `s` - The input string
///
/// # Returns
///
/// A title-cased string
fn to_title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>()
                        + &chars.collect::<String>().to_lowercase()
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Cleans and formats a folder name according to defined rules.
///
/// This function applies several transformations to normalize folder names:
/// - Removes everything after `[FULL ALBUM]` or `(FULL ALBUM)` (case insensitive)
/// - Removes all brackets `[]` and parentheses `()` with their content
/// - Replaces underscores `_`, pipes `|` and slashes `/` with dashes `-`
/// - Normalizes multiple spaces into a single space
/// - Capitalizes each word (first letter uppercase, rest lowercase)
/// - Removes spaces and dashes at the beginning/end of the string
///
/// # Arguments
///
/// * `name` - The raw folder name to clean
///
/// # Returns
///
/// A cleaned and formatted string ready to use as a folder name
///
/// # Examples
///
/// ```
/// use youtube_chapter_splitter::utils::clean_folder_name;
///
/// let result = clean_folder_name("MARIGOLD - Oblivion Gate [Full Album] (70s Rock)");
/// assert_eq!(result, "Marigold - Oblivion Gate");
/// ```
pub fn clean_folder_name(name: &str) -> String {
    // Remove everything after (FULL ALBUM) or [FULL ALBUM] (case insensitive)
    let without_suffix = RE_FULL_ALBUM.replace_all(name, "");

    // Remove remaining [] and () with their content
    let cleaned = RE_BRACKETS.replace_all(&without_suffix, "");

    // Replace underscores, pipes and slashes with dashes
    let with_dashes = cleaned.replace(['_', '|', '/'], "-");

    // Remove trailing - Full Album (without brackets) after pipe replacement
    let with_dashes = RE_FULL_ALBUM_UNBRACKETED.replace_all(&with_dashes, "");

    // Clean multiple spaces
    let normalized = RE_SPACES.replace_all(&with_dashes, " ");

    // Capitalize each word
    let capitalized = normalized
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>()
                        + chars.as_str().to_lowercase().as_str()
                }
            }
        })
        .collect::<Vec<String>>()
        .join(" ");

    // Clean dashes and spaces at beginning/end
    capitalized.trim().trim_matches('-').trim().to_string()
}

/// Formats a duration in seconds as a readable string.
///
/// Converts a duration in seconds to a formatted string:
/// - `Mm SSs` format if duration is less than an hour
/// - `Hh MMm SSs` format if duration is an hour or more
///
/// # Arguments
///
/// * `seconds` - The duration in seconds (can be decimal)
///
/// # Returns
///
/// A formatted string representing the duration
///
/// # Examples
///
/// ```
/// use youtube_chapter_splitter::utils::format_duration;
///
/// assert_eq!(format_duration(90.0), "1m 30s");
/// assert_eq!(format_duration(3661.0), "1h 01m 01s");
/// ```
pub fn format_duration(seconds: f64) -> String {
    let hours = (seconds / 3600.0).floor() as u32;
    let minutes = ((seconds % 3600.0) / 60.0).floor() as u32;
    let secs = (seconds % 60.0).floor() as u32;

    if hours > 0 {
        format!("{}h {:02}m {:02}s", hours, minutes, secs)
    } else {
        format!("{}m {:02}s", minutes, secs)
    }
}

/// Formats a duration in seconds in short format.
///
/// Converts a duration in seconds to a compact string in `Mm SSs` format,
/// without displaying hours even if the duration exceeds one hour.
///
/// # Arguments
///
/// * `seconds` - The duration in seconds (can be decimal)
///
/// # Returns
///
/// A formatted string in short format
///
/// # Examples
///
/// ```
/// use youtube_chapter_splitter::utils::format_duration_short;
///
/// assert_eq!(format_duration_short(343.0), "5m 43s");
/// ```
pub fn format_duration_short(seconds: f64) -> String {
    let minutes = (seconds / 60.0).floor() as u32;
    let secs = (seconds % 60.0).floor() as u32;
    format!("{}m {:02}s", minutes, secs)
}

/// Parses a YouTube video title to extract artist and album.
///
/// This function analyzes a video title and attempts to extract the artist name
/// and album title based on common naming conventions.
///
/// # Expected format
///
/// - `"ARTIST - ALBUM [...]"` (dash separator)
/// - `"ARTIST | ALBUM [...]"` (pipe separator)
///
/// # Arguments
///
/// * `title` - The YouTube video title
///
/// # Returns
///
/// A tuple `(artist, album)` where:
/// - If parsing succeeds: both values are extracted and cleaned
/// - If parsing fails: `("Unknown Artist", cleaned_title)`
///
/// # Examples
///
/// ```
/// use youtube_chapter_splitter::utils::parse_artist_album;
///
/// let (artist, album) = parse_artist_album("Pink Floyd - Dark Side [1973]");
/// assert_eq!(artist, "Pink Floyd");
/// assert_eq!(album, "Dark Side");
/// ```
pub fn parse_artist_album(title: &str) -> (String, String) {
    parse_artist_album_with_source(title).0
}

/// Parses a YouTube video title to extract artist and album with source information.
///
/// This is a version of `parse_artist_album` that also returns the source
/// (detected vs default) for use in UI display.
///
/// # Returns
///
/// A tuple containing:
/// - `((String, String), MetadataSource, MetadataSource)` - (artist, album) tuple, artist source, album source
///
/// # Examples
///
/// ```
/// use youtube_chapter_splitter::utils::parse_artist_album_with_source;
/// use youtube_chapter_splitter::ui::MetadataSource;
///
/// let ((artist, album), artist_src, album_src) = parse_artist_album_with_source("Pink Floyd - Dark Side [1973]");
/// assert_eq!(artist, "Pink Floyd");
/// assert_eq!(album, "Dark Side");
/// assert_eq!(artist_src, MetadataSource::Detected);
/// ```
pub fn parse_artist_album_with_source(
    title: &str,
) -> (
    (String, String),
    crate::ui::MetadataSource,
    crate::ui::MetadataSource,
) {
    use crate::ui::MetadataSource;

    // Remove everything after (FULL ALBUM) or [FULL ALBUM]
    let without_suffix = RE_FULL_ALBUM.replace_all(title, "");

    // Remove remaining [] and ()
    let cleaned = RE_BRACKETS.replace_all(&without_suffix, "");

    // Normalize various dash types to hyphen for splitting
    // Handle: - (hyphen), – (en dash), — (em dash), ― (horizontal bar)
    let normalized = cleaned.replace(['–', '—', '―'], "-");

    // Split by - or |
    let parts: Vec<&str> = if normalized.contains(" - ") {
        normalized.split(" - ").collect()
    } else if normalized.contains(" | ") {
        normalized.split(" | ").collect()
    } else {
        vec![normalized.trim()]
    };

    if parts.len() >= 2 {
        let artist = clean_folder_name(parts[0].trim());
        let album = clean_folder_name(parts[1].trim());
        (
            (artist, album),
            MetadataSource::Detected,
            MetadataSource::Detected,
        )
    } else {
        let cleaned_title = clean_folder_name(normalized.trim());
        (
            ("Unknown Artist".to_string(), cleaned_title),
            MetadataSource::Default,
            MetadataSource::Default,
        )
    }
}

/// Cleans a chapter title for use as a filename.
///
/// This function removes track numbering prefixes and replaces
/// invalid characters for file systems.
///
/// # Applied transformations
///
/// - Removes prefixes like `"1 - "`, `"01. "`, `"Track 5: "`
/// - Replaces forbidden characters (`/`, `\`, `:`, `*`, `?`, `"`, `<`, `>`, `|`) with `_`
/// - Applies title case (first letter of each word capitalized)
///
/// # Arguments
///
/// * `title` - The raw chapter title
///
/// # Returns
///
/// A cleaned title safe for use as a filename
///
/// # Examples
///
/// ```
/// use youtube_chapter_splitter::utils::sanitize_title;
///
/// assert_eq!(sanitize_title("1 - Song Name"), "Song Name");
/// // Note: to_title_case treats underscores as part of words, so only first char is capitalized
/// assert_eq!(sanitize_title("Track 5: Test/Song"), "Test_song");
/// ```
pub fn sanitize_title(title: &str) -> String {
    // Remove track numbers at the beginning
    let title = RE_TRACK_PREFIX.replace(title, "");

    // Replace invalid characters
    let sanitized: String = title
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect();

    // Apply title case
    to_title_case(&sanitized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_folder_name() {
        assert_eq!(
            clean_folder_name(
                "MARIGOLD - Oblivion Gate [Full Album] (70s Psychedelic Blues Acid Rock)"
            ),
            "Marigold - Oblivion Gate"
        );

        assert_eq!(
            clean_folder_name("Artist_Name - Album_Title [2024]"),
            "Artist-name - Album-title"
        );

        assert_eq!(
            clean_folder_name("test_album (bonus tracks) [remastered]"),
            "Test-album"
        );

        assert_eq!(
            clean_folder_name(
                "PURPLE DREAMS - WANDERING SHADOWS (FULL ALBUM) | 70s Progressive/Psychedelic Rock"
            ),
            "Purple Dreams - Wandering Shadows"
        );

        // Test for pipe-separated full album (like Chronomancer | MAGNUM OPUS | FULL ALBUM)
        assert_eq!(
            clean_folder_name("Chronomancer | MAGNUM OPUS | FULL ALBUM (Progressive Rock)"),
            "Chronomancer - Magnum Opus"
        );

        // Test for hyphen-separated full album without brackets
        assert_eq!(
            clean_folder_name("Artist Name - Album Name - Full Album"),
            "Artist Name - Album Name"
        );
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(90.0), "1m 30s");
        assert_eq!(format_duration(3661.0), "1h 01m 01s");
        assert_eq!(format_duration(45.0), "0m 45s");
    }

    #[test]
    fn test_format_duration_short() {
        assert_eq!(format_duration_short(343.0), "5m 43s");
        assert_eq!(format_duration_short(90.0), "1m 30s");
    }

    #[test]
    fn test_sanitize_title() {
        assert_eq!(sanitize_title("1 - Song Name"), "Song Name");
        assert_eq!(sanitize_title("Track 5: Another Song"), "Another Song");
        // Note: to_title_case treats underscores as part of words, so only first char is capitalized
        assert_eq!(
            sanitize_title("Invalid/Characters:Here"),
            "Invalid_characters_here"
        );
        // New test cases for title case
        assert_eq!(sanitize_title("THE UNHOLY"), "The Unholy");
        assert_eq!(sanitize_title("1 the unholy"), "The Unholy");
    }

    #[test]
    fn test_parse_artist_album() {
        let (artist, album) = parse_artist_album("Pink Floyd - Dark Side of the Moon [1973]");
        assert_eq!(artist, "Pink Floyd");
        assert_eq!(album, "Dark Side Of The Moon");

        // Test with en dash (–)
        let (artist, album) = parse_artist_album("MARIGOLD – ACCIDENTAL SENSE [FULL ALBUM]");
        assert_eq!(artist, "Marigold");
        assert_eq!(album, "Accidental Sense");

        // Test with em dash (—)
        let (artist, album) = parse_artist_album("Artist — Album Name");
        assert_eq!(artist, "Artist");
        assert_eq!(album, "Album Name");
    }
}
