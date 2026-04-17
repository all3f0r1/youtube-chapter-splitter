use youtube_chapter_splitter::{MissingToolsError, YtcsError};

#[test]
fn test_error_display() {
    let err = YtcsError::DownloadError("Test error".to_string());
    assert_eq!(err.to_string(), "Download error: Test error");
}

#[test]
fn test_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let ytcs_err: YtcsError = io_err.into();
    assert!(matches!(ytcs_err, YtcsError::IoError(_)));
}

#[test]
fn test_error_from_json() {
    let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
    let ytcs_err: YtcsError = json_err.into();
    assert!(matches!(ytcs_err, YtcsError::JsonError(_)));
}

#[test]
fn test_error_types() {
    let errors = vec![
        YtcsError::DownloadError("test".to_string()),
        YtcsError::AudioError("test".to_string()),
        YtcsError::ChapterError("test".to_string()),
        YtcsError::InvalidUrl("test".to_string()),
        YtcsError::MissingTools(MissingToolsError {
            missing_ytdlp: true,
            missing_ffmpeg: true,
            missing_deno: true,
        }),
        YtcsError::Other("test".to_string()),
    ];

    for err in errors {
        assert!(!err.to_string().is_empty());
    }
}

#[test]
fn test_result_type() {
    fn returns_result() -> youtube_chapter_splitter::Result<i32> {
        Ok(42)
    }

    assert_eq!(returns_result().unwrap(), 42);
}

#[test]
fn test_result_error() {
    fn returns_error() -> youtube_chapter_splitter::Result<i32> {
        Err(YtcsError::Other("test error".to_string()))
    }

    assert!(returns_error().is_err());
}

#[test]
fn test_missing_tools_error_tools_to_install() {
    assert!(
        MissingToolsError {
            missing_ytdlp: true,
            missing_ffmpeg: false,
            missing_deno: false,
        }
        .tools_to_install()
        .contains(&"yt-dlp")
    );
    assert_eq!(
        MissingToolsError {
            missing_ytdlp: false,
            missing_ffmpeg: true,
            missing_deno: false,
        }
        .tools_to_install(),
        vec!["ffmpeg"]
    );
    assert_eq!(
        MissingToolsError {
            missing_ytdlp: true,
            missing_ffmpeg: true,
            missing_deno: false,
        }
        .tools_to_install(),
        vec!["yt-dlp", "ffmpeg"]
    );
    assert_eq!(
        MissingToolsError {
            missing_ytdlp: false,
            missing_ffmpeg: false,
            missing_deno: true,
        }
        .tools_to_install(),
        vec!["deno"]
    );
}
