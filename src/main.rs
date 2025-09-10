use eframe::egui;
use std::sync::{Arc, Mutex};

mod hooks;
mod recorder;
mod player;
mod events;

use hooks::GlobalHooks;
use recorder::MacroRecorder;
use player::MacroPlayer;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_min_inner_size([800.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Macro Recorder - Rust Edition",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(MacroApp::new(cc))
        }),
    )
}

#[derive(Debug, Clone)]
enum AppState {
    Idle,
    Recording,
    RecordingPaused,
    Playing,
    PlayingPaused,
}

struct MacroApp {
    state: AppState,
    recorder: Arc<Mutex<MacroRecorder>>,
    player: Arc<Mutex<MacroPlayer>>,
    hooks: Arc<Mutex<GlobalHooks>>,
    
    // UI state
    current_file: Option<String>,
    log_messages: Vec<String>,
    max_log_lines: usize,
    
    // Statistics
    events_recorded: usize,
    events_played: usize,
    recording_time: f32,
    
    // Settings
    show_mouse_moves: bool,
    playback_speed: f32,
}

impl MacroApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let hooks = Arc::new(Mutex::new(GlobalHooks::new()));
        let recorder = Arc::new(Mutex::new(MacroRecorder::new()));
        let player = Arc::new(Mutex::new(MacroPlayer::new()));
        
        let _hooks_clone = hooks.clone();
        let recorder_clone = recorder.clone();
        
        {
            let mut hooks_guard = hooks.lock().unwrap();
            hooks_guard.set_callback(Box::new(move |event| {
                if let Ok(mut recorder) = recorder_clone.lock() {
                    recorder.add_event(event);
                }
            }));
        }
        
        Self {
            state: AppState::Idle,
            recorder,
            player,
            hooks,
            current_file: None,
            log_messages: Vec::new(),
            max_log_lines: 1000,
            events_recorded: 0,
            events_played: 0,
            recording_time: 0.0,
            show_mouse_moves: true,
            playback_speed: 1.0,
        }
    }
    
    fn add_log(&mut self, message: String) {
        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f");
        self.log_messages.push(format!("[{}] {}", timestamp, message));
        
        // Keep only recent messages
        if self.log_messages.len() > self.max_log_lines {
            self.log_messages.drain(0..self.log_messages.len() - self.max_log_lines);
        }
    }
    
    fn start_recording(&mut self) {
        if matches!(self.state, AppState::Idle) {
            self.state = AppState::Recording;
            self.events_recorded = 0;
            self.recording_time = 0.0;
            
            if let Ok(mut recorder) = self.recorder.lock() {
                recorder.clear();
                recorder.start();
            }
            
            let install_result = if let Ok(mut hooks) = self.hooks.lock() {
                hooks.install()
            } else {
                Err("Failed to lock hooks".to_string())
            };
            
            match install_result {
                Ok(_) => {
                    self.add_log("🔴 Recording started - Global hooks active".to_string());
                    self.add_log("📝 Hotkeys: Ctrl+P (pause), Ctrl+Q (stop)".to_string());
                }
                Err(e) => {
                    self.add_log(format!("❌ Failed to install hooks: {}", e));
                    self.state = AppState::Idle;
                }
            }
        }
    }
    
    fn pause_resume_recording(&mut self) {
        match self.state {
            AppState::Recording => {
                self.state = AppState::RecordingPaused;
                if let Ok(mut recorder) = self.recorder.lock() {
                    recorder.pause();
                }
                self.add_log("⏸️ Recording paused".to_string());
            }
            AppState::RecordingPaused => {
                self.state = AppState::Recording;
                if let Ok(mut recorder) = self.recorder.lock() {
                    recorder.resume();
                }
                self.add_log("▶️ Recording resumed".to_string());
            }
            AppState::Playing => {
                self.state = AppState::PlayingPaused;
                if let Ok(mut player) = self.player.lock() {
                    player.pause();
                }
                self.add_log("⏸️ Playback paused".to_string());
            }
            AppState::PlayingPaused => {
                self.state = AppState::Playing;
                if let Ok(mut player) = self.player.lock() {
                    player.resume();
                }
                self.add_log("▶️ Playback resumed".to_string());
            }
            _ => {}
        }
    }
    
    fn stop_current_action(&mut self) {
        match self.state {
            AppState::Recording | AppState::RecordingPaused => {
                if let Ok(mut hooks) = self.hooks.lock() {
                    hooks.uninstall();
                }
                
                if let Ok(mut recorder) = self.recorder.lock() {
                    recorder.stop();
                    self.events_recorded = recorder.get_events().len();
                }
                
                self.state = AppState::Idle;
                self.add_log(format!("🛑 Recording stopped - {} events captured", self.events_recorded));
                
                self.add_log("💾 Use 'Save As' to save your recording".to_string());
            }
            AppState::Playing | AppState::PlayingPaused => {
                if let Ok(mut player) = self.player.lock() {
                    player.stop();
                }
                self.state = AppState::Idle;
                self.add_log("🛑 Playback stopped".to_string());
            }
            _ => {}
        }
    }
    
    fn save_recording(&mut self, path: &str) {
        let save_result = if let Ok(recorder) = self.recorder.lock() {
            recorder.save_to_file(path)
        } else {
            Err("Failed to lock recorder".into())
        };
        
        match save_result {
            Ok(_) => {
                self.current_file = Some(path.to_string());
                self.add_log(format!("💾 Saved to: {}", path));
            }
            Err(e) => {
                self.add_log(format!("❌ Save failed: {}", e));
            }
        }
    }
    
    fn load_recording(&mut self, path: &str) {
        let load_result = if let Ok(mut player) = self.player.lock() {
            player.load_from_file(path)
        } else {
            Err("Failed to lock player".into())
        };
        
        match load_result {
            Ok(event_count) => {
                self.current_file = Some(path.to_string());
                self.add_log(format!("📁 Loaded {} events from: {}", event_count, path));
            }
            Err(e) => {
                self.add_log(format!("❌ Load failed: {}", e));
            }
        }
    }
    
    fn start_playback(&mut self) {
        if matches!(self.state, AppState::Idle) && self.current_file.is_some() {
            self.state = AppState::Playing;
            self.events_played = 0;
            
            let speed = self.playback_speed;
            if let Ok(mut player) = self.player.lock() {
                player.set_speed(speed);
                player.start();
            }
            self.add_log(format!("▶️ Playback started ({}x speed)", speed));
        }
    }
}

