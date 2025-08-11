//! UI–Rust Binding
//!
//! Connects Slint UI components with Rust business logic.

use slint::{ComponentHandle, Weak};
use std::borrow::Cow;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, warn};

use crate::app::events::TrackInfo;
use crate::app::{AppEvent, EventBus};
use crate::audio::traits::{PlaybackControl, PlaybackState, PlaybackStatus, VolumeControl};
use crate::audio::{AudioEngine, AudioEngineStatus};
use crate::error::{Result, UiError};

// Include Slint-generated components
slint::include_modules!();

/// UI binding for the main window
pub struct MainWindowBinding {
    /// Slint main window instance
    window: MainWindow,

    /// Weak reference for use in callbacks
    weak_window: Weak<MainWindow>,

    /// Event bus
    event_bus: EventBus,
}

impl MainWindowBinding {
    /// Create a new main window binding
    pub fn new(event_bus: EventBus) -> Result<Self> {
        info!("Creating main window UI binding");

        let window = MainWindow::new()
            .map_err(|e| UiError::Slint(format!("Failed to create main window: {}", e)))?;

        let weak_window = window.as_weak();

        let mut binding = Self {
            window,
            weak_window,
            event_bus,
        };

        // Set up UI event handlers
        binding.setup_ui_event_handlers()?;

        // Initialize UI state
        binding.set_initial_state();

        info!("Main window UI binding created successfully");
        Ok(binding)
    }

    /// Set up UI event handlers for the main window
    fn setup_ui_event_handlers(&mut self) -> Result<()> {
        debug!("Setting up UI event handlers");

        let event_bus = self.event_bus.clone();

        // Play/Pause button
        self.window.on_play_pause_clicked({
            let event_bus = event_bus.clone();
            move || {
                debug!("Play/pause button clicked");
                if let Err(e) = event_bus.publish(AppEvent::TogglePlayback) {
                    error!("Failed to publish play/pause event: {}", e);
                }
            }
        });

        // Stop button
        self.window.on_stop_clicked({
            let event_bus = event_bus.clone();
            move || {
                debug!("Stop button clicked");
                if let Err(e) = event_bus.publish(AppEvent::PlaybackStopped) {
                    error!("Failed to publish stop event: {}", e);
                }
            }
        });

        // Next track button
        self.window.on_next_track({
            let event_bus = event_bus.clone();
            move || {
                debug!("Next track button clicked");
                if let Err(e) = event_bus.publish(AppEvent::NextTrack) {
                    error!("Failed to publish next track event: {}", e);
                }
            }
        });

        // Previous track button
        self.window.on_previous_track({
            let event_bus = event_bus.clone();
            move || {
                debug!("Previous track button clicked");
                if let Err(e) = event_bus.publish(AppEvent::PreviousTrack) {
                    error!("Failed to publish previous track event: {}", e);
                }
            }
        });

        // Volume change
        self.window.on_volume_changed({
            let event_bus = event_bus.clone();
            move |volume| {
                debug!("Volume changed to: {:.2}", volume);
                if let Err(e) = event_bus.publish(AppEvent::VolumeChanged(volume)) {
                    error!("Failed to publish volume change event: {}", e);
                }
            }
        });

        // Seek operation
        self.window.on_seek({
            let event_bus = event_bus.clone();
            move |position| {
                debug!("Seek to position: {:.2}", position);
                if let Err(e) =
                    event_bus.publish(AppEvent::PlaybackPositionChanged(position as f64))
                {
                    error!("Failed to publish seek event: {}", e);
                }
            }
        });

        // Visualizer change
        self.window.on_visualizer_changed({
            let event_bus = event_bus.clone();
            move |visualizer_type| {
                debug!("Visualizer changed to: {}", visualizer_type);
                let visualizer_id = match visualizer_type.as_str() {
                    "Spectrum Bars" => "spectrum_bars",
                    "Waveform" => "waveform",
                    "Circle Spectrum" => "circle_spectrum",
                    "Particle System" => "particle_system",
                    "3D Spectrum" => "spectrum_3d",
                    "VU Meters" => "vu_meters",
                    _ => "spectrum_bars",
                };
                
                if let Err(e) =
                    event_bus.publish(AppEvent::VisualizerChanged(visualizer_id.to_string()))
                {
                    error!("Failed to publish visualizer change event: {}", e);
                }
            }
        });

        // Visualizer sensitivity change
        self.window.on_visualizer_sensitivity_changed({
            let event_bus = event_bus.clone();
            move |sensitivity| {
                debug!("Visualizer sensitivity changed to: {:.2}", sensitivity);
                if let Err(e) = event_bus.publish(AppEvent::VisualizerConfigChanged {
                    sensitivity,
                    color_scheme: "default".to_string(), // TODO: Make this configurable
                }) {
                    error!("Failed to publish sensitivity change event: {}", e);
                }
            }
        });

        // Load track button
        self.window.on_load_track_clicked({
            let event_bus = event_bus.clone();
            move || {
                debug!("Load track button clicked");
                // For now, we'll use a dummy file path
                // In a real implementation, this would open a file dialog
                let dummy_path = "/path/to/sample.mp3";
                info!("Would open file dialog to load track (placeholder: {})", dummy_path);
                
                // TODO: Implement file dialog and actual track loading
                // For demo purposes, we could load a sample file if available
            }
        });

        // Fullscreen toggle
        self.window.on_fullscreen_toggled({
            move || {
                debug!("Fullscreen toggle clicked");
                // TODO: Implement fullscreen visualizer mode
                warn!("Fullscreen visualizer not yet implemented");
            }
        });

        debug!("UI event handlers setup completed");
        Ok(())
    }

