//! Visualizer engine and plugins

pub mod canvas;
pub mod engine;
pub mod traits;
pub mod plugins;

// Re-export main types
pub use canvas::SoftwareCanvas;
pub use engine::{VisualizerEngine, VisualizerEvent, VisualizerMetrics, VisualizerState};
pub use traits::{
    BlendMode, Canvas, Color, ColorScheme, ParameterType, PluginValue, Point, Rect,
    VisualizationConfig, Visualizer, VisualizerMetadata,
};

// Re-export plugins
pub use plugins::{create_builtin_visualizers, validate_visualizer, SpectrumBarsVisualizer};

use crate::error::VisualizerError;

/// Visualizer system manager
/// 
/// This is the main entry point for the visualizer system,
/// providing a high-level interface for managing visualizations.
pub struct VisualizerSystem {
    engine: VisualizerEngine,
}

impl VisualizerSystem {
    /// Create a new visualizer system
    pub fn new(width: u32, height: u32) -> Result<Self, VisualizerError> {
        let engine = VisualizerEngine::new(width, height)?;
        
        // Set default visualizer
        engine.set_visualizer("spectrum_bars")?;
        
        Ok(Self { engine })
    }

    /// Get reference to the engine
    pub fn engine(&self) -> &VisualizerEngine {
        &self.engine
    }

    /// Start visualization
    pub fn start(&self) -> Result<(), VisualizerError> {
        self.engine.start()
    }
    
    /// Stop visualization
    pub fn stop(&self) -> Result<(), VisualizerError> {
        self.engine.stop()
    }

    /// Update with spectrum data
    pub fn update(&self, spectrum_data: crate::audio::analysis::SpectrumData) -> Result<(), VisualizerError> {
        self.engine.update_spectrum(spectrum_data)
    }
}
