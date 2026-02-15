use clap::Parser;
use colored::Colorize;
use ui::MetadataSource;
use youtube_chapter_splitter::{Result, audio, downloader, ui, utils, yt_dlp_progress};

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

/// Progress callback for track splitting
fn track_progress_callback(track_number: usize, total_tracks: usize, title: &str, duration: &str) {
    ui::print_track_progress(track_number, total_tracks, title, duration);
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Print minimal header
    ui::print_header();

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

    // Get video information
    ui::print_section_header("Fetching video information");
    let video_info = downloader::get_video_info(&clean_url)?;

    // Determine initial metadata sources and values
    let (mut artist, mut album, mut artist_source, mut album_source) = if let (Some(a), Some(al)) =
        (&cli.artist, &cli.album)
    {
        // Both forced by user
        (
            utils::clean_folder_name(a),
            utils::clean_folder_name(al),
            MetadataSource::Forced,
            MetadataSource::Forced,
        )
    } else if let Some(a) = &cli.artist {
        // Only artist forced
        let ((_, parsed_album), _, _) = utils::parse_artist_album_with_source(&video_info.title);
        (
            utils::clean_folder_name(a),
            parsed_album,
            MetadataSource::Forced,
            MetadataSource::Detected,
        )
    } else if let Some(al) = &cli.album {
        // Only album forced
        let ((parsed_artist, _), _, _) = utils::parse_artist_album_with_source(&video_info.title);
        (
            parsed_artist,
            utils::clean_folder_name(al),
            MetadataSource::Detected,
            MetadataSource::Forced,
        )
    } else {
        // Nothing forced - parse from title
        let ((artist, album), artist_src, album_src) =
            utils::parse_artist_album_with_source(&video_info.title);
        (artist, album, artist_src, album_src)
    };

    // Prompt for metadata if unknown and not forced
    if artist == "Unknown Artist" && cli.artist.is_none() {
        let (input_artist, input_album) = ui::prompt_metadata(
            &video_info.title,
            &artist,
            &utils::clean_folder_name(&video_info.title),
        );
        artist = input_artist;
        album = input_album;
        artist_source = MetadataSource::Forced;
        album_source = MetadataSource::Forced;
    } else if album == utils::clean_folder_name(&video_info.title)
        && cli.album.is_none()
        && !utils::clean_folder_name(&video_info.title).contains(" - ")
    {
        // Only album is unknown
        let (input_artist, input_album) = ui::prompt_metadata(&video_info.title, &artist, &album);
        artist = input_artist;
        album = input_album;
        artist_source = MetadataSource::Forced;
        album_source = MetadataSource::Forced;
    }

    // Display video metadata in unified tree style
    ui::print_video_metadata_tree(
        &video_info.title,
        &utils::format_duration(video_info.duration),
        video_info.chapters.len(),
        &artist,
        &album,
        artist_source,
        album_source,
    );

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
    match downloader::download_thumbnail(&clean_url, &output_dir) {
        Ok(thumb_path) => {
            ui::print_artwork_section(thumb_path.to_str().unwrap_or("cover.jpg"));
        }
        Err(_) => {
            ui::print_artwork_section("");
        }
    }

    // Download audio
    ui::print_audio_section_header();
    let temp_audio = output_dir.join("temp_audio.mp3");
    let audio_file =
        yt_dlp_progress::download_audio_with_progress(&clean_url, &temp_audio, None, None, None)?;
    ui::print_audio_complete(audio_file.to_str().unwrap_or("audio.mp3"));

    // Determine chapters to use
    let chapters_to_use = if !video_info.chapters.is_empty() {
        video_info.chapters
    } else {
        audio::detect_silence_chapters(&audio_file, -30.0, 2.0)?
    };

    // Split audio with metadata (with progress callback)
    let cover_path = output_dir.join("cover.jpg");
    ui::print_splitting_section_header(chapters_to_use.len());

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
        Some(track_progress_callback),
    )?;

    ui::print_splitting_complete();

    // Clean up temporary file
    std::fs::remove_file(&audio_file).ok();

    // Display final result
    ui::print_final_result(&output_dir);

    Ok(())
}
