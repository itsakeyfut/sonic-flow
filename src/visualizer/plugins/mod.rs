//! Visualizer plugins module
//!
//! This module contains built-in visualizer implementations and
//! provides utilities for managing external plugins.

pub mod spectrum_bars;

pub use spectrum_bars::SpectrumBarsVisualizer;

/// Re-export commonly used types for plugin development
pub use crate::visualizer::traits::{
    BlendMode, Canvas, Color, ColorScheme, ConfigParameter, ParameterType, PluginValue, Point,
    Rect, VisualizationConfig, Visualizer, VisualizerMetadata, VisualizerRegistry,
};

/// Plugin registration macro
///
/// This macro helps register visualizer plugins with the engine.
///
/// # Example
///
/// ```rust
/// use sonic_flow::visualizer::plugins::register_visualizer;
///
/// struct MyVisualizer;
/// impl Visualizer for MyVisualizer { /* implementation */ }
///
/// register_visualizer!("my_visualizer", MyVisualizer);
/// ```
#[macro_export]
macro_rules! register_visualizer {
    ($id:expr, $visualizer_type:ty) => {
        pub fn create_visualizer() -> Box<dyn $crate::visualizer::traits::Visualizer> {
            Box::new(<$visualizer_type>::new())
        }

        pub fn visualizer_id() -> &'static str {
            $id
        }
    };
}

/// Utility function to create all built-in visualizers
pub fn create_builtin_visualizers() -> VisualizerRegistry {
    let mut visualizers = VisualizerRegistry::new();

    // Spectrum bars visualizer
    visualizers.insert(
        "spectrum_bars".to_string(),
        Box::new(|| -> Box<dyn Visualizer> { Box::new(SpectrumBarsVisualizer::new()) })
            as Box<dyn Fn() -> Box<dyn crate::visualizer::traits::Visualizer> + Send + Sync>,
    );

    // TODO: Add more built-in visualizers here
    // visualizers.insert("waveform".to_string(), Box::new(|| Box::new(WaveformVisualizer::new())));
    // visualizers.insert("circle_spectrum".to_string(), Box::new(|| Box::new(CircleSpectrumVisualizer::new())));

    visualizers
}

/// Validate a visualizer implementation
pub fn validate_visualizer(
    visualizer: &dyn crate::visualizer::traits::Visualizer,
) -> Result<(), String> {
    let metadata = visualizer.metadata();

    // Check required fields
    if metadata.id.is_empty() {
        return Err("Visualizer ID cannot be empty".to_string());
    }

    if metadata.name.is_empty() {
        return Err("Visualizer name cannot be empty".to_string());
    }

    if metadata.version.is_empty() {
        return Err("Visualizer version cannot be empty".to_string());
    }

    // Validate update rate
    let update_rate = visualizer.preferred_update_rate();
    if update_rate == 0 || update_rate > 240 {
        return Err(format!(
            "Invalid update rate: {} (must be 1-240 FPS)",
            update_rate
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_visualizers_creation() {
        let visualizers = create_builtin_visualizers();

        // Should have at least spectrum_bars
        assert!(visualizers.contains_key("spectrum_bars"));

        // Test creating spectrum bars visualizer
        let factory = visualizers.get("spectrum_bars").unwrap();
        let visualizer = factory();

        let metadata = visualizer.metadata();
        assert_eq!(metadata.id, "spectrum_bars");
        assert!(!metadata.name.is_empty());
    }

    #[test]
    fn test_visualizer_validation() {
        let visualizers = create_builtin_visualizers();
        let factory = visualizers.get("spectrum_bars").unwrap();
        let visualizer = factory();

        // Should pass validation
        assert!(validate_visualizer(visualizer.as_ref()).is_ok());
    }
}
