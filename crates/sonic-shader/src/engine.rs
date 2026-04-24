//! Shader engine management module

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use tracing::{info, warn, debug};

use crate::types::{CompiledShader, ShaderCompilationError, GPURenderingError};
use crate::compiler::ShaderCompiler;
use crate::renderer::GPURenderer;
use sonic_core::audio::analysis::SpectrumData;

/// Shader engine for managing multiple shaders and visualizations
pub struct ShaderEngine {
    /// Shader compiler
    compiler: ShaderCompiler,
    /// Loaded shaders
    shaders: HashMap<String, CompiledShader>,
    /// Active shader name
    active_shader: Option<String>,
    /// Engine configuration
    config: EngineConfig,
    /// GPU renderer
    gpu_renderer: Option<GPURenderer>,
    /// Rendering surface
    surface: Option<Arc<Mutex<wgpu::Surface<'static>>>>,
}

/// Engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Default compilation target
    pub default_target: CompilationTarget,
    /// Optimization level
    pub optimization_level: OptimizationLevel,
    /// Maximum shaders to keep in memory
    pub max_shaders: usize,
    /// Enable shader caching
    pub enable_caching: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            default_target: CompilationTarget::Vulkan,
            optimization_level: OptimizationLevel::Full,
            max_shaders: 10,
            enable_caching: true,
        }
    }
}

impl ShaderEngine {
    /// Create a new shader engine
    pub fn new() -> Self {
        Self {
            compiler: ShaderCompiler::new(),
            shaders: HashMap::new(),
            active_shader: None,
            config: EngineConfig::default(),
            gpu_renderer: None,
            surface: None,
        }
    }

    /// Create a shader engine with custom configuration
    pub fn with_config(config: EngineConfig) -> Self {
        let compiler = ShaderCompiler::with_target(config.default_target)
            .with_optimization(config.optimization_level);

        Self {
            compiler,
            shaders: HashMap::new(),
            active_shader: None,
            config,
            gpu_renderer: None,
            surface: None,
        }
    }

    /// Initialize GPU rendering
    pub async fn initialize_gpu(&mut self, window: &'static winit::window::Window) -> Result<(), GPURenderingError> {
        info!("Initializing GPU rendering for shader engine");
        
        let mut gpu_renderer = GPURenderer::new(window).await?;
        gpu_renderer.initialize_pipeline().await?;
        
        self.gpu_renderer = Some(gpu_renderer);
        info!("GPU rendering initialized successfully");
        
        Ok(())
    }

    /// Load a shader from source
    pub fn load_shader(
        &mut self,
        name: &str,
        source: &str,
        vertex_entry: &str,
        fragment_entry: &str,
    ) -> Result<(), GPURenderingError> {
        info!("Loading shader: {}", name);

        // Check if we need to evict a shader
        if self.shaders.len() >= self.config.max_shaders && !self.shaders.contains_key(name) {
            self.evict_oldest_shader();
        }

        // Compile and load shader using GPU renderer
        if let Some(gpu_renderer) = &mut self.gpu_renderer {
            let shader = gpu_renderer.load_shader(source, vertex_entry, fragment_entry)?;
            self.shaders.insert(name.to_string(), shader);
            info!("Shader loaded successfully: {}", name);
        } else {
            return Err(GPURenderingError::PipelineCreation("GPU renderer not initialized".to_string()));
        }
        
        Ok(())
    }

    /// Load a shader from file
    pub fn load_shader_from_file(
        &mut self,
        name: &str,
        file_path: &str,
        vertex_entry: &str,
        fragment_entry: &str,
    ) -> Result<(), GPURenderingError> {
        let source = std::fs::read_to_string(file_path)
            .map_err(|e| GPURenderingError::PipelineCreation(format!("Failed to read shader file: {}", e)))?;

        self.load_shader(name, &source, vertex_entry, fragment_entry)
    }

    /// Get a shader by name
    pub fn get_shader(&self, name: &str) -> Option<&CompiledShader> {
        self.shaders.get(name)
    }

    /// Get a shader by name (mutable)
    pub fn get_shader_mut(&mut self, name: &str) -> Option<&mut CompiledShader> {
        self.shaders.get_mut(name)
    }

    /// Activate a shader
    pub fn activate_shader(&mut self, name: &str) -> Result<(), GPURenderingError> {
        if self.shaders.contains_key(name) {
            self.active_shader = Some(name.to_string());
            
            // Set the shader in the GPU renderer
            if let Some(gpu_renderer) = &mut self.gpu_renderer {
                if let Some(shader) = self.shaders.get(name) {
                    gpu_renderer.set_shader(shader.clone())?;
                }
            }
            
            info!("Activated shader: {}", name);
            Ok(())
        } else {
            Err(GPURenderingError::PipelineCreation(format!("Shader not found: {}", name)))
        }
    }

    /// Get active shader
    pub fn active_shader(&self) -> Option<&CompiledShader> {
        self.active_shader.as_ref().and_then(|name| self.shaders.get(name))
    }

    /// Get active shader name
    pub fn active_shader_name(&self) -> Option<&String> {
        self.active_shader.as_ref()
    }

    /// List all loaded shaders
    pub fn shader_names(&self) -> Vec<String> {
        self.shaders.keys().cloned().collect()
    }

    /// Remove a shader
    pub fn remove_shader(&mut self, name: &str) -> bool {
        let removed = self.shaders.remove(name).is_some();
        
        // If we removed the active shader, clear it
        if removed && self.active_shader.as_ref() == Some(&name.to_string()) {
            self.active_shader = None;
        }
        
        if removed {
            info!("Removed shader: {}", name);
        }
        
        removed
    }

