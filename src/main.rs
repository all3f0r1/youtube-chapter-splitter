use clap::{Parser, Subcommand};
use colored::Colorize;
use std::io::Write;
use std::path::PathBuf;
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

    /// Snap chapter cuts to silence (extra ffmpeg pass; default on in config; forces refinement if config has it off)
    #[arg(long)]
    refine_chapters: bool,

    /// Print target folder and chapter plan without downloading or splitting
    #[arg(long)]
    dry_run: bool,

    /// Minimal output (still prints output folder path when done, and dry-run lines)
    #[arg(short, long)]
    quiet: bool,

    /// Skip thumbnail download for this run
    #[arg(long)]
    no_cover: bool,

    /// Use existing `temp_audio.*` in the album folder if present instead of downloading
    #[arg(long)]
    skip_download: bool,
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
        log::info!("Single video URL (no playlist parameter)");
        return Ok(vec![canonical_video_url(raw)?]);
    }

    log::info!(
        "Playlist parameter detected; behavior={:?}",
        cfg.playlist_behavior
    );

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
            log::info!("Playlist expanded to {} videos", info.videos.len());
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
                log::info!("User chose full playlist ({} videos)", info.videos.len());
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

/// When `total > 1`, used with `playlist_prefix_index` to disambiguate output folders.
struct BatchCtx {
    index: usize,
    total: usize,
}

fn run_dry_run(urls: &[String], cli: &Cli, cfg: &config::Config) -> Result<()> {
    for (i, url) in urls.iter().enumerate() {
        let vi = downloader::get_video_info(url, cfg.cookies_from_browser.as_deref())?;
        let ((artist, album), _, _) = utils::parse_artist_album_with_source(&vi.title);
        let mut folder_name = cfg.format_directory(&artist, &album);
        if urls.len() > 1 && cfg.playlist_prefix_index {
            folder_name = format!("{:02} - {}", i + 1, folder_name);
        }
        let base = cli
            .output
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| cfg.get_output_dir());
        let out_dir = base.join(&folder_name);
        let chapter_note = if !vi.chapters.is_empty() {
            format!("{} (YouTube chapters)", vi.chapters.len())
        } else if let Some(desc) = vi
            .description
            .as_deref()
            .map(str::trim)
            .filter(|d| !d.is_empty())
        {
            match chapters_from_description::parse_chapters_from_description(desc, vi.duration) {
                Ok(c) if c.len() >= 2 => format!("{} (from description)", c.len()),
                _ => "silence detection after download".to_string(),
            }
        } else {
            "silence detection after download".to_string()
        };
        println!("URL         {}", url);
        println!("  output    {}", out_dir.display());
        println!("  chapters  {}", chapter_note);
        println!("  format    {:?}", cfg.audio_format);
        if i + 1 < urls.len() {
            println!();
        }
    }
    Ok(())
}

fn process_single_video(
    video_url: &str,
    cli: &Cli,
    app_config: &config::Config,
    batch: Option<BatchCtx>,
) -> Result<()> {
    let clean_url = video_url.to_string();

    ui::print_section_header("Fetching video information");
    let video_info =
        downloader::get_video_info(&clean_url, app_config.cookies_from_browser.as_deref())?;

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

    let mut folder_name = app_config.format_directory(&artist, &album);
    if let Some(b) = &batch
        && b.total > 1
        && app_config.playlist_prefix_index
    {
        folder_name = format!("{:02} - {}", b.index + 1, folder_name);
    }
    let base_output = cli
        .output
        .as_ref()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| app_config.get_output_dir());
    let output_dir = base_output.join(&folder_name);
    std::fs::create_dir_all(&output_dir)?;

    let want_cover = app_config.download_cover && !cli.no_cover;
    if want_cover {
        match downloader::download_thumbnail_from_info(
            &video_info,
            &clean_url,
            &output_dir,
            app_config.cookies_from_browser.as_deref(),
        ) {
            Ok(thumb_path) => {
                ui::print_artwork_saved(thumb_path.to_str().unwrap_or("cover.jpg"));
            }
            Err(e) => {
                log::warn!("Thumbnail download failed: {}", e);
                ui::print_artwork_failed(
                    "could not retrieve cover (run with RUST_LOG=warn for details)",
                );
            }
        }
    } else {
        ui::print_artwork_disabled();
    }

    ui::print_audio_section_header();
    let ext = app_config.audio_format.extension();
    let temp_audio = output_dir.join(format!("temp_audio.{ext}"));
    let download_opts = YtdlpDownloadOpts::from(app_config);

    let audio_file = if cli.skip_download {
        let p = temp_audio.clone();
        if p.exists() && std::fs::metadata(&p).map(|m| m.len() > 0).unwrap_or(false) {
            log::info!("Using existing file at {}", p.display());
            p
        } else {
            return Err(YtcsError::DownloadError(format!(
                "--skip-download: expected non-empty file at {}",
                p.display()
            )));
        }
    } else {
        yt_dlp_progress::download_audio_with_progress(
            &clean_url,
            &temp_audio,
            app_config.cookies_from_browser.as_deref(),
            download_opts,
            None,
            None,
        )?
    };
    ui::print_audio_complete(audio_file.to_str().unwrap_or("audio"));

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
        log::info!(
            "Refining chapters (window={}s noise={}dB min_silence={}s)",
            app_config.refine_silence_window,
            app_config.refine_noise_db,
            app_config.refine_min_silence
        );
        chapters_to_use = chapter_refinement::refine_chapters_with_silence(
            &chapters_to_use,
            &audio_file,
            app_config.refine_silence_window,
            app_config.refine_noise_db,
            app_config.refine_min_silence,
        )?;
    }

    let cover_path = downloader::album_cover_path(&output_dir);
    ui::print_splitting_section_header(chapters_to_use.len());

    let extra_date = video_info
        .upload_date
        .as_deref()
        .and_then(utils::upload_date_to_id3_date);
    let extra_genre = video_info.genre.as_deref();
    let extra_comment = video_info
        .webpage_url
        .as_deref()
        .or(Some(clean_url.as_str()));

    let output_files = audio::split_audio_by_chapters(
        &audio_file,
        &chapters_to_use,
        &output_dir,
        &artist,
        &album,
        if want_cover {
            cover_path.as_deref()
        } else {
            None
        },
        &app_config.filename_format,
        app_config.audio_format,
        app_config.audio_quality,
        extra_date.as_deref(),
        extra_genre,
        extra_comment,
        app_config.overwrite_existing,
        Some(track_progress_callback),
    )?;

    if app_config.create_playlist {
        let m3u = audio::write_m3u_playlist(&output_dir, &output_files)?;
        if !ui::is_output_quiet() {
            ui::print_section_header("Playlist");
            println!("  └─ {}", m3u.display());
        }
    }

    ui::print_splitting_complete();

    std::fs::remove_file(&audio_file).ok();

    ui::print_final_result(&output_dir);

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "✗".red().bold(), format!("{}", e).red());
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
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

    ui::set_output_quiet(cli.quiet);

    ui::print_header();

    if let Err(e) = downloader::check_dependencies() {
        handle_missing_dependencies(e, &app_config.dependency_auto_install)?;
    }

    let video_urls = resolve_video_urls(url, &app_config)?;

    if cli.dry_run {
        return run_dry_run(&video_urls, &cli, &app_config);
    }

    let n = video_urls.len();
    for (i, video_url) in video_urls.iter().enumerate() {
        if i > 0 {
            println!();
        }
        let batch = (n > 1).then_some(BatchCtx { index: i, total: n });
        process_single_video(video_url, &cli, &app_config, batch)?;
    }

    Ok(())
}
