//! Common types and error definitions for the Sonic Shader engine

use std::sync::{Arc, Mutex};

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec2, Vec4};
use thiserror::Error;
use wgpu::{BindGroup, BindGroupLayout, Buffer, RenderPipeline, ShaderModule};

use sonic_core::audio::analysis::SpectrumData;

/// Error types for shader compilation and GPU operations
#[derive(Debug, Error)]
pub enum ShaderCompilationError {
    #[error("Slang compilation failed: {0}")]
    SlangCompilation(String),

    #[error("Shader validation failed: {0}")]
    Validation(String),

    #[error("Entry point not found: {0}")]
    EntryPointNotFound(String),

    #[error("Uniform buffer binding failed: {0}")]
    UniformBinding(String),

    #[error("GPU resource allocation failed: {0}")]
    ResourceAllocation(String),
}

/// Error types for GPU rendering operations
#[derive(Debug, Error)]
pub enum GPURenderingError {
    #[error("Device initialization failed: {0}")]
    DeviceInit(String),

    #[error("Surface creation failed: {0}")]
    SurfaceCreation(String),

    #[error("Pipeline creation failed: {0}")]
    PipelineCreation(String),

    #[error("Buffer creation failed: {0}")]
    BufferCreation(String),

    #[error("Rendering failed: {0}")]
    Rendering(String),

    #[error("GPU memory exhausted")]
    OutOfMemory,
}

/// Audio visualization uniform buffer data
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct AudioVisualizationUniforms {
    /// Model-view-projection matrix
    pub model_view_projection: [[f32; 4]; 4],
    /// Current time in seconds
    pub time: f32,
    /// Audio sensitivity multiplier
    pub sensitivity: f32,
    /// Color scheme parameters (RGBA)
    pub color_scheme: [f32; 4],
    /// Audio spectrum magnitudes (128 frequency bins)
    pub spectrum_data: [f32; 128],
    /// Audio levels (L, R, LFE, Center)
    pub audio_levels: [f32; 4],
    /// Custom effect parameters
    pub effect_params: [f32; 8],
}

impl Default for AudioVisualizationUniforms {
    fn default() -> Self {
        Self {
            model_view_projection: Mat4::IDENTITY.to_cols_array_2d(),
            time: 0.0,
            sensitivity: 1.0,
            color_scheme: [1.0, 0.5, 0.2, 1.0], // Orange
            spectrum_data: [0.0; 128],
            audio_levels: [0.0; 4],
            effect_params: [0.0; 8],
        }
    }
}

/// Compiled shader information
#[derive(Debug, Clone)]
pub struct CompiledShader {
    /// Shader module for wgpu
    pub module: Arc<ShaderModule>,
    /// Vertex shader entry point
    pub vertex_entry: String,
    /// Fragment shader entry point
    pub fragment_entry: String,
    /// Bind group layout for uniforms
    pub bind_group_layout: Arc<BindGroupLayout>,
    /// Shader metadata
    pub metadata: ShaderMetadata,
}

/// Shader metadata information
#[derive(Debug, Clone)]
pub struct ShaderMetadata {
    /// Shader name
    pub name: String,
    /// Shader version
    pub version: String,
    /// Author information
    pub author: String,
    /// Description
    pub description: String,
    /// Required uniform buffer size
    pub uniform_buffer_size: usize,
    /// Supported features
    pub features: Vec<String>,
}

/// GPU renderer trait for visualizer implementations
pub trait GPURenderer: Send + Sync {
    /// Initialize GPU resources
    fn initialize(&mut self, device: &wgpu::Device) -> Result<(), GPURenderingError>;

    /// Update audio data for visualization
    fn update_audio_data(&mut self, spectrum: &SpectrumData) -> Result<(), GPURenderingError>;

    /// Render current frame
    fn render(
        &mut self,
        view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<(), GPURenderingError>;

    /// Resize render target
    fn resize(&mut self, width: u32, height: u32) -> Result<(), GPURenderingError>;

    /// Get the render pipeline
    fn pipeline(&self) -> Option<&RenderPipeline>;

    /// Get the bind group
    fn bind_group(&self) -> Option<&BindGroup>;
}

/// GPU-based shader canvas for rendering
#[derive(Debug)]
pub struct ShaderCanvas {
    /// Canvas width
    pub width: u32,
    /// Canvas height
    pub height: u32,
    /// Blend mode
    pub blend_mode: BlendMode,
    /// GPU device
    pub device: Arc<wgpu::Device>,
    /// GPU queue
    pub queue: Arc<wgpu::Queue>,
    /// Shared surface
    pub surface: Arc<Mutex<wgpu::Surface<'static>>>,
    /// Render pipeline
    pub render_pipeline: Option<wgpu::RenderPipeline>,
    /// Uniform buffer
    pub uniform_buffer: Option<wgpu::Buffer>,
    /// Bind group
    pub bind_group: Option<wgpu::BindGroup>,
}

/// Blend modes for rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    /// Normal blending
    Normal,
    /// Additive blending
    Add,
    /// Multiplicative blending
    Multiply,
    /// Screen blending
    Screen,
    /// Overlay blending
    Overlay,
}

/// Color representation
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    /// Create a new color
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Create a color from RGB values (alpha = 1.0)
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Convert to wgpu color
    pub fn to_wgpu(self) -> wgpu::Color {
        wgpu::Color {
            r: self.r as f64,
            g: self.g as f64,
            b: self.b as f64,
            a: self.a as f64,
        }
    }

    /// Convert to Vec4
    pub fn to_vec4(self) -> Vec4 {
        Vec4::new(self.r, self.g, self.b, self.a)
    }
}

/// Point in 2D space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    /// Create a new point
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Convert to Vec2
    pub fn to_vec2(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

/// Rectangle in 2D space
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Create a new rectangle
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Get the center point
    pub fn center(self) -> Point {
        Point::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Check if a point is inside the rectangle
    pub fn contains(self, point: Point) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.width
            && point.y >= self.y
            && point.y <= self.y + self.height
    }
}

/// Canvas trait for rendering operations
pub trait Canvas: Send + Sync {
    /// Get canvas size
    fn size(&self) -> (u32, u32);

    /// Set blend mode
    fn set_blend_mode(&mut self, mode: BlendMode);

    /// Clear canvas with color
    fn clear(&mut self, color: Color) -> Result<(), GPURenderingError>;

    /// Draw a rectangle
    fn draw_rect(&mut self, rect: Rect, color: Color) -> Result<(), GPURenderingError>;

    /// Draw spectrum bars
    fn draw_spectrum_bars(&mut self, data: &SpectrumData) -> Result<(), GPURenderingError>;

    /// Draw waveform
    fn draw_waveform(&mut self, data: &[f32]) -> Result<(), GPURenderingError>;

    /// Present the rendered frame
    fn present(&mut self) -> Result<(), GPURenderingError>;
}
