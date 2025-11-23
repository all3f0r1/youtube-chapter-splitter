use clap::{Parser, Subcommand};
use colored::Colorize;
use std::io::{self, Write};
use youtube_chapter_splitter::{audio, config, downloader, playlist, utils, Result};

#[derive(Parser)]
#[command(name = "ytcs")]
#[command(about = "YouTube Chapter Splitter - Download and split YouTube videos into MP3 tracks", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// YouTube video URL(s) (if no subcommand)
    #[arg(value_name = "URL", required_unless_present = "command")]
    urls: Vec<String>,

    /// Output directory (overrides config)
    #[arg(short, long)]
    output: Option<String>,

    /// Force artist name (overrides auto-detection)
    #[arg(short, long)]
    artist: Option<String>,

    /// Force album name (overrides auto-detection)
    #[arg(short = 'A', long)]
    album: Option<String>,

    /// Skip downloading cover art
    #[arg(long)]
    no_cover: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Show current configuration
    Config,

    /// Set a configuration value
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },

    /// Reset configuration to defaults
    Reset,
}

fn clean_url(url: &str) -> String {
    // Extract only the video ID, remove playlist and other parameters
    if let Some(id_start) = url.find("v=") {
        let id_end = url[id_start + 2..]
            .find(&['&', '#'][..])
            .map(|i| id_start + 2 + i)
            .unwrap_or(url.len());
        let video_id = &url[id_start + 2..id_end];
        format!("https://www.youtube.com/watch?v={}", video_id)
    } else {
        url.to_string()
    }
}

