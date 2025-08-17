//! # Sonic Shader
//!
//! HLSL shader engine for Sonic Flow GPU-accelerated audio visualization.
//!
//! This library provides GPU-accelerated rendering capabilities using HLSL shader language
//! and wgpu for cross-platform graphics programming.
//!
//! ## Features
//!
//! - **HLSL Integration**: DirectX shader language support (with future Slang compatibility)
//! - **GPU Acceleration**: Hardware-accelerated rendering
//! - **Cross-platform**: Vulkan, Metal, DirectX 12 support
//! - **Real-time Performance**: Optimized for 120FPS visualization
//! - **Audio Visualization**: Specialized for audio spectrum rendering
//!
//! ## Status: Phase 1 Implementation
//!
//! Currently implementing basic HLSL to WGSL conversion with future GPU rendering capabilities.
//!
//! ## Quick Start
//!
//! ```no_run
//! use sonic_shader::{ShaderEngine, ShaderCompiler, CompiledShader};
//! use sonic_core::audio::analysis::SpectrumData;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Initialize shader engine
//!     let mut engine = ShaderEngine::new().await?;
//!     
//!     // Compile shader
//!     let compiler = ShaderCompiler::new();
//!     let shader_source = include_str!("shaders/spectrum_bars.hlsl");
//!     let shader = compiler.compile_shader(shader_source, "vertexMain", "fragmentMain")?;
//!     
//!     // Create visualization
//!     engine.create_visualization(shader).await?;
//!     
//!     Ok(())
//! }
//! ```

#![deny(unsafe_op_in_unsafe_fn)]
#![warn(
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    clippy::all,
    clippy::pedantic,
    clippy::nursery
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]

// Re-exports from sonic-core
pub use sonic_core::{Error, Result};

// Public API
pub use compiler::*;
pub use engine::*;
pub use pipeline::*;
pub use types::*;

// Modules
pub mod compiler;
pub mod engine;
pub mod pipeline;
pub mod renderer;
pub mod types;

// Re-export commonly used types
pub use types::{
    AudioVisualizationUniforms,
    CompiledShader,
    ShaderCompilationError,
    GPURenderer,
    ShaderCanvas,
};
pub use engine::ShaderEngine;
pub use compiler::ShaderCompiler;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_compiler_creation() {
        let compiler = ShaderCompiler::new();
        assert_eq!(compiler.target(), CompilationTarget::Vulkan);
        assert_eq!(compiler.optimization_level(), OptimizationLevel::Full);
    }

    #[test]
    fn test_uniforms_default() {
        let uniforms = AudioVisualizationUniforms::default();
        assert_eq!(uniforms.time, 0.0);
        assert_eq!(uniforms.sensitivity, 1.0);
        assert_eq!(uniforms.spectrum_data[0], 0.0);
    }

    #[test]
    fn test_color_creation() {
        let color = Color::rgb(1.0, 0.5, 0.25);
        assert_eq!(color.r, 1.0);
        assert_eq!(color.g, 0.5);
        assert_eq!(color.b, 0.25);
        assert_eq!(color.a, 1.0);
    }

    #[test]
    fn test_point_creation() {
        let point = Point::new(10.0, 20.0);
        assert_eq!(point.x, 10.0);
        assert_eq!(point.y, 20.0);
    }

    #[test]
    fn test_rect_contains() {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let inside_point = Point::new(50.0, 50.0);
        let outside_point = Point::new(150.0, 150.0);
        
        assert!(rect.contains(inside_point));
        assert!(!rect.contains(outside_point));
    }
}
