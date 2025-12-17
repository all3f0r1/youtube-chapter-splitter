use once_cell::sync::Lazy;
use regex::Regex;

// Regex compilées une seule fois au démarrage
static RE_FULL_ALBUM: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\s*[\[(]full\s+album[\])].*$").unwrap());

static RE_BRACKETS: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[.*?\]|\(.*?\)|\[.*$|\(.*$").unwrap());

static RE_SPACES: Lazy<Regex> = Lazy::new(|| Regex::new(r"\s+").unwrap());

// Updated regex to also match "N)" format (e.g., "1) Title", "12) Title")
static RE_TRACK_PREFIX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^\s*(?:Track\s+)?\d+\s*[-.:)]\s*").unwrap());

static RE_GENRE_TAGS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\s+\d{2,4}s?\s+[\w\s]+[•·].*$").unwrap());

// Regex for emoji separators (crown, star, etc.) commonly used in YouTube titles
static RE_EMOJI_SEPARATOR: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\s*[\p{Emoji_Presentation}\p{Extended_Pictographic}]+\s*").unwrap());

/// Capitalizes each word in a string (Title Case).
///
/// This function converts each word to have its first letter uppercase
/// and the rest lowercase.
///
/// # Arguments
///
/// * `s` - The string to capitalize
///
/// # Returns
///
/// A string with each word capitalized
///
/// # Examples
///
/// ```
/// use youtube_chapter_splitter::utils::capitalize_words;
///
/// assert_eq!(capitalize_words("HELLO WORLD"), "Hello World");
/// assert_eq!(capitalize_words("hello world"), "Hello World");
/// ```
pub fn capitalize_words(s: &str) -> String {
    s.split_whitespace()
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
        .join(" ")
}

/// Nettoie et formate le nom de dossier selon les règles définies.
///
/// Cette fonction applique plusieurs transformations pour normaliser les noms de dossiers :
/// - Retire tout le contenu après `[FULL ALBUM]` ou `(FULL ALBUM)` (insensible à la casse)
/// - Supprime les tags de genre musical (ex: "70s Psychedelic • Progressive Rock")
/// - Supprime tous les crochets `[]` et parenthèses `()` avec leur contenu
/// - Remplace les underscores `_`, pipes `|` et slashes `/` par des tirets `-`
/// - Normalise les espaces multiples en un seul espace
/// - Capitalise chaque mot (première lettre en majuscule, reste en minuscule)
/// - Supprime les espaces et tirets en début/fin de chaîne
///
/// # Arguments
///
/// * `name` - Le nom de dossier brut à nettoyer
///
/// # Returns
///
/// Une chaîne nettoyée et formatée, prête à être utilisée comme nom de dossier
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
    // Retirer tout après (FULL ALBUM) ou [FULL ALBUM] (case insensitive)
    let without_suffix = RE_FULL_ALBUM.replace_all(name, "");

    // Retirer les tags de genre musical (ex: "70s Psychedelic • Progressive Rock")
    let without_genre = RE_GENRE_TAGS.replace_all(&without_suffix, "");

    // Retirer les [] et () avec leur contenu restants
    let cleaned = RE_BRACKETS.replace_all(&without_genre, "");

    // Remplacer les underscores, pipes et slashes par des tirets
    let with_dashes = cleaned.replace(['_', '|', '/'], "-");

    // Nettoyer les espaces multiples
    let normalized = RE_SPACES.replace_all(&with_dashes, " ");

    // Capitaliser chaque mot
    let capitalized = capitalize_words(&normalized);

    // Nettoyer les tirets et espaces en début/fin
    capitalized.trim().trim_matches('-').trim().to_string()
}

/// Formate une durée en secondes au format lisible.
///
/// Convertit une durée en secondes en une chaîne formatée :
/// - Format `Mm SSs` si la durée est inférieure à une heure
/// - Format `Hh MMm SSs` si la durée est d'une heure ou plus
///
/// # Arguments
///
/// * `seconds` - La durée en secondes (peut être décimale)
///
/// # Returns
///
/// Une chaîne formatée représentant la durée
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

