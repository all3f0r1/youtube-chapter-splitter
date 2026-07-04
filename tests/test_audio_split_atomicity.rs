//! Regression tests for `audio::split_audio_by_chapters`' upfront validation
//! and atomic-rename behavior (see CLAUDE.md / review: tracks must be
//! validated before any ffmpeg process runs, and a partially-encoded track
//! must never appear under its final filename).
//!
//! These only need a local `ffmpeg`/`ffprobe` (no network, no yt-dlp): the
//! fixture audio is synthesized with ffmpeg's `anullsrc` lavfi source.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use youtube_chapter_splitter::{AudioFormat, Chapter, audio};

fn ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Creates a `duration_secs`-long silent mono audio fixture (and a fresh,
/// empty output directory next to it) under a unique temp directory.
fn make_fixture(name: &str, duration_secs: f64) -> (PathBuf, PathBuf, PathBuf) {
    let root = std::env::temp_dir().join(format!("ytcs_split_atomicity_{}", name));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();

    let audio_file = root.join("source.wav");
    let status = Command::new("ffmpeg")
        .args([
            "-y",
            "-f",
            "lavfi",
            "-i",
            "anullsrc=r=8000:cl=mono",
            "-t",
            &duration_secs.to_string(),
            audio_file.to_str().unwrap(),
        ])
        .output()
        .expect("failed to run ffmpeg")
        .status;
    assert!(status.success(), "fixture generation must succeed");

    let output_dir = root.join("output");
    fs::create_dir_all(&output_dir).unwrap();

    (root, audio_file, output_dir)
}

fn cleanup(root: &Path) {
    let _ = fs::remove_dir_all(root);
}

#[test]
fn test_split_rejects_duplicate_filenames_from_template_before_any_ffmpeg_call() {
    if !ffmpeg_available() {
        eprintln!("Skipping: ffmpeg not available");
        return;
    }
    let (root, audio_file, output_dir) = make_fixture("template_collision", 4.0);

    // "%t" alone (no %n) means two chapters with the same sanitized title
    // collide on one output filename.
    let chapters = vec![
        Chapter::new("Intro".to_string(), 0.0, 2.0),
        Chapter::new("Intro".to_string(), 2.0, 4.0),
    ];

    let result = audio::split_audio_by_chapters(
        &audio_file,
        &chapters,
        &output_dir,
        "Artist",
        "Album",
        None,
        "%t",
        AudioFormat::Mp3,
        128,
        None,
        None,
        None,
        true,
        None,
    );

    assert!(
        result.is_err(),
        "duplicate output filenames must be rejected"
    );
    let entries: Vec<_> = fs::read_dir(&output_dir).unwrap().collect();
    assert!(
        entries.is_empty(),
        "no ffmpeg process should have run before the collision was caught"
    );

    cleanup(&root);
}

#[test]
fn test_split_fails_upfront_and_leaves_earlier_tracks_untouched_on_conflict() {
    if !ffmpeg_available() {
        eprintln!("Skipping: ffmpeg not available");
        return;
    }
    let (root, audio_file, output_dir) = make_fixture("existing_conflict", 6.0);

    let chapters = vec![
        Chapter::new("Intro".to_string(), 0.0, 3.0),
        Chapter::new("Outro".to_string(), 3.0, 6.0),
    ];

    // Simulate a stale file left by a previous, unrelated run: track 2's
    // final name already exists, and overwrite_existing is false.
    let stale_path = output_dir.join("02 - Outro.mp3");
    fs::write(&stale_path, b"stale-do-not-touch").unwrap();

    let result = audio::split_audio_by_chapters(
        &audio_file,
        &chapters,
        &output_dir,
        "Artist",
        "Album",
        None,
        "%n - %t",
        AudioFormat::Mp3,
        128,
        None,
        None,
        None,
        false, // overwrite_existing
        None,
    );

    assert!(
        result.is_err(),
        "existing target with overwrite off must fail"
    );
    assert!(
        !output_dir.join("01 - Intro.mp3").exists(),
        "track 1 must not be written: the conflict on track 2 must be caught \
         before any ffmpeg process runs, not discovered mid-batch"
    );
    assert_eq!(
        fs::read(&stale_path).unwrap(),
        b"stale-do-not-touch",
        "the pre-existing file must be left exactly as-is"
    );

    cleanup(&root);
}

#[test]
fn test_split_leaves_no_temp_files_behind_on_success() {
    if !ffmpeg_available() {
        eprintln!("Skipping: ffmpeg not available");
        return;
    }
    let (root, audio_file, output_dir) = make_fixture("clean_temp", 4.0);

    let chapters = vec![
        Chapter::new("Intro".to_string(), 0.0, 2.0),
        Chapter::new("Outro".to_string(), 2.0, 4.0),
    ];

    let output_files = audio::split_audio_by_chapters(
        &audio_file,
        &chapters,
        &output_dir,
        "Artist",
        "Album",
        None,
        "%n - %t",
        AudioFormat::Mp3,
        128,
        None,
        None,
        None,
        true,
        None,
    )
    .unwrap();

    assert_eq!(output_files.len(), 2);
    for f in &output_files {
        assert!(f.exists());
    }

    // No scratch `.ytcs-tmp-*` files left in the output directory.
    let leftovers: Vec<_> = fs::read_dir(&output_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().contains("ytcs-tmp"))
        .collect();
    assert!(
        leftovers.is_empty(),
        "temp files must be renamed away on success, found: {:?}",
        leftovers
    );

    cleanup(&root);
}