    /// Initialize the UI with default state
    fn set_initial_state(&self) {
        debug!("Setting initial UI state");

        self.window.set_is_playing(false);
        self.window.set_is_paused(false);
        self.window.set_current_track("No track loaded".into());
        self.window.set_playback_state("Stopped".into());
        self.window.set_volume(0.8);
        self.window.set_progress(0.0);
        self.window.set_position_text("0:00".into());
        self.window.set_duration_text("0:00".into());

        // Visualizer state
        self.window.set_visualizer_type("spectrum_bars".into());
        self.window.set_visualizer_sensitivity(1.0);

        // Visualizer quality info
        self.window.set_file_format("".into());
        self.window.set_sample_rate("".into());
        self.window.set_bit_depth("".into());

        debug!("Initial UI state set");
    }

    /// Update playback state in the UI
    pub fn update_playback_state(&self, is_playing: bool, is_paused: bool, state_text: &str) {
        self.window.set_is_playing(is_playing);
        self.window.set_is_paused(is_paused);
        self.window.set_playback_state(state_text.into());

        debug!("Updated playback state: playing={}, paused={}, state={}", 
                is_playing, is_paused, state_text);
    }

    /// Update the displayed current track info
    pub fn update_current_track(&self, track_info: Option<&TrackInfo>) {
        let track_text = match track_info {
            Some(track) => {
                let title = track.title.as_deref().unwrap_or("Unknown Title");
                let artist = track.artist.as_deref().unwrap_or("Unknown Artist");
                format!("{} - {}", artist, title)
            }
            None => "No track loaded".to_string(),
        };

        self.window.set_current_track(track_text.into());

        // Update audio format information if available
        if let Some(track) = track_info {
            // Extract file extension as format
            let format = track.file_path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("Unknown")
                .to_uppercase();
            
            self.window.set_file_format(format.into());
            
            // TODO: Get actual sample rate and bit depth from track metadata
            self.window.set_sample_rate("44.1 kHz".into());
            self.window.set_bit_depth("16 bit".into());
        } else {
            self.window.set_file_format("".into());
            self.window.set_sample_rate("".into());
            self.window.set_bit_depth("".into());
        }
        
        debug!("Updated current track: {:?}", track_info.map(|t| &t.title));
    }

    /// Update volume in the UI
    pub fn update_volume(&self, volume: f32) {
        self.window.set_volume(volume);
        debug!("Updated volume: {:.2}", volume);
    }