/// Formate une durée en secondes au format court.
///
/// Convertit une durée en secondes en une chaîne compacte au format `Mm SSs`,
/// sans afficher les heures même si la durée dépasse une heure.
///
/// # Arguments
///
/// * `seconds` - La durée en secondes (peut être décimale)
///
/// # Returns
///
/// Une chaîne formatée au format court
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
/// Cette fonction détecte automatiquement les formats courants de titres YouTube :
/// - `"ARTIST - ALBUM [...]"` (séparateur tiret)
/// - `"ARTIST | ALBUM [...]"` (séparateur pipe)
/// - `"ARTIST 👑 ALBUM"` (séparateur emoji)
///
/// # Arguments
///
/// * `title` - Le titre de la vidéo YouTube
/// * `uploader` - Le nom de la chaîne YouTube (utilisé comme fallback pour l'artiste)
///
/// # Returns
///
/// Un tuple `(artiste, album)` où :
/// - Si le parsing réussit : les deux valeurs sont extraites et nettoyées
/// - Si le parsing échoue : `(uploader, titre_nettoyé)` ou `("Unknown Artist", titre_nettoyé)` si uploader est vide
///
/// # Examples
///
/// ```
/// use youtube_chapter_splitter::utils::parse_artist_album;
///
/// let (artist, album) = parse_artist_album("Pink Floyd - Dark Side [1973]", "SomeChannel");
/// assert_eq!(artist, "Pink Floyd");
/// assert_eq!(album, "Dark Side");
///
/// // With emoji separator
/// let (artist, album) = parse_artist_album("Black Crown Crows 👑 When We Two Parted", "SomeChannel");
/// assert_eq!(artist, "Black Crown Crows");
/// assert_eq!(album, "When We Two Parted");
/// ```
pub fn parse_artist_album(title: &str, uploader: &str) -> (String, String) {
    // Retirer tout après (FULL ALBUM) ou [FULL ALBUM]
    let without_suffix = RE_FULL_ALBUM.replace_all(title, "");

    // Retirer les [] et () restants
    let cleaned = RE_BRACKETS.replace_all(&without_suffix, "");

    // Try to split by emoji first (e.g., "Artist 👑 Album")
    let emoji_parts: Vec<&str> = RE_EMOJI_SEPARATOR.split(&cleaned).collect();
    if emoji_parts.len() >= 2 {
        // Filter out empty parts and take first two non-empty parts
        let non_empty: Vec<&str> = emoji_parts
            .iter()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        if non_empty.len() >= 2 {
            let artist = clean_folder_name(non_empty[0]);
            let album = clean_folder_name(non_empty[1]);
            return (artist, album);
        }
    }

    // Normaliser les tirets collés (ex: "Mammoth-" -> "Mammoth - ")
    // Utiliser une regex pour remplacer tous les tirets par " - "
    let normalized = cleaned
        .as_ref()
        .split('-')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>()
        .join(" - ");

    // Séparer par - (tiret), – (tiret long/em-dash), ou |
    let parts: Vec<&str> = if normalized.contains(" - ") {
        normalized.split(" - ").collect()
    } else if normalized.contains(" – ") {
        normalized.split(" – ").collect()
    } else if normalized.contains(" | ") {
        normalized.split(" | ").collect()
    } else {
        vec![normalized.trim()]
    };

    if parts.len() >= 2 {
        let artist = clean_folder_name(parts[0].trim());
        let album = clean_folder_name(parts[1].trim());
        (artist, album)
    } else {
        let cleaned_title = clean_folder_name(cleaned.trim());
        // Utiliser le nom de la chaîne comme artiste si disponible
        let artist = if uploader.is_empty() || uploader == "Unknown" {
            "Unknown Artist".to_string()
        } else {
            clean_folder_name(uploader)
        };
        (artist, cleaned_title)
    }
}

