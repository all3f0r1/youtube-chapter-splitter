use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;
use youtube_chapter_splitter::{
    audio, chapters, downloader, utils, Result, YtcsError,
};

#[derive(Parser)]
#[command(name = "ytcs")]
#[command(about = "YouTube Chapter Splitter - Download YouTube videos and split them into MP3 tracks", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Download a YouTube video and split it into tracks based on chapters
    Download {
        /// YouTube video URL
        #[arg(short, long)]
        url: String,

        /// Output directory (default: ./output)
        #[arg(short, long, default_value = "./output")]
        output: PathBuf,

        /// Use silence detection if no chapters are found
        #[arg(short, long, default_value = "true")]
        detect_silence: bool,

        /// Silence threshold in dB (for automatic detection)
        #[arg(long, default_value = "-30")]
        silence_threshold: f64,

        /// Minimum silence duration in seconds
        #[arg(long, default_value = "2.0")]
        min_silence_duration: f64,
    },

    /// Split an existing audio file based on timestamps
    Split {
        /// Path to the audio file to split
        #[arg(short, long)]
        input: PathBuf,

        /// Output directory
        #[arg(short, long, default_value = "./output")]
        output: PathBuf,

        /// JSON file containing chapters
        #[arg(short, long)]
        chapters: Option<PathBuf>,

        /// Use silence detection
        #[arg(short, long)]
        detect_silence: bool,

        /// Silence threshold in dB
        #[arg(long, default_value = "-30")]
        silence_threshold: f64,

        /// Minimum silence duration in seconds
        #[arg(long, default_value = "2.0")]
        min_silence_duration: f64,

        /// Album name for metadata
        #[arg(short, long, default_value = "Album")]
        album: String,
    },

    /// Display information about a YouTube video
    Info {
        /// YouTube video URL
        #[arg(short, long)]
        url: String,
    },

    /// Install missing dependencies
    Install {
        /// Tool to install (yt-dlp or ffmpeg)
        #[arg(short, long)]
        tool: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Check system dependencies at startup
    if let Err(e) = downloader::check_dependencies() {
        eprintln!("{}", format!("⚠ {}", e).yellow());
        eprintln!();
        eprintln!("{}", "Would you like to install the missing dependencies? (y/n)".bold());
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).ok();
        
        if input.trim().to_lowercase() == "y" {
            // Extract tool names from error message
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

    match cli.command {
        Commands::Download {
            url,
            output,
            detect_silence,
            silence_threshold,
            min_silence_duration,
        } => {
            println!("{}", "=== YouTube Chapter Splitter ===".bold().cyan());
            println!();

            // Get video information
            println!("{}", "Fetching video information...".yellow());
            let video_info = downloader::get_video_info(&url)?;

            println!("{} {}", "Title:".bold(), video_info.title);
            println!("{} {}", "Duration:".bold(), utils::format_duration(video_info.duration));
            println!("{} {}", "Tracks found:".bold(), video_info.chapters.len());
            println!();

            // Create output directory with cleaned name
            let clean_title = utils::clean_folder_name(&video_info.title);
            let output_dir = output.join(&clean_title);
            std::fs::create_dir_all(&output_dir)?;

            // Download audio
            let temp_audio = output_dir.join("temp_audio");
            println!("{}", "Downloading audio...".yellow());
            let audio_file = downloader::download_audio(&url, &temp_audio)?;
            println!("{} {}", "✓ Audio downloaded:".green(), audio_file.display());
            println!();

            // Determine chapters to use
            let chapters_to_use = if !video_info.chapters.is_empty() {
                println!("{}", "Using YouTube tracks".green());
                video_info.chapters
            } else if detect_silence {
                println!("{}", "No tracks found, detecting automatically...".yellow());
                audio::detect_silence_chapters(&audio_file, silence_threshold, min_silence_duration)?
            } else {
                return Err(YtcsError::ChapterError(
                    "No tracks found and silence detection disabled".to_string()
                ));
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

            // Split audio
            let output_files = audio::split_audio_by_chapters(
                &audio_file,
                &chapters_to_use,
                &output_dir,
                &clean_title,
            )?;

            // Clean up temporary file
            std::fs::remove_file(&audio_file).ok();

            println!();
            println!("{}", "✓ Processing completed successfully!".bold().green());
            println!("{} {}", "Files created:".bold(), output_files.len());
            println!("{} {}", "Directory:".bold(), output_dir.display());
        }

        Commands::Split {
            input,
            output,
            chapters: chapters_file,
            detect_silence,
            silence_threshold,
            min_silence_duration,
            album,
        } => {
            println!("{}", "=== Audio Splitter ===".bold().cyan());
            println!();

            if !input.exists() {
                return Err(YtcsError::IoError(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Input file not found",
                )));
            }

            let chapters_to_use = if let Some(chapters_path) = chapters_file {
                let json_content = std::fs::read_to_string(chapters_path)?;
                chapters::parse_chapters_from_json(&json_content)?
            } else if detect_silence {
                println!("{}", "Detecting tracks automatically...".yellow());
                audio::detect_silence_chapters(&input, silence_threshold, min_silence_duration)?
            } else {
                return Err(YtcsError::ChapterError(
                    "You must provide a chapters file or enable silence detection".to_string()
                ));
            };

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

            std::fs::create_dir_all(&output)?;
            let output_files = audio::split_audio_by_chapters(
                &input,
                &chapters_to_use,
                &output,
                &album,
            )?;

            println!();
            println!("{}", "✓ Processing completed successfully!".bold().green());
            println!("{} {}", "Files created:".bold(), output_files.len());
            println!("{} {}", "Directory:".bold(), output.display());
        }

        Commands::Info { url } => {
            println!("{}", "=== Video Information ===".bold().cyan());
            println!();

            let video_info = downloader::get_video_info(&url)?;

            println!("{} {}", "Title:".bold(), video_info.title);
            println!("{} {}", "ID:".bold(), video_info.video_id);
            println!("{} {}", "Duration:".bold(), utils::format_duration(video_info.duration));
            println!("{} {}", "Tracks:".bold(), video_info.chapters.len());

            if !video_info.chapters.is_empty() {
                println!();
                println!("{}", "Track list:".bold());
                for (i, chapter) in video_info.chapters.iter().enumerate() {
                    println!(
                        "  {}. {} [{}]",
                        i + 1,
                        chapter.title,
                        utils::format_duration_short(chapter.duration())
                    );
                }
            } else {
                println!();
                println!("{}", "No tracks found in this video.".yellow());
                println!("{}", "Use --detect-silence for automatic detection.".yellow());
            }
        }

        Commands::Install { tool } => {
            downloader::install_dependency(&tool)?;
        }
    }

    Ok(())
}
