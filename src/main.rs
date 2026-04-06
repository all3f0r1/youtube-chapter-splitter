use clap::{Parser, Subcommand};
use colored::Colorize;
use std::io::Write;
use ui::MetadataSource;
use youtube_chapter_splitter::{
    Result, YtcsError, audio, chapter_refinement, chapters_from_description, config, downloader,
    playlist, ui, utils, yt_dlp_progress, yt_dlp_progress::YtdlpDownloadOpts,
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

    /// Snap chapter cuts to silence (extra ffmpeg pass; combined with `refine_chapters` in config)
    #[arg(long)]
    refine_chapters: bool,
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

fn canonical_video_url(url: &str) -> Result<String> {
    let id = downloader::extract_video_id(url)?;
    Ok(format!("https://www.youtube.com/watch?v={}", id))
}

fn is_playlist_only_page(url: &str) -> bool {
    let u = url.to_lowercase();
    (u.contains("youtube.com/playlist?") || u.contains("music.youtube.com/playlist?"))
        && !u.contains("watch?v=")
}

fn resolve_video_urls(raw: &str, cfg: &config::Config) -> Result<Vec<String>> {
    use config::PlaylistBehavior;
    let cookies = cfg.cookies_from_browser.as_deref();

    if playlist::is_playlist_url(raw).is_none() {
        return Ok(vec![canonical_video_url(raw)?]);
    }

    match cfg.playlist_behavior {
        PlaylistBehavior::VideoOnly => {
            if is_playlist_only_page(raw) {
                return Err(YtcsError::InvalidUrl(
                    "Playlist-only links need playlist_behavior = playlist_only (run ytcs config) or a watch?v= URL."
                        .to_string(),
                ));
            }
            Ok(vec![canonical_video_url(
                &playlist::remove_playlist_param(raw),
            )?])
        }
        PlaylistBehavior::PlaylistOnly => {
            let info = playlist::get_playlist_info(raw, cookies)?;
            Ok(info.videos.iter().map(|v| v.url.clone()).collect())
        }
        PlaylistBehavior::Ask => {
            eprintln!(
                "{}",
                "This URL includes a playlist. Download all videos? (y/n)".bold()
            );
            print!("> ");
            std::io::stdout().flush().ok();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).ok();
            if input.trim().eq_ignore_ascii_case("y") {
                let info = playlist::get_playlist_info(raw, cookies)?;
                Ok(info.videos.iter().map(|v| v.url.clone()).collect())
            } else if is_playlist_only_page(raw) {
                Err(YtcsError::InvalidUrl(
                    "Playlist-only URLs cannot be reduced to one video; answer y to download the playlist."
                        .to_string(),
                ))
            } else {
                Ok(vec![canonical_video_url(
                    &playlist::remove_playlist_param(raw),
                )?])
            }
        }
    }
}

/// Progress callback for track splitting
fn track_progress_callback(track_number: usize, total_tracks: usize, title: &str, duration: &str) {
    ui::print_track_progress(track_number, total_tracks, title, duration);
}

fn handle_missing_dependencies(e: YtcsError, behavior: &config::AutoInstallBehavior) -> Result<()> {
    let missing = match &e {
        YtcsError::MissingTools(m) => *m,
        _ => return Err(e),
    };

    eprintln!("{}", format!("⚠ {}", e).yellow());
    eprintln!();

    let install_all = || -> Result<()> {
        for tool in missing.tools_to_install() {
            downloader::install_dependency(tool)?;
        }
        println!();
        downloader::check_dependencies()?;
        Ok(())
    };

    match behavior {
        config::AutoInstallBehavior::Never => Err(e),
        config::AutoInstallBehavior::Always => install_all(),
        config::AutoInstallBehavior::Prompt => {
            eprintln!(
                "{}",
                "Would you like to install the missing dependencies? (y/n)".bold()
            );

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).ok();

            if input.trim().eq_ignore_ascii_case("y") {
                install_all()
            } else {
                Err(e)
            }
        }
    }
}

fn process_single_video(video_url: &str, cli: &Cli, app_config: &config::Config) -> Result<()> {
    let clean_url = video_url.to_string();

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
    let download_opts = YtdlpDownloadOpts::from(app_config);
    let audio_file = yt_dlp_progress::download_audio_with_progress(
        &clean_url,
        &temp_audio,
        app_config.cookies_from_browser.as_deref(),
        download_opts,
        None,
        None,
    )?;
    ui::print_audio_complete(audio_file.to_str().unwrap_or("audio.mp3"));

    let (mut chapters_to_use, used_silence_only) = if !video_info.chapters.is_empty() {
        (video_info.chapters.clone(), false)
    } else if let Some(desc) = video_info
        .description
        .as_deref()
        .map(str::trim)
        .filter(|d| !d.is_empty())
    {
        match chapters_from_description::parse_chapters_from_description(desc, video_info.duration)
        {
            Ok(c) if c.len() >= 2 => (c, false),
            _ => (
                audio::detect_silence_chapters(&audio_file, -30.0, 2.0)?,
                true,
            ),
        }
    } else {
        (
            audio::detect_silence_chapters(&audio_file, -30.0, 2.0)?,
            true,
        )
    };

    if !used_silence_only && (cli.refine_chapters || app_config.refine_chapters) {
        chapters_to_use = chapter_refinement::refine_chapters_with_silence(
            &chapters_to_use,
            &audio_file,
            5.0,
            -35.0,
            1.5,
        )?;
    }

    let cover_path = output_dir.join("cover.jpg");
    ui::print_splitting_section_header(chapters_to_use.len());

    let output_files = audio::split_audio_by_chapters(
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
        app_config.overwrite_existing,
        Some(track_progress_callback),
    )?;

    if app_config.create_playlist {
        let m3u = audio::write_m3u_playlist(&output_dir, &output_files)?;
        ui::print_section_header("Playlist");
        println!("  └─ {}", m3u.display());
    }

    ui::print_splitting_complete();

    std::fs::remove_file(&audio_file).ok();

    ui::print_final_result(&output_dir);

    Ok(())
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

    let video_urls = resolve_video_urls(url, &app_config)?;

    for (i, video_url) in video_urls.iter().enumerate() {
        if i > 0 {
            println!();
        }
        process_single_video(video_url, &cli, &app_config)?;
    }

    Ok(())
}
