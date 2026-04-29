use std::fs;
use youtube_chapter_splitter::downloader::{download_thumbnail, extract_video_id};

#[test]
fn test_extract_video_id_standard() {
    let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
    let id = extract_video_id(url).unwrap();
    assert_eq!(id, "dQw4w9WgXcQ");
}

#[test]
fn test_extract_video_id_with_params() {
    let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=PLtest&index=1";
    let id = extract_video_id(url).unwrap();
    assert_eq!(id, "dQw4w9WgXcQ");
}

#[test]
fn test_extract_video_id_short_url() {
    let url = "https://youtu.be/dQw4w9WgXcQ";
    let id = extract_video_id(url).unwrap();
    assert_eq!(id, "dQw4w9WgXcQ");
}

#[test]
fn test_extract_video_id_invalid() {
    let url = "https://example.com/not-youtube";
    assert!(extract_video_id(url).is_err());
}

#[test]
fn test_extract_video_id_malformed() {
    let url = "not a url at all";
    assert!(extract_video_id(url).is_err());
}

#[test]
fn test_video_id_length() {
    // YouTube video IDs are always 11 characters
    let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
    let id = extract_video_id(url).unwrap();
    assert_eq!(id.len(), 11);
}

/// Network: exercises `download_thumbnail` (CDN fallbacks, no extra yt-dlp call).
#[test]
#[ignore]
fn download_thumbnail_reachability() {
    let dir = std::env::temp_dir().join("ytcs_thumb_reach_test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let r = download_thumbnail("https://www.youtube.com/watch?v=Yl-1cFRQ7Es", &dir);
    assert!(r.is_ok(), "{:?}", r);
    assert!(r.unwrap().exists());
    let _ = fs::remove_dir_all(&dir);
}
