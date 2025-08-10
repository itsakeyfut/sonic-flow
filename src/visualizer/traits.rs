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
