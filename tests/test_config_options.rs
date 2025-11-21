use youtube_chapter_splitter::config::Config;

#[test]
fn test_config_default_values() {
    let config = Config::default();

    assert_eq!(config.audio_quality, 192);
    assert_eq!(config.overwrite_existing, false);
    assert_eq!(config.max_retries, 3);
    assert_eq!(config.create_playlist, false);
    assert_eq!(config.download_cover, true);
}

#[test]
fn test_config_audio_quality() {
    let mut config = Config::default();

    config.audio_quality = 128;
    assert_eq!(config.audio_quality, 128);

    config.audio_quality = 192;
    assert_eq!(config.audio_quality, 192);
}

#[test]
fn test_config_overwrite_existing() {
    let mut config = Config::default();

    config.overwrite_existing = true;
    assert_eq!(config.overwrite_existing, true);

    config.overwrite_existing = false;
    assert_eq!(config.overwrite_existing, false);
}

#[test]
fn test_config_max_retries() {
    let mut config = Config::default();

    config.max_retries = 5;
    assert_eq!(config.max_retries, 5);

    config.max_retries = 0;
    assert_eq!(config.max_retries, 0);
}

#[test]
fn test_config_create_playlist() {
    let mut config = Config::default();

    config.create_playlist = true;
    assert_eq!(config.create_playlist, true);

    config.create_playlist = false;
    assert_eq!(config.create_playlist, false);
}

#[test]
fn test_config_format_filename() {
    let config = Config::default();

    let result = config.format_filename(1, "Oblivion Gate", "Marigold", "Oblivion Gate");
    assert_eq!(result, "01 - Oblivion Gate");
}

#[test]
fn test_config_format_filename_custom() {
    let mut config = Config::default();
    config.filename_format = "%a - %n - %t".to_string();

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
    let mut config = Config::default();
    config.directory_format = "%a/%A".to_string();

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
    "#;

    let config: Config = toml::from_str(toml_str).unwrap();

    assert_eq!(config.audio_quality, 128);
    assert_eq!(config.overwrite_existing, true);
    assert_eq!(config.max_retries, 5);
    assert_eq!(config.create_playlist, true);
}