fn check_dependencies() -> Result<()> {
    let mut missing = Vec::new();

    if std::process::Command::new("yt-dlp")
        .arg("--version")
        .output()
        .is_err()
    {
        missing.push("yt-dlp");
    }

    if std::process::Command::new("ffmpeg")
        .arg("-version")
        .output()
        .is_err()
    {
        missing.push("ffmpeg");
    }

    if !missing.is_empty() {
        eprintln!(
            "{}",
            format!("⚠ Missing dependencies: {}", missing.join(", ")).yellow()
        );
        eprintln!();
        eprintln!(
            "{}",
            "Would you like to install the missing dependencies? (y/n)".bold()
        );
        eprintln!("Or install manually:");
        eprintln!("  Ubuntu/Debian: sudo apt install yt-dlp ffmpeg");
        eprintln!("  macOS: brew install yt-dlp ffmpeg");
        eprintln!("  Windows: winget install yt-dlp ffmpeg");
        std::process::exit(1);
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle subcommands
    if let Some(command) = cli.command {
        return match command {
            Commands::Config => config::show_config(),
            Commands::Set { key, value } => config::set_config(&key, &value),
            Commands::Reset => config::reset_config(),
        };
    }

    // Main download flow
    if cli.urls.is_empty() {
        eprintln!("{}", "Error: No URL provided".red().bold());
        std::process::exit(1);
    }

    check_dependencies()?;

    // Afficher l'en-tête TUI moderne
    youtube_chapter_splitter::ui::print_header();

    // Load config
    let config = config::Config::load()?;

    // Process each URL
    let total_urls = cli.urls.len();
    for (index, url) in cli.urls.iter().enumerate() {
        if total_urls > 1 {
            println!();
            println!("{}", format!("=== Processing URL {}/{} ===", index + 1, total_urls).cyan().bold());
            println!("{}", url.bright_blue());
            println!();
        }

        process_single_url(url, &cli, &config)?;
    }

    if total_urls > 1 {
        println!();
        println!("{}", format!("✓ All {} URL(s) processed successfully!", total_urls).green().bold());
    }

    Ok(())
}

fn process_single_url(url: &str, cli: &Cli, config: &config::Config) -> Result<()> {
    use config::PlaylistBehavior;

    // Check if URL contains a playlist
    if let Some(_playlist_id) = playlist::is_playlist_url(url) {
        println!("{}", "Playlist detected!".yellow().bold());
        println!();

        // Determine action based on config
        let should_download_playlist = match config.playlist_behavior {
            PlaylistBehavior::Ask => {
                // Ask user what to do
                print!(
                    "{}",
                    "Do you want to download (v)ideo only or (p)laylist? [v/p]: ".bold()
                );
                io::stdout().flush()?;

                let mut choice = String::new();
                io::stdin().read_line(&mut choice)?;
                let choice = choice.trim().to_lowercase();

                choice == "p" || choice == "playlist"
            }
            PlaylistBehavior::VideoOnly => {
                println!("{}", "Downloading video only (configured behavior)...".green());
                println!();
                false
            }
            PlaylistBehavior::PlaylistOnly => {
                println!("{}", "Downloading entire playlist (configured behavior)...".green());
                println!();
                true
            }
        };

        if should_download_playlist {
            return download_playlist(url, cli, config);
        } else {
            println!("{}", "Downloading video only...".green());
            println!();
        }
    }

    use youtube_chapter_splitter::ui::{self, Status, TrackProgress};

    let url = clean_url(url);
    println!("{}", "Fetching video information...".yellow());
    let video_info = downloader::get_video_info(&url)?;
    
    // Afficher les infos vidéo avec le nouveau TUI
    ui::print_video_info(
        &video_info.title,
        &utils::format_duration(video_info.duration),
        video_info.chapters.len()
    );

    // Determine output directory
    let output_dir = if let Some(ref output) = cli.output {
        std::path::PathBuf::from(shellexpand::tilde(output).to_string())
    } else {
        config.get_output_dir()
    };

    // Parse artist and album
    let (artist, album) = if let (Some(a), Some(al)) = (&cli.artist, &cli.album) {
        (utils::clean_folder_name(a), utils::clean_folder_name(al))
    } else {
        utils::parse_artist_album(&video_info.title)
    };

    // Create output directory using config format
    let dir_name = config.format_directory(&artist, &album);
    let output_dir = output_dir.join(&dir_name);
    std::fs::create_dir_all(&output_dir)?;

    // Download cover art and audio with TUI
    let download_cover = !cli.no_cover && config.download_cover;
    
    let mut cover_status = if download_cover { Status::InProgress } else { Status::Pending };
    let mut audio_status = Status::Pending;
    
    // Afficher l'état initial
    ui::print_download_section(cover_status, audio_status);
    
    // Download cover
    if download_cover {
        match downloader::download_thumbnail(&video_info.thumbnail_url, &output_dir) {
            Ok(_) => cover_status = Status::Success,
            Err(e) => {
                cover_status = Status::Failed;
                ui::print_warning(&format!("Could not download artwork: {}", e));
            }
        }
        // Mettre à jour l'affichage
        ui::clear_lines(5);
        ui::print_download_section(cover_status, audio_status);
    }
    
    // Download audio
    audio_status = Status::InProgress;
    ui::clear_lines(5);
    ui::print_download_section(cover_status, audio_status);
    
    let audio_file = match downloader::download_audio(&url, &output_dir.join("temp_audio.mp3")) {
        Ok(file) => {
            audio_status = Status::Success;
            ui::clear_lines(5);
            ui::print_download_section(cover_status, audio_status);
            file
        }
        Err(e) => {
            audio_status = Status::Failed;
            ui::clear_lines(5);
            ui::print_download_section(cover_status, audio_status);
            return Err(e);
        }
    };

    // Get chapters
    let chapters_to_use = if !video_info.chapters.is_empty() {
        println!("{}", "Using YouTube tracks".green());
        video_info.chapters
    } else {
        println!("{}", "No tracks found, detecting automatically...".yellow());
        audio::detect_silence_chapters(&audio_file, -30.0, 2.0)?
    };
    println!();

    let (final_chapters, final_artist, final_album) = (chapters_to_use, artist, album);

    // Split audio with metadata (using config format)
    let cover_path = if download_cover {
        Some(output_dir.join("cover.jpg"))
    } else {
        None
    };

    // Initialiser les pistes pour l'affichage TUI
    let mut tracks: Vec<TrackProgress> = final_chapters.iter().enumerate().map(|(i, ch)| {
        TrackProgress {
            number: i + 1,
            title: ch.title.clone(),
            status: Status::Pending,
            progress: 0,
        }
    }).collect();
    
    // Afficher l'état initial
    ui::print_track_section(&tracks);
    
    // Découper toutes les pistes (la fonction split_audio_by_chapters gère déjà la progression)
    // On va simuler la progression visuellement
    let output_files = audio::split_audio_by_chapters(
        &audio_file,
        &final_chapters,
        &output_dir,
        &final_artist,
        &final_album,
        cover_path.as_deref(),
        &config,
    )?;
    
    // Marquer toutes les pistes comme réussies
    for track in &mut tracks {
        track.status = Status::Success;
        track.progress = 100;
    }
    ui::clear_lines(tracks.len() + 3);
    ui::print_track_section(&tracks);
    
    // Clean up temporary audio file
    std::fs::remove_file(&audio_file).ok();
    
    // Message de succès
    ui::print_success(
        &format!("✓ All {} tracks created successfully!", output_files.len()),
        &output_dir.display().to_string()
    );

    Ok(())
}
// Fonction à ajouter à main.rs

fn download_playlist(url: &str, cli: &Cli, cfg: &config::Config) -> Result<()> {
    println!("{}", "Fetching playlist information...".yellow());

    let playlist_info = playlist::get_playlist_info(url)?;

    println!("{} {}", "Playlist:".bold(), playlist_info.title);
    println!("{} {}", "Videos:".bold(), playlist_info.videos.len());
    println!();

    // Display all videos
    for (i, video) in playlist_info.videos.iter().enumerate() {
        println!(
            "  {}. {} [{}]",
            i + 1,
            video.title,
            utils::format_duration(video.duration)
        );
    }
    println!();

    // Confirm
    print!(
        "{}",
        format!("Download {} videos? [y/n]: ", playlist_info.videos.len()).bold()
    );
    io::stdout().flush()?;

    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm)?;

    if !confirm.trim().to_lowercase().starts_with('y') {
        println!("{}", "Cancelled.".yellow());
        return Ok(());
    }

    println!();
    println!(
        "{}",
        format!(
            "Starting download of {} videos...",
            playlist_info.videos.len()
        )
        .green()
        .bold()
    );
    println!();

    // Determine output directory
    let output_dir = if let Some(ref output) = cli.output {
        std::path::PathBuf::from(shellexpand::tilde(output).to_string())
    } else {
        cfg.get_output_dir()
    };

    // Create playlist directory
    let playlist_dir = output_dir.join(utils::sanitize_title(&playlist_info.title));
    std::fs::create_dir_all(&playlist_dir)?;

    let mut successful = 0;
    let mut failed = 0;

    // Download each video
    for (i, video) in playlist_info.videos.iter().enumerate() {
        println!(
            "{}",
            format!("=== Video {}/{} ===", i + 1, playlist_info.videos.len())
                .bold()
                .cyan()
        );
        println!("{} {}", "Title:".bold(), video.title);
        println!();

        match download_single_video(&video.url, cli, cfg, &playlist_dir) {
            Ok(_) => {
                successful += 1;
                println!("{}", "✓ Video completed successfully!".green().bold());
            }
            Err(e) => {
                failed += 1;
                println!("{} {}", "✗ Video failed:".red().bold(), e);
            }
        }

        println!();
    }

    // Summary
    println!("{}", "=== Playlist Download Complete ===".bold().cyan());
    println!(
        "{} {}/{}",
        "Successful:".green(),
        successful,
        playlist_info.videos.len()
    );
    if failed > 0 {
        println!(
            "{} {}/{}",
            "Failed:".red(),
            failed,
            playlist_info.videos.len()
        );
    }
    println!("{} {}", "Directory:".bold(), playlist_dir.display());

    // Create M3U playlist if configured
    if cfg.create_playlist {
        create_m3u_playlist(&playlist_dir, &playlist_info.title)?;
    }

    Ok(())
}