impl eframe::App for MacroApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.input(|i| {
            if i.modifiers.ctrl {
                if i.key_pressed(egui::Key::R) {
                    self.start_recording();
                } else if i.key_pressed(egui::Key::P) {
                    self.pause_resume_recording();
                } else if i.key_pressed(egui::Key::Q) {
                    self.stop_current_action();
                }
            }
        });
        
        if let Ok(recorder) = self.recorder.lock() {
            self.events_recorded = recorder.get_events().len();
            self.recording_time = recorder.get_duration();
        }
        
        if let Ok(player) = self.player.lock() {
            self.events_played = player.get_current_position();
        }
        
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("📁 Open .mcr").clicked() {
                        self.load_recording("demo.mcr");
                        ui.close_menu();
                    }
                    
                    if ui.button("💾 Save As").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Macro files", &["mcr"])
                            .save_file() {
                            self.save_recording(&path.display().to_string());
                        }
                        ui.close_menu();
                    }
                    
                    ui.separator();
                    
                    if ui.button("🚪 Exit").clicked() {
                        self.stop_current_action();
                        
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        ui.close_menu();
                    }
                });
                
                ui.menu_button("Settings", |ui| {
                    ui.checkbox(&mut self.show_mouse_moves, "Show mouse moves in log");
                    
                    ui.horizontal(|ui| {
                        ui.label("Playback speed:");
                        ui.add(egui::Slider::new(&mut self.playback_speed, 0.1..=5.0)
                            .text("x"));
                    });
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let (color, text) = match self.state {
                        AppState::Idle => (egui::Color32::GRAY, "⚪ Idle"),
                        AppState::Recording => (egui::Color32::RED, "🔴 Recording"),
                        AppState::RecordingPaused => (egui::Color32::YELLOW, "⏸️ Rec Paused"),
                        AppState::Playing => (egui::Color32::GREEN, "▶️ Playing"),
                        AppState::PlayingPaused => (egui::Color32::YELLOW, "⏸️ Play Paused"),
                    };
                    
                    ui.colored_label(color, text);
                });
            });
        });
        
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let can_record = matches!(self.state, AppState::Idle);
                let can_pause = matches!(self.state, AppState::Recording | AppState::RecordingPaused | AppState::Playing | AppState::PlayingPaused);
                let can_stop = !matches!(self.state, AppState::Idle);
                let can_play = matches!(self.state, AppState::Idle) && self.current_file.is_some();
                
                if ui.add_enabled(can_record, egui::Button::new("🔴 Record (Ctrl+R)")).clicked() {
                    self.start_recording();
                }
                
                if ui.add_enabled(can_play, egui::Button::new("▶️ Play")).clicked() {
                    self.start_playback();
                }
                
                let pause_text = match self.state {
                    AppState::RecordingPaused | AppState::PlayingPaused => "▶️ Resume (Ctrl+P)",
                    _ => "⏸️ Pause (Ctrl+P)",
                };
                
                if ui.add_enabled(can_pause, egui::Button::new(pause_text)).clicked() {
                    self.pause_resume_recording();
                }
                
                if ui.add_enabled(can_stop, egui::Button::new("⏹️ Stop (Ctrl+Q)")).clicked() {
                    self.stop_current_action();
                }
                
                ui.separator();
                
                ui.label(format!("📊 Events: {}", self.events_recorded));
                if self.recording_time > 0.0 {
                    ui.label(format!("⏱️ Time: {:.1}s", self.recording_time));
                }
                
                if let Some(file) = &self.current_file {
                    ui.label(format!("📄 File: {}", 
                        std::path::Path::new(file).file_name()
                            .unwrap_or_default().to_string_lossy()));
                }
            });
        });
        
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("🔥 Global Hotkeys:");
                ui.label("Ctrl+R (Record) | Ctrl+P (Pause/Resume) | Ctrl+Q (Stop)");
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.hyperlink_to("🦀 Rust Edition", "https://github.com");
                });
            });
        });
        
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.heading("📝 Activity Log");
                    ui.separator();
                    
                    for message in &self.log_messages {
                        let color = if message.contains("❌") {
                            egui::Color32::RED
                        } else if message.contains("🔴") || message.contains("▶️") {
                            egui::Color32::GREEN
                        } else if message.contains("⏸️") {
                            egui::Color32::YELLOW
                        } else if message.contains("💾") || message.contains("📁") {
                            egui::Color32::BLUE
                        } else {
                            ui.visuals().text_color()
                        };
                        
                        ui.colored_label(color, message);
                    }
                    
                    if self.log_messages.is_empty() {
                        ui.colored_label(egui::Color32::GRAY, "🚀 Welcome to Macro Recorder Rust Edition!");
                        ui.colored_label(egui::Color32::GRAY, "📌 Click 'Record' or press Ctrl+R to start recording");
                        ui.colored_label(egui::Color32::GRAY, "⚡ This version uses native Windows hooks for maximum performance");
                    }
                });
        });
        
        ctx.request_repaint();
    }
    
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        log::info!("Application shutting down - performing cleanup");
        
        self.stop_current_action();
        
        if let Ok(mut player) = self.player.lock() {
            player.stop();
        }
        
        if let Ok(mut hooks) = self.hooks.lock() {
            hooks.uninstall();
        }
        
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        log::info!("Application cleanup completed");
    }
}
