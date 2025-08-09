//! Main application controller

use std::sync::Arc;

use tracing::{info, debug, error};
use tokio::sync::RwLock;

use crate::audio::{AudioEngine, AudioEngineBuilder};
use crate::config::ConfigManager;
use crate::{Result, Error};

use super::{AppState, EventBus};
use super::lifecycle::LifecycleManager;

/// Main application controller that orchestrates all subsystems
pub struct AppController {
    /// Audio engine for playback
    audio_engine: Arc<RwLock<AudioEngine>>,
    /// Configuration manager
    config_manager: Arc<ConfigManager>,
    /// Event bus for inter-component communication
    event_bus: Arc<EventBus>,
    /// Application state
    app_state: Arc<RwLock<AppState>>,
    /// Lifecycle manager
    lifecycle_manager: LifecycleManager,
}

impl AppController {
    /// Create a new application controller
    pub async fn new() -> Result<Self> {
        info!("Initializing application controller");

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
                .map_err(|e| Error::Audio(e))?
        ));
        debug!("Audio engine initialized");

        // Initialize event bus
        let event_bus = Arc::new(EventBus::new());
        debug!("Event bus initialized");

        // Initialize application state
        let app_state = Arc::new(RwLock::new(AppState::default()));
        debug!("Application state initialized");

        Ok(Self {
            audio_engine,
            config_manager,
            event_bus,
            app_state,
            lifecycle_manager,
        })
    }

    /// Run the application main loop
    pub async fn run(self) -> Result<()> {
        info!("Starting application controller main loop");

        // Start background tasks
        let audio_task = self.start_audio_monitoring_task();
        let event_task = self.start_event_processing_task();

        // Wait for shutdown signal
        let shutdown_task = tokio::spawn(async move {
            self.lifecycle_manager.wait_for_shutdown().await
        });

        // Wait for either shutdown signal or critical error
        tokio::select! {
            result = audio_task => {
                match result {
                    Ok(Ok(())) => info!("Audio monitoring task completed"),
                    Ok(Err(e)) => error!("Audio monitoring task failed: {}", e),
                    Err(e) => error!("Audio monitoring task panicked: {}", e),
                }
            }
            result = event_task => {
                match result {
                    Ok(Ok(())) => info!("Event processing task completed"),
                    Ok(Err(e)) => error!("Event processing task failed: {}", e),
                    Err(e) => error!("Event processing task panicked: {}", e),
                }
            }
            result = shutdown_task => {
                match result {
                    Ok(Ok(())) => info!("Shutdown signal received"),
                    Ok(Err(e)) => error!("Shutdown monitoring failed: {}", e),
                    Err(e) => error!("Shutdown task panicked: {}", e),
                }
            }
        }

        // Perform cleanup
        self.shutdown().await?;

        info!("Application controller main loop completed");
        Ok(())
    }

    /// Start the audio monitoring background task
    fn start_audio_monitoring_task(&self) -> tokio::task::JoinHandle<Result<()>> {
        let audio_engine = Arc::clone(&self.audio_engine);
        let app_state = Arc::clone(&self.app_state);
        let event_bus = Arc::clone(&self.event_bus);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(100));
            
            loop {
                interval.tick().await;

                // Update application state with current audio engine state
                let audio_engine = audio_engine.read().await;
                let current_state = audio_engine.state();
                let current_position = audio_engine.position();
                let current_volume = audio_engine.volume();
                let is_muted = audio_engine.is_muted();
                let current_track = audio_engine.current_track();
                drop(audio_engine);

                // Update app state
                {
                    let mut state = app_state.write().await;
                    let playback = &mut state.playback;
                    
                    let old_state = playback.is_playing;
                    playback.is_playing = current_state == crate::audio::PlaybackState::Playing;
                    playback.position = current_position.as_secs_f64();
                    playback.volume = current_volume;
                    playback.is_muted = is_muted;
                    playback.current_track = current_track;

                    // Publish events for state changes
                    if old_state != playback.is_playing {
                        if playback.is_playing {
                            if let Some(track_id) = current_track {
                                let _ = event_bus.publish(crate::app::AppEvent::PlaybackStarted(track_id));
                            }
                        } else {
                            let _ = event_bus.publish(crate::app::AppEvent::PlaybackPaused);
                        }
                    }
                }

                // Check for any critical errors
                if matches!(current_state, crate::audio::PlaybackState::Error(_)) {
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

        tokio::spawn(async move {
            let mut event_receiver = event_bus.subscribe();

            while let Ok(event) = event_receiver.recv().await {
                match event {
                    crate::app::AppEvent::PlaybackStarted(track_id) => {
                        info!("Playback started for track: {}", track_id);
                        // Could trigger UI updates, logging, etc.
                    }
                    crate::app::AppEvent::PlaybackPaused => {
                        info!("Playback paused");
                    }
                    crate::app::AppEvent::PlaybackStopped => {
                        info!("Playback stopped");
                    }
                    crate::app::AppEvent::VolumeChanged(volume) => {
                        info!("Volume changed to: {}", volume);
                        // Update audio engine if the change came from UI
                        let mut engine = audio_engine.write().await;
                        engine.set_volume(volume);
                    }
                    crate::app::AppEvent::ApplicationExit => {
                        info!("Application exit requested");
                        break;
                    }
                    _ => {
                        debug!("Received event: {:?}", event);
                    }
                }
            }

            Ok(())
        })
    }

    /// Shutdown the application controller
    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down application controller");

        // Perform cleanup operations
        self.lifecycle_manager.shutdown().await?;

        debug!("Application controller shutdown completed");
        Ok(())
    }

    /// Get a reference to the audio engine
    pub fn audio_engine(&self) -> Arc<RwLock<AudioEngine>> {
        Arc::clone(&self.audio_engine)
    }

    /// Get a reference to the configuration manager
    pub fn config_manager(&self) -> Arc<ConfigManager> {
        Arc::clone(&self.config_manager)
    }

    /// Get a reference to the event bus
    pub fn event_bus(&self) -> Arc<EventBus> {
        Arc::clone(&self.event_bus)
    }

    /// Get a reference to the application state
    pub fn app_state(&self) -> Arc<RwLock<AppState>> {
        Arc::clone(&self.app_state)
    }
}
