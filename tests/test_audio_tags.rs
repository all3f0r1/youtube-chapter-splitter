/// Test pour vérifier que les tags ID3 sont correctement préservés
/// lors de l'ajout de la pochette d'album avec lofty
#[cfg(test)]
mod audio_tags_tests {
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    /// Crée un fichier MP3 de test avec des métadonnées
    fn create_test_mp3_with_metadata(
        path: &std::path::Path,
        title: &str,
        artist: &str,
        album: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Créer un fichier audio silencieux de 1 seconde
        let output = Command::new("ffmpeg")
            .args([
                "-f",
                "lavfi",
                "-i",
                "anullsrc=r=44100:cl=mono",
                "-t",
                "1",
                "-c:a",
                "libmp3lame",
                "-q:a",
                "2",
                "-metadata",
                &format!("title={}", title),
                "-metadata",
                &format!("artist={}", artist),
                "-metadata",
                &format!("album={}", album),
                "-y",
                path.to_str().unwrap(),
            ])
            .output()?;

        if !output.status.success() {
            return Err(format!(
                "Failed to create test MP3: {}",
                String::from_utf8_lossy(&output.stderr)
            )
            .into());
        }

        Ok(())
    }

    /// Vérifie les métadonnées d'un fichier MP3
    fn verify_metadata(
        path: &std::path::Path,
    ) -> Result<(String, String, String), Box<dyn std::error::Error>> {
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "quiet",
                "-print_format",
                "json",
                "-show_format",
                path.to_str().unwrap(),
            ])
            .output()?;

        if !output.status.success() {
            return Err("Failed to read metadata".into());
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let data: serde_json::Value = serde_json::from_str(&json_str)?;

        let title = data["format"]["tags"]["title"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let artist = data["format"]["tags"]["artist"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let album = data["format"]["tags"]["album"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok((title, artist, album))
    }

    #[test]
    #[ignore] // Ignorer par défaut car nécessite ffmpeg
    fn test_metadata_preservation_with_lofty() {
        // Créer un répertoire de test temporaire
        let test_dir = PathBuf::from("/tmp/ytcs_test");
        fs::create_dir_all(&test_dir).unwrap();

        let test_file = test_dir.join("test.mp3");
        let test_title = "Test Song";
        let test_artist = "Test Artist";
        let test_album = "Test Album";

        // Créer un MP3 avec des métadonnées
        create_test_mp3_with_metadata(&test_file, test_title, test_artist, test_album).unwrap();

        // Vérifier que les métadonnées initiales sont présentes
        let (title_before, artist_before, album_before) = verify_metadata(&test_file).unwrap();
        assert_eq!(title_before, test_title);
        assert_eq!(artist_before, test_artist);
        assert_eq!(album_before, test_album);

        // Créer une image de test (1x1 pixel JPEG)
        let cover_path = test_dir.join("cover.jpg");
        let jpeg_data = vec![
            0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00,
            0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xFF, 0xDB, 0x00, 0x43, 0x00, 0x08, 0x06, 0x06,
            0x07, 0x06, 0x05, 0x08, 0x07, 0x07, 0x07, 0x09, 0x09, 0x08, 0x0A, 0x0C, 0x14, 0x0D,
            0x0C, 0x0B, 0x0B, 0x0C, 0x19, 0x12, 0x13, 0x0F, 0x14, 0x1D, 0x1A, 0x1F, 0x1E, 0x1D,
            0x1A, 0x1C, 0x1C, 0x20, 0x24, 0x2E, 0x27, 0x20, 0x22, 0x2C, 0x23, 0x1C, 0x1C, 0x28,
            0x37, 0x29, 0x2C, 0x30, 0x31, 0x34, 0x34, 0x34, 0x1F, 0x27, 0x39, 0x3D, 0x38, 0x32,
            0x3C, 0x2E, 0x33, 0x34, 0x32, 0xFF, 0xC0, 0x00, 0x0B, 0x08, 0x00, 0x01, 0x00, 0x01,
            0x01, 0x01, 0x11, 0x00, 0xFF, 0xC4, 0x00, 0x14, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0xFF, 0xC4,
            0x00, 0x14, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0xDA, 0x00, 0x08, 0x01, 0x01, 0x00, 0x00,
            0x3F, 0x00, 0x7F, 0xFF, 0xD9,
        ];
        fs::write(&cover_path, jpeg_data).unwrap();

        // Simuler l'ajout du cover avec lofty
        // Note: On utilise directement le code de audio.rs
        use lofty::config::WriteOptions;
        use lofty::picture::{Picture, PictureType};
        use lofty::prelude::*;
        use lofty::probe::Probe;
        use std::io::Read;

        let mut cover_file = fs::File::open(&cover_path).unwrap();
        let mut cover_data = Vec::new();
        cover_file.read_to_end(&mut cover_data).unwrap();

        let mut tagged_file = Probe::open(&test_file)
            .unwrap()
            .guess_file_type()
            .unwrap()
            .read()
            .unwrap();

        let mut cover_reader = &cover_data[..];
        let mut picture = Picture::from_reader(&mut cover_reader).unwrap();
        picture.set_pic_type(PictureType::CoverFront);
        picture.set_description(Some("Album Cover".to_string()));

        let tag = match tagged_file.primary_tag_mut() {
            Some(primary_tag) => primary_tag,
            None => {
                let tag_type = tagged_file.primary_tag_type();
                tagged_file.insert_tag(lofty::tag::Tag::new(tag_type));
                tagged_file.primary_tag_mut().unwrap()
            }
        };

        tag.push_picture(picture);
        tagged_file
            .save_to_path(&test_file, WriteOptions::default())
            .unwrap();

        // Vérifier que les métadonnées sont toujours présentes après l'ajout du cover
        let (title_after, artist_after, album_after) = verify_metadata(&test_file).unwrap();

        println!(
            "Before: title='{}', artist='{}', album='{}'",
            title_before, artist_before, album_before
        );
        println!(
            "After:  title='{}', artist='{}', album='{}'",
            title_after, artist_after, album_after
        );

        assert_eq!(title_after, test_title, "Title should be preserved");
        assert_eq!(artist_after, test_artist, "Artist should be preserved");
        assert_eq!(album_after, test_album, "Album should be preserved");

        // Nettoyer
        fs::remove_dir_all(&test_dir).ok();
    }
}
