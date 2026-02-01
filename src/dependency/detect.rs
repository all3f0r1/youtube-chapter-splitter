//! Platform and dependency detection
//!
//! This module provides functionality to detect the current platform
//! and check for the presence of required external dependencies.

use std::process::Command;

/// Platform detection result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux { distro: LinuxDistro },
    MacOS,
    Windows,
}

/// Linux distribution for package manager detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinuxDistro {
    Debian, // apt
    Fedora, // dnf
    Arch,   // pacman
    Other,  // Unknown
}

/// Installation method for the current platform
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallMethod {
    Apt,        // Debian/Ubuntu
    Dnf,        // Fedora
    Pacman,     // Arch
    Brew,       // macOS
    Winget,     // Windows
    Chocolatey, // Windows
    Scoop,      // Windows
    Pip,        // Python pip (yt-dlp only)
    Manual,     // User must install manually
}

/// Status of a dependency
#[derive(Debug, Clone)]
pub struct DependencyStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub install_method: Option<InstallMethod>,
}

impl Platform {
    /// Detect the current platform
    pub fn detect() -> Platform {
        #[cfg(target_os = "linux")]
        {
            Platform::Linux {
                distro: Self::detect_linux_distro(),
            }
        }
        #[cfg(target_os = "macos")]
        {
            Platform::MacOS
        }
        #[cfg(target_os = "windows")]
        {
            Platform::Windows
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            // Fallback for unsupported platforms
            Platform::Linux {
                distro: LinuxDistro::Other,
            }
        }
    }

    /// Detect Linux distribution by checking for package managers
    #[cfg(target_os = "linux")]
    fn detect_linux_distro() -> LinuxDistro {
        // Check for package managers in order
        if Self::command_exists("apt-get") || Self::command_exists("apt") {
            LinuxDistro::Debian
        } else if Self::command_exists("dnf") {
            LinuxDistro::Fedora
        } else if Self::command_exists("pacman") {
            LinuxDistro::Arch
        } else {
            LinuxDistro::Other
        }
    }

    /// Check if a command exists on the system
    fn command_exists(cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Get the install method for this platform
    pub fn install_method(&self) -> InstallMethod {
        match self {
            Platform::Linux { distro } => match distro {
                LinuxDistro::Debian => InstallMethod::Apt,
                LinuxDistro::Fedora => InstallMethod::Dnf,
                LinuxDistro::Arch => InstallMethod::Pacman,
                LinuxDistro::Other => InstallMethod::Pip,
            },
            Platform::MacOS => {
                if Self::command_exists("brew") {
                    InstallMethod::Brew
                } else {
                    InstallMethod::Manual
                }
            }
            Platform::Windows => {
                // Check for Windows package managers
                if Self::command_exists("winget") {
                    InstallMethod::Winget
                } else if Self::command_exists("choco") {
                    InstallMethod::Chocolatey
                } else if Self::command_exists("scoop") {
                    InstallMethod::Scoop
                } else {
                    InstallMethod::Pip
                }
            }
        }
    }
}

impl DependencyStatus {
    /// Check if a dependency is installed
    pub fn check(name: &str) -> Self {
        let installed = Self::is_installed(name);
        let version = if installed {
            Self::get_version(name)
        } else {
            None
        };

        Self {
            installed,
            version,
            install_method: if installed {
                None
            } else {
                Some(Platform::detect().install_method())
            },
        }
    }

    /// Check if a command is available
    fn is_installed(name: &str) -> bool {
        Command::new(name)
            .arg("--version")
            .output()
            .or_else(|_| Command::new(name).arg("-version").output())
            .or_else(|_| Command::new(name).arg("version").output())
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Get version of a dependency
    fn get_version(name: &str) -> Option<String> {
        Command::new(name)
            .arg("--version")
            .output()
            .or_else(|_| Command::new(name).arg("-version").output())
            .or_else(|_| Command::new(name).arg("version").output())
            .ok()
            .and_then(|output| {
                let stdout = String::from_utf8(output.stdout).ok()?;
                Some(stdout.lines().next()?.to_string())
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        let platform = Platform::detect();
        // Should not panic
        match platform {
            Platform::Linux { .. } | Platform::MacOS | Platform::Windows => {}
        }
    }

    #[test]
    fn test_install_method() {
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
