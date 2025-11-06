use regex::Regex;

/// Nettoie et formate le nom de dossier selon les règles :
/// - Retire les [] et () avec leur contenu
/// - Remplace _ par - entre artiste et album
/// - Capitalise les mots
pub fn clean_folder_name(name: &str) -> String {
    // Retirer les [] et () avec leur contenu
    let re_brackets = Regex::new(r"\[.*?\]|\(.*?\)").unwrap();
    let cleaned = re_brackets.replace_all(name, "");
    
    // Remplacer les underscores par des tirets
    let with_dashes = cleaned.replace('_', "-");
    
    // Nettoyer les espaces multiples
    let re_spaces = Regex::new(r"\s+").unwrap();
    let normalized = re_spaces.replace_all(&with_dashes, " ");
    
    // Capitaliser chaque mot
    let capitalized = normalized
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + chars.as_str().to_lowercase().as_str()
                }
            }
        })
        .collect::<Vec<String>>()
        .join(" ");
    
    // Nettoyer les tirets et espaces en début/fin
    capitalized.trim().trim_matches('-').trim().to_string()
}

/// Formate une durée en secondes en format MM:SS ou HH:MM:SS
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

/// Formate une durée en secondes en format court (ex: "5m 43s")
pub fn format_duration_short(seconds: f64) -> String {
    let minutes = (seconds / 60.0).floor() as u32;
    let secs = (seconds % 60.0).floor() as u32;
    format!("{}m {:02}s", minutes, secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_folder_name() {
        assert_eq!(
            clean_folder_name("MARIGOLD - Oblivion Gate [Full Album] (70s Psychedelic Blues Acid Rock)"),
            "Marigold - Oblivion Gate"
        );
        
        assert_eq!(
            clean_folder_name("Artist_Name - Album_Title [2024]"),
            "Artist-Name - Album-Title"
        );
        
        assert_eq!(
            clean_folder_name("test_album (bonus tracks) [remastered]"),
            "Test-Album"
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
}
