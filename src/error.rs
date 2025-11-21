//! Gestion des erreurs pour YouTube Chapter Splitter.
//!
//! Ce module définit les types d'erreurs personnalisés utilisés dans toute l'application.

use thiserror::Error;

/// Type d'erreur personnalisé pour YouTube Chapter Splitter.
///
/// Cette énumération regroupe tous les types d'erreurs possibles rencontrés
/// lors de l'exécution de l'application.
#[derive(Error, Debug)]
pub enum YtcsError {
    /// Erreur survenue lors du téléchargement d'une vidéo ou d'une miniature.
    #[error("Download error: {0}")]
    DownloadError(String),

    /// Erreur survenue lors du traitement audio (découpage, conversion, etc.).
    #[error("Audio processing error: {0}")]
    AudioError(String),

    /// Erreur survenue lors de l'analyse ou de la manipulation des chapitres.
    #[error("Chapter parsing error: {0}")]
    ChapterError(String),

    /// URL YouTube invalide ou mal formatée.
    #[error("Invalid YouTube URL: {0}")]
    InvalidUrl(String),

    /// Outil système requis manquant (yt-dlp, ffmpeg, etc.).
    #[error("Missing system tool: {0}")]
    MissingTool(String),

    /// Erreur d'entrée/sortie (fichiers, réseau, etc.).
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Erreur de parsing JSON (métadonnées de vidéo, chapitres, etc.).
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Erreur de compilation ou d'exécution d'expression régulière.
    #[error("Regex error: {0}")]
    RegexError(#[from] regex::Error),

    /// Erreur de configuration (fichier TOML, valeurs invalides, etc.).
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Erreur générique pour les cas non couverts par les autres variantes.
    #[error("Generic error: {0}")]
    Other(String),
}

/// Type alias pour `Result<T, YtcsError>`.
///
/// Simplifie la signature des fonctions en utilisant notre type d'erreur personnalisé.
pub type Result<T> = std::result::Result<T, YtcsError>;
