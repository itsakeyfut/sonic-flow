//! Spectrum bars visualizer implementations
//! 
//! This visualizer displays frequency spectrum data as vertical bars,
//! providing a classic and intuitive visualization of audio content.

use std::collections::HashMap;
use std::time::Instant;

use crate::audio::analysis::SpectrumData;
use crate::error::VisualizerError;
use crate::visualizer::traits::{
    BlendMode, Canvas, Color, ColorScheme, ConfigParameter, ParameterType, PluginValue, Point,
    Rect, VisualizationConfig, Visualizer, VisualizerMetadata,
};

/// Spectrum bars visualizer configuration
#[derive(Debug, Clone)]
pub struct SpectrumBarsConfig {
    /// Number of frequency bars to display
    pub bar_count: usize,
    /// Width ratio of bars (0.0 - 1.0, where 1.0 means no gaps)
    pub bar_width_ratio: f32,
    /// Minimum bar height (as fraction of canvas height)
    pub min_bar_height: f32,
    /// Maximum bar height (as fraction of canvas height)
    pub max_bar_height: f32,
    /// Peak hold time in seconds
    pub peak_hold_time: f32,
    /// Peak fall speed (fraction per second)
    pub peak_fall_speed: f32,
    /// Enable peak indicators
    pub show_peaks: bool,
    /// Logarithmic frequency scaling
    pub logarithmic_scale: bool,
    /// Bar style
    pub bar_style: BarStyle,
    /// Gradient direction
    pub gradient_direction: GradientDirection,
}

/// Bar rendering styles
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BarStyle {
    /// Solid filled bars
    Solid,
    /// Outlined bars
    Outlined,
    /// Rounded bars
    Rounded,
    /// Line bars (thin lines)
    Line,
}

/// Gradient directions for bars
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GradientDirection {
    /// Vertical gradient (bottom to top)
    Vertical,
    /// Horizontal gradient (left to right)
    Horizontal,
    /// Radial gradient from center
    Radial,
    /// No gradient (solid color)
    None,
}

/// Individual bar state
#[derive(Debug, Clone)]
struct BarState {
    /// Current height (0.0 - 1.0)
    height: f32,
    /// Peak height (0.0 - 1.0)
    peak_height: f32,
    /// Peak hold start time
    peak_hold_start: Instant,
    /// Smoothed height for animation
    smoothed_height: f32,
}

/// Spectrum bars visualizer
pub struct SpectrumBarsVisualizer {
    /// Visualizer configuration
    config: SpectrumBarsConfig,
    /// Visualization settings
    vis_config: VisualizationConfig,
    /// Bar states
    bars: Vec<BarState>,
    /// Last update time
    last_update: Instant,
    /// Frequency bin mapping for logarithmic scaling
    frequency_bins: Vec<usize>,
    /// Maximum amplitude for normalization
    max_amplitude: f32,
    /// Smoothing factor for animations
    smoothing_factor: f32,
}

impl SpectrumBarsVisualizer {
    /// Create a new spectrum bars visualizer
    pub fn new() -> Self {
        let config = SpectrumBarsConfig::default();
        let bar_count = config.bar_count;

        Self {
            config,
            vis_config: VisualizationConfig::default(),
            bars: vec![BarState::default(); bar_count],
            last_update: Instant::now(),
            frequency_bins: Vec::new(),
            max_amplitude: 1.0,
            smoothing_factor: 0.8,
        }
    }

    /// Update frequency bin mapping based on configuration
    fn update_frequency_mapping(&mut self, spectrum_size: usize) {
        self.frequency_bins.clear();

        if self.config.logarithmic_scale {
            // Logarithmic frequency mapping
            self.create_log_frequency_mapping(spectrum_size);
        } else {
            // Linear frequency mapping
            self.create_linear_frequency_mapping(spectrum_size);
        }
    }

    /// Create logarithmic frequency mapping
    fn create_log_frequency_mapping(&mut self, spectrum_size: usize) {
        let (min_freq, max_freq) = self.vis_config.frequency_range;
        let log_min = min_freq.ln();
        let log_max = max_freq.ln();
        let log_step = (log_max - log_min) / self.config.bar_count as f32;

        let nyquist = 22050.0; // Assuming 44.1kHz sample rate
        let freq_per_bin = nyquist / spectrum_size as f32;

        for i in 0..self.config.bar_count {
            let freq = (log_min + i as f32 * log_step).exp();
            let bin = (freq / freq_per_bin) as usize;
            self.frequency_bins.push(bin.min(spectrum_size - 1));
        }
    }

    /// Create linear frequency mapping
    fn create_linear_frequency_mapping(&mut self, spectrum_size: usize) {
        let (min_freq, max_freq) = self.vis_config.frequency_range;
        let freq_step = (max_freq - min_freq) / self.config.bar_count as f32;

        let nyquist = 22050.0; // Assuming 44.1kHz sample rate
        let freq_per_bin = nyquist / spectrum_size as f32;

        for i in 0..self.config.bar_count {
            let freq = min_freq + i as f32 * freq_step;
            let bin = (freq / freq_per_bin) as usize;
            self.frequency_bins.push(bin.min(spectrum_size - 1));
        }
    }
}