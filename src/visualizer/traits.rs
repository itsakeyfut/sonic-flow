//! Visualizer traits and plugin API
//! 
//! This module defines the core traits for visualizer plugins and
//! the unified interface for the visualizer system.

use std::collections::HashMap;
use std::time::Duration;

use crate::audio::analysis::SpectrumData;
use crate::error::VisualizerError;

/// Visualizer plugin trait
/// 
/// All visualizer implementations must implement this trait to be
/// compatible with the Sonic Flow plugin system.
pub trait Visualizer: Send + Sync {
    /// Get visualizer metadata
    fn metadata(&self) -> VisualizerMetadata;

    /// Initialize the visualizer with configuration
    fn initialize(&mut self, config: &VisualizerConfig) -> Result<(), VisualizerError>;

    /// Update the visualizer with new spectrum data
    fn update(&mut self, spectrum_data: &SpectrumData) -> Result<(), VisualizerError>;

    /// Render the visualizer to a canvas
    fn render(&self, canvas: &mut dyn Canvas) -> Result<(), VisualizerError>;

    /// Configure the visualizer with new settings
    fn configure(&mut self, settings: &HashMap<String, PluginValue>) -> Result<(), VisualizerError>;

    /// Reset the visualizer state
    fn reset(&mut self);

    /// Check if the visualizer supports real-time rendering
    fn supports_realtime(&self) -> bool {
        true
    }

    /// Get the preferred update rate(FPS)
    fn preferred_update_rate(&self) -> u32 {
        60
    }
}

/// Visualizer metadata information
#[derive(Debug, Clone)]
pub struct VisualizerMetadata {
    /// Unique identifier for the visualizer
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Version string
    pub version: String,
    /// Author information
    pub author: String,
    /// Description
    pub description: String,
    /// Configuration schema
    pub config_schema: Vec<ConfigParameter>,
}

/// Visualization configuration
#[derive(Debug, Clone)]
pub struct VisualizationConfig {
    /// Sensitivity multiplier (0.0 - 2.0)
    pub sensitivity: f32,
    /// Frequency range for analysis (Hz)
    pub frequency_range: (f32, f32),
    /// Color scheme
    pub color_scheme: ColorScheme,
    /// Animation speed multiplier
    pub animation_speed: f32,
    /// Enable smoothing
    pub smoothing: bool,
    /// Auto gain control
    pub auto_gain: bool,
    /// Custom parameters for specific visualizers
    pub custom_params: HashMap<String, PluginValue>,
}