/// Nettoie un titre de chapitre pour l'utiliser comme nom de fichier.
///
/// Cette fonction supprime les préfixes de numérotation de piste, applique
/// la capitalisation Title Case, et remplace les caractères invalides pour
/// les systèmes de fichiers.
///
/// # Transformations appliquées
///
/// - Supprime les préfixes comme `"1 - "`, `"01. "`, `"Track 5: "`, `"1) "`
/// - Applique Title Case (première lettre de chaque mot en majuscule)
/// - Remplace les caractères interdits (`/`, `\`, `:`, `*`, `?`, `"`, `<`, `>`, `|`) par `_`
///
/// # Arguments
///
/// * `title` - Le titre brut du chapitre
///
/// # Returns
///
/// Un titre nettoyé, sûr pour une utilisation comme nom de fichier
///
/// # Examples
///
/// ```
/// use youtube_chapter_splitter::utils::sanitize_title;
///
/// assert_eq!(sanitize_title("1 - Song Name"), "Song Name");
/// assert_eq!(sanitize_title("1) Dunes of Dread"), "Dunes Of Dread");
/// assert_eq!(sanitize_title("VOICES BENEATH THE RAIN"), "Voices Beneath The Rain");
/// assert_eq!(sanitize_title("Track 5: Test/Song"), "Test_song");
/// ```
pub fn sanitize_title(title: &str) -> String {
    // Retirer les numéros de piste au début
    let title = RE_TRACK_PREFIX.replace(title, "");

    // Apply Title Case capitalization
    let title_cased = capitalize_words(&title);

    // Remplacer les caractères invalides
    title_cased
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
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
        assert_eq!(
            sanitize_title("Invalid/Characters:Here"),
            "Invalid_characters_here"
        );
    }

    #[test]
    fn test_sanitize_title_with_parenthesis_prefix() {
        // Bug fix: "1) Dunes of Dread" should become "Dunes Of Dread"
        assert_eq!(sanitize_title("1) Dunes of Dread"), "Dunes Of Dread");
        assert_eq!(sanitize_title("12) Some Track Name"), "Some Track Name");
        assert_eq!(sanitize_title("5) HELLO WORLD"), "Hello World");
    }

    #[test]
    fn test_sanitize_title_uppercase() {
        // Bug fix: "VOICES BENEATH THE RAIN" should become "Voices Beneath The Rain"
        assert_eq!(
            sanitize_title("VOICES BENEATH THE RAIN"),
            "Voices Beneath The Rain"
        );
        assert_eq!(sanitize_title("RIVERS OF TOMORROW"), "Rivers Of Tomorrow");
        assert_eq!(sanitize_title("THE LONG RETURN"), "The Long Return");
    }

    #[test]
    fn test_parse_artist_album() {
        let (artist, album) =
            parse_artist_album("Pink Floyd - Dark Side of the Moon [1973]", "SomeChannel");
        assert_eq!(artist, "Pink Floyd");
        assert_eq!(album, "Dark Side Of The Moon");

        // Test avec em-dash et FULL ALBUM
        let (artist, album) = parse_artist_album(
            "Arcane Voyage – Third (FULL ALBUM) 70s Progressive • Psychedelic Rock",
            "SomeChannel",
        );
        assert_eq!(artist, "Arcane Voyage");
        assert_eq!(album, "Third");

        // Test avec fallback sur le nom de la chaîne
        let (artist, album) = parse_artist_album("Some Album Title", "HasvAlner");
        assert_eq!(artist, "Hasvalner"); // clean_folder_name capitalise le nom
        assert_eq!(album, "Some Album Title");

        // Test avec uploader vide
        let (artist, album) = parse_artist_album("Some Album Title", "");
        assert_eq!(artist, "Unknown Artist");
        assert_eq!(album, "Some Album Title");
    }

    #[test]
    fn test_parse_artist_album_with_emoji_separator() {
        // Bug fix: "Dark Chord - Black Crown Crows 👑 When We Two Parted" should parse correctly
        // The channel name "Dark Chord" should be ignored, emoji should be the separator
        let (artist, album) =
            parse_artist_album("Black Crown Crows 👑 When We Two Parted", "Dark Chord");
        assert_eq!(artist, "Black Crown Crows");
        assert_eq!(album, "When We Two Parted");

        // Test with other emojis
        let (artist, album) = parse_artist_album("Some Artist ⭐ Some Album", "Channel");
        assert_eq!(artist, "Some Artist");
        assert_eq!(album, "Some Album");
    }

    #[test]
    fn test_capitalize_words() {
        assert_eq!(capitalize_words("HELLO WORLD"), "Hello World");
        assert_eq!(capitalize_words("hello world"), "Hello World");
        assert_eq!(
            capitalize_words("VOICES BENEATH THE RAIN"),
            "Voices Beneath The Rain"
        );
        assert_eq!(capitalize_words("mixed CASE words"), "Mixed Case Words");
    }
}
