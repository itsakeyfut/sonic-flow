//! Reverb effect implementation

use super::AudioEffect;
use crate::error::AudioError;
use crate::Result;

/// Reverb configuration
#[derive(Debug, Clone)]
pub struct ReverbConfig {
    /// Room size (0.0 to 1.0)
    pub room_size: f32,
    /// Damping factor (0.0 to 1.0)
    pub damping: f32,
    /// Wet level (0.0 to 1.0)
    pub wet_level: f32,
    /// Dry level (0.0 to 1.0)
    pub dry_level: f32,
}

impl Default for ReverbConfig {
    fn default() -> Self {
        Self {
            room_size: 0.5,
            damping: 0.5,
            wet_level: 0.3,
            dry_level: 0.7,
        }
    }
}
