use clap::{Parser, Subcommand};
use colored::Colorize;
use youtube_chapter_splitter::{audio, config, downloader, utils, Result};

#[derive(Parser)]
#[command(name = "ytcs")]
#[command(about = "YouTube Chapter Splitter - Download and split YouTube videos into MP3 tracks", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// YouTube video URL (if no subcommand)
    #[arg(value_name = "URL", required_unless_present = "command")]
    url: Option<String>,

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

    if !std::process::Command::new("yt-dlp")
        .arg("--version")
        .output()
        .is_ok()
    {
        missing.push("yt-dlp");
    }

    if !std::process::Command::new("ffmpeg")
        .arg("-version")
        .output()
        .is_ok()
    {
        missing.push("ffmpeg");
    }

    if !missing.is_empty() {
        eprintln!("{}", format!("⚠ Missing dependencies: {}", missing.join(", ")).yellow());
        eprintln!();
        eprintln!("{}", "Would you like to install the missing dependencies? (y/n)".bold());
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
    let url = cli.url.expect("URL is required");
    
    check_dependencies()?;

    println!("{}", "=== YouTube Chapter Splitter ===".bold().cyan());
    println!();

    let url = clean_url(&url);
    println!("{}", "Fetching video information...".yellow());
    let video_info = downloader::get_video_info(&url)?;
    println!("{} {}", "Title:".bold(), video_info.title);
    println!("{} {}", "Duration:".bold(), utils::format_duration(video_info.duration));
    println!("{} {}", "Tracks found:".bold(), video_info.chapters.len());
    println!();

    // Load config
    let cfg = config::Config::load()?;
    
    // Determine output directory
    let output_dir = if let Some(ref output) = cli.output {
        std::path::PathBuf::from(shellexpand::tilde(output).to_string())
    } else {
        cfg.get_output_dir()
    };

    // Parse artist and album
    let (artist, album) = if let (Some(a), Some(al)) = (&cli.artist, &cli.album) {
        (utils::clean_folder_name(a), utils::clean_folder_name(al))
    } else {
        utils::parse_artist_album(&video_info.title)
    };

    // Create output directory using config format
    let dir_name = cfg.format_directory(&artist, &album);
    let output_dir = output_dir.join(&dir_name);
    std::fs::create_dir_all(&output_dir)?;

    // Download cover art (unless --no-cover)
    let download_cover = !cli.no_cover && cfg.download_cover;
    if download_cover {
        println!("{}", "Downloading album artwork...".yellow());
        let thumb_path = output_dir.join("cover.jpg");
        match downloader::download_thumbnail(&video_info.thumbnail_url, &thumb_path) {
            Ok(_) => println!("{} {}", "✓ Artwork saved:".green(), thumb_path.display()),
            Err(e) => println!("{} {}", "⚠ Could not download artwork:".yellow(), e),
        }
        println!();
    }

    // Download audio
    println!("{}", "Downloading audio...".yellow());
    let audio_file = downloader::download_audio(&url, &output_dir.join("temp_audio.mp3"))?;
    println!("{} {}", "✓ Audio downloaded:".green(), audio_file.display());
    println!();

    // Get chapters
    let chapters_to_use = if !video_info.chapters.is_empty() {
        println!("{}", "Using YouTube tracks".green());
        video_info.chapters
    } else {
        println!("{}", "No tracks found, detecting automatically...".yellow());
        audio::detect_silence_chapters(&audio_file, -30.0, 2.0)?
    };

    // Display chapters
    println!();
    println!("{}", "Tracks to create:".bold());
    for (i, chapter) in chapters_to_use.iter().enumerate() {
        println!(
            "  {}. {} [{}]",
            i + 1,
            chapter.title,
            utils::format_duration_short(chapter.duration())
        );
    }
    println!();
    
    let (final_chapters, final_artist, final_album) = (chapters_to_use, artist, album);

    // Split audio with metadata (using config format)
    let cover_path = if download_cover {
        Some(output_dir.join("cover.jpg"))
    } else {
        None
    };
    
    let output_files = audio::split_audio_by_chapters(
        &audio_file,
        &final_chapters,
        &output_dir,
        &final_artist,
        &final_album,
        cover_path.as_deref(),
        &cfg,
    )?;

    // Clean up temporary audio file
    std::fs::remove_file(&audio_file).ok();

    println!();
    println!("{}", "✓ Processing completed successfully!".bold().green());
    println!("{} {}", "Files created:".bold(), output_files.len());
    println!("{} {}", "Directory:".bold(), output_dir.display());

    Ok(())
}
