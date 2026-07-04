//! Regression tests for `Config::validate()`.
//!
//! These cover values that a hand-edited `config.toml` can produce but that
//! the interactive wizard would normally reject at input time.

use youtube_chapter_splitter::config::Config;

fn valid_config() -> Config {
    Config::default()
}

#[test]
fn test_validate_accepts_default_config() {
    assert!(valid_config().validate().is_ok());
}

#[test]
fn test_validate_rejects_arbitrary_audio_quality() {
    let config = Config {
        audio_quality: 256,
        ..valid_config()
    };
    assert!(config.validate().is_err());
}

#[test]
fn test_validate_rejects_zero_max_retries() {
    let config = Config {
        max_retries: 0,
        ..valid_config()
    };
    assert!(config.validate().is_err());
}

#[test]
fn test_validate_rejects_negative_refine_silence_window() {
    let config = Config {
        refine_silence_window: -5.0,
        ..valid_config()
    };
    assert!(config.validate().is_err());
}

#[test]
fn test_validate_rejects_zero_refine_min_silence() {
    let config = Config {
        refine_min_silence: 0.0,
        ..valid_config()
    };
    assert!(config.validate().is_err());
}

#[test]
fn test_validate_rejects_non_finite_refine_noise_db() {
    let config = Config {
        refine_noise_db: f64::NAN,
        ..valid_config()
    };
    assert!(config.validate().is_err());
}

#[test]
fn test_validate_rejects_empty_filename_format() {
    let config = Config {
        filename_format: "   ".to_string(),
        ..valid_config()
    };
    assert!(config.validate().is_err());
}

#[test]
fn test_validate_rejects_path_separator_in_filename_format() {
    let config = Config {
        filename_format: "%a/%n - %t".to_string(),
        ..valid_config()
    };
    assert!(config.validate().is_err());
}

#[test]
fn test_validate_rejects_path_separator_in_directory_format() {
    let config = Config {
        directory_format: "%a/%A".to_string(),
        ..valid_config()
    };
    assert!(config.validate().is_err());
}

#[test]
fn test_load_rejects_hand_edited_invalid_config() {
    let toml_str = r#"
        audio_quality = 999
    "#;
    let config: Config = toml::from_str(toml_str).unwrap();
    assert!(config.validate().is_err());
}
