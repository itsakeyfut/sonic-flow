//! HLSL shader compiler integration (with future Slang compatibility)

use std::sync::Arc;

use tracing::{debug, info, warn};
use wgpu::{BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType, Device, ShaderModule, ShaderModuleDescriptor, ShaderStages};

use crate::types::{CompiledShader, ShaderCompilationError, ShaderMetadata};

/// HLSL shader compiler (with future Slang compatibility)
pub struct ShaderCompiler {
    /// Target platform for compilation
    target: CompilationTarget,
    /// Optimization level
    optimization_level: OptimizationLevel,
    /// Include paths for shader compilation
    include_paths: Vec<String>,
}

/// Compilation target platform
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilationTarget {
    /// Vulkan SPIR-V
    Vulkan,
    /// Metal Shading Language
    Metal,
    /// DirectX 12 HLSL
    DirectX12,
    /// WebGPU WGSL
    WGSL,
}

/// Optimization level for shader compilation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationLevel {
    /// No optimization
    None,
    /// Basic optimization
    Basic,
    /// Full optimization
    Full,
}

impl Default for ShaderCompiler {
    fn default() -> Self {
        Self {
            target: CompilationTarget::Vulkan,
            optimization_level: OptimizationLevel::Full,
            include_paths: vec!["shaders/".to_string()],
        }
    }
}

impl ShaderCompiler {
    /// Create a new shader compiler
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a compiler with specific target
    pub fn with_target(target: CompilationTarget) -> Self {
        Self {
            target,
            ..Default::default()
        }
    }

    /// Set optimization level
    pub fn with_optimization(mut self, level: OptimizationLevel) -> Self {
        self.optimization_level = level;
        self
    }

    /// Add include path
    pub fn add_include_path(mut self, path: String) -> Self {
        self.include_paths.push(path);
        self
    }

    /// Compile an HLSL shader (with future Slang compatibility)
    pub fn compile_shader(
        &self,
        source: &str,
        vertex_entry: &str,
        fragment_entry: &str,
        device: &Device,
    ) -> Result<CompiledShader, ShaderCompilationError> {
        info!("Compiling shader with target: {:?}", self.target);

        // Extract shader metadata from source
        let metadata = self.extract_metadata(source)?;

        // Compile shader using HLSL (with future Slang compatibility)
        let compiled_source = self.compile_hlsl_to_wgsl(source)?;

        // Create wgpu shader module
        let module = self.create_shader_module(device, &compiled_source)?;

        // Create bind group layout
        let bind_group_layout = self.create_bind_group_layout(device)?;

        Ok(CompiledShader {
            module: Arc::new(module),
            vertex_entry: vertex_entry.to_string(),
            fragment_entry: fragment_entry.to_string(),
            bind_group_layout: Arc::new(bind_group_layout),
            metadata,
        })
    }

    /// Compile HLSL shader to WGSL (with future Slang compatibility)
    fn compile_hlsl_to_wgsl(&self, source: &str) -> Result<String, ShaderCompilationError> {
        debug!("Compiling HLSL shader to WGSL");
        
        // Validate shader source
        self.validate_shader_source(source)?;
        
        // TODO: Implement actual HLSL compilation
        // For now, we'll use a placeholder that converts HLSL-like syntax to WGSL
        // This should be replaced with actual HLSL compilation when the integration is complete
        
        let mut wgsl_source = source.to_string();
        
        // Replace HLSL-specific syntax with WGSL equivalents
        wgsl_source = wgsl_source.replace("cbuffer", "");
        wgsl_source = wgsl_source.replace(": register(b0)", "");
        wgsl_source = wgsl_source.replace("SV_Position", "builtin(position)");
        wgsl_source = wgsl_source.replace("SV_Target", "");
        wgsl_source = wgsl_source.replace("SV_InstanceID", "builtin(instance_index)");
        wgsl_source = wgsl_source.replace("TEXCOORD0", "location(1)");
        wgsl_source = wgsl_source.replace("POSITION", "location(0)");
        wgsl_source = wgsl_source.replace("float2", "vec2<f32>");
        wgsl_source = wgsl_source.replace("float3", "vec3<f32>");
        wgsl_source = wgsl_source.replace("float4", "vec4<f32>");
        wgsl_source = wgsl_source.replace("float4x4", "mat4x4<f32>");
        wgsl_source = wgsl_source.replace("mul(", "(");
        wgsl_source = wgsl_source.replace("sin(", "sin(");
        wgsl_source = wgsl_source.replace("smoothstep(", "smoothstep(");
        
        // Add WGSL-specific structure
        let wgsl_header = r#"
@group(0) @binding(0) var<uniform> uniforms: AudioVisualizationUniforms;

struct AudioVisualizationUniforms {
    modelViewProjection: mat4x4<f32>,
    time: f32,
    sensitivity: f32,
    colorScheme: vec4<f32>,
    spectrumData: array<f32, 128>,
    audioLevels: vec4<f32>,
    effectParams: array<f32, 8>,
}

"#;
        
        let final_wgsl = format!("{}{}", wgsl_header, wgsl_source);
        
        Ok(final_wgsl)
    }

    /// Validate shader source
    fn validate_shader_source(&self, source: &str) -> Result<(), ShaderCompilationError> {
        // Basic validation
        if source.is_empty() {
            return Err(ShaderCompilationError::Validation("Shader source is empty".to_string()));
        }

        // Check for required entry points
        if !source.contains("vertexMain") && !source.contains("main") {
            warn!("No vertex entry point found in shader");
        }

        if !source.contains("fragmentMain") && !source.contains("main") {
            warn!("No fragment entry point found in shader");
        }

        Ok(())
    }

