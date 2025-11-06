use eframe::egui;
use youtube_chapter_splitter::{audio, chapters, downloader, utils, Result};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "YouTube Chapter Splitter",
        options,
        Box::new(|_cc| Ok(Box::new(YtcsApp::default()))),
    )
}

#[derive(Default)]
struct YtcsApp {
    url: String,
    output_dir: String,
    detect_silence: bool,
    silence_threshold: f64,
    min_silence_duration: f64,
    
    status: String,
    progress: f32,
    video_info: Option<VideoInfoDisplay>,
    
    dependencies_checked: bool,
    missing_deps: Vec<String>,
    
    processing: Arc<Mutex<bool>>,
}

#[derive(Clone)]
struct VideoInfoDisplay {
    title: String,
    duration: String,
    tracks: Vec<TrackDisplay>,
}

#[derive(Clone)]
struct TrackDisplay {
    number: usize,
    title: String,
    duration: String,
}

impl YtcsApp {
    fn check_dependencies(&mut self) {
        if self.dependencies_checked {
            return;
        }
        
        self.dependencies_checked = true;
        self.missing_deps.clear();
        
        if let Err(e) = downloader::check_dependencies() {
            let error_msg = e.to_string();
            if error_msg.contains("yt-dlp") {
                self.missing_deps.push("yt-dlp".to_string());
            }
            if error_msg.contains("ffmpeg") {
                self.missing_deps.push("ffmpeg".to_string());
            }
            
            if !self.missing_deps.is_empty() {
                self.status = format!("⚠ Missing dependencies: {}", self.missing_deps.join(", "));
            }
        } else {
            self.status = "✓ All dependencies installed".to_string();
        }
    }
    
    fn install_dependency(&mut self, tool: &str) {
        self.status = format!("Installing {}...", tool);
        
        match downloader::install_dependency(tool) {
            Ok(_) => {
                self.status = format!("✓ {} installed successfully", tool);
                self.missing_deps.retain(|d| d != tool);
                self.dependencies_checked = false;
            }
            Err(e) => {
                self.status = format!("✗ Failed to install {}: {}", tool, e);
            }
        }
    }
    
    fn fetch_video_info(&mut self) {
        if self.url.is_empty() {
            self.status = "Please enter a YouTube URL".to_string();
            return;
        }
        
        self.status = "Fetching video information...".to_string();
        self.video_info = None;
        
        let url = self.url.clone();
        
        match tokio::runtime::Runtime::new() {
            Ok(rt) => {
                match rt.block_on(async { downloader::get_video_info(&url) }) {
                    Ok(info) => {
                        let tracks: Vec<TrackDisplay> = info.chapters.iter().enumerate().map(|(i, ch)| {
                            TrackDisplay {
                                number: i + 1,
                                title: ch.title.clone(),
                                duration: utils::format_duration_short(ch.duration()),
                            }
                        }).collect();
                        
                        self.video_info = Some(VideoInfoDisplay {
                            title: info.title.clone(),
                            duration: utils::format_duration(info.duration),
                            tracks,
                        });
                        
                        self.status = format!("✓ Found {} tracks", info.chapters.len());
                    }
                    Err(e) => {
                        self.status = format!("✗ Error: {}", e);
                    }
                }
            }
            Err(e) => {
                self.status = format!("✗ Runtime error: {}", e);
            }
        }
    }
    
