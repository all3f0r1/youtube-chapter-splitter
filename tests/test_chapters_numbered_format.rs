// Tests for the numbered chapter format "N - Title (MM:SS)"

use youtube_chapter_splitter::chapters_from_description::parse_chapters_from_description;

#[test]
fn test_numbered_format_basic() {
    let description = r#"
1 - The Cornerstone of Some Dream (0:00)
2 - Architects of Inner Time (Part I) (4:24)
3 - The Ritual of the Octagonal Chamber (11:01)
"#;

    let duration = 900.0; // 15 minutes
    let chapters = parse_chapters_from_description(description, duration).unwrap();

    assert_eq!(chapters.len(), 3);
    assert_eq!(chapters[0].title, "The Cornerstone of Some Dream");
    assert_eq!(chapters[0].start_time, 0.0);
    assert_eq!(chapters[1].title, "Architects of Inner Time (Part I)");
    assert_eq!(chapters[1].start_time, 4.0 * 60.0 + 24.0);
    assert_eq!(chapters[2].title, "The Ritual of the Octagonal Chamber");
    assert_eq!(chapters[2].start_time, 11.0 * 60.0 + 1.0);
}

#[test]
fn test_numbered_format_with_parentheses_in_title() {
    let description = r#"
1 - Colors at the Bottom of the Gesture (Instrumental) (0:00)
2 - Mirror Against the Firmament (Suite in Three Parts) (5:30)
3 - Architects of Inner Time (Part II: The Awakening) (10:15)
"#;

    let duration = 900.0;
    let chapters = parse_chapters_from_description(description, duration).unwrap();

    assert_eq!(chapters.len(), 3);
    assert_eq!(
        chapters[0].title,
        "Colors at the Bottom of the Gesture (Instrumental)"
    );
    assert_eq!(
        chapters[1].title,
        "Mirror Against the Firmament (Suite in Three Parts)"
    );
    assert_eq!(
        chapters[2].title,
        "Architects of Inner Time (Part II_ The Awakening)"
    ); // Note: colons are replaced with underscore
}

#[test]
fn test_numbered_format_mixed_with_standard() {
    // The parser should detect the numbered format even if there are other lines
    let description = r#"
Album: Test Album
Artist: Test Artist

1 - Track One (0:00)
2 - Track Two (3:45)
3 - Track Three (7:30)

Released: 2024
"#;

    let duration = 600.0;
    let chapters = parse_chapters_from_description(description, duration).unwrap();

    assert_eq!(chapters.len(), 3);
    assert_eq!(chapters[0].title, "Track One");
    assert_eq!(chapters[1].title, "Track Two");
    assert_eq!(chapters[2].title, "Track Three");
}

#[test]
fn test_numbered_format_double_digit_numbers() {
    let description = r#"
8 - Track Eight (25:00)
9 - Track Nine (28:30)
10 - Track Ten (32:15)
11 - Track Eleven (36:00)
12 - Track Twelve (40:45)
"#;

    let duration = 2700.0; // 45 minutes
    let chapters = parse_chapters_from_description(description, duration).unwrap();

    assert_eq!(chapters.len(), 5);
    assert_eq!(chapters[0].title, "Track Eight");
    assert_eq!(chapters[0].start_time, 25.0 * 60.0);
    assert_eq!(chapters[4].title, "Track Twelve");
    assert_eq!(chapters[4].start_time, 40.0 * 60.0 + 45.0);
}

#[test]
fn test_numbered_format_with_hour_timestamps() {
    let description = r#"
1 - Long Track One (0:00:00)
2 - Long Track Two (1:15:30)
3 - Long Track Three (2:30:45)
"#;

    let duration = 10800.0; // 3 hours
    let chapters = parse_chapters_from_description(description, duration).unwrap();

    assert_eq!(chapters.len(), 3);
    assert_eq!(chapters[0].start_time, 0.0);
    assert_eq!(chapters[1].start_time, 1.0 * 3600.0 + 15.0 * 60.0 + 30.0);
    assert_eq!(chapters[2].start_time, 2.0 * 3600.0 + 30.0 * 60.0 + 45.0);
}

#[test]
fn test_standard_format_still_works() {
    // Ensure standard format still works
    let description = r#"
0:00 - Track One
3:45 - Track Two
7:30 - Track Three
"#;

    let duration = 600.0;
    let chapters = parse_chapters_from_description(description, duration).unwrap();

    assert_eq!(chapters.len(), 3);
    assert_eq!(chapters[0].title, "Track One");
    assert_eq!(chapters[1].title, "Track Two");
    assert_eq!(chapters[2].title, "Track Three");
}

#[test]
fn test_numbered_format_sanitization() {
    // Verify that special characters are properly sanitized
    let description = r#"
1 - Track: With Colon (0:00)
2 - Track/With/Slash (3:00)
3 - Track\With\Backslash (6:00)
4 - Track|With|Pipe (9:00)
"#;

    let duration = 720.0;
    let chapters = parse_chapters_from_description(description, duration).unwrap();

    assert_eq!(chapters.len(), 4);
    // Colons, slashes, backslashes, and pipes should be replaced
    assert_eq!(chapters[0].title, "Track_ With Colon");
    assert_eq!(chapters[1].title, "Track_With_Slash");
    assert_eq!(chapters[2].title, "Track_With_Backslash");
    assert_eq!(chapters[3].title, "Track_With_Pipe");
}
