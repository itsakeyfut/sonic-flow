//! Shader engine management module

use std::collections::HashMap;

use tracing::{info, warn};

use crate::types::{CompiledShader, ShaderCompilationError};
use crate::compiler::ShaderCompiler;

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
        }
    }

    /// Load a shader from source
    pub fn load_shader(
        &mut self,
        name: &str,
        _source: &str,
        _vertex_entry: &str,
        _fragment_entry: &str,
    ) -> Result<(), ShaderCompilationError> {
        info!("Loading shader: {}", name);

        // Check if we need to evict a shader
        if self.shaders.len() >= self.config.max_shaders && !self.shaders.contains_key(name) {
            self.evict_oldest_shader();
        }

        // Compile shader (placeholder - requires GPU device)
        // TODO: Implement actual shader compilation when GPU context is available
        // let shader = self.compiler.compile_shader(source, vertex_entry, fragment_entry, &device)?;
        
        // For now, return an error indicating GPU context is required
        return Err(ShaderCompilationError::SlangCompilation(
            "Shader compilation requires GPU device context".to_string()
        ));
        Ok(())
    }

    /// Load a shader from file
    pub fn load_shader_from_file(
        &mut self,
        name: &str,
        file_path: &str,
        vertex_entry: &str,
        fragment_entry: &str,
    ) -> Result<(), ShaderCompilationError> {
        let source = std::fs::read_to_string(file_path)
            .map_err(|e| ShaderCompilationError::SlangCompilation(format!("Failed to read shader file: {}", e)))?;

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
    pub fn activate_shader(&mut self, name: &str) -> Result<(), ShaderCompilationError> {
        if self.shaders.contains_key(name) {
            self.active_shader = Some(name.to_string());
            info!("Activated shader: {}", name);
            Ok(())
        } else {
            Err(ShaderCompilationError::EntryPointNotFound(format!("Shader not found: {}", name)))
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
