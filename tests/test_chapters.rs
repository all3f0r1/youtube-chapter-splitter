use youtube_chapter_splitter::chapters::{Chapter, format_timestamp, parse_timestamp};

#[test]
fn test_chapter_creation() {
    let chapter = Chapter::new("Test Chapter".to_string(), 0.0, 120.0);
    assert_eq!(chapter.title, "Test Chapter");
    assert_eq!(chapter.start_time, 0.0);
    assert_eq!(chapter.end_time, 120.0);
}

#[test]
fn test_chapter_duration() {
    let chapter = Chapter::new("Test".to_string(), 10.0, 70.0);
    assert_eq!(chapter.duration(), 60.0);
}

#[test]
fn test_sanitize_title() {
    let chapter = Chapter::new("1 - Song Name".to_string(), 0.0, 100.0);
    assert_eq!(chapter.sanitize_title(), "Song Name");

    let chapter2 = Chapter::new("Track 5: Test/Song".to_string(), 0.0, 100.0);
    assert_eq!(chapter2.sanitize_title(), "Test_Song");
}

#[test]
fn test_parse_timestamp_seconds() {
    assert_eq!(parse_timestamp("42").unwrap(), 42.0);
}

#[test]
fn test_parse_timestamp_minutes_seconds() {
    assert_eq!(parse_timestamp("5:30").unwrap(), 330.0);
}

#[test]
fn test_parse_timestamp_hours_minutes_seconds() {
    assert_eq!(parse_timestamp("1:23:45").unwrap(), 5025.0);
}

#[test]
fn test_parse_timestamp_invalid() {
    assert!(parse_timestamp("invalid").is_err());
    assert!(parse_timestamp("1:2:3:4").is_err());
}

#[test]
fn test_format_timestamp_short() {
    assert_eq!(format_timestamp(90.0), "01:30");
}

#[test]
fn test_format_timestamp_long() {
    assert_eq!(format_timestamp(3661.0), "01:01:01");
}

#[test]
fn test_chapter_serialization() {
    let chapter = Chapter::new("Test".to_string(), 0.0, 100.0);
    let json = serde_json::to_string(&chapter).unwrap();
    let deserialized: Chapter = serde_json::from_str(&json).unwrap();

    assert_eq!(chapter.title, deserialized.title);
    assert_eq!(chapter.start_time, deserialized.start_time);
    assert_eq!(chapter.end_time, deserialized.end_time);
}
