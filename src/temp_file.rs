//! Module de gestion RAII des fichiers temporaires
//!
//! Ce module fournit une structure `TempFile` qui supprime automatiquement
//! les fichiers temporaires lorsqu'elle sort du scope, utilisant le pattern RAII.

use std::fs;
use std::path::{Path, PathBuf};

/// Fichier temporaire avec nettoyage automatique via RAII
///
/// Le fichier est automatiquement supprimé lorsque `TempFile` est dropped,
/// sauf si `keep()` a été appelé.
///
/// # Examples
///
/// ```no_run
/// use youtube_chapter_splitter::temp_file::TempFile;
/// use std::path::Path;
///
/// {
///     let temp = TempFile::new(Path::new("/tmp/audio.mp3"));
///     // Use the file...
///     // Le fichier sera automatiquement supprimé ici
/// }
/// ```
#[derive(Debug)]
pub struct TempFile {
    path: PathBuf,
    keep: bool,
}

impl TempFile {
    /// Crée un nouveau fichier temporaire
    ///
    /// # Arguments
    ///
    /// * `path` - Chemin du fichier temporaire
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use youtube_chapter_splitter::temp_file::TempFile;
    /// use std::path::Path;
    ///
    /// let temp = TempFile::new(Path::new("/tmp/audio.mp3"));
    /// ```
    pub fn new(path: &Path) -> Self {
        log::debug!("Created temp file: {:?}", path);
        Self {
            path: path.to_path_buf(),
            keep: false,
        }
    }

    /// Returns the path of the temporary file
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use youtube_chapter_splitter::temp_file::TempFile;
    /// use std::path::Path;
    ///
    /// let temp = TempFile::new(Path::new("/tmp/audio.mp3"));
    /// println!("Temp file path: {:?}", temp.path());
    /// ```
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Conserve le fichier après le drop
    ///
    /// Par défaut, le fichier est supprimé automatiquement.
    /// Appeler cette méthode empêche la suppression.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use youtube_chapter_splitter::temp_file::TempFile;
    /// use std::path::Path;
    ///
    /// let mut temp = TempFile::new(Path::new("/tmp/audio.mp3"));
    /// temp.keep(); // Le fichier ne sera pas supprimé
    /// ```
    pub fn keep(&mut self) {
        log::debug!("Keeping temp file: {:?}", self.path);
        self.keep = true;
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        if !self.keep && self.path.exists() {
            match fs::remove_file(&self.path) {
                Ok(_) => log::debug!("Removed temp file: {:?}", self.path),
                Err(e) => log::warn!("Failed to remove temp file {:?}: {}", self.path, e),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_temp_file_auto_cleanup() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_temp_file_auto_cleanup.txt");

        // Create the file
        File::create(&path).unwrap();
        assert!(path.exists());

        {
            let _temp = TempFile::new(&path);
            assert!(path.exists());
        } // temp is dropped here

        // File should be removed
        assert!(!path.exists());
    }

    #[test]
    fn test_temp_file_keep() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_temp_file_keep.txt");

        // Create the file
        File::create(&path).unwrap();
        assert!(path.exists());

        {
            let mut temp = TempFile::new(&path);
            temp.keep();
        } // temp is dropped here

        // File should still exist
        assert!(path.exists());

        // Clean up
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn test_temp_file_non_existent() {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_temp_file_non_existent.txt");

        // Make sure file doesn't exist
        if path.exists() {
            fs::remove_file(&path).unwrap();
        }

        {
            let _temp = TempFile::new(&path);
            // File doesn't exist, drop should not panic
        }
    }
}
