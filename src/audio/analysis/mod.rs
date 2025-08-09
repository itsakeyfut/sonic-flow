//! Audio analysis and spectrum processing

pub mod fft;
pub mod spectrum;
pub mod meter;

use std::time::Duration;

/// Spectrum analysis data
#[derive(Debug, Clone)]
pub struct SpectrumData {
    /// Frequency bands (typically 32, 64, or 128)
    pub bands: Vec<f32>,
    /// Peak level (-1.0 to 1.0)
    pub peak_level: f32,
    /// RMS level (0.0 to 1.0)
    pub rms_level: f32,
    /// Timestamp when data was captured
    pub timestamp: std::time::Instant,
}
