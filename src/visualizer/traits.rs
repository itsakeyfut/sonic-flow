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

/// Color scheme for visualizers
#[derive(Debug, Clone)]
pub struct ColorScheme {
    /// Primary color
    pub primary: Color,
    /// Secondary color
    pub secondary: Color,
    /// Background color
    pub background: Color,
    /// Gradient colors (from low to high intensity)
    pub gradient: Vec<Color>,
}

/// RGBA color representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

/// Configuration parameter for plugin configuration
#[derive(Debug, Clone)]
pub struct ConfigParameter {
    /// Parameter name
    pub name: String,
    /// Human-readable label
    pub label: String,
    /// Parameter type
    pub param_type: ParameterType,
    /// Default value
    pub default_value: PluginValue,
    /// Minimum value (for numeric types)
    pub min_value: Option<PluginValue>,
    /// Maximum value (for numeric types)
    pub max_value: Option<PluginValue>,
    /// Description
    pub description: String,
}

/// Parameter types for configuration
#[derive(Debug, Clone)]
pub enum ParameterType {
    Float,
    Integer,
    Boolean,
    String,
    Color,
    Enum(Vec<String>),
}

/// Plugin configuration value
#[derive(Debug, Clone, PartialEq)]
pub enum PluginValue {
    Float(f32),
    Integer(f32),
    Boolean(f32),
    String(String),
    Color(Color),
}

/// Canvas trait for rendering
pub trait Canvas {
    /// Get canvas dimensions
    fn size(&self) -> (u32, u32);

    /// Clear the canvas with a color
    fn clear(&mut self, color: Color);

    /// Draw a rectangle
    fn draw_rect(&mut self, rect: Rect, color: Color);

    /// Draw a line
    fn draw_line(&mut self, start: Point, end: Point, color: Color, width: f32);

    /// Draw a Circle
    fn draw_circle(&mut self, center: Point, radius: f32, color: Color);

    /// Draw text
    fn draw_text(&mut self, text: &str, position: Point, color: Color, size: f32);

    /// Draw a polygon
    fn draw_polygon(&mut self, points: &[Point], color: Color);

    /// Set blend mode
    fn set_blend_mode(&mut self, mode: BlendMode);
}

/// Rectangle definition
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Point definition
#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

/// Blend modes for rendering
#[derive(Debug, Clone, Copy)]
pub enum BlendMode {
    Normal,
    Add,
    Multiply,
    Screen,
}

// Implementations

impl Color {
    /// Create a new RGBA color
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create a new RGB color with full opacity
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }
}