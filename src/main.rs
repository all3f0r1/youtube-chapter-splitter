use clap::{Parser, Subcommand};
use colored::Colorize;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use youtube_chapter_splitter::{
    audio, config, download_audio_with_progress, downloader, playlist, print_refinement_report,
    refine_chapters_with_silence, utils, Result,
};

#[derive(Parser)]
#[command(name = "ytcs")]
#[command(about = "YouTube Chapter Splitter - Download and split YouTube videos into MP3 tracks", long_about = None)]
#[command(version)]
struct Cli {
    /// YouTube video URL(s) - can be used without 'download' subcommand
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

    /// Skip downloading cover art
    #[arg(long)]
    no_cover: bool,

    /// Refine chapter markers using silence detection (improves split accuracy)
    #[arg(long, default_value = "true")]
    refine_chapters: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Parser)]
struct DownloadArgs {
    /// YouTube video URL(s)
    #[arg(value_name = "URL", required = true)]
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

    /// Refine chapter markers using silence detection (improves split accuracy)
    #[arg(long, default_value = "true")]
    refine_chapters: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Download and split YouTube video(s) (default)
    Download(DownloadArgs),

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
        Some(Commands::Download(args)) => handle_download(args),
        None => {
            // Si aucune commande mais des URLs sont fournies, traiter comme download
            if !cli.urls.is_empty() {
                let args = DownloadArgs {
                    urls: cli.urls,
                    output: cli.output,
                    artist: cli.artist,
                    album: cli.album,
                    no_cover: cli.no_cover,
                    refine_chapters: cli.refine_chapters,
                };
                handle_download(args)
            } else {
                // Aucune URL fournie, afficher l'aide
                eprintln!("{}", "Error: No URL provided".red().bold());
                eprintln!();
                eprintln!("Usage: ytcs <URL> [OPTIONS]");
                eprintln!("       ytcs download <URL> [OPTIONS]");
                eprintln!("       ytcs config");
                eprintln!("       ytcs set <KEY> <VALUE>");
                eprintln!("       ytcs reset");
                eprintln!();
                eprintln!("For more information, run: ytcs --help");
                std::process::exit(1);
            }
        }
    }
}

fn handle_download(cli: DownloadArgs) -> Result<()> {
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
            println!(
                "{}",
                format!("=== Processing URL {}/{} ===", index + 1, total_urls)
                    .cyan()
                    .bold()
            );
            println!("{}", url.bright_blue());
            println!();
        }

        process_single_url(url, &cli, &config)?;
    }

    if total_urls > 1 {
        println!();
        println!(
            "{}",
            format!("✓ All {} URL(s) processed successfully!", total_urls)
                .green()
                .bold()
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
                println!("{}", "Playlist detected!".yellow().bold());
                println!();
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
            PlaylistBehavior::VideoOnly => false,
            PlaylistBehavior::PlaylistOnly => {
                println!("{}", "Playlist detected!".yellow().bold());
                println!();
                println!(
                    "{}",
                    "Downloading entire playlist (configured behavior)...".green()
                );
                println!();
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

    println!("{}", "Fetching video information...".yellow());
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
        ui::print_info("Refining chapter markers with silence detection...");
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

    // 4. Télécharger la couverture et l'audio
    let keep_cover = !cli.no_cover && config.download_cover;
    let assets = download_cover_and_audio(
        &url,
        &video_ctx.info,
        &output_dir,
        keep_cover,
        cookies_from_browser,
    )?;

    // 5. Récupérer les chapitres avec fallback
    let chapters =
        get_chapters_with_fallback(&video_ctx.info, &assets.audio_file, cli.refine_chapters)?;

    // 6. Découper en pistes
    split_into_tracks(
        &chapters,
        &assets.audio_file,
        &output_dir,
        &video_ctx.artist,
        &video_ctx.album,
        assets.cover_data.as_deref(),
        config,
    )?;

    // Message de succès
    ui::print_success(&output_dir.display().to_string());

    Ok(())
}

fn download_playlist(url: &str, cli: &DownloadArgs, cfg: &config::Config) -> Result<()> {
    println!("{}", "Fetching playlist information...".yellow());

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

    // Download cover art (unless --no-cover)
    let download_cover = !cli.no_cover && cfg.download_cover;
    if download_cover {
        match downloader::download_thumbnail(&video_info.thumbnail_url, &output_dir) {
            Ok(thumb_path) => println!("{} {}", "✓ Artwork saved:".green(), thumb_path.display()),
            Err(e) => println!("{} {}", "⚠ Could not download artwork:".yellow(), e),
        }
    }

    // Download audio
    let audio_file = downloader::download_audio(
        &url,
        &output_dir.join("temp_audio.mp3"),
        cookies_from_browser,
        None,
    )?;

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
