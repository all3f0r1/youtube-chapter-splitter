use clap::{Parser, Subcommand};
use colored::Colorize;
use ui::MetadataSource;
use youtube_chapter_splitter::{
    Result, YtcsError, audio, config, downloader, ui, utils, yt_dlp_progress,
    yt_dlp_progress::YtdlpDownloadOpts,
};

#[derive(Parser)]
#[command(name = "ytcs")]
#[command(about = "YouTube Chapter Splitter - Download and split YouTube videos into MP3 tracks", long_about = None)]
#[command(version)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// YouTube video URL
    url: Option<String>,

    /// Output directory (overrides config default_output_dir)
    #[arg(short, long)]
    output: Option<String>,

    /// Force artist name (overrides auto-detection)
    #[arg(short, long)]
    artist: Option<String>,

    /// Force album name (overrides auto-detection)
    #[arg(short = 'A', long)]
    album: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show or edit settings (interactive; use --show to print current file only)
    Config {
        /// Print current configuration and exit (no prompts)
        #[arg(long, short = 's')]
        show: bool,
    },
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

/// Progress callback for track splitting
fn track_progress_callback(track_number: usize, total_tracks: usize, title: &str, duration: &str) {
    ui::print_track_progress(track_number, total_tracks, title, duration);
}

fn handle_missing_dependencies(e: YtcsError, behavior: &config::AutoInstallBehavior) -> Result<()> {
    eprintln!("{}", format!("⚠ {}", e).yellow());
    eprintln!();

    match behavior {
        config::AutoInstallBehavior::Never => Err(e),
        config::AutoInstallBehavior::Always => {
            if e.to_string().contains("yt-dlp") {
                downloader::install_dependency("yt-dlp")?;
            }
            if e.to_string().contains("ffmpeg") {
                downloader::install_dependency("ffmpeg")?;
            }
            println!();
            downloader::check_dependencies()?;
            Ok(())
        }
        config::AutoInstallBehavior::Prompt => {
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
                downloader::check_dependencies()?;
                Ok(())
            } else {
                Err(e)
            }
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(Commands::Config { show }) = &cli.command {
        if *show {
            config::print_config_summary()?;
        } else {
            config::run_interactive_config_wizard()?;
        }
        return Ok(());
    }

    let url = cli.url.as_ref().ok_or_else(|| {
        YtcsError::ConfigError("Missing URL. Usage: ytcs <URL> | ytcs config [--show]".to_string())
    })?;

    let app_config = config::Config::load()?;

    ui::print_header();

    if let Err(e) = downloader::check_dependencies() {
        handle_missing_dependencies(e, &app_config.dependency_auto_install)?;
    }

    let clean_url = clean_url(url);

    ui::print_section_header("Fetching video information");
    let video_info = downloader::get_video_info(&clean_url)?;

    let (mut artist, mut album, mut artist_source, mut album_source) = if let (Some(a), Some(al)) =
        (&cli.artist, &cli.album)
    {
        (
            utils::clean_folder_name(a),
            utils::clean_folder_name(al),
            MetadataSource::Forced,
            MetadataSource::Forced,
        )
    } else if let Some(a) = &cli.artist {
        let ((_, parsed_album), _, _) = utils::parse_artist_album_with_source(&video_info.title);
        (
            utils::clean_folder_name(a),
            parsed_album,
            MetadataSource::Forced,
            MetadataSource::Detected,
        )
    } else if let Some(al) = &cli.album {
        let ((parsed_artist, _), _, _) = utils::parse_artist_album_with_source(&video_info.title);
        (
            parsed_artist,
            utils::clean_folder_name(al),
            MetadataSource::Detected,
            MetadataSource::Forced,
        )
    } else {
        let ((artist, album), artist_src, album_src) =
            utils::parse_artist_album_with_source(&video_info.title);
        (artist, album, artist_src, album_src)
    };

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
        let (input_artist, input_album) = ui::prompt_metadata(&video_info.title, &artist, &album);
        artist = input_artist;
        album = input_album;
        artist_source = MetadataSource::Forced;
        album_source = MetadataSource::Forced;
    }

    ui::print_video_metadata_tree(
        &video_info.title,
        &utils::format_duration(video_info.duration),
        video_info.chapters.len(),
        &artist,
        &album,
        artist_source,
        album_source,
    );

    let folder_name = app_config.format_directory(&artist, &album);
    let base_output = cli
        .output
        .as_ref()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| app_config.get_output_dir());
    let output_dir = base_output.join(&folder_name);
    std::fs::create_dir_all(&output_dir)?;

    if app_config.download_cover {
        match downloader::download_thumbnail(&clean_url, &output_dir) {
            Ok(thumb_path) => {
                ui::print_artwork_section(thumb_path.to_str().unwrap_or("cover.jpg"));
            }
            Err(_) => {
                ui::print_artwork_section("");
            }
        }
    } else {
        ui::print_artwork_section("");
    }

    ui::print_audio_section_header();
    let temp_audio = output_dir.join("temp_audio.mp3");
    let download_opts = YtdlpDownloadOpts::from(&app_config);
    let audio_file = yt_dlp_progress::download_audio_with_progress(
        &clean_url,
        &temp_audio,
        app_config.cookies_from_browser.as_deref(),
        download_opts,
        None,
        None,
    )?;
    ui::print_audio_complete(audio_file.to_str().unwrap_or("audio.mp3"));

    let chapters_to_use = if !video_info.chapters.is_empty() {
        video_info.chapters
    } else {
        audio::detect_silence_chapters(&audio_file, -30.0, 2.0)?
    };

    let cover_path = output_dir.join("cover.jpg");
    ui::print_splitting_section_header(chapters_to_use.len());

    let _output_files = audio::split_audio_by_chapters(
        &audio_file,
        &chapters_to_use,
        &output_dir,
        &artist,
        &album,
        if app_config.download_cover && cover_path.exists() {
            Some(&cover_path)
        } else {
            None
        },
        &app_config.filename_format,
        Some(track_progress_callback),
    )?;

    ui::print_splitting_complete();

    std::fs::remove_file(&audio_file).ok();

    ui::print_final_result(&output_dir);

    Ok(())
}
