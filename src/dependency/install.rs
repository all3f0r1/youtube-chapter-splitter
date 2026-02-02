//! Dependency installation
//!
//! This module provides functionality to install missing dependencies
//! with platform-specific commands.

use colored::Colorize;
use std::io::{self, Write};
use std::process::Command;

use crate::error::Result;
use crate::error::YtcsError;

use super::detect::LinuxDistro;
use super::{InstallMethod, Platform};

/// Dependency installer
pub struct DependencyInstaller {
    platform: Platform,
}

impl DependencyInstaller {
    /// Create a new installer for the current platform
    pub fn new() -> Self {
        Self {
            platform: Platform::detect(),
        }
    }

    /// Install a dependency by name
    pub fn install(&self, name: &str) -> Result<()> {
        use super::detect::DependencyStatus;

        // Check if already installed before attempting installation
        let status = DependencyStatus::check(name);
        if status.installed {
            let version_msg = if let Some(v) = &status.version {
                format!(" (version {})", v)
            } else {
                String::new()
            };
            eprintln!(
                "{}",
                format!("✓ {} is already installed{}", name, version_msg).green()
            );
            return Ok(());
        }

        let method = self.platform.install_method();
        let command = self.get_install_command(name, method)?;

        eprintln!("{}", format!("Installing {}...", name).yellow());
        eprintln!("{}", format!("Running: {}", command.join(" ")).dimmed());

        let status = Command::new(&command[0])
            .args(&command[1..])
            .spawn()?
            .wait()?;

        // Verify installation was successful
        let post_install_status = DependencyStatus::check(name);
        if post_install_status.installed {
            let version_msg = if let Some(v) = &post_install_status.version {
                format!(" (version {})", v)
            } else {
                String::new()
            };
            eprintln!(
                "{}",
                format!("✓ {} installed successfully{}", name, version_msg).green()
            );
            Ok(())
        } else if status.success() {
            // Package manager reported success but command not found
            // This can happen with PATH issues - notify the user
            eprintln!(
                "{}",
                format!(
                    "⚠ {} was installed but may not be in your PATH. You may need to restart your terminal or log out and back in.",
                    name
                ).yellow()
            );
            Ok(())
        } else {
            Err(YtcsError::InstallError(format!(
                "Failed to install {}. Command exited with status: {}",
                name, status
            )))
        }
    }

    /// Check if user wants to install dependencies
    pub fn prompt_install(missing: &[&str]) -> Result<bool> {
        eprintln!(
            "{}",
            format!("⚠ Missing dependencies: {}", missing.join(", ")).yellow()
        );
        eprintln!();
        eprintln!("{}", "Install dependencies automatically? [Y/n]".bold());
        eprintln!();
        eprintln!("Installation will use your system's package manager:");
        eprintln!("  • Linux: apt, dnf, or pacman (auto-detected)");
        eprintln!("  • macOS: Homebrew");
        eprintln!("  • Windows: winget or pip");
        eprintln!();
        eprint!("Or install manually. Install? ");

        io::stdout().flush()?;

        let mut response = String::new();
        io::stdin().read_line(&mut response)?;

        let response = response.trim().to_lowercase();
        Ok(response.is_empty() || response == "y" || response == "yes")
    }

    /// Get the install command for a dependency
    fn get_install_command(&self, name: &str, method: InstallMethod) -> Result<Vec<String>> {
        Ok(match method {
            InstallMethod::Apt => {
                vec![
                    "sudo".to_string(),
                    "apt".to_string(),
                    "install".to_string(),
                    "-y".to_string(),
                    name.to_string(),
                ]
            }
            InstallMethod::Dnf => {
                vec![
                    "sudo".to_string(),
                    "dnf".to_string(),
                    "install".to_string(),
                    "-y".to_string(),
                    name.to_string(),
                ]
            }
            InstallMethod::Pacman => {
                vec![
                    "sudo".to_string(),
                    "pacman".to_string(),
                    "-S".to_string(),
                    "--noconfirm".to_string(),
                    name.to_string(),
                ]
            }
            InstallMethod::Brew => {
                vec!["brew".to_string(), "install".to_string(), name.to_string()]
            }
            InstallMethod::Winget => {
                vec![
                    "winget".to_string(),
                    "install".to_string(),
                    "--accept-package-agreements".to_string(),
                    "--accept-source-agreements".to_string(),
                    name.to_string(),
                ]
            }
            InstallMethod::Chocolatey => {
                vec![
                    "choco".to_string(),
                    "install".to_string(),
                    "-y".to_string(),
                    name.to_string(),
                ]
            }
            InstallMethod::Scoop => {
                vec!["scoop".to_string(), "install".to_string(), name.to_string()]
            }
            InstallMethod::Pip => {
                // For yt-dlp specifically
                vec![
                    "python".to_string(),
                    "-m".to_string(),
                    "pip".to_string(),
                    "install".to_string(),
                    "--upgrade".to_string(),
                    name.to_string(),
                ]
            }
            InstallMethod::Manual => {
                return Err(YtcsError::InstallError(format!(
                    "Manual installation required for {}. Please install {} using your system package manager.",
                    name, name
                )));
            }
        })
    }

    /// Get manual installation instructions for the current platform
    pub fn get_manual_instructions(&self) -> &'static str {
        match self.platform {
            Platform::Linux { distro } => match distro {
                LinuxDistro::Debian => "sudo apt install yt-dlp ffmpeg",
                LinuxDistro::Fedora => "sudo dnf install yt-dlp ffmpeg",
                LinuxDistro::Arch => "sudo pacman -S yt-dlp ffmpeg",
                LinuxDistro::Other => {
                    "# Install using your package manager\nsudo apt install yt-dlp ffmpeg  # Debian/Ubuntu\nsudo dnf install yt-dlp ffmpeg  # Fedora\nsudo pacman -S yt-dlp ffmpeg  # Arch"
                }
            },
            Platform::MacOS => "brew install yt-dlp ffmpeg",
            Platform::Windows => {
                "# Install via winget\nwinget install yt-dlp ffmpeg\n\n# Or via pip\npython -m pip install --upgrade yt-dlp\n# ffmpeg: download from https://ffmpeg.org/download.html"
            }
        }
    }
}

impl Default for DependencyInstaller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_installer_creation() {
        let installer = DependencyInstaller::new();
        // Should not panic
        let _ = installer.platform;
    }
}
