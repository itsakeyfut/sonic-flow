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
