//! Main UI binding with integrated audio support
//!
//! This module provides the primary UI binding for the Sonic Flow music player
//! with audio integration using event loop based architecture and safe channel communication.

use slint::{ComponentHandle, Weak};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use sonic_core::Result;
use crate::audio_bridge::{AudioCommand, AudioIntegration};
use crate::bindings::MainWindow; // Use existing MainWindow

/// UI command from callbacks
#[derive(Debug)]
pub enum UiCommand {
    LoadFile(PathBuf),
    TogglePlayback,
    Stop,
    SetVolume(f32),
    NextTrack,
    PreviousTrack,
    SkipForward(f64),
    SkipBackward(f64),
    Seek(f32),
}

/// Main UI binding with audio integration
pub struct MainWindowBinding {
    /// Slint main window instance
    window: MainWindow,
    /// Audio integration manager
    audio_integration: Arc<AudioIntegration>,
    /// UI command sender for safe callback communication
    ui_command_tx: mpsc::UnboundedSender<UiCommand>,
    /// Command processor task handle
    _command_processor: tokio::task::JoinHandle<()>,
}

impl MainWindowBinding {
    /// Create a new main window binding
    pub fn new() -> Result<Self> {
        info!("Creating main window binding");

        let window = MainWindow::new().map_err(|e| {
            sonic_core::Error::Application(format!("Failed to create main window: {}", e))
        })?;

        // Create audio integration with weak window reference
        let audio_integration = Arc::new(AudioIntegration::new(window.as_weak())?);

        // Create UI command channel
        let (ui_command_tx, ui_command_rx) = mpsc::unbounded_channel();

        // Start command processor
        let command_processor = Self::start_command_processor(
            Arc::clone(&audio_integration),
            ui_command_rx,
        );

        let binding = Self {
            window,
            audio_integration,
            ui_command_tx,
            _command_processor: command_processor,
        };

        // Set up UI callbacks
        binding.setup_callbacks();

        // Set initial UI state
        binding.set_initial_state();

        // Request initial status
        binding.audio_integration.send_command(AudioCommand::RequestStatus);

        info!("Main window binding created successfully");
        Ok(binding)
    }

    /// Start command processor
    fn start_command_processor(
        audio_integration: Arc<AudioIntegration>,
        mut ui_command_rx: mpsc::UnboundedReceiver<UiCommand>,
    ) -> tokio::task::JoinHandle<()> {
        // Extract the command sender from AudioIntegration to avoid Send issues
        let audio_command_tx = audio_integration.get_command_sender();
        
        tokio::spawn(async move {
            info!("UI command processor started");

            while let Some(command) = ui_command_rx.recv().await {
                debug!("Processing UI command: {:?}", command);

                let audio_command = match command {
                    UiCommand::LoadFile(path) => AudioCommand::LoadAndPlay(path),
                    UiCommand::TogglePlayback => AudioCommand::TogglePlayback,
                    UiCommand::Stop => AudioCommand::Stop,
                    UiCommand::SetVolume(volume) => AudioCommand::SetVolume(volume),
                    UiCommand::NextTrack => AudioCommand::NextTrack,
                    UiCommand::PreviousTrack => AudioCommand::PreviousTrack,
                    UiCommand::SkipForward(seconds) => AudioCommand::SkipForward(seconds),
                    UiCommand::SkipBackward(seconds) => AudioCommand::SkipBackward(seconds),
                    UiCommand::Seek(position) => {
                        // Convert relative position to duration (placeholder)
                        use std::time::Duration;
                        let seek_position = Duration::from_secs_f32(position * 300.0); // TODO: Use actual duration
                        AudioCommand::Seek(seek_position)
                    }
                };

                if let Err(_) = audio_command_tx.send(audio_command) {
                    error!("Failed to send audio command");
                }
            }

            info!("UI command processor ended");
        })
    }

