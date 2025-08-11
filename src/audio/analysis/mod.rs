//! Audio analysis and spectrum processing

pub mod fft;
// pub mod spectrum;
// pub mod meter;

use std::time::Duration;
pub use fft::SpectrumAnalyzer;

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

impl SpectrumData {
    /// Create new spectrum data
    pub fn new(bands: Vec<f32>, peak_level: f32, rms_level: f32) -> Self {
        Self {
            bands,
            peak_level,
            rms_level,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Get the number of frequency bands
    pub fn band_count(&self) -> usize {
        self.bands.len()
    }

    /// Check if the data is recent (within specified age)
    pub fn is_recent(&self, max_age: Duration) -> bool {
        self.timestamp.elapsed() <= max_age
    }
}
