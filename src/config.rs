//! Module de gestion de la configuration persistante.
//!
//! Ce module gère la configuration de l'application stockée dans un fichier TOML.

use crate::error::{Result, YtcsError};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Configuration de l'application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Répertoire de téléchargement par défaut
    pub default_output_dir: Option<String>,
    
    /// Télécharger la pochette d'album
    pub download_cover: bool,
    
    /// Format du nom de fichier
    /// Placeholders disponibles:
    /// - %n: numéro de piste (01, 02, etc.)
    /// - %t: titre de la piste
    /// - %a: artiste
    /// - %A: album
    pub filename_format: String,
    
    /// Format du nom de répertoire
    /// Placeholders disponibles:
    /// - %a: artiste
    /// - %A: album
    pub directory_format: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_output_dir: None,
            download_cover: true,
            filename_format: "%n - %t".to_string(),
            directory_format: "%a - %A".to_string(),
        }
    }
}

impl Config {
    /// Obtenir le chemin du fichier de configuration
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| YtcsError::ConfigError("Could not find config directory".to_string()))?;
        
        let ytcs_config_dir = config_dir.join("ytcs");
        fs::create_dir_all(&ytcs_config_dir)?;
        
        Ok(ytcs_config_dir.join("config.toml"))
    }
    
    /// Charger la configuration depuis le fichier
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if !config_path.exists() {
            // Créer une configuration par défaut
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }
        
        let content = fs::read_to_string(&config_path)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| YtcsError::ConfigError(format!("Failed to parse config: {}", e)))?;
        
        Ok(config)
    }
    
    /// Sauvegarder la configuration dans le fichier
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let content = toml::to_string_pretty(self)
            .map_err(|e| YtcsError::ConfigError(format!("Failed to serialize config: {}", e)))?;
        
        fs::write(&config_path, content)?;
        Ok(())
    }
    
    /// Obtenir le répertoire de sortie par défaut
    pub fn get_output_dir(&self) -> PathBuf {
        if let Some(ref dir) = self.default_output_dir {
            PathBuf::from(shellexpand::tilde(dir).to_string())
        } else {
            // Fallback: ~/Music ou ~/
            if let Some(music_dir) = dirs::audio_dir() {
                music_dir
            } else {
                dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
            }
        }
    }
    
    /// Formater le nom de fichier selon le template
    pub fn format_filename(&self, track_number: usize, title: &str, artist: &str, album: &str) -> String {
        self.filename_format
            .replace("%n", &format!("{:02}", track_number))
            .replace("%t", title)
            .replace("%a", artist)
            .replace("%A", album)
    }
    
    /// Formater le nom de répertoire selon le template
    pub fn format_directory(&self, artist: &str, album: &str) -> String {
        self.directory_format
            .replace("%a", artist)
            .replace("%A", album)
    }
}

/// Afficher la configuration actuelle
pub fn show_config() -> Result<()> {
    let config = Config::load()?;
    let config_path = Config::config_path()?;
    
    println!("Configuration file: {}", config_path.display());
    println!();
    println!("Current settings:");
    println!("  default_output_dir = {:?}", config.default_output_dir.unwrap_or_else(|| "~/Music (default)".to_string()));
    println!("  download_cover     = {}", config.download_cover);
    println!("  filename_format    = \"{}\"", config.filename_format);
    println!("  directory_format   = \"{}\"", config.directory_format);
    println!();
    println!("Available placeholders:");
    println!("  Filename: %n (track number), %t (title), %a (artist), %A (album)");
    println!("  Directory: %a (artist), %A (album)");
    
    Ok(())
}

/// Définir une valeur de configuration
pub fn set_config(key: &str, value: &str) -> Result<()> {
    let mut config = Config::load()?;
    
    match key {
        "default_output_dir" => {
            config.default_output_dir = if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            };
            println!("✓ Set default_output_dir to: {}", value);
        }
        "download_cover" => {
            config.download_cover = value.parse::<bool>()
                .map_err(|_| YtcsError::ConfigError("Value must be 'true' or 'false'".to_string()))?;
            println!("✓ Set download_cover to: {}", config.download_cover);
        }
        "filename_format" => {
            config.filename_format = value.to_string();
            println!("✓ Set filename_format to: \"{}\"", value);
        }
        "directory_format" => {
            config.directory_format = value.to_string();
            println!("✓ Set directory_format to: \"{}\"", value);
        }
        _ => {
            return Err(YtcsError::ConfigError(format!("Unknown config key: {}", key)));
        }
    }
    
    config.save()?;
    println!("✓ Configuration saved to: {}", Config::config_path()?.display());
    
    Ok(())
}

/// Réinitialiser la configuration aux valeurs par défaut
pub fn reset_config() -> Result<()> {
    let config = Config::default();
    config.save()?;
    println!("✓ Configuration reset to defaults");
    println!("✓ Configuration saved to: {}", Config::config_path()?.display());
    Ok(())
}