    /// Set up UI callbacks
    fn setup_callbacks(&self) {
        // Load track button
        self.window.on_load_track_clicked({
            let ui_command_tx = self.ui_command_tx.clone();
            
            move || {
                debug!("Load track button clicked");
                
                // Use blocking file dialog on main thread
                let result = Self::open_file_dialog_sync();
                
                match result {
                    Ok(Some(path)) => {
                        info!("Selected file: {}", path.display());
                        
                        if let Err(_) = ui_command_tx.send(UiCommand::LoadFile(path)) {
                            error!("Failed to send load file command");
                        }
                    }
                    Ok(None) => {
                        debug!("No file selected");
                    }
                    Err(e) => {
                        error!("File dialog error: {}", e);
                    }
                }
            }
        });

        // Play/Pause button
        self.window.on_play_pause_clicked({
            let ui_command_tx = self.ui_command_tx.clone();
            
            move || {
                debug!("Play/pause button clicked");
                
                if let Err(_) = ui_command_tx.send(UiCommand::TogglePlayback) {
                    error!("Failed to send toggle playback command");
                }
            }
        });

        // Stop button
        self.window.on_stop_clicked({
            let ui_command_tx = self.ui_command_tx.clone();
            
            move || {
                debug!("Stop button clicked");
                
                if let Err(_) = ui_command_tx.send(UiCommand::Stop) {
                    error!("Failed to send stop command");
                }
            }
        });

        // Volume control
        self.window.on_volume_changed({
            let ui_command_tx = self.ui_command_tx.clone();
            
            move |volume| {
                debug!("Volume changed to: {:.2}", volume);
                
                if let Err(_) = ui_command_tx.send(UiCommand::SetVolume(volume)) {
                    error!("Failed to send volume command");
                }
            }
        });

        // Next track button
        self.window.on_next_track({
            let ui_command_tx = self.ui_command_tx.clone();
            
            move || {
                debug!("Next track button clicked");
                
                if let Err(_) = ui_command_tx.send(UiCommand::NextTrack) {
                    error!("Failed to send next track command");
                }
            }
        });

        // Previous track button
        self.window.on_previous_track({
            let ui_command_tx = self.ui_command_tx.clone();
            
            move || {
                debug!("Previous track button clicked");
                
                if let Err(_) = ui_command_tx.send(UiCommand::PreviousTrack) {
                    error!("Failed to send previous track command");
                }
            }
        });

        // Skip backward (10 seconds)
        self.window.on_skip_backward({
            let ui_command_tx = self.ui_command_tx.clone();
            
            move || {
                debug!("Skip backward button clicked");
                
                if let Err(_) = ui_command_tx.send(UiCommand::SkipBackward(10.0)) {
                    error!("Failed to send skip backward command");
                }
            }
        });

        // Skip forward (10 seconds)
        self.window.on_skip_forward({
            let ui_command_tx = self.ui_command_tx.clone();
            
            move || {
                debug!("Skip forward button clicked");
                
                if let Err(_) = ui_command_tx.send(UiCommand::SkipForward(10.0)) {
                    error!("Failed to send skip forward command");
                }
            }
        });

        // Seek control
        self.window.on_seek({
            let ui_command_tx = self.ui_command_tx.clone();
            
            move |position| {
                debug!("Seek to position: {:.2}", position);
                
                if let Err(_) = ui_command_tx.send(UiCommand::Seek(position)) {
                    error!("Failed to send seek command");
                }
            }
        });

        self.window.on_shuffle_toggled(|| debug!("Shuffle toggled"));
        self.window.on_repeat_toggled(|| debug!("Repeat toggled"));
        self.window.on_playlist_load_files(|| debug!("Playlist load files"));
        self.window.on_playlist_load_folder(|| debug!("Playlist load folder"));
        self.window.on_playlist_save(|| debug!("Playlist save"));
        self.window.on_playlist_clear(|| debug!("Playlist clear"));
        self.window.on_playlist_toggle_collapsed(|| debug!("Playlist toggle collapsed"));
        self.window.on_visualizer_changed(|_| debug!("Visualizer changed"));
        self.window.on_visualizer_sensitivity_changed(|_| debug!("Visualizer sensitivity changed"));
        self.window.on_visualizer_smoothing_changed(|_| debug!("Visualizer smoothing changed"));
        self.window.on_visualizer_preset_selected(|_| debug!("Visualizer preset selected"));
        self.window.on_fullscreen_toggled(|| debug!("Fullscreen toggled"));
    }

    /// Set initial UI state
    fn set_initial_state(&self) {
        // Set default values
        self.window.set_is_playing(false);
        self.window.set_is_paused(false);
        self.window.set_volume(0.8);
        self.window.set_track_title("No track loaded".into());
        self.window.set_track_artist("".into());
        self.window.set_track_album("".into());
        self.window.set_track_year("".into());
        self.window.set_track_genre("".into());
        self.window.set_position_text("00:00".into());
        self.window.set_duration_text("00:00".into());
        self.window.set_progress(0.0);
        self.window.set_file_format("".into());
        self.window.set_sample_rate("".into());
        self.window.set_channels("".into());
        self.window.set_bit_depth("".into());
        self.window.set_bitrate("".into());
        self.window.set_visualizer_sensitivity(1.0);
        self.window.set_visualizer_smoothing(0.5);
        self.window.set_visualizer_type("Spectrum Bars".into());
        self.window.set_playlist_collapsed(false);
        self.window.set_playback_state("Ready".into());
    }

    /// Run the UI main loop
    pub fn run(&self) -> Result<()> {
        info!("Starting main UI loop");
        self.window.run().map_err(|e| {
            sonic_core::Error::Application(format!("UI main loop error: {}", e))
        })
    }

    /// Open file dialog synchronously
    fn open_file_dialog_sync() -> Result<Option<PathBuf>> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use rfd::FileDialog;

            let file = FileDialog::new()
                .add_filter("Audio Files", &["mp3", "flac", "wav", "ogg", "m4a", "aac"])
                .add_filter("MP3 Files", &["mp3"])
                .add_filter("FLAC Files", &["flac"])
                .add_filter("WAV Files", &["wav"])
                .add_filter("OGG Files", &["ogg"])
                .add_filter("All Files", &["*"])
                .set_title("Select Audio File")
                .pick_file();

            Ok(file)
        }

        #[cfg(target_arch = "wasm32")]
        {
            Err(crate::error::Error::Application(
                "File dialog not supported on WASM".to_string(),
            ))
        }
    }

    /// Get window reference
    pub fn window(&self) -> &MainWindow {
        &self.window
    }
}
