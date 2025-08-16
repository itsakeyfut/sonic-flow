//! Main application controller

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::audio::traits::{
    PlaybackControl, PlaybackState, PlaybackStatus, TrackLoader, VolumeControl,
};
use crate::audio::{AudioEngine, AudioEngineBuilder};
use crate::config::ConfigManager;
use crate::ui::UiSystem;
use crate::visualizer::VisualizerSystem;
use crate::{Error, Result};

use super::events::{AppEvent, EventBus};
use super::lifecycle::LifecycleManager;
use super::state::{AppState, StateManager};

/// Main application controller that orchestrates all subsystems
pub struct AppController {
    /// Audio engine for playback
    audio_engine: Arc<RwLock<AudioEngine>>,

    /// Configuration manager
    config_manager: Arc<ConfigManager>,

    /// Event bus for inter-component communication
    event_bus: Arc<EventBus>,

    /// Application state
    state_manager: Arc<StateManager>,

    /// UI system
    ui_system: UiSystem,

    /// Visualizer system
    visualizer_system: Arc<VisualizerSystem>,
}

impl AppController {
    /// Create a new application controller
    pub async fn new() -> Result<Self> {
        info!("Initializing Sonic Flow application controller");

        // Initialize lifecycle manager and perform startup
        let lifecycle_manager = LifecycleManager::new();
        lifecycle_manager.startup().await?;

        // Initialize configuration manager
        let config_manager = Arc::new(ConfigManager::new()?);
        debug!("Configuration manager initialized");

        // Initialize audio engine with configuration
        let audio_engine = Arc::new(RwLock::new(
            AudioEngineBuilder::new()
                .with_volume(0.8)
                .with_buffer_size(1024)
                .with_sample_rate(44100)
                .build()
                .map_err(Error::Audio)?,
        ));
        debug!("Audio engine initialized");

        // Initialize event bus
        let event_bus = Arc::new(EventBus::new());
        debug!("Event bus initialized");

        // Initialize application state
        let state_manager = Arc::new(StateManager::new());
        debug!("Application state initialized");

        let visualizer_system =
            Arc::new(VisualizerSystem::new(800, 600).map_err(|e| Error::Visualizer(e))?);
        debug!("Visualizer system initialized");

        // Initialize UI system - fixed to use single event bus reference
        let ui_system = UiSystem::new(EventBus::clone(&event_bus))?;
        debug!("UI system initialized");

        Ok(Self {
            audio_engine,
            config_manager,
            event_bus,
            state_manager,
            ui_system,
            visualizer_system,
        })
    }

    /// Run the application main loop
    pub async fn run(mut self) -> Result<()> {
        info!("Starting Sonic Flow application controller main loop");

        // Create lifecycle manager for shutdown handling
        let lifecycle_manager = LifecycleManager::new();

        // Start background tasks
        let audio_task = self.start_audio_monitoring_task();
        let event_task = self.start_event_processing_task();
        let ui_update_task = self.start_ui_update_task();
        let spectrum_task = self.start_spectrum_analysis_task();

        // Show the main window
        self.ui_system.main_window().show()?;

        // Run UI in the main thread
        let ui_result = tokio::select! {
            result = audio_task => {
                error!("Audio task ended unexpectedly: {:?}", result);
                Err(Error::Application("Audio task ended".to_string()))
            }
            result = event_task => {
                error!("Event task ended unexpectedly: {:?}", result);
                Err(Error::Application("Event task ended".to_string()))
            }
            result = ui_update_task => {
                error!("UI update task ended unexpectedly: {:?}", result);
                Err(Error::Application("UI update task ended".to_string()))
            }
            result = spectrum_task => {
                error!("Spectrum task ended unexpectedly: {:?}", result);
                Err(Error::Application("Spectrum task ended".to_string()))
            }
            // Run UI in current thread
            ui_result = async {
                self.ui_system.run()
            } => {
                match ui_result {
                    Ok(()) => {
                        info!("UI completed normally");
                        Ok(())
                    }
                    Err(e) => {
                        error!("UI error: {}", e);
                        Err(e)
                    }
                }
            }
        };

        // Perform cleanup
        self.shutdown().await?;

        info!("Sonic Flow application controller main loop completed");
        ui_result
    }

    /// Open a file dialog to select an audio file
    async fn open_file_dialog() -> Result<Option<std::path::PathBuf>> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use rfd::AsyncFileDialog;