    /// Extract metadata from shader source
    fn extract_metadata(&self, source: &str) -> Result<ShaderMetadata, ShaderCompilationError> {
        let mut metadata = ShaderMetadata {
            name: "Unknown".to_string(),
            version: "1.0.0".to_string(),
            author: "Sonic Flow".to_string(),
            description: "Audio visualization shader".to_string(),
            uniform_buffer_size: std::mem::size_of::<crate::types::AudioVisualizationUniforms>(),
            features: Vec::new(),
        };

        // Parse shader comments for metadata
        for line in source.lines() {
            let line = line.trim();
            
            if line.starts_with("//! @title") {
                metadata.name = line.replace("//! @title", "").trim().to_string();
            } else if line.starts_with("//! @version") {
                metadata.version = line.replace("//! @version", "").trim().to_string();
            } else if line.starts_with("//! @author") {
                metadata.author = line.replace("//! @author", "").trim().to_string();
            } else if line.starts_with("//! @description") {
                metadata.description = line.replace("//! @description", "").trim().to_string();
            }
        }

        Ok(metadata)
    }

    /// Create wgpu shader module
    fn create_shader_module(&self, device: &Device, source: &str) -> Result<ShaderModule, ShaderCompilationError> {
        let descriptor = ShaderModuleDescriptor {
            label: Some("Sonic Shader"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        };

        Ok(device.create_shader_module(descriptor))
    }

    /// Create bind group layout for uniforms
    fn create_bind_group_layout(&self, device: &Device) -> Result<BindGroupLayout, ShaderCompilationError> {
        let entries = &[
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ];

        let descriptor = BindGroupLayoutDescriptor {
            label: Some("Audio Visualization Bind Group Layout"),
            entries,
        };

        Ok(device.create_bind_group_layout(&descriptor))
    }

    /// Compile shader from file
    pub fn compile_from_file(
        &self,
        file_path: &str,
        vertex_entry: &str,
        fragment_entry: &str,
        device: &Device,
    ) -> Result<CompiledShader, ShaderCompilationError> {
        let source = std::fs::read_to_string(file_path)
            .map_err(|e| ShaderCompilationError::SlangCompilation(format!("Failed to read shader file: {}", e)))?;

        self.compile_shader(&source, vertex_entry, fragment_entry, device)
    }

    /// Get compilation target
    pub fn target(&self) -> CompilationTarget {
        self.target
    }

    /// Get optimization level
    pub fn optimization_level(&self) -> OptimizationLevel {
        self.optimization_level
    }

    /// Test HLSL compilation without GPU device
    pub fn test_hlsl_compilation(&self, source: &str) -> Result<String, ShaderCompilationError> {
        self.compile_hlsl_to_wgsl(source)
    }
}

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
    fn test_shader_compiler_with_target() {
        let compiler = ShaderCompiler::with_target(CompilationTarget::Metal);
        assert_eq!(compiler.target(), CompilationTarget::Metal);
    }

    #[test]
    fn test_shader_validation() {
        let compiler = ShaderCompiler::new();
        
        // Valid shader
        let valid_source = r#"
            //! @title Test Shader
            //! @version 1.0.0
            //! @author Test Author
            //! @description Test shader
            
            struct VertexInput {
                float2 position : POSITION;
            };
            
            void vertexMain(VertexInput input) {
                // Vertex shader implementation
            }
            
            void fragmentMain() {
                // Fragment shader implementation
            }
        "#;
        
        assert!(compiler.validate_shader_source(valid_source).is_ok());
        
        // Invalid shader (empty)
        assert!(compiler.validate_shader_source("").is_err());
    }

    #[test]
    fn test_metadata_extraction() {
        let compiler = ShaderCompiler::new();
        let source = r#"
            //! @title Test Shader
            //! @version 2.0.0
            //! @author Test Author
            //! @description Test shader description
        "#;
        
        let metadata = compiler.extract_metadata(source).unwrap();
        assert_eq!(metadata.name, "Test Shader");
        assert_eq!(metadata.version, "2.0.0");
        assert_eq!(metadata.author, "Test Author");
        assert_eq!(metadata.description, "Test shader description");
    }

    #[test]
    fn test_hlsl_compilation() {
        let compiler = ShaderCompiler::new();
        let hlsl_source = r#"
            struct VertexInput {
                float2 position : POSITION;
            };
            
            void vertexMain(VertexInput input) {
                // Vertex shader implementation
            }
        "#;
        
        let result = compiler.test_hlsl_compilation(hlsl_source);
        assert!(result.is_ok());
        
        let wgsl_source = result.unwrap();
        assert!(wgsl_source.contains("location(0)"));
        assert!(wgsl_source.contains("AudioVisualizationUniforms"));
    }

    #[test]
    fn test_spectrum_bars_shader_compilation() {
        let compiler = ShaderCompiler::new();
        let spectrum_source = include_str!("shaders/spectrum_bars.hlsl");
        
        let result = compiler.test_hlsl_compilation(spectrum_source);
        assert!(result.is_ok());
        
        let wgsl_source = result.unwrap();
        assert!(wgsl_source.contains("vertexMain"));
        assert!(wgsl_source.contains("fragmentMain"));
        assert!(wgsl_source.contains("spectrumData"));
        assert!(wgsl_source.contains("vec2<f32>"));
    }

    #[test]
    fn test_simple_test_shader_compilation() {
        let compiler = ShaderCompiler::new();
        let simple_source = include_str!("shaders/simple_test.hlsl");
        
        let result = compiler.test_hlsl_compilation(simple_source);
        assert!(result.is_ok());
        
        let wgsl_source = result.unwrap();
        assert!(wgsl_source.contains("vertexMain"));
        assert!(wgsl_source.contains("fragmentMain"));
        assert!(wgsl_source.contains("sin("));
        assert!(wgsl_source.contains("vec3<f32>"));
    }
}
