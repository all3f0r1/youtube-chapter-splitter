use youtube_chapter_splitter::YtcsError;

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
        YtcsError::MissingTool("test".to_string()),
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