            let file = AsyncFileDialog::new()
                .add_filter("Audio Files", &["mp3", "flac", "wav", "ogg", "m4a", "aac"])
                .add_filter("MP3 Files", &["mp3"])
                .add_filter("FLAC Files", &["flac"])
                .add_filter("WAV Files", &["wav"])
                .add_filter("All Files", &["*"])
                .set_title("Select Audio File")
                .pick_file()
                .await;

            Ok(file.map(|f| f.path().to_path_buf()))
        }

        #[cfg(target_arch = "wasm32")]
        {
            Err(Error::Application(
                "File dialog not supported on WASM".to_string(),
            ))
        }
    }

    /// Start the audio monitoring background task
    fn start_audio_monitoring_task(&self) -> tokio::task::JoinHandle<Result<()>> {
        let audio_engine = Arc::clone(&self.audio_engine);
        let state_manager = Arc::clone(&self.state_manager);
        let event_bus = Arc::clone(&self.event_bus);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(100)); // 10Hz for status updates

            loop {
                interval.tick().await;

                // Update application state with current audio engine state
                let audio_engine = audio_engine.read().await;
                let current_state = audio_engine.state();
                let current_position = audio_engine.position();
                let current_volume = audio_engine.volume();
                let is_muted = audio_engine.is_muted();
                let current_track = audio_engine.current_track();
                let duration = audio_engine.duration();

                drop(audio_engine);

                // Update app state
                {
                    let mut state = state_manager.write();
                    let playback = &mut state.playback;

                    let old_playing = playback.is_playing;
                    playback.is_playing = current_state == PlaybackState::Playing;
                    playback.position = current_position.as_secs_f64();
                    playback.volume = current_volume;
                    playback.is_muted = is_muted;
                    playback.current_track = current_track;

                    if let Some(dur) = duration {
                        playback.duration = dur.as_secs_f64();
                    }

                    // Publish events for state changes
                    if old_playing != playback.is_playing {
                        if playback.is_playing {
                            if let Some(track_id) = current_track {
                                let _ = event_bus.publish(AppEvent::PlaybackStarted(track_id));
                            }
                        } else {
                            let _ = event_bus.publish(AppEvent::PlaybackPaused);
                        }
                    }
                }

                // Check for any critical errors
                if matches!(current_state, PlaybackState::Error(_)) {
                    error!("Audio engine is in error state: {:?}", current_state);
                }
            }
        })
    }

    /// Start the spectrum analysis background task
    fn start_spectrum_analysis_task(&self) -> tokio::task::JoinHandle<Result<()>> {
        let audio_engine = Arc::clone(&self.audio_engine);
        let visualizer_system = Arc::clone(&self.visualizer_system);

        tokio::spawn(async move {
            // Subscribe to spectrum data from audio engine
            let mut spectrum_receiver = {
                let engine = audio_engine.read().await;
                engine.subscribe_spectrum()
            };

            info!("Spectrum analysis task started");

            // Process spectrum data and send to visualizer
            while let Ok(spectrum_data) = spectrum_receiver.recv().await {
                debug!(
                    "Received spectrum data with {} bands",
                    spectrum_data.bands.len()
                );

                // Update visualizer with spectrum data
                if let Err(e) = visualizer_system.update(spectrum_data) {
                    error!("Failed to update visualizer with spectrum data: {}", e);
                } else {
                    debug!("Successfully updated visualizer with spectrum data");
                }
            }

            warn!("Spectrum receiver channel closed");
            Ok(())
        })
    }

    /// Start the event processing background task
    fn start_event_processing_task(&self) -> tokio::task::JoinHandle<Result<()>> {
        let event_bus = Arc::clone(&self.event_bus);
        let audio_engine = Arc::clone(&self.audio_engine);
        let state_manager = Arc::clone(&self.state_manager);
        let visualizer_system = Arc::clone(&self.visualizer_system);

        tokio::spawn(async move {
            let mut event_receiver = event_bus.subscribe();

            while let Ok(event) = event_receiver.recv().await {
                match event {
                    AppEvent::LoadTrackRequested => {
                        debug!("Load track requested, opening file dialog");

                        // Handle file dialog in a separate task
                        let audio_engine = Arc::clone(&audio_engine);
                        let event_bus = Arc::clone(&event_bus);

                        tokio::spawn(async move {
                            match Self::open_file_dialog().await {
                                Ok(Some(path)) => {
                                    info!("Selected file: {}", path.display());

                                    // Load track into audio engine
                                    match audio_engine.write().await.load_track(&path).await {
                                        Ok(track_id) => {
                                            info!("Track loaded successfully: {}", track_id);

                                            // Set as current track
                                            if let Err(e) = audio_engine
                                                .write()
                                                .await
                                                .set_current_track(track_id)
                                                .await
                                            {
                                                error!("Failed to set current track: {}", e);
                                                return;
                                            }

                                            // Get track info and publish event
                                            if let Some(track_info) =
                                                audio_engine.read().await.get_track_info(track_id)
                                            {
                                                let app_track_info =
                                                    crate::app::events::TrackInfo {
                                                        id: track_info.id,
                                                        title: track_info.title.clone(),
                                                        artist: track_info.artist.clone(),
                                                        album: track_info.album.clone(),
                                                        duration: track_info
                                                            .duration
                                                            .unwrap_or(std::time::Duration::ZERO),
                                                        file_path: track_info.path.clone(),
                                                    };

                                                if let Err(e) = event_bus
                                                    .publish(AppEvent::TrackChanged(app_track_info))
                                                {
                                                    error!(
                                                        "Failed to publish track changed event: {}",
                                                        e
                                                    );
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to load track: {}", e);
                                        }
                                    }
                                }
                                Ok(None) => {
                                    debug!("File dialog cancelled");
                                }
                                Err(e) => {
                                    error!("File dialog error: {}", e);
                                }
                            }
                        });
                    }

                    AppEvent::TogglePlayback => {
                        let mut engine = audio_engine.write().await;
                        let current_state = engine.state();

                        match current_state {
                            PlaybackState::Playing => {
                                if let Err(e) = engine.pause().await {
                                    error!("Failed to pause playback: {}", e);
                                } else {
                                    info!("Playback paused");
                                }
                            }
                            PlaybackState::Paused | PlaybackState::Stopped => {
                                if let Err(e) = engine.play().await {
                                    error!("Failed to start playback: {}", e);
                                } else {
                                    info!("Playback started");
                                    // Start visualizer when playing
                                    if let Err(e) = visualizer_system.start() {
                                        error!("Failed to start visualizer: {}", e);
                                    }
                                }
                            }
                            _ => {
                                debug!(
                                    "Toggle playback ignored in current state: {:?}",
                                    current_state
                                );
                            }
                        }
                    }

                    AppEvent::PlaybackStopped => {
                        let mut engine = audio_engine.write().await;
                        if let Err(e) = engine.stop().await {
                            error!("Failed to stop playback: {}", e);
                        } else {
                            info!("Playback stopped");
                            // Stop visualizer
                            if let Err(e) = visualizer_system.stop() {
                                error!("Failed to stop visualizer: {}", e);
                            }
                        }
                    }

                    AppEvent::NextTrack => {
                        let mut engine = audio_engine.write().await;
                        if let Err(e) = engine.next_track().await {
                            error!("Failed to switch to next track: {}", e);
                        } else {
                            info!("Switched to next track");
                        }
                    }

                    AppEvent::PreviousTrack => {
                        let mut engine = audio_engine.write().await;
                        if let Err(e) = engine.previous_track().await {
                            error!("Failed to switch to previous track: {}", e);
                        } else {
                            info!("Switched to previous track");
                        }
                    }

                    AppEvent::VolumeChanged(volume) => {
                        let mut engine = audio_engine.write().await;
                        engine.set_volume(volume);
                        debug!("Volume changed to: {:.2}", volume);
                    }

                    AppEvent::PlaybackPositionChanged(position) => {
                        let mut engine = audio_engine.write().await;
                        let duration = engine
                            .duration()
                            .unwrap_or(std::time::Duration::from_secs(300));
                        let seek_duration =
                            std::time::Duration::from_secs_f64(position * duration.as_secs_f64());

                        if let Err(e) = engine.seek(seek_duration).await {
                            error!("Failed to seek: {}", e);
                        } else {
                            debug!("Seek to position: {:.2}s", position);
                        }
                    }

                    AppEvent::VisualizerChanged(visualizer_type) => {
                        if let Err(e) = visualizer_system.engine().set_visualizer(&visualizer_type)
                        {
                            error!("Failed to change visualizer: {}", e);
                        } else {
                            state_manager.update_ui_state(|ui| {
                                ui.active_visualizer = visualizer_type.clone();
                            });
                            info!("Visualizer changed to: {}", visualizer_type);
                        }
                    }

                    AppEvent::TrackChanged(track) => {
                        state_manager.update_player_state(|player| {
                            player.current_track = Some(track.id);
                            player.duration = track.duration.as_secs_f64();
                            player.position = 0.0;
                        });
                        info!("Track changed: {:?}", track.title);
                    }

                    AppEvent::ApplicationExit => {
                        info!("Application exit requested");
                        break;
                    }

                    _ => {
                        debug!("Received unhandled event: {:?}", event);
                    }
                }
            }

            Ok(())
        })
    }

    /// Start the UI update background task
    fn start_ui_update_task(&self) -> tokio::task::JoinHandle<Result<()>> {
        let state_manager = Arc::clone(&self.state_manager);
        let main_window_weak = self.ui_system.main_window().as_weak();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));
            let mut last_state = state_manager.get_state();

            loop {
                interval.tick().await;

                let current_state = state_manager.get_state();

                // Check if state has changed
                if Self::state_changed(&last_state, &current_state) {
                    // Update UI
                    if let Some(window) = main_window_weak.upgrade() {
                        // Update playback state
                        let is_playing = current_state.playback.is_playing;
                        let is_paused = !is_playing && current_state.playback.position > 0.0;
                        let state_text = if is_playing {
                            "Playing"
                        } else if is_paused {
                            "Paused"
                        } else {
                            "Stopped"
                        };

                        window.set_is_playing(is_playing);
                        window.set_is_paused(is_paused);
                        window.set_playback_state(state_text.into());

                        // Update volume
                        window.set_volume(current_state.playback.volume);

                        // Update progress
                        let progress = if current_state.playback.duration > 0.0 {
                            (current_state.playback.position / current_state.playback.duration)
                                as f32
                        } else {
                            0.0
                        };
                        window.set_progress(progress);

                        // Update time displays
                        let position_text =
                            Self::format_duration_from_secs(current_state.playback.position);
                        let duration_text =
                            Self::format_duration_from_secs(current_state.playback.duration);
                        window.set_position_text(position_text.into());
                        window.set_duration_text(duration_text.into());

                        // Update visualizer type
                        window
                            .set_visualizer_type(current_state.ui.active_visualizer.clone().into());

                        debug!(
                            "UI updated - Playing: {}, Volume: {:.2}",
                            is_playing, current_state.playback.volume
                        );
                    }

                    last_state = current_state;
                }
            }
        })
    }

    /// Check if state has changed significantly
    fn state_changed(old: &AppState, new: &AppState) -> bool {
        old.playback.is_playing != new.playback.is_playing
            || (old.playback.volume - new.playback.volume).abs() > 0.01
            || (old.playback.position - new.playback.position).abs() > 0.5
            || old.ui.active_visualizer != new.ui.active_visualizer
    }

    /// Format duration from seconds
    fn format_duration_from_secs(secs: f64) -> String {
        let total_seconds = secs as u64;
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    /// Shutdown the application controller
    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down Sonic Flow application controller");

        // Stop visualizer
        if let Err(e) = self.visualizer_system.stop() {
            error!("Failed to stop visualizer during shutdown: {}", e);
        }

        // Create a lifecycle manager for shutdown
        let lifecycle_manager = LifecycleManager::new();
        lifecycle_manager.shutdown().await?;

        debug!("Application controller shutdown completed");
        Ok(())
    }

    /// Get visualizer system reference
    pub fn visualizer_system(&self) -> &VisualizerSystem {
        &self.visualizer_system
    }

    /// Get current visualizer frame
    pub fn get_visualizer_frame(&self) -> Vec<u8> {
        self.visualizer_system.get_frame()
    }

    /// Get visualizer canvas size
    pub fn get_visualizer_size(&self) -> (u32, u32) {
        self.visualizer_system.size()
    }

    /// Load and play a test track for demonstration
    pub async fn load_test_track(&self, path: &std::path::Path) -> Result<()> {
        info!("Loading test track: {}", path.display());

        let mut engine = self.audio_engine.write().await;

        // Load the track
        let track_id = engine.load_track(path).await.map_err(Error::Audio)?;

        // Set as current track
        engine
            .set_current_track(track_id)
            .await
            .map_err(Error::Audio)?;

        info!("Test track loaded successfully: {}", track_id);
        Ok(())
    }
}
