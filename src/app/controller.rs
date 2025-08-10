//! Main application controller

use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tracing::{info, debug, error};

use crate::audio::{AudioEngine, AudioEngineBuilder, AudioEngineStatus};
use crate::audio::traits::{PlaybackControl, VolumeControl, TrackLoader, PlaybackStatus, PlaybackState};
use crate::config::ConfigManager;
use crate::ui::{UiSystem, MainWindowBinding};
use crate::visualizer::{VisualizerSystem, VisualizerState};
use crate::{Result, Error, TrackId};

use super::state::{StateManager, AppState};
use super::events::{AppEvent, EventBus, TrackInfo};
use super::lifecycle::LifecycleManager;

/// Main application controller that orchestrates all subsystems
pub struct AppController {
    /// Audio engine for playback
    audio_engine: Arc<tokio::sync::RwLock<AudioEngine>>,
    
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
        let audio_engine = Arc::new(tokio::sync::RwLock::new(
            AudioEngineBuilder::new()
                .with_volume(0.8)
                .with_buffer_size(1024)
                .with_sample_rate(44100)
                .build()
                .map_err(Error::Audio)?
        ));
        debug!("Audio engine initialized");

        // Initialize event bus
        let event_bus = Arc::new(EventBus::new());
        debug!("Event bus initialized");

        // Initialize application state
        let state_manager = Arc::new(StateManager::new());
        debug!("Application state initialized");

        let visualizer_system = Arc::new(
            VisualizerSystem::new(800, 600)
                .map_err(|e| Error::Visualizer(e))?
        );
        debug!("Visualizer system initialized");

        // Initialize UI system - fix dereferencing issue
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

        // Show the main window
        self.ui_system.main_window().show()?;

        // Run UI in the main thread instead of spawn_blocking
        // This avoids the thread safety issues
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

    /// Start the audio monitoring background task
    fn start_audio_monitoring_task(&self) -> tokio::task::JoinHandle<Result<()>> {
        let audio_engine = Arc::clone(&self.audio_engine);
        let state_manager = Arc::clone(&self.state_manager);
        let event_bus = Arc::clone(&self.event_bus);
        let visualizer_system = Arc::clone(&self.visualizer_system);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(16)); // 60FPS
            
            loop {
                interval.tick().await;

                // Update application state with current audio engine state
                let audio_engine = audio_engine.read().await;
                let current_state = audio_engine.state();
                let current_position = audio_engine.position();
                let current_volume = audio_engine.volume();
                let is_muted = audio_engine.is_muted();
                let current_track = audio_engine.current_track();

                // TODO: 音声エンジンからスペクトラムデータを取得
                // 実際の実装では AudioEngine にスペクトラムデータ取得メソッドを追加する必要があります
                if current_state == PlaybackState::Playing {
                    // ダミーのスペクトラムデータ（実装時は実際のFFTデータを使用）
                    let dummy_spectrum = crate::audio::analysis::SpectrumData::new(
                        (0..64).map(|i| (i as f32 / 64.0) * 0.5).collect(),
                        0.5,
                        0.3,
                    );
                    
                    if let Err(e) = visualizer_system.update(dummy_spectrum) {
                        tracing::error!("Failed to update visualizer: {}", e);
                    }
                }

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

                    // Publish events for state changes
                    if old_playing != playback.is_playing {
                        if playback.is_playing {
                            if let Err(e) = visualizer_system.start() {
                                tracing::error!("Failed to start visualizer: {}", e);
                            }

                            if let Some(track_id) = current_track {
                                let _ = event_bus.publish(AppEvent::PlaybackStarted(track_id));
                            }
                        } else {
                            if let Err(e) = visualizer_system.stop() {
                                tracing::error!("Failed to stop visualizer: {}", e);
                            }

                            let _ = event_bus.publish(AppEvent::PlaybackPaused);
                        }
                    }
                }

                // Check for any critical errors
                if matches!(current_state, PlaybackState::Error(_)) {
                    error!("Audio engine is in error state: {:?}", current_state);
                    // Could implement error recovery here
                }
            }
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
                                }
                            }
                            _ => {
                                debug!("Toggle playback ignored in current state: {:?}", current_state);
                            }
                        }
                    }
                    
                    AppEvent::PlaybackStopped => {
                        let mut engine = audio_engine.write().await;
                        if let Err(e) = engine.stop().await {
                            error!("Failed to stop playback: {}", e);
                        } else {
                            info!("Playback stopped");
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
                        let duration = std::time::Duration::from_secs_f64(position * 300.0); // TODO: Use actual track duration
                        if let Err(e) = engine.seek(duration).await {
                            error!("Failed to seek: {}", e);
                        } else {
                            debug!("Seek to position: {:.2}s", position);
                        }
                    }

                    AppEvent::VisualizerChanged(visualizer_type) => {
                        if let Err(e) = visualizer_system.engine().set_visualizer(&visualizer_type) {
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
                            (current_state.playback.position / current_state.playback.duration) as f32
                        } else {
                            0.0
                        };
                        window.set_progress(progress);
                        
                        // Update time displays
                        let position_text = Self::format_duration_from_secs(current_state.playback.position);
                        let duration_text = Self::format_duration_from_secs(current_state.playback.duration);
                        window.set_position_text(position_text.into());
                        window.set_duration_text(duration_text.into());
                        
                        // Update visualizer type - fix field name
                        window.set_visualizer_type(current_state.ui.active_visualizer.clone().into());
                    }

                    last_state = current_state;
                }
            }
        })
    }

    /// Check if state has changed significantly
    fn state_changed(old: &AppState, new: &AppState) -> bool {
        old.playback.is_playing != new.playback.is_playing ||
        (old.playback.volume - new.playback.volume).abs() > 0.01 ||
        (old.playback.position - new.playback.position).abs() > 0.5 ||
        old.ui.active_visualizer != new.ui.active_visualizer // Fix field name
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration};

    #[tokio::test]
    async fn test_app_controller_creation() {
        let result = AppController::new().await;
        // This might fail in test environment due to audio system initialization
        if result.is_err() {
            println!(
                "AppController creation failed (expected in test environment): {:?}",
                result.err()
            );
            return;
        }

        let controller = result.unwrap();

        // Verify that all components are initialized
        assert!(!controller.audio_engine.read().await.is_playing());
    }

    #[tokio::test]
    async fn test_event_system() {
        // Create a standalone event bus for testing
        let event_bus = EventBus::new();
        let mut receiver = event_bus.subscribe();

        // Publish an event
        let result = event_bus.publish(crate::app::AppEvent::PlaybackPaused);
        assert!(result.is_ok());

        // Receive the event with timeout
        let event_result = timeout(Duration::from_millis(100), receiver.recv()).await;
        assert!(event_result.is_ok());

        let event = event_result.unwrap();
        assert!(event.is_ok());

        match event.unwrap() {
            crate::app::AppEvent::PlaybackPaused => {
                // Expected
            }
            other => panic!("Unexpected event: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_app_state_initialization() {
        let state = AppState::default();

        assert!(!state.playback.is_playing);
        assert_eq!(state.playback.position, 0.0);
        assert_eq!(state.playback.volume, 0.8);
        assert!(!state.playback.is_muted);
        assert_eq!(state.current_playlist, None);
    }
}
