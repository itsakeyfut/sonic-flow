//! Crossfade effect implementation

use super::AudioEffect;
use crate::error::AudioError;
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

impl AudioEffect for Crossfade {
    fn process(
        &mut self,
        buffer: &mut [f32],
        sample_rate: u32,
        _channels: u16,
    ) -> Result<(), AudioError> {
        if self.fade_state == CrossfadeState::Inactive {
            return Ok(());
        }

        self.sample_rate = sample_rate;
        let samples_per_ms = sample_rate as f32 / 1000.0;
        let fade_samples = (self.fade_duration.as_millis() as f32 * samples_per_ms) as usize;

        if fade_samples == 0 {
            return Ok(());
        }

        let fade_increment = 1.0 / fade_samples as f32;

        for sample in buffer.iter_mut() {
            let multiplier = match self.fade_state {
                CrossfadeState::FadingOut => 1.0 - self.fade_position,
                CrossfadeState::FadingIn => self.fade_position,
                CrossfadeState::Inactive => 1.0,
            };

            *sample *= multiplier;

            self.fade_position += fade_increment;

            if self.fade_position >= 1.0 {
                self.fade_state = CrossfadeState::Inactive;
                self.fade_position = 0.0;
                break;
            }
        }

        Ok(())
    }

    fn reset(&mut self) {
        self.fade_state = CrossfadeState::Inactive;
        self.fade_position = 0.0;
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}
