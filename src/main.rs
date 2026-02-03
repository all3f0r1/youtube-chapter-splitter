use clap::{Parser, Subcommand};
use colored::Colorize;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use youtube_chapter_splitter::{
    Result, audio, config, download_audio_with_progress, downloader, error::YtcsError, playlist,
    print_refinement_report, refine_chapters_with_silence, temp_file::TempFile, ui,
    utils, ytdlp_helper,
};

#[derive(Parser)]
#[command(name = "ytcs")]
#[command(about = "YouTube Chapter Splitter - Download and split YouTube videos into MP3 tracks", long_about = None)]
#[command(version)]
struct Cli {
    /// YouTube video URL(s)
    ///
    /// If not provided: launch interactive TUI
    /// If provided without --cli: launch TUI with URL pre-filled
    /// If provided with --cli: plain-text download mode
    #[arg(value_name = "URL")]
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

    /// Force plain-text mode (no TUI rendering, for scripting/piping)
    #[arg(long)]
    cli: bool,

    /// Force yt-dlp update before downloading
    #[arg(long)]
    force_update: bool,

    #[command(subcommand)]
    command: Option<Commands>,
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

    /// Update yt-dlp to the latest version
    UpdateYtdlp,
}

/// Arguments for download processing
#[derive(Debug, Clone)]
struct DownloadArgs {
    urls: Vec<String>,
    output: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    force_update: bool,
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

/// Check dependencies and offer to install if missing (US1: Dependency Auto-Detection)
fn check_and_install_dependencies() -> Result<()> {
    use youtube_chapter_splitter::dependency::{DependencyInstaller, DependencyState};

    let state = DependencyState::check_all();

    if !state.all_present() {
        let missing = state.missing();
        let installer = DependencyInstaller::new();

        // Check config for auto-install preference
        let config = config::Config::load()?;
        let should_install = match config.dependency_auto_install {
            config::AutoInstallBehavior::Always => true,
            config::AutoInstallBehavior::Never => false,
            config::AutoInstallBehavior::Prompt => DependencyInstaller::prompt_install(&missing)?,
        };

        if should_install {
            for dep in &missing {
                if let Err(e) = installer.install(dep) {
                    eprintln!("{}", format!("✗ Failed to install {}: {}", dep, e).red());
                    eprintln!();
                    eprintln!("Manual installation:");
                    eprintln!("{}", installer.get_manual_instructions());
                    return Err(e);
                }
            }
        } else {
            eprintln!(
                "{}",
                "Dependencies required but auto-install is disabled.".yellow()
            );
            eprintln!();
            eprintln!("Manual installation:");
            eprintln!("{}", installer.get_manual_instructions());
            return Err(YtcsError::MissingTool(missing.join(", ")));
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    // Initialize logger
    // Set RUST_LOG environment variable to control log level:
    // RUST_LOG=debug ytcs <url>  (for debug logs)
    // RUST_LOG=info ytcs <url>   (for info logs, default)
    // RUST_LOG=warn ytcs <url>   (for warnings only)
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .format_timestamp(None)
        .format_module_path(false)
        .init();

    log::debug!("ytcs started");

    let cli = Cli::parse();

    // Handle commands
    match cli.command {
        Some(Commands::Config) => config::show_config(),
        Some(Commands::Set { key, value }) => config::set_config(&key, &value),
        Some(Commands::Reset) => config::reset_config(),
        Some(Commands::UpdateYtdlp) => handle_ytdlp_update(),
        None => {
            // No subcommand - determine mode based on URLs and --cli flag
            if cli.cli {
                // --cli flag: use plain-text CLI mode
                if cli.urls.is_empty() {
                    eprintln!("{}", "Error: --cli requires a URL".red().bold());
                    eprintln!();
                    eprintln!("Usage: ytcs --cli <URL> [OPTIONS]");
                    std::process::exit(1);
                }
                let args = DownloadArgs {
                    urls: cli.urls,
                    output: cli.output,
                    artist: cli.artist,
                    album: cli.album,
                    force_update: cli.force_update,
                };
                handle_download(args)
            } else {
                // Default mode: TUI
                #[cfg(feature = "tui")]
                {
                    // If URL provided, start TUI with URL pre-filled
                    // Otherwise, start TUI on welcome screen
                    let initial_url = if cli.urls.is_empty() {
                        None
                    } else {
                        Some(cli.urls.join("\n"))
                    };
                    return youtube_chapter_splitter::run_tui(initial_url);
                }
                #[cfg(not(feature = "tui"))]
                {
                    if cli.urls.is_empty() {
                        eprintln!("{}", "Error: No URL provided".red().bold());
                        eprintln!();
                        eprintln!("Usage: ytcs <URL> [OPTIONS]");
                        eprintln!("       ytcs --cli <URL> [OPTIONS]");
                        eprintln!("       ytcs config");
                        eprintln!("       ytcs set <KEY> <VALUE>");
                        eprintln!("       ytcs reset");
                        eprintln!("       ytcs update-ytdlp");
                        eprintln!();
                        eprintln!("For more information, run: ytcs --help");
                        std::process::exit(1);
                    } else {
                        let args = DownloadArgs {
                            urls: cli.urls,
                            output: cli.output,
                            artist: cli.artist,
                            album: cli.album,
                            force_update: cli.force_update,
                        };
                        handle_download(args)
                    }
                }
            }
        }
    }
}

/// Handle the update-ytdlp command
fn handle_ytdlp_update() -> Result<()> {
    use youtube_chapter_splitter::ytdlp_helper;

    println!("{}", "Updating yt-dlp to the latest version...".cyan());
    println!();

    // Check current version
    if let Some(info) = ytdlp_helper::get_ytdlp_version() {
        println!(
            "{}",
            format!(
                "Current version: {} ({} days old)",
                info.version,
                info.days_since_release.unwrap_or(0)
            )
            .dimmed()
        );
        println!();
    }

    match ytdlp_helper::update_ytdlp() {
        Ok(()) => {
            println!();
            println!("{}", "✓ yt-dlp is up to date!".green());
            Ok(())
        }
        Err(e) => {
            println!();
            println!("{}", format!("✗ Update failed: {}", e).red());
            println!();
            println!("{}", "To update manually, run:".yellow());
            println!("  pip install --upgrade yt-dlp");
            println!("  or");
            println!("  python -m pip install --upgrade yt-dlp");
            Err(e)
        }
    }
}

fn handle_download(cli: DownloadArgs) -> Result<()> {
    check_and_install_dependencies()?;

    // Load config
    let config = config::Config::load()?;

    // Handle --force-update flag
    if cli.force_update
        || (ytdlp_helper::should_check_for_update(config.ytdlp_update_interval_days)
            && config.ytdlp_auto_update)
    {
        // Check if update is needed
        if let Some(info) = ytdlp_helper::get_ytdlp_version()
            && (info.is_outdated || cli.force_update)
        {
            let days_old = info.days_since_release.unwrap_or(0);
            if days_old >= 7 || cli.force_update {
                println!("Updating yt-dlp ({} days old)...", days_old);
                if let Err(e) = ytdlp_helper::update_ytdlp() {
                    println!("  Warning: {}", e);
                }
            }
        }
    }

    // Afficher l'en-tête TUI moderne
    ui::print_header();
    ui::print_blank_line();

    // Process each URL
    let total_urls = cli.urls.len();
    for (index, url) in cli.urls.iter().enumerate() {
        if total_urls > 1 {
            println!("[{}/{}] {}", index + 1, total_urls, url);
        }

        process_single_url(url, &cli, &config)?;
    }

    if total_urls > 1 {
        println!(
            "{}",
            format!("✓ All {} URL(s) processed", total_urls).green()
        );
    }

    Ok(())
}

// ============================================================================
// HELPER FUNCTIONS FOR PROCESS_SINGLE_URL
// ============================================================================

/// Structure contenant les informations vidéo traitées
struct VideoContext {
    info: downloader::VideoInfo,
    artist: String,
    album: String,
}

/// Structure contenant les fichiers téléchargés
struct DownloadedAssets {
    audio_file: PathBuf,
    cover_data: Option<Vec<u8>>,
    _temp_audio: youtube_chapter_splitter::temp_file::TempFile,
    _temp_cover: youtube_chapter_splitter::temp_file::TempFile,
}

/// 1. Gère la détection de playlist et demande à l'utilisateur
///
/// Returns `true` if a playlist should be downloaded, `false` otherwise
fn handle_playlist_detection(url: &str, config: &config::Config) -> Result<bool> {
    use config::PlaylistBehavior;

    if let Some(_playlist_id) = playlist::is_playlist_url(url) {
        let should_download_playlist = match config.playlist_behavior {
            PlaylistBehavior::Ask => {
                print!(
                    "{}",
                    "Playlist detected. Download [v]ideo or [p]laylist? ".bold()
                );
                io::stdout().flush()?;

                let mut choice = String::new();
                io::stdin().read_line(&mut choice)?;
                let choice = choice.trim().to_lowercase();

                choice == "p" || choice == "playlist"
            }
            PlaylistBehavior::VideoOnly => false,
            PlaylistBehavior::PlaylistOnly => {
                println!("{}", "Downloading entire playlist...".green());
                true
            }
        };

        return Ok(should_download_playlist);
    }

    Ok(false)
}

/// 2. Retrieves video information and displays it
///
/// Returns a `VideoContext` containing all necessary information
fn fetch_and_display_video_info(
    url: &str,
    cli_artist: Option<&String>,
    cli_album: Option<&String>,
    cookies_from_browser: Option<&str>,
) -> Result<VideoContext> {
    use youtube_chapter_splitter::ui;

    let video_info = downloader::get_video_info(url, cookies_from_browser)?;

    // Parse artist and album pour l'affichage
    let (artist_display, album_display) = if let (Some(a), Some(al)) = (cli_artist, cli_album) {
        (utils::clean_folder_name(a), utils::clean_folder_name(al))
    } else {
        utils::parse_artist_album(&video_info.title, &video_info.uploader)
    };

    // Afficher les infos vidéo
    let display_title = format!("{} - {}", artist_display, album_display);
    let has_chapters = !video_info.chapters.is_empty();
    ui::print_video_info(
        &display_title,
        &utils::format_duration(video_info.duration),
        video_info.chapters.len(),
        !has_chapters,
        false,
    );
    ui::print_blank_line();

    Ok(VideoContext {
        info: video_info,
        artist: artist_display,
        album: album_display,
    })
}

/// 3. Sets up the output directory
///
/// Creates the directory if necessary and returns its path
fn setup_output_directory(
    cli_output: Option<&String>,
    artist: &str,
    album: &str,
    config: &config::Config,
) -> Result<PathBuf> {
    let base_dir = if let Some(ref output) = cli_output {
        PathBuf::from(shellexpand::tilde(output).to_string())
    } else {
        config.get_output_dir()
    };

    let dir_name = config.format_directory(artist, album);
    let output_dir = base_dir.join(&dir_name);
    std::fs::create_dir_all(&output_dir)?;

    Ok(output_dir)
}

/// 4. Downloads the cover and audio
///
/// Returns a `DownloadedAssets` structure containing the files and data
fn download_cover_and_audio(
    url: &str,
    video_info: &downloader::VideoInfo,
    output_dir: &Path,
    keep_cover: bool,
    cookies_from_browser: Option<&str>,
) -> Result<DownloadedAssets> {
    use youtube_chapter_splitter::ui::Status;
    use youtube_chapter_splitter::{temp_file::TempFile, ui};

    ui::print_section_header("Downloading the album...");

    // Download cover
    let cover_path = output_dir.join("cover.jpg");
    let mut temp_cover = TempFile::new(&cover_path);

    let cover_status;
    let cover_downloaded =
        match downloader::download_thumbnail(&video_info.thumbnail_url, output_dir) {
            Ok(_) => {
                cover_status = Status::Success;
                if keep_cover {
                    temp_cover.keep();
                }
                true
            }
            Err(e) => {
                cover_status = Status::Failed;
                ui::print_warning(&format!("Could not download artwork: {}", e));
                false
            }
        };

    ui::print_cover_status(cover_status);

    // Download audio with real-time progress bar
    let temp_audio_path = output_dir.join("temp_audio.mp3");
    let temp_audio = TempFile::new(&temp_audio_path);

    let audio_file = download_audio_with_progress(
        url,
        temp_audio.path(),
        cookies_from_browser,
        None, // Progress bar will be created automatically
    )?;

    // Charger l'image de couverture si elle existe
    let cover_data = if cover_downloaded {
        audio::load_cover_image(&cover_path)?
    } else {
        None
    };

    Ok(DownloadedAssets {
        audio_file,
        cover_data,
        _temp_audio: temp_audio,
        _temp_cover: temp_cover,
    })
}

/// 5. Retrieves chapters with fallback strategy
///
/// 3-step strategy:
/// 1. YouTube chapters
/// 2. Description parsing
/// 3. Silence detection
///
/// If chapters are found and `refine` is true, chapter markers are refined
/// using silence detection for improved split accuracy.
fn get_chapters_with_fallback(
    video_info: &downloader::VideoInfo,
    audio_file: &Path,
    refine: bool,
) -> Result<Vec<youtube_chapter_splitter::chapters::Chapter>> {
    use youtube_chapter_splitter::ui;

    let original_chapters = if !video_info.chapters.is_empty() {
        video_info.chapters.clone()
    } else {
        ui::print_info("No chapters found in video metadata, checking description...");
        match youtube_chapter_splitter::chapters_from_description::parse_chapters_from_description(
            &video_info.description,
            video_info.duration,
        ) {
            Ok(chapters) => {
                ui::print_info(&format!(
                    "Found {} chapters in description!",
                    chapters.len()
                ));
                chapters
            }
            Err(_) => {
                ui::print_info("No chapters in description, detecting from silence...");
                audio::detect_silence_chapters(audio_file, -30.0, 2.0)?
            }
        }
    };

    // Affiner les chapitres avec la détection de silence si demandé
    if refine && !original_chapters.is_empty() {
        match refine_chapters_with_silence(
            &original_chapters,
            audio_file,
            5.0,   // fenêtre de ±5 secondes
            -35.0, // seuil de silence en dB
            1.0,   // durée minimale de silence en secondes
        ) {
            Ok(refined) => {
                print_refinement_report(&original_chapters, &refined);
                Ok(refined)
            }
            Err(e) => {
                ui::print_warning(&format!(
                    "Chapter refinement failed: {}. Using original chapters.",
                    e
                ));
                Ok(original_chapters)
            }
        }
    } else {
        Ok(original_chapters)
    }
}

/// 6. Splits the audio into individual tracks
///
/// Uses chapters and metadata to create MP3 files
fn split_into_tracks(
    chapters: &[youtube_chapter_splitter::chapters::Chapter],
    audio_file: &Path,
    output_dir: &Path,
    artist: &str,
    album: &str,
    cover_data: Option<&[u8]>,
    config: &config::Config,
) -> Result<()> {
    use youtube_chapter_splitter::{progress, ui};

    ui::print_section_header("Splitting into the album...");

    for (i, ch) in chapters.iter().enumerate() {
        let track_number = i + 1;
        let duration_str = utils::format_duration(ch.duration());

        // Construire le nom de fichier selon le format
        let formatted_name = config
            .filename_format
            .replace("%n", &format!("{:02}", track_number))
            .replace("%t", &ch.title)
            .replace("%a", artist)
            .replace("%A", album);

        let message = format!("{} ({})", formatted_name, duration_str);
        let pb = progress::create_track_progress(&message);

        // Découper la piste
        audio::split_single_track(audio::TrackSplitParams {
            input_file: audio_file,
            chapter: ch,
            track_number,
            total_tracks: chapters.len(),
            output_dir,
            artist,
            album,
            cover_data,
            config,
        })?;

        pb.finish_and_clear();
        println!("  ✓ {} ({})", formatted_name, duration_str);
    }

    Ok(())
}

// ============================================================================
// REFACTORED PROCESS_SINGLE_URL
// ============================================================================

fn process_single_url(url: &str, cli: &DownloadArgs, config: &config::Config) -> Result<()> {
    use youtube_chapter_splitter::ui;

    // 1. Vérifier si c'est une playlist
    if handle_playlist_detection(url, config)? {
        return download_playlist(url, cli, config);
    }

    // Nettoyer l'URL
    let url = clean_url(url);

    // Vérifier que c'est bien une URL YouTube
    if !url.contains("youtube.com") && !url.contains("youtu.be") {
        ui::print_error("Invalid YouTube URL. Please provide a valid YouTube video URL.");
        return Err(youtube_chapter_splitter::error::YtcsError::InvalidUrl(
            "Not a YouTube URL".to_string(),
        ));
    }

    // 2. Récupérer et afficher les informations vidéo
    let cookies_from_browser = config.cookies_from_browser.as_deref();
    let video_ctx = fetch_and_display_video_info(
        &url,
        cli.artist.as_ref(),
        cli.album.as_ref(),
        cookies_from_browser,
    )?;

    // 3. Configurer le répertoire de sortie
    let output_dir = setup_output_directory(
        cli.output.as_ref(),
        &video_ctx.artist,
        &video_ctx.album,
        config,
    )?;

    // 4. Download cover and audio
    let keep_cover = config.download_cover;
    let assets = download_cover_and_audio(
        &url,
        &video_ctx.info,
        &output_dir,
        keep_cover,
        cookies_from_browser,
    )?;

    // 5. Get chapters with fallback (refinement always enabled)
    let chapters = get_chapters_with_fallback(&video_ctx.info, &assets.audio_file, true)?;

    // 6. Split into tracks
    ui::print_blank_line();
    split_into_tracks(
        &chapters,
        &assets.audio_file,
        &output_dir,
        &video_ctx.artist,
        &video_ctx.album,
        assets.cover_data.as_deref(),
        config,
    )?;

    // Success message
    ui::print_blank_line();
    ui::print_success(&output_dir.display().to_string());

    Ok(())
}

fn download_playlist(url: &str, cli: &DownloadArgs, cfg: &config::Config) -> Result<()> {
    let cookies_from_browser = cfg.cookies_from_browser.as_deref();
    let playlist_info = playlist::get_playlist_info(url, cookies_from_browser)?;

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
        println!("[{}/{}] {}", i + 1, playlist_info.videos.len(), video.title);

        match download_single_video(&video.url, cli, cfg, &playlist_dir) {
            Ok(_) => {
                successful += 1;
            }
            Err(e) => {
                failed += 1;
                println!("  ✗ {}", e);
            }
        }
    }

    // Summary
    println!(
        "{} {}/{} successful",
        "✓".green(),
        successful,
        playlist_info.videos.len()
    );
    if failed > 0 {
        println!(
            "{} {}/{} failed",
            "✗".red(),
            failed,
            playlist_info.videos.len()
        );
    }

    // Create M3U playlist if configured
    if cfg.create_playlist {
        create_m3u_playlist(&playlist_dir, &playlist_info.title)?;
    }

    Ok(())
}

fn download_single_video(
    url: &str,
    cli: &DownloadArgs,
    cfg: &config::Config,
    base_output_dir: &std::path::Path,
) -> Result<()> {
    let url = clean_url(url);
    let cookies_from_browser = cfg.cookies_from_browser.as_deref();

    let video_info = downloader::get_video_info(&url, cookies_from_browser)?;

    // Parse artist and album
    let (artist, album) = if let (Some(a), Some(al)) = (&cli.artist, &cli.album) {
        (utils::clean_folder_name(a), utils::clean_folder_name(al))
    } else {
        utils::parse_artist_album(&video_info.title, &video_info.uploader)
    };

    // Create output directory using config format
    let dir_name = cfg.format_directory(&artist, &album);
    let output_dir = base_output_dir.join(&dir_name);
    std::fs::create_dir_all(&output_dir)?;

    // Download cover art if enabled in config
    if cfg.download_cover {
        match downloader::download_thumbnail(&video_info.thumbnail_url, &output_dir) {
            Ok(thumb_path) => println!("{} {}", "✓ Artwork saved:".green(), thumb_path.display()),
            Err(e) => println!("{} {}", "⚠ Could not download artwork:".yellow(), e),
        }
    }

    // Download audio
    let temp_audio_path = output_dir.join("temp_audio.mp3");
    let _temp_file = TempFile::new(&temp_audio_path);

    let audio_file =
        downloader::download_audio(&url, &temp_audio_path, cookies_from_browser, None)?;

    // Get chapters
    let chapters_to_use = if !video_info.chapters.is_empty() {
        video_info.chapters
    } else {
        audio::detect_silence_chapters(&audio_file, -30.0, 2.0)?
    };

    // Split audio with metadata
    let cover_path = if cfg.download_cover {
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

    // Temp file is automatically cleaned up by _temp_file when it goes out of scope
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
