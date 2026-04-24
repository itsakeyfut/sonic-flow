//! UI–Rust Binding
//!
//! Connects Slint UI components with Rust business logic.

use slint::{ComponentHandle, Weak};
use std::time::Duration;
use tracing::{debug, error, info, warn};

use sonic_core::audio::engine::TrackInfo;
use sonic_core::{Result, Error as UiError};

// Include Slint-generated components
slint::include_modules!();

/// UI binding for the main window
pub struct MainWindowBinding {
    /// Slint main window instance
    window: MainWindow,

    /// Weak reference for use in callbacks
    weak_window: Weak<MainWindow>,


}

impl MainWindowBinding {
    /// Create a new main window binding
    pub fn new() -> Result<Self> {
        info!("Creating main window UI binding");

        let window = MainWindow::new()
            .map_err(|e| UiError::Application(format!("Failed to create main window: {}", e)))?;

        let weak_window = window.as_weak();

        let mut binding = Self {
            window,
            weak_window,
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

        // Play/Pause button
        self.window.on_play_pause_clicked(move || {
            debug!("Play/pause button clicked");
            // TODO: Implement event handling
        });

        // Stop button
        self.window.on_stop_clicked(move || {
            debug!("Stop button clicked");
            // TODO: Implement event handling
        });

        // Next track button
        self.window.on_next_track(move || {
            debug!("Next track button clicked");
            // TODO: Implement event handling
        });

        // Previous track button
        self.window.on_previous_track(move || {
            debug!("Previous track button clicked");
            // TODO: Implement event handling
        });

        // Volume change
        self.window.on_volume_changed(move |volume| {
            debug!("Volume changed to: {:.2}", volume);
            // TODO: Implement event handling
        });

        // Seek operation
        self.window.on_seek(move |position| {
            debug!("Seek to position: {:.2}", position);
            // TODO: Implement event handling
        });

        // Visualizer change
        self.window.on_visualizer_changed(move |visualizer_type| {
            debug!("Visualizer changed to: {}", visualizer_type);
            // TODO: Implement event handling
        });

        // Visualizer sensitivity change
        self.window.on_visualizer_sensitivity_changed(move |sensitivity| {
            debug!("Visualizer sensitivity changed to: {:.2}", sensitivity);
            // TODO: Implement event handling
        });

        // Load track button - simplified to just publish event
        self.window.on_load_track_clicked(move || {
            debug!("Load track button clicked");
            // TODO: Implement event handling
        });

        // Fullscreen toggle
        self.window.on_fullscreen_toggled({
            move || {
                debug!("Fullscreen toggle clicked");
                warn!("Fullscreen visualizer not yet implemented");
            }
        });

        // Skip controls - add new handlers for 10 second skip
        self.window.on_skip_backward(move || {
            debug!("Skip backward button clicked");
            // TODO: Implement event handling
        });

        self.window.on_skip_forward(move || {
            debug!("Skip forward button clicked");
            // TODO: Implement event handling
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
        self.window.set_visualizer_smoothing(0.8);

        // Playlist state
        self.window.set_playlist_collapsed(false);
        self.window.set_current_track_index(-1);
        self.window.set_total_tracks(0);
        self.window.set_total_duration("00:00".into());

        // Audio quality info
        self.window.set_file_format("".into());
        self.window.set_sample_rate("".into());
        self.window.set_bit_depth("".into());
        self.window.set_bitrate("".into());
        self.window.set_channels("".into());

        // Track metadata
        self.window.set_track_title("".into());
        self.window.set_track_artist("".into());
        self.window.set_track_album("".into());
        self.window.set_track_year("".into());
        self.window.set_track_genre("".into());

        debug!("Initial UI state set");
    }

    /// Update playback state in the UI
    pub fn update_playback_state(&self, is_playing: bool, is_paused: bool, state_text: &str) {
        self.window.set_is_playing(is_playing);
        self.window.set_is_paused(is_paused);
        self.window.set_playback_state(state_text.into());

        debug!(
            "Updated playback state: playing={}, paused={}, state={}",
            is_playing, is_paused, state_text
        );
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
            let format = track
                .path
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("Unknown")
                .to_uppercase();

            self.window.set_file_format(format.into());
            self.window.set_sample_rate("44.1 kHz".into()); // TODO: Get from track
            self.window.set_bit_depth("16 bit".into()); // TODO: Get from track
        } else {
            self.window.set_file_format("".into());
            self.window.set_sample_rate("".into());
            self.window.set_bit_depth("".into());
        }

        debug!("Updated current track: {:?}", track_info.map(|t| &t.title));
    }

    /// Clear track metadata
    pub fn clear_track_metadata(&self) {
        self.window.set_track_title("".into());
        self.window.set_track_artist("".into());
        self.window.set_track_album("".into());
        self.window.set_track_year("".into());
        self.window.set_track_genre("".into());
    }

    /// Update audio quality information
    pub fn update_audio_quality(&self, format: &str, sample_rate: u32, bit_depth: u16, bitrate: Option<u32>, channels: u16) {
        self.window.set_file_format(format.into());
        self.window.set_sample_rate(format!("{} Hz", sample_rate).into());
        self.window.set_bit_depth(format!("{} bit", bit_depth).into());
        self.window.set_channels(format!("{}", channels).into());
        
        if let Some(bitrate) = bitrate {
            self.window.set_bitrate(format!("{} kbps", bitrate / 1000).into());
        } else {
            self.window.set_bitrate("Lossless".into());
        }
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
            .set_position_text(Self::format_duration(position).into());
        self.window
            .set_duration_text(Self::format_duration(duration).into());

        debug!(
            "Updated progress: {:.2}% ({:?}/{:?})",
            progress * 100.0,
            position,
            duration
        );
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
        debug!(
            "Updated visualizer type: {} ({})",
            display_name, visualizer_type
        );
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
            .map_err(|e| UiError::Application(format!("Failed to show window: {}", e)))?;
        info!("Main window shown");
        Ok(())
    }

    /// Hide the main window
    pub fn hide(&self) -> Result<()> {
        self.window
            .hide()
            .map_err(|e| UiError::Application(format!("Failed to hide window: {}", e)))?;
        info!("Main window hidden");
        Ok(())
    }

    /// Run the main window event loop (blocking)
    pub fn run(&self) -> Result<()> {
        info!("Running main window");
        self.window
            .run()
            .map_err(|e| UiError::Application(format!("Window run failed: {}", e)))?;
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
    pub fn update_from_audio_status(&self, status: &sonic_core::audio::engine::AudioEngineStatus) {
        use sonic_core::audio::traits::PlaybackState;

        // Update playback state
        let (is_playing, is_paused, state_text) = match status.state {
            PlaybackState::Playing => (true, false, "Playing"),
            PlaybackState::Paused => (false, true, "Paused"),
            PlaybackState::Stopped => (false, false, "Stopped"),
            PlaybackState::Buffering => (false, false, "Buffering"),
            PlaybackState::Error(ref _e) => (false, false, "Error"),
        };

        self.update_playback_state(is_playing, is_paused, state_text);
        self.update_volume(status.volume);

        if let Some(duration) = status.duration {
            self.update_progress(status.position, duration);
        }

        debug!("Updated UI from audio status: {:?}", status.state);
    }

    /// Show an error message in the UI
    pub fn show_error(&self, error: &str) {
        error!("UI Error: {}", error);
        // TODO: Implement actual error display in UI
    }

    /// Show an info message in the UI
    pub fn show_info(&self, message: &str) {
        info!("UI Info: {}", message);
        // TODO: Implement actual info display in UI
    }

    /// Format a `Duration` as `mm:ss`
    fn format_duration(duration: Duration) -> String {
        let total_seconds = duration.as_secs();
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }
}