    /// Update playback progress in the UI
    pub fn update_progress(&self, position: Duration, duration: Duration) {
        let progress = if duration.as_secs() > 0 {
            position.as_secs_f32() / duration.as_secs_f32()
        } else {
            0.0
        };

        self.window.set_progress(progress);
        self.window
            .set_position_text(format_duration(position).into());
        self.window
            .set_duration_text(format_duration(duration).into());

        debug!("Updated progress: {:.2}% ({:?}/{:?})", 
               progress * 100.0, position, duration);
    }

    /// Update the selected visualizer type in the UI
    pub fn update_visualizer_type(&self, visualizer_type: &str) {
        let display_name = match visualizer_type {
            "spectrum_bars" => "Spectrum Bars",
            "waveform" => "Waveform",
            "circle_spectrum" => "Circle Spectrum",
            "particle_system" => "Particle System",
            "spectrum_3d" => "3D Spectrum",
            "vu_meters" => "VU Meters",
            _ => "Spectrum Bars",
        };
        
        self.window.set_visualizer_type(display_name.into());
        debug!("Updated visualizer type: {} ({})", display_name, visualizer_type);
    }

    /// Update visualizer sensitivity
    pub fn update_visualizer_sensitivity(&self, sensitivity: f32) {
        self.window.set_visualizer_sensitivity(sensitivity);
        debug!("Updated visualizer sensitivity: {:.2}", sensitivity);
    }

    /// Show the main window
    pub fn show(&self) -> Result<()> {
        self.window
            .show()
            .map_err(|e| UiError::Slint(format!("Failed to show window: {}", e)))?;
        info!("Main window shown");
        Ok(())
    }

    /// Hide the main window
    pub fn hide(&self) -> Result<()> {
        self.window
            .hide()
            .map_err(|e| UiError::Slint(format!("Failed to hide window: {}", e)))?;
        info!("Main window hidden");
        Ok(())
    }

    /// Run the main window event loop (blocking)
    pub fn run(&self) -> Result<()> {
        info!("Running main window");
        self.window
            .run()
            .map_err(|e| UiError::Slint(format!("Window run failed: {}", e)))?;
        Ok(())
    }

    /// Get a weak reference to the main window
    pub fn as_weak(&self) -> Weak<MainWindow> {
        self.weak_window.clone()
    }

    /// Get a reference to the main window handle
    pub fn window(&self) -> &MainWindow {
        &self.window
    }

    /// Update all UI state from an audio engine status
    pub fn update_from_audio_status(&self, status: &AudioEngineStatus) {
        // Update playback state
        let (is_playing, is_paused, state_text): (bool, bool, Cow<'static, str>) = match status.state {
            PlaybackState::Playing   => (true,  false, Cow::Borrowed("Playing")),
            PlaybackState::Paused    => (false, true,  Cow::Borrowed("Paused")),
            PlaybackState::Stopped   => (false, false, Cow::Borrowed("Stopped")),
            PlaybackState::Buffering => (false, false, Cow::Borrowed("Buffering")),
            PlaybackState::Error(ref e) => (false, false, Cow::Owned(format!("Error: {}", e))),
        };

        self.update_playback_state(is_playing, is_paused, &state_text);
        self.update_volume(status.volume);
        
        if let Some(duration) = status.duration {
            self.update_progress(status.position, duration);
        }
        
        debug!("Updated UI from audio status: {:?}", status.state);
    }
}

/// Format a `Duration` as `mm:ss`
fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}

/// Helper for safely updating UI state
pub struct UiStateUpdater {
    window_weak: Weak<MainWindow>,
}

impl UiStateUpdater {
    /// Create a new UI state updater
    pub fn new(window_weak: Weak<MainWindow>) -> Self {
        Self { window_weak }
    }

    /// Safely update the UI
    pub fn update_ui<F>(&self, f: F)
    where
        F: FnOnce(&MainWindow),
    {
        if let Some(window) = self.window_weak.upgrade() {
            f(&window);
        } else {
            warn!("Attempted to update UI but window is no longer available");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::EventBus;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(65)), "01:05");
        assert_eq!(format_duration(Duration::from_secs(3661)), "61:01");
        assert_eq!(format_duration(Duration::from_secs(0)), "00:00");
    }

    // Note: UI-related tests require the Slint runtime and
    // should be implemented as integration tests in a real environment.
}
