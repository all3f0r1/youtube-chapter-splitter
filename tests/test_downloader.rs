use youtube_chapter_splitter::downloader::extract_video_id;

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
