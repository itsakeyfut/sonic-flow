//! Sonic Flow GPU Shader Library
//! 
//! This library provides GPU-accelerated audio visualization capabilities
//! using modern shader languages and WebGPU.

pub mod compiler;
pub mod pipeline;
pub mod renderer;
pub mod types;
pub mod audio_bridge;
pub mod effects_manager;
pub mod performance;
pub mod engine;

pub use compiler::ShaderCompiler;
pub use pipeline::RenderingPipeline;
pub use renderer::GPURenderer;
pub use engine::ShaderEngine;
pub use types::*;
pub use audio_bridge::{AudioVisualizationBridge, VisualizationLoop, VisualizationSettings};
pub use effects_manager::{EffectsManager, EffectConfig, EffectType, EffectPreset};
pub use performance::{
    PerformanceMonitor, PerformanceMetrics, PerformanceThresholds, 
    PerformanceBenchmark, BenchmarkResults, OptimizationSuggestion,
};