    /// Clear all shaders
    pub fn clear_shaders(&mut self) {
        self.shaders.clear();
        self.active_shader = None;
        info!("Cleared all shaders");
    }

    /// Get shader count
    pub fn shader_count(&self) -> usize {
        self.shaders.len()
    }

    /// Get engine configuration
    pub fn config(&self) -> &EngineConfig {
        &self.config
    }

    /// Update engine configuration
    pub fn update_config(&mut self, config: EngineConfig) {
        self.config = config;
        
        // Recreate compiler with new settings
        self.compiler = ShaderCompiler::with_target(self.config.default_target)
            .with_optimization(self.config.optimization_level);
            
        info!("Updated engine configuration");
    }

    /// Evict the oldest shader (FIFO)
    fn evict_oldest_shader(&mut self) {
        if let Some((name, _)) = self.shaders.iter().next() {
            let name = name.clone();
            self.remove_shader(&name);
            warn!("Evicted oldest shader: {}", name);
        }
    }

    /// Validate all shaders
    pub fn validate_shaders(&self) -> Vec<String> {
        let mut errors = Vec::new();
        
        for (name, shader) in &self.shaders {
            if shader.metadata.name.is_empty() {
                errors.push(format!("Shader '{}' has empty name", name));
            }
            
            if shader.metadata.version.is_empty() {
                errors.push(format!("Shader '{}' has empty version", name));
            }
        }
        
        errors
    }

    /// Get shader statistics
    pub fn statistics(&self) -> EngineStatistics {
        EngineStatistics {
            total_shaders: self.shaders.len(),
            active_shader: self.active_shader.clone(),
            max_shaders: self.config.max_shaders,
            caching_enabled: self.config.enable_caching,
        }
    }

    /// Update audio data for visualization
    pub fn update_audio_data(&mut self, spectrum: &SpectrumData) -> Result<(), GPURenderingError> {
        if let Some(gpu_renderer) = &mut self.gpu_renderer {
            gpu_renderer.update_audio_data(spectrum)?;
            debug!("Updated audio data for visualization");
        }
        Ok(())
    }

    /// Render a frame
    pub fn render(&mut self) -> Result<(), GPURenderingError> {
        if let Some(gpu_renderer) = &mut self.gpu_renderer {
            gpu_renderer.render()?;
            debug!("Rendered frame");
        }
        Ok(())
    }

    /// Resize the render target
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), GPURenderingError> {
        if let Some(gpu_renderer) = &mut self.gpu_renderer {
            gpu_renderer.resize(width, height)?;
            info!("Resized render target to {}x{}", width, height);
        }
        Ok(())
    }

    /// Get GPU adapter info
    pub fn gpu_info(&self) -> Option<wgpu::AdapterInfo> {
        self.gpu_renderer.as_ref().map(|renderer| renderer.adapter_info())
    }

    /// Check if GPU rendering is available
    pub fn is_gpu_available(&self) -> bool {
        self.gpu_renderer.is_some()
    }

    /// Set blend mode
    pub fn set_blend_mode(&mut self, mode: crate::types::BlendMode) -> Result<(), GPURenderingError> {
        if let Some(gpu_renderer) = &mut self.gpu_renderer {
            gpu_renderer.set_blend_mode(mode)?;
            info!("Set blend mode: {:?}", mode);
        }
        Ok(())
    }

    /// Get frame count
    pub fn frame_count(&self) -> u64 {
        self.gpu_renderer.as_ref().map(|renderer| renderer.frame_count()).unwrap_or(0)
    }
}

/// Engine statistics
#[derive(Debug, Clone)]
pub struct EngineStatistics {
    /// Total number of loaded shaders
    pub total_shaders: usize,
    /// Name of active shader
    pub active_shader: Option<String>,
    /// Maximum number of shaders
    pub max_shaders: usize,
    /// Whether caching is enabled
    pub caching_enabled: bool,
}

// Re-export types from compiler module
pub use crate::compiler::{CompilationTarget, OptimizationLevel};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = ShaderEngine::new();
        assert_eq!(engine.shader_count(), 0);
        assert!(engine.active_shader().is_none());
    }

    #[test]
    fn test_engine_with_config() {
        let config = EngineConfig {
            default_target: CompilationTarget::Metal,
            optimization_level: OptimizationLevel::Basic,
            max_shaders: 5,
            enable_caching: false,
        };
        
        let engine = ShaderEngine::with_config(config.clone());
        assert_eq!(engine.config().default_target, CompilationTarget::Metal);
        assert_eq!(engine.config().max_shaders, 5);
    }

    #[test]
    fn test_shader_management() {
        let mut engine = ShaderEngine::new();
        
        // Test shader loading (this will fail without actual GPU device)
        // let result = engine.load_shader("test", "test source", "main", "main");
        // assert!(result.is_err()); // Should fail without GPU context
        
        // Test shader activation
        let result = engine.activate_shader("nonexistent");
        assert!(result.is_err());
        
        // Test shader removal
        assert!(!engine.remove_shader("nonexistent"));
        
        // Test statistics
        let stats = engine.statistics();
        assert_eq!(stats.total_shaders, 0);
        assert!(stats.active_shader.is_none());
    }

    #[test]
    fn test_config_default() {
        let config = EngineConfig::default();
        assert_eq!(config.default_target, CompilationTarget::Vulkan);
        assert_eq!(config.optimization_level, OptimizationLevel::Full);
        assert_eq!(config.max_shaders, 10);
        assert!(config.enable_caching);
    }
}
