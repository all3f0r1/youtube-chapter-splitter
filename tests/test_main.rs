//! End-to-end tests that exercise the actual compiled `ytcs` binary via
//! `CARGO_BIN_EXE_ytcs`, instead of re-implementing pieces of `main.rs`
//! privately inside the test (which can stay green while the real function
//! regresses). No network access or external tools (yt-dlp/ffmpeg) required:
//! every scenario here fails (or succeeds) before any of that is invoked.

use std::path::PathBuf;
use std::process::Command;

fn ytcs_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ytcs"))
}

#[test]
fn test_binary_help_lists_documented_flags() {
    let output = ytcs_command().arg("--help").output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("YouTube Chapter Splitter"));
    assert!(stdout.contains("--skip-download"));
    assert!(stdout.contains("--non-interactive"));
    assert!(stdout.contains("--dry-run"));
}

#[test]
fn test_binary_no_arguments_prints_usage_and_fails() {
    let output = ytcs_command().output().unwrap();
    assert!(!output.status.success());

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined.contains("Usage"));
}

#[test]
fn test_binary_config_show_prints_settings() {
    let output = ytcs_command().args(["config", "--show"]).output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Configuration file:"));
    assert!(stdout.contains("audio_quality"));
    assert!(stdout.contains("filename_format"));
}

#[test]
fn test_binary_rejects_non_youtube_url_with_generic_error() {
    // Fails at URL parsing, well before any network/tool access, so this is
    // deterministic. A plain parse failure (not a missing-input situation)
    // must exit 1, not the non-interactive-specific exit code 2.
    let output = ytcs_command()
        .args(["not-a-youtube-url", "--non-interactive"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Invalid YouTube URL") || stderr.contains("Unable to extract"));
}

/// Points `dirs::config_dir()` (and therefore `Config::config_path()`) at an
/// isolated, per-test directory via `XDG_CONFIG_HOME`, so the test can seed a
/// `config.toml` without touching the developer's real `~/.config/ytcs`.
/// Linux-only: `dirs` resolves the config directory differently on macOS
/// (`~/Library/Application Support`, ignores `XDG_CONFIG_HOME`) and Windows
/// (`%APPDATA%`), which would need separate seeding logic to exercise safely.
#[cfg(target_os = "linux")]
fn isolated_config_home(unique_name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ytcs_test_{}_{}_{:?}",
        unique_name,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(dir.join("ytcs")).unwrap();
    dir
}

#[cfg(target_os = "linux")]
#[test]
fn test_binary_non_interactive_playlist_ask_exits_with_code_2() {
    let config_home = isolated_config_home("playlist_ask");
    std::fs::write(
        config_home.join("ytcs/config.toml"),
        "playlist_behavior = \"ask\"\n",
    )
    .unwrap();

    // Resolving whether to expand a playlist happens before any network call
    // (it only inspects the URL string), so this deterministically reaches
    // the non-interactive gate without needing yt-dlp or connectivity.
    let output = ytcs_command()
        .args([
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=PLxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "--non-interactive",
        ])
        .env("XDG_CONFIG_HOME", &config_home)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("non-interactive") || stderr.contains("playlist_behavior"));

    let _ = std::fs::remove_dir_all(&config_home);
}
