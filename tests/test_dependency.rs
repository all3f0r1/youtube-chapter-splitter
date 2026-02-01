//! Tests for dependency detection and installation

use youtube_chapter_splitter::dependency::{
    DependencyState, DependencyStatus, InstallMethod, Platform,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_state_all_present() {
        let state = DependencyState::check_all();
        // We don't assert specific results since yt-dlp/ffmpeg may not be installed in CI
        // Just verify the check doesn't panic
        let _missing = state.missing();
    }

    #[test]
    fn test_dependency_state_missing() {
        let state = DependencyState {
            ytdlp: DependencyStatus {
                installed: false,
                version: None,
                install_method: None,
            },
            ffmpeg: DependencyStatus {
                installed: false,
                version: None,
                install_method: None,
            },
        };

        assert!(!state.all_present());
        assert_eq!(state.missing().len(), 2);
    }

    #[test]
    fn test_platform_detection() {
        let platform = Platform::detect();
        // Should not panic and return a valid platform
        match platform {
            Platform::Linux { .. } | Platform::MacOS | Platform::Windows => {}
        }
    }

    #[test]
    fn test_install_method_for_platform() {
        let platform = Platform::detect();
        let method = platform.install_method();
        // Should return a valid install method
        match method {
            InstallMethod::Apt
            | InstallMethod::Dnf
            | InstallMethod::Pacman
            | InstallMethod::Brew
            | InstallMethod::Winget
            | InstallMethod::Chocolatey
            | InstallMethod::Scoop
            | InstallMethod::Pip
            | InstallMethod::Manual => {}
        }
    }
}
