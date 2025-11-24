use youtube_chapter_splitter::config::Config;

#[test]
fn test_config_default_values() {
    let config = Config::default();

    assert_eq!(config.audio_quality, 192);
    assert!(!config.overwrite_existing);
    assert_eq!(config.max_retries, 3);
    assert!(!config.create_playlist);
    assert!(config.download_cover);
}

#[test]
fn test_config_audio_quality() {
    let config = Config {
        audio_quality: 128,
        ..Default::default()
    };
    assert_eq!(config.audio_quality, 128);

    let config2 = Config {
        audio_quality: 192,
        ..Default::default()
    };
    assert_eq!(config2.audio_quality, 192);
}

#[test]
fn test_config_overwrite_existing() {
    let config = Config {
        overwrite_existing: true,
        ..Default::default()
    };
    assert!(config.overwrite_existing);

    let config2 = Config::default();
    assert!(!config2.overwrite_existing);
}

#[test]
fn test_config_max_retries() {
    let config = Config {
        max_retries: 5,
        ..Default::default()
    };
    assert_eq!(config.max_retries, 5);

    let config2 = Config {
        max_retries: 0,
        ..Default::default()
    };
    assert_eq!(config2.max_retries, 0);
}

#[test]
fn test_config_create_playlist() {
    let config = Config {
        create_playlist: true,
        ..Default::default()
    };
    assert!(config.create_playlist);

    let config2 = Config::default();
    assert!(!config2.create_playlist);
}

#[test]
fn test_config_format_filename() {
    let config = Config::default();

    let result = config.format_filename(1, "Oblivion Gate", "Marigold", "Oblivion Gate");
    assert_eq!(result, "01 - Oblivion Gate");
}

#[test]
fn test_config_format_filename_custom() {
    let config = Config {
        filename_format: "%a - %n - %t".to_string(),
        ..Default::default()
    };

    let result = config.format_filename(5, "Eternal Pyre", "Marigold", "Oblivion Gate");
    assert_eq!(result, "Marigold - 05 - Eternal Pyre");
}

#[test]
fn test_config_format_directory() {
    let config = Config::default();

    let result = config.format_directory("Marigold", "Oblivion Gate");
    assert_eq!(result, "Marigold - Oblivion Gate");
}

#[test]
fn test_config_format_directory_custom() {
    let config = Config {
        directory_format: "%a/%A".to_string(),
        ..Default::default()
    };

    let result = config.format_directory("Marigold", "Oblivion Gate");
    assert_eq!(result, "Marigold/Oblivion Gate");
}

#[test]
fn test_config_serialization() {
    let config = Config::default();

    let toml_str = toml::to_string(&config).unwrap();
    assert!(toml_str.contains("audio_quality"));
    assert!(toml_str.contains("overwrite_existing"));
    assert!(toml_str.contains("max_retries"));
    assert!(toml_str.contains("create_playlist"));
}

#[test]
fn test_config_deserialization() {
    let toml_str = r#"
        download_cover = true
        filename_format = "%n - %t"
        directory_format = "%a - %A"
        audio_quality = 128
        overwrite_existing = true
        max_retries = 5
        create_playlist = true
        playlist_behavior = "ask"
    "#;

    let config: Config = toml::from_str(toml_str).unwrap();

    assert_eq!(config.audio_quality, 128);
    assert!(config.overwrite_existing);
    assert_eq!(config.max_retries, 5);
    assert!(config.create_playlist);
}