fn download_single_video(
    url: &str,
    cli: &Cli,
    cfg: &config::Config,
    base_output_dir: &std::path::Path,
) -> Result<()> {
    let url = clean_url(url);

    let video_info = downloader::get_video_info(&url)?;

    // Parse artist and album
    let (artist, album) = if let (Some(a), Some(al)) = (&cli.artist, &cli.album) {
        (utils::clean_folder_name(a), utils::clean_folder_name(al))
    } else {
        utils::parse_artist_album(&video_info.title)
    };

    // Create output directory using config format
    let dir_name = cfg.format_directory(&artist, &album);
    let output_dir = base_output_dir.join(&dir_name);
    std::fs::create_dir_all(&output_dir)?;

    // Download cover art (unless --no-cover)
    let download_cover = !cli.no_cover && cfg.download_cover;
    if download_cover {
        match downloader::download_thumbnail(&video_info.thumbnail_url, &output_dir) {
            Ok(thumb_path) => println!("{} {}", "✓ Artwork saved:".green(), thumb_path.display()),
            Err(e) => println!("{} {}", "⚠ Could not download artwork:".yellow(), e),
        }
    }

    // Download audio
    let audio_file = downloader::download_audio(&url, &output_dir.join("temp_audio.mp3"))?;

    // Get chapters
    let chapters_to_use = if !video_info.chapters.is_empty() {
        video_info.chapters
    } else {
        audio::detect_silence_chapters(&audio_file, -30.0, 2.0)?
    };

    // Split audio with metadata
    let cover_path = if download_cover {
        Some(output_dir.join("cover.jpg"))
    } else {
        None
    };

    audio::split_audio_by_chapters(
        &audio_file,
        &chapters_to_use,
        &output_dir,
        &artist,
        &album,
        cover_path.as_deref(),
        cfg,
    )?;

    // Clean up temporary audio file
    std::fs::remove_file(&audio_file).ok();

    Ok(())
}

fn create_m3u_playlist(directory: &std::path::Path, playlist_name: &str) -> Result<()> {
    use std::fs::File;
    use std::io::Write;

    let m3u_path = directory.join(format!("{}.m3u", utils::sanitize_title(playlist_name)));
    let mut file = File::create(&m3u_path)?;

    writeln!(file, "#EXTM3U")?;
    writeln!(file, "#PLAYLIST:{}", playlist_name)?;

    // Find all MP3 files recursively
    let mut mp3_files: Vec<_> = std::fs::read_dir(directory)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("mp3"))
                .unwrap_or(false)
        })
        .collect();

    // Sort by filename
    mp3_files.sort_by_key(|entry| entry.file_name());

    for entry in mp3_files {
        let path = entry.path();
        let relative_path = path
            .strip_prefix(directory)
            .unwrap_or(&path)
            .to_string_lossy();
        writeln!(file, "{}", relative_path)?;
    }

    println!(
        "{} {}",
        "✓ Playlist file created:".green(),
        m3u_path.display()
    );

    Ok(())
}
