//! Audio integration for UI using event loop pattern
//!
//! This module provides the audio integration logic that works with
//! existing MainWindowBinding using Slint's event loop best practices.

use slint::{invoke_from_event_loop, Timer, TimerMode, Weak};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use sonic_core::audio::player_manager::{PlayerManager, PlayerStatus};
use sonic_core::Result;
use crate::bindings::MainWindow; // Use the existing MainWindow from bindings

/// Audio control commands that can be sent to the audio system
#[derive(Debug)]
pub enum AudioCommand {
    /// Load and play a file
    LoadAndPlay(PathBuf),
    /// Toggle playback (play/pause)
    TogglePlayback,
    /// Stop playback
    Stop,
    /// Set volume (0.0 - 1.0)
    SetVolume(f32),
    /// Seek to position
    Seek(Duration),
    /// Next track
    NextTrack,
    /// Previous track
    PreviousTrack,
    /// Skip forward (seconds)
    SkipForward(f64),
    /// Skip backward (seconds)
    SkipBackward(f64),
    /// Request current status
    RequestStatus,
}

/// UI update events from the audio system
#[derive(Debug, Clone)]
pub enum UiUpdateEvent {
    /// Player status has changed
    PlayerStatusChanged(PlayerStatus),
    /// Error occurred
    ErrorOccurred(String),
    /// Track loading started
    TrackLoadingStarted(PathBuf),
    /// Track loading completed
    TrackLoadingCompleted {
        path: PathBuf,
        success: bool,
        error: Option<String>,
    },
}

/// Audio integration manager for UI
pub struct AudioIntegration {
    /// Audio command sender
    audio_command_tx: mpsc::UnboundedSender<AudioCommand>,
    /// Status update timer
    status_timer: Timer,
}

impl AudioIntegration {
    /// Create a new audio integration
    pub fn new(main_window: Weak<MainWindow>) -> Result<Self> {
        info!("Creating audio integration");

        // Create communication channels
        let (audio_command_tx, audio_command_rx) = mpsc::unbounded_channel();
        let (ui_update_tx, ui_update_rx) = mpsc::unbounded_channel();

        // Start the audio control thread
        Self::start_audio_thread(audio_command_rx, ui_update_tx);

        // Start the UI update processor
        Self::start_ui_update_processor(main_window.clone(), ui_update_rx);

        // Create and setup status timer
        let status_timer = Timer::default();
        Self::setup_status_timer(&status_timer, audio_command_tx.clone());

        let integration = Self {
            audio_command_tx,
            status_timer,
        };

        info!("Audio integration created successfully");
        Ok(integration)
    }

    /// Send an audio command
    pub fn send_command(&self, command: AudioCommand) {
        if let Err(_) = self.audio_command_tx.send(command) {
            error!("Failed to send audio command");
        }
    }

    /// Get a cloned command sender for use in other threads
    pub fn get_command_sender(&self) -> mpsc::UnboundedSender<AudioCommand> {
        self.audio_command_tx.clone()
    }

    /// Start the audio control thread
    fn start_audio_thread(
        mut command_rx: mpsc::UnboundedReceiver<AudioCommand>,
        ui_update_tx: mpsc::UnboundedSender<UiUpdateEvent>,
    ) {
        tokio::spawn(async move {
            info!("Audio control thread started");

            // Initialize player manager
            let player_manager = match PlayerManager::new() {
                Ok(manager) => Arc::new(manager),
                Err(e) => {
                    error!("Failed to create player manager: {}", e);
                    let _ = ui_update_tx.send(UiUpdateEvent::ErrorOccurred(
                        format!("Failed to initialize audio system: {}", e)
                    ));
                    return;
                }
            };

            while let Some(command) = command_rx.recv().await {
                debug!("Processing audio command: {:?}", command);

                match command {
                    AudioCommand::LoadAndPlay(path) => {
                        let _ = ui_update_tx.send(UiUpdateEvent::TrackLoadingStarted(path.clone()));

                        match player_manager.load_and_play(path.clone()).await {
                            Ok(()) => {
                                let _ = ui_update_tx.send(UiUpdateEvent::TrackLoadingCompleted {
                                    path,
                                    success: true,
                                    error: None,
                                });
                            }
                            Err(e) => {
                                let error_msg = e.to_string();
                                let _ = ui_update_tx.send(UiUpdateEvent::TrackLoadingCompleted {
                                    path,
                                    success: false,
                                    error: Some(error_msg.clone()),
                                });
                                let _ = ui_update_tx.send(UiUpdateEvent::ErrorOccurred(error_msg));
                            }
                        }
                    }

                    AudioCommand::TogglePlayback => {
                        let status = player_manager.get_status().await;
                        let result = if status.is_playing {
                            player_manager.pause().await
                        } else {
                            player_manager.play().await
                        };

                        if let Err(e) = result {
                            let _ = ui_update_tx.send(UiUpdateEvent::ErrorOccurred(e.to_string()));
                        }
                    }

                    AudioCommand::Stop => {
                        if let Err(e) = player_manager.stop().await {
                            let _ = ui_update_tx.send(UiUpdateEvent::ErrorOccurred(e.to_string()));
                        }
                    }

                    AudioCommand::SetVolume(volume) => {
                        if let Err(e) = player_manager.set_volume(volume).await {
                            let _ = ui_update_tx.send(UiUpdateEvent::ErrorOccurred(e.to_string()));
                        }
                    }

                    AudioCommand::Seek(position) => {
                        warn!("Seeking not yet implemented: {:?}", position);
                    }

                    AudioCommand::NextTrack => {
                        info!("Next track requested");
                    }

                    AudioCommand::PreviousTrack => {
                        info!("Previous track requested");
                    }

                    AudioCommand::SkipForward(seconds) => {
                        info!("Skip forward {} seconds", seconds);
                    }

                    AudioCommand::SkipBackward(seconds) => {
                        info!("Skip backward {} seconds", seconds);
                    }

                    AudioCommand::RequestStatus => {
                        let status = player_manager.get_status().await;
                        let _ = ui_update_tx.send(UiUpdateEvent::PlayerStatusChanged(status));
                    }
                }
            }

            info!("Audio control thread ended");
        });
    }