    fn start_download(&mut self) {
        if self.url.is_empty() {
            self.status = "Please enter a YouTube URL".to_string();
            return;
        }
        
        let url = self.url.clone();
        let output = if self.output_dir.is_empty() {
            PathBuf::from("./output")
        } else {
            PathBuf::from(&self.output_dir)
        };
        let detect_silence = self.detect_silence;
        let silence_threshold = self.silence_threshold;
        let min_silence_duration = self.min_silence_duration;
        
        self.status = "Starting download...".to_string();
        self.progress = 0.0;
        
        // Dans une vraie application, ceci devrait être dans un thread séparé
        match tokio::runtime::Runtime::new() {
            Ok(rt) => {
                match rt.block_on(async {
                    // Obtenir les informations
                    let video_info = downloader::get_video_info(&url)?;
                    
                    // Créer le répertoire de sortie
                    let clean_title = utils::clean_folder_name(&video_info.title);
                    let output_dir = output.join(&clean_title);
                    std::fs::create_dir_all(&output_dir)?;
                    
                    // Télécharger l'audio
                    let temp_audio = output_dir.join("temp_audio");
                    let audio_file = downloader::download_audio(&url, &temp_audio)?;
                    
                    // Déterminer les chapitres
                    let chapters_to_use = if !video_info.chapters.is_empty() {
                        video_info.chapters
                    } else if detect_silence {
                        audio::detect_silence_chapters(&audio_file, silence_threshold, min_silence_duration)?
                    } else {
                        return Err(youtube_chapter_splitter::YtcsError::ChapterError(
                            "No tracks found and silence detection disabled".to_string()
                        ));
                    };
                    
                    // Diviser l'audio
                    let output_files = audio::split_audio_by_chapters(
                        &audio_file,
                        &chapters_to_use,
                        &output_dir,
                        &clean_title,
                    )?;
                    
                    // Nettoyer
                    std::fs::remove_file(&audio_file).ok();
                    
                    Ok::<(usize, PathBuf), youtube_chapter_splitter::YtcsError>((output_files.len(), output_dir))
                }) {
                    Ok((count, dir)) => {
                        self.status = format!("✓ Success! Created {} tracks in {}", count, dir.display());
                        self.progress = 1.0;
                    }
                    Err(e) => {
                        self.status = format!("✗ Error: {}", e);
                        self.progress = 0.0;
                    }
                }
            }
            Err(e) => {
                self.status = format!("✗ Runtime error: {}", e);
            }
        }
    }
}

impl eframe::App for YtcsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Vérifier les dépendances au premier lancement
        if !self.dependencies_checked {
            self.check_dependencies();
        }
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎵 YouTube Chapter Splitter");
            ui.add_space(10.0);
            
            // Section des dépendances
            if !self.missing_deps.is_empty() {
                ui.colored_label(egui::Color32::from_rgb(255, 165, 0), "⚠ Missing Dependencies");
                ui.horizontal(|ui| {
                    for dep in self.missing_deps.clone() {
                        if ui.button(format!("Install {}", dep)).clicked() {
                            self.install_dependency(&dep);
                        }
                    }
                });
                ui.add_space(10.0);
            }
            
            // URL Input
            ui.horizontal(|ui| {
                ui.label("YouTube URL:");
                ui.text_edit_singleline(&mut self.url);
            });
            
            ui.add_space(5.0);
            
            // Output directory
            ui.horizontal(|ui| {
                ui.label("Output Directory:");
                ui.text_edit_singleline(&mut self.output_dir);
                if self.output_dir.is_empty() {
                    ui.label("(default: ./output)");
                }
            });
            
            ui.add_space(10.0);
            
            // Options
            ui.checkbox(&mut self.detect_silence, "Enable silence detection if no tracks found");
            
            if self.detect_silence {
                ui.horizontal(|ui| {
                    ui.label("Silence threshold (dB):");
                    ui.add(egui::Slider::new(&mut self.silence_threshold, -50.0..=-10.0));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Min silence duration (s):");
                    ui.add(egui::Slider::new(&mut self.min_silence_duration, 0.5..=5.0));
                });
            }
            
            ui.add_space(10.0);
            
            // Buttons
            ui.horizontal(|ui| {
                if ui.button("📋 Fetch Info").clicked() {
                    self.fetch_video_info();
                }
                
                if ui.button("⬇ Download & Split").clicked() {
                    self.start_download();
                }
            });
            
            ui.add_space(10.0);
            
            // Progress bar
            if self.progress > 0.0 {
                ui.add(egui::ProgressBar::new(self.progress).show_percentage());
            }
            
            // Status
            if !self.status.is_empty() {
                ui.separator();
                ui.label(&self.status);
            }
            
            // Video info display
            if let Some(ref info) = self.video_info {
                ui.separator();
                ui.add_space(10.0);
                
                ui.heading("Video Information");
                ui.label(format!("Title: {}", info.title));
                ui.label(format!("Duration: {}", info.duration));
                ui.label(format!("Tracks: {}", info.tracks.len()));
                
                ui.add_space(5.0);
                
                if !info.tracks.is_empty() {
                    ui.label("Track List:");
                    egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                        for track in &info.tracks {
                            ui.label(format!("  {}. {} [{}]", track.number, track.title, track.duration));
                        }
                    });
                }
            }
        });
    }
}
