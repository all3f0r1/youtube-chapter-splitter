//! YouTube Chapter Splitter - Bibliothèque pour télécharger et découper des vidéos YouTube.
//!
//! Cette bibliothèque fournit des outils pour :
//! - Télécharger des vidéos YouTube et extraire l'audio en MP3
//! - Parser les chapitres depuis les métadonnées YouTube
//! - Découper l'audio en pistes individuelles basées sur les chapitres
//! - Ajouter des métadonnées ID3 complètes et des pochettes d'album
//!
//! # Exemple d'utilisation
//!
//! ```no_run
//! use youtube_chapter_splitter::{downloader, audio, config, Result};
//! use std::path::PathBuf;
//!
//! fn main() -> Result<()> {
//!     let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
//!     
//!     // Récupérer les informations de la vidéo
//!     let video_info = downloader::get_video_info(url)?;
//!     
//!     // Télécharger l'audio
//!     let output_path = PathBuf::from("temp_audio");
//!     let audio_file = downloader::download_audio(url, &output_path)?;
//!     
//!     // Découper par chapitres
//!     let output_dir = PathBuf::from("output");
//!     let cfg = config::Config::default();
//!     audio::split_audio_by_chapters(
//!         &audio_file,
//!         &video_info.chapters,
//!         &output_dir,
//!         "Artist Name",
//!         "Album Name",
//!         None,
//!         &cfg,
//!     )?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! # Modules
//!
//! - [`error`] - Gestion des erreurs personnalisées
//! - [`chapters`] - Structures et fonctions pour les chapitres
//! - [`downloader`] - Téléchargement de vidéos et métadonnées
//! - [`audio`] - Traitement et découpage audio
//! - [`utils`] - Fonctions utilitaires (formatage, nettoyage)

pub mod audio;
pub mod chapters;
pub mod config;
pub mod downloader;
pub mod error;
pub mod playlist;
pub mod ui;
pub mod utils;

pub use chapters::Chapter;
pub use downloader::VideoInfo;
pub use error::{Result, YtcsError};
