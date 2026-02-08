use clap::Parser;
use colored::Colorize;
use youtube_chapter_splitter::{Result, audio, downloader, utils, yt_dlp_progress};

#[derive(Parser)]
#[command(name = "ytcs")]
#[command(about = "YouTube Chapter Splitter - Download and split YouTube videos into MP3 tracks", long_about = None)]
#[command(version)]
struct Cli {
    /// YouTube video URL
    url: String,

    /// Output directory (default: ~/Music)
    #[arg(short, long)]
    output: Option<String>,

    /// Force artist name (overrides auto-detection)
    #[arg(short, long)]
    artist: Option<String>,

    /// Force album name (overrides auto-detection)
    #[arg(short = 'A', long)]
    album: Option<String>,
}

fn clean_url(url: &str) -> String {
    // Extract only the video ID, remove playlist and other parameters
    if let Some(id_start) = url.find("v=") {
        let id_part = &url[id_start + 2..];
        if let Some(amp_pos) = id_part.find('&') {
            format!("https://www.youtube.com/watch?v={}", &id_part[..amp_pos])
        } else {
            format!("https://www.youtube.com/watch?v={}", id_part)
        }
    } else {
        url.to_string()
    }
}

fn get_default_music_dir() -> std::path::PathBuf {
    if let Some(music_dir) = dirs::audio_dir() {
        music_dir
    } else {
        // Fallback to home directory if audio_dir is not available
        dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("Music")
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Check system dependencies at startup
    if let Err(e) = downloader::check_dependencies() {
        eprintln!("{}", format!("⚠ {}", e).yellow());
        eprintln!();
        eprintln!(
            "{}",
            "Would you like to install the missing dependencies? (y/n)".bold()
        );

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();

        if input.trim().to_lowercase() == "y" {
            if e.to_string().contains("yt-dlp") {
                downloader::install_dependency("yt-dlp")?;
            }
            if e.to_string().contains("ffmpeg") {
                downloader::install_dependency("ffmpeg")?;
            }
            println!();
        } else {
            return Err(e);
        }
    }

    // Clean the URL
    let clean_url = clean_url(&cli.url);

    // Print header with borders
    println!("{}", "================================".cyan());
    println!("{}", "=== YouTube Chapter Splitter ===".bold().cyan());
    println!("{}", "================================".cyan());
    println!();

    // Get video information
    println!(
        "{}",
        ">> Fetching video information <<".bright_cyan().bold()
    );
    let video_info = downloader::get_video_info(&clean_url)?;

    // Show both raw title and cleaned title
    println!("Video Title: {}", video_info.title);
    let cleaned_title = utils::clean_folder_name(&video_info.title);
    println!("Cleaned Title: {}", cleaned_title);
    println!("Duration: {}", utils::format_duration(video_info.duration));
    println!("Tracks found: {}", video_info.chapters.len());
    println!();

    // Parse artist and album from title or use forced values
    let (artist, album) = if let (Some(a), Some(al)) = (&cli.artist, &cli.album) {
        // Clean user-forced values
        (utils::clean_folder_name(a), utils::clean_folder_name(al))
    } else {
        utils::parse_artist_album(&video_info.title)
    };

    // Create output directory with cleaned name
    let folder_name = format!("{} - {}", artist, album);
    let base_output = cli
        .output
        .as_ref()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(get_default_music_dir);
    let output_dir = base_output.join(&folder_name);
    std::fs::create_dir_all(&output_dir)?;

    // Download thumbnail
    println!("{}", ">> Downloading album artwork <<".bright_cyan().bold());
    match downloader::download_thumbnail(&clean_url, &output_dir) {
        Ok(thumb_path) => {
            println!("✓ Artwork saved:");
            println!("{}", thumb_path.display());
        }
        Err(e) => {
            println!("{} {}", "⚠ Could not download artwork:".yellow(), e);
        }
    }
    println!();

    // Download audio
    let temp_audio = output_dir.join("temp_audio");
    println!("{}", ">> Downloading audio <<".bright_cyan().bold());
    let audio_file =
        yt_dlp_progress::download_audio_with_progress(&clean_url, &temp_audio, None, None, None)?;

    // Determine chapters to use
    let chapters_to_use = if !video_info.chapters.is_empty() {
        video_info.chapters
    } else {
        audio::detect_silence_chapters(&audio_file, -30.0, 2.0)?
    };

    // Split audio with metadata
    let cover_path = output_dir.join("cover.jpg");
    println!(
        "{}",
        format!(
            ">> Splitting audio into {} tracks <<",
            chapters_to_use.len()
        )
        .bright_cyan()
        .bold()
    );
    let _output_files = audio::split_audio_by_chapters(
        &audio_file,
        &chapters_to_use,
        &output_dir,
        &artist,
        &album,
        if cover_path.exists() {
            Some(&cover_path)
        } else {
            None
        },
    )?;

    // Clean up temporary file
    std::fs::remove_file(&audio_file).ok();

    println!();
    println!("{}", "Tracks to create:".bold());
    for (i, chapter) in chapters_to_use.iter().enumerate() {
        println!(
            "✓ {}. {} [{}]",
            i + 1,
            chapter.display_title(),
            utils::format_duration_short(chapter.duration())
        );
    }

    println!();
    println!("{}", "✓ Processing completed successfully!".bold().green());
    println!("Directory:");
    println!("{}", output_dir.display());

    Ok(())
}
