//! Visualizer plugins module
//! 
//! This module contains built-in visualizer implementations and
//! provides utilities for managing external plugins.

pub mod spectrum_bars;

pub use spectrum_bars::SpectrumBarsVisualizer;

/// Re-export commonly used types for plugin development
pub use crate::visualizer::traits::{
    BlendMode, Canvas, Color, ColorScheme, ConfigParameter, ParameterType, PluginValue, Point,
    Rect, VisualizationConfig, Visualizer, VisualizerMetadata,
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
