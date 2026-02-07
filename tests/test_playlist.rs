use youtube_chapter_splitter::downloader::extract_video_id;
use youtube_chapter_splitter::playlist::*;

#[test]
fn test_is_playlist_url_with_playlist() {
    let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf";
    // Always returns None (video only mode)
    assert!(is_playlist_url(url).is_none());
}

#[test]
fn test_is_playlist_url_without_playlist() {
    let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
    assert!(is_playlist_url(url).is_none());
}

#[test]
fn test_is_playlist_url_playlist_only() {
    let url = "https://www.youtube.com/playlist?list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf";
    // Always returns None (video only mode)
    assert!(is_playlist_url(url).is_none());
}

#[test]
fn test_extract_video_id_standard() {
    let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
    assert_eq!(extract_video_id(url).ok(), Some("dQw4w9WgXcQ".to_string()));
}

#[test]
fn test_extract_video_id_short() {
    let url = "https://youtu.be/dQw4w9WgXcQ";
    assert_eq!(extract_video_id(url).ok(), Some("dQw4w9WgXcQ".to_string()));
}

#[test]
fn test_extract_video_id_with_params() {
    let url =
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf&t=42s";
    assert_eq!(extract_video_id(url).ok(), Some("dQw4w9WgXcQ".to_string()));
}

#[test]
fn test_extract_video_id_invalid() {
    let url = "https://www.youtube.com/";
    assert!(extract_video_id(url).is_err());
}

#[test]
fn test_remove_playlist_param_with_list() {
    let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ&list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf";
    let clean = remove_playlist_param(url);
    assert_eq!(clean, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
}

#[test]
fn test_remove_playlist_param_without_list() {
    let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
    let clean = remove_playlist_param(url);
    assert_eq!(clean, url);
}

#[test]
fn test_remove_playlist_param_list_first_param() {
    let url = "https://www.youtube.com/watch?list=PLrAXtmErZgOeiKm4sgNOknGvNjby9efdf&v=dQw4w9WgXcQ";
    let clean = remove_playlist_param(url);
    assert_eq!(clean, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
}