    /// Start the UI update processor using invoke_from_event_loop
    fn start_ui_update_processor(
        main_window: Weak<MainWindow>,
        mut ui_update_rx: mpsc::UnboundedReceiver<UiUpdateEvent>,
    ) {
        tokio::spawn(async move {
            info!("UI update processor started");

            while let Some(event) = ui_update_rx.recv().await {
                debug!("Processing UI update event: {:?}", event);

                // Use invoke_from_event_loop to safely update UI from background thread
                let window_weak = main_window.clone();
                let result = invoke_from_event_loop(move || {
                    if let Some(window) = window_weak.upgrade() {
                        Self::process_ui_update(&window, event);
                    }
                });

                if let Err(e) = result {
                    error!("Failed to invoke UI update from event loop: {:?}", e);
                }
            }

            info!("UI update processor ended");
        });
    }

    /// Process UI update on the main thread
    fn process_ui_update(window: &MainWindow, event: UiUpdateEvent) {
        match event {
            UiUpdateEvent::PlayerStatusChanged(status) => {
                Self::update_player_status(window, status);
            }

            UiUpdateEvent::ErrorOccurred(error_msg) => {
                error!("Audio error: {}", error_msg);
                window.set_playback_state("Error".into());
            }

            UiUpdateEvent::TrackLoadingStarted(path) => {
                info!("Loading started: {}", path.display());
                window.set_playback_state("Loading...".into());
            }

            UiUpdateEvent::TrackLoadingCompleted { path, success, error } => {
                if success {
                    info!("Loading completed: {}", path.display());
                    window.set_playback_state("Ready".into());
                } else {
                    error!("Loading failed: {} - {:?}", path.display(), error);
                    window.set_playback_state("Error".into());
                }
            }
        }
    }

    /// Update player status in UI
    fn update_player_status(window: &MainWindow, status: PlayerStatus) {
        // Update playback state
        window.set_is_playing(status.is_playing);
        window.set_is_paused(status.is_paused);

        // Update volume
        window.set_volume(status.volume);

        // Update track information
        if let Some(ref track_path) = status.current_track {
            let filename = track_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string();

            window.set_track_title(filename.into());
            window.set_track_artist("Unknown Artist".into());
            window.set_track_album("Unknown Album".into());

            // Set file format
            if let Some(ext) = track_path.extension().and_then(|s| s.to_str()) {
                window.set_file_format(ext.to_uppercase().into());
            }
        } else {
            window.set_track_title("No track loaded".into());
            window.set_track_artist("".into());
            window.set_track_album("".into());
            window.set_file_format("".into());
        }

        // Update playback state text
        let state = if status.is_playing {
            "Playing"
        } else if status.is_paused {
            "Paused"
        } else {
            "Stopped"
        };
        window.set_playback_state(state.into());

        // Update timing information
        let position_text = format!(
            "{:02}:{:02}",
            status.position.as_secs() / 60,
            status.position.as_secs() % 60
        );
        window.set_position_text(position_text.into());

        if let Some(duration) = status.duration {
            let duration_text = format!(
                "{:02}:{:02}",
                duration.as_secs() / 60,
                duration.as_secs() % 60
            );
            window.set_duration_text(duration_text.into());

            // Update progress
            if duration.as_secs() > 0 {
                let progress = status.position.as_secs_f32() / duration.as_secs_f32();
                window.set_progress(progress.clamp(0.0, 1.0));
            }
        } else {
            window.set_duration_text("--:--".into());
            window.set_progress(0.0);
        }

        // Update audio quality information
        if let Some(ref format) = status.format {
            window.set_sample_rate(format!("{} Hz", format.sample_rate).into());
            window.set_channels(format.channels.to_string().into());
            window.set_bit_depth(format!("{} bit", format.bit_depth).into());
        }
    }

    /// Setup status update timer
    fn setup_status_timer(timer: &Timer, audio_command_tx: mpsc::UnboundedSender<AudioCommand>) {
        timer.start(
            TimerMode::Repeated,
            Duration::from_millis(100), // Update every 100ms
            move || {
                // Request status update every 100ms during playback
                let _ = audio_command_tx.send(AudioCommand::RequestStatus);
            }
        );
    }
}

impl Drop for AudioIntegration {
    fn drop(&mut self) {
        info!("Audio integration dropping");
        self.status_timer.stop();
    }
}
