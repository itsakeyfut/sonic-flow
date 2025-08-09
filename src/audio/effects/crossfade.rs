//! Crossfade effect implementation

use super::AudioEffect;
use crate::error::AudioError;
use crate::Result;
use std::time::Duration;

/// Crossfade effect for smooth track transitions
pub struct Crossfade {
    /// Crossfade duration
    fade_duration: Duration,
    /// Current fade state
    fade_state: CrossfadeState,
    /// Fade position (0.0 to 1.0)
    fade_position: f32,
    /// Sample rate for timing calculations
    sample_rate: u32,
    /// Enabled state
    enabled: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum CrossfadeState {
    Inactive,
    FadingOut,
    FadingIn,
}

impl Crossfade {
    /// Create a new crossfade effect
    pub fn new(fade_duration: Duration) -> Self {
        Self {
            fade_duration,
            fade_state: CrossfadeState::Inactive,
            fade_position: 0.0,
            sample_rate: 44100,
            enabled: true,
        }
    }

    /// Start a fade out
    pub fn start_fade_out(&mut self) {
        self.fade_state = CrossfadeState::FadingOut;
        self.fade_position = 0.0;
    }

    /// Start a fade in
    pub fn start_fade_in(&mut self) {
        self.fade_state = CrossfadeState::FadingIn;
        self.fade_position = 0.0;
    }

    /// Check if currently fading
    pub fn is_fading(&self) -> bool {
        self.fade_state != CrossfadeState::Inactive
    }

    /// Set fade duration
    pub fn set_fade_duration(&mut self, duration: Duration) {
        self.fade_duration = duration;
    }

    /// Get fade duration
    pub fn fade_duration(&self) -> Duration {
        self.fade_duration
    }
}
