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

    /// Get current frame data
    pub fn get_frame(&self) -> Vec<u8> {
        self.engine.get_frame()
    }

    /// Get canvas size
    pub fn size(&self) -> (u32, u32) {
        self.engine.canvas_size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::analysis::SpectrumData;
    
    #[tokio::test]
    async fn test_visualizer_system_creation() {
        let system = VisualizerSystem::new(800, 600).unwrap();
        assert_eq!(system.size(), (800, 600));
    }
    
    #[tokio::test]
    async fn test_visualizer_system_workflow() {
        let system = VisualizerSystem::new(400, 300).unwrap();
        
        // Start visualization
        system.start().unwrap();
        
        // Update with test data
        let spectrum_data = SpectrumData::new(
            vec![0.1, 0.2, 0.3, 0.4, 0.5],
            0.5,
            0.3,
        );
        system.update(spectrum_data).unwrap();
        
        // Get frame
        let frame = system.get_frame();
        assert_eq!(frame.len(), 400 * 300 * 4); // RGBA
        
        // Stop visualization
        system.stop().unwrap();
    }
}
