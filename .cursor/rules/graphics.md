# Sonic Flow - Graphics Programming with Slang

## 🎨 Graphics Programming Overview

Sonic Flow uses **Slang** as the primary shader language for GPU-accelerated audio visualization. This provides high-performance, real-time graphics rendering with modern GPU capabilities.

### Core Graphics Principles

- **GPU-First Design**: All visualizers must run on GPU for optimal performance
- **Slang Integration**: Use Slang shader language as the primary graphics technology
- **Real-time Performance**: Target 120FPS for visualizer rendering
- **Cross-platform Compatibility**: Support Windows, macOS, and Linux
- **Modular Shaders**: Reusable shader components and effects

## 🛠️ Technology Stack

### Primary Graphics Stack

- **Shader Language**: [Slang](https://shader-slang.org/docs/) (Microsoft's modern shader language)
- **GPU API**: wgpu (WebGPU-like abstraction for cross-platform support)
- **Rendering Pipeline**: Custom GPU pipeline optimized for audio visualization
- **Integration**: Slint UI + GPU canvas bridge

### Secondary Technologies

- **Vulkan**: Primary GPU API target (highest performance)
- **Metal**: macOS/iOS optimization
- **DirectX 12**: Windows optimization
- **WGSL**: WebGPU compatibility

## 📚 External References

### Core Graphics Documentation

- **Slang Documentation**: https://shader-slang.org/docs/
- **Slang User Guide**: https://shader-slang.org/docs/user-guide
- **Slang Language Specification**: https://shader-slang.org/docs/language-specification
- **Slang Compilation API**: https://shader-slang.org/docs/compilation-api
- **Slang Reflection API**: https://shader-slang.org/docs/reflection-api

### GPU API Documentation

- **wgpu Documentation**: https://docs.rs/wgpu/latest/wgpu/
- **Vulkan Documentation**: https://www.khronos.org/vulkan/
- **Metal Documentation**: https://developer.apple.com/metal/
- **DirectX 12 Documentation**: https://docs.microsoft.com/en-us/windows/win32/direct3d12/

### Tutorials and Examples

- **Write Your First Slang Shader**: https://shader-slang.org/docs/my-first-shader
- **Using Slang Generics**: https://shader-slang.org/docs/slang-generics
- **Slang Parameter Blocks**: https://shader-slang.org/docs/parameter-blocks
- **Migrating from HLSL to Slang**: https://shader-slang.org/docs/coming-from-hlsl
- **Migrating from GLSL to Slang**: https://shader-slang.org/docs/coming-from-glsl

## 🏗️ Architecture Design

### Graphics Pipeline Architecture

```
┌─────────────────────────────────────────────┐
│              Slint UI Layer                 │
├─────────────────────────────────────────────┤
│         Shader Canvas Bridge                │
├─────────────────────────────────────────────┤
│           Slang Shader Engine               │
│  ┌─────────────────────────────────────────┐ │
│  │  Vertex Shader    │   Fragment Shader   │ │
│  │  (Geometry)       │   (Visual Effects)  │ │
│  └─────────────────────────────────────────┘ │
├─────────────────────────────────────────────┤
│            wgpu/GPU API Layer               │
├─────────────────────────────────────────────┤
│         Vulkan/Metal/DirectX 12             │
└─────────────────────────────────────────────┘
```

### Module Structure

```
libs/
├── sonic-shader/           # Slang shader engine
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── compiler.rs     # Slang compiler integration
│       ├── pipeline.rs     # Rendering pipeline
│       ├── shaders/        # Shader files
│       │   ├── spectrum_bars.slang
│       │   ├── waveform.slang
│       │   ├── particles.slang
│       │   └── effects/
│       │       ├── bloom.slang
│       │       ├── glow.slang
│       │       └── distortion.slang
│       └── renderer.rs     # GPU renderer
│
└── sonic-visualizer/       # Extended visualizer
    └── src/
        ├── shader_canvas.rs # GPU canvas implementation
        └── plugins/
            ├── gpu_spectrum_bars.rs  # GPU spectrum bars
            ├── gpu_waveform.rs       # GPU waveform
            ├── gpu_particles.rs      # GPU particle system
            └── gpu_3d_spectrum.rs    # 3D spectrum visualization
```

## 🎯 Performance Requirements

### Graphics Performance Targets

- **Visualizer Rendering**: ≤ 8.3ms (120FPS target)
- **Shader Compilation**: ≤ 100ms (first load)
- **GPU Memory Usage**: ≤ 50MB per visualizer
- **GPU Utilization**: ≤ 30% during normal operation
- **Frame Time Variance**: ≤ 2ms (stable 60FPS minimum)

### Optimization Guidelines

- **Batch Rendering**: Group similar draw calls
- **Instanced Rendering**: Use for repeated geometry
- **Texture Atlasing**: Minimize texture bindings
- **Uniform Buffer Objects**: Efficient data transfer
- **Compute Shaders**: For complex audio processing

## 📝 Implementation Guidelines

### Shader Development

#### 1. **Shader File Organization**

```slang
// shaders/spectrum_bars.slang
//! @title Spectrum Bars Visualizer
//! @description Real-time audio spectrum visualization
//! @author Sonic Flow Team
//! @version 1.0.0

#include "common/uniforms.slang"
#include "common/audio_data.slang"

struct VertexInput {
    float2 position : POSITION;
    float2 uv : TEXCOORD0;
    uint instance_id : SV_InstanceID;
};

struct VertexOutput {
    float4 position : SV_Position;
    float2 uv : TEXCOORD0;
    uint instance_id : SV_InstanceID;
};

// ... shader implementation
```

#### 2. **Uniform Buffer Design**

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AudioVisualizationUniforms {
    pub model_view_projection: [[f32; 4]; 4],
    pub time: f32,
    pub sensitivity: f32,
    pub color_scheme: [f32; 4],
    pub spectrum_data: [f32; 128],  // Audio spectrum magnitudes
    pub audio_levels: [f32; 4],     // L, R, LFE, Center
    pub effect_params: [f32; 8],    // Custom effect parameters
}
```

#### 3. **Shader Compilation Strategy**

```rust
pub struct ShaderCompiler {
    session: slang::Session,
    target: slang::CompileTarget,
}

impl ShaderCompiler {
    pub fn compile_shader(&self, source: &str, entry_point: &str) -> Result<CompiledShader> {
        let request = slang::CompileRequest {
            source: source.to_string(),
            entry_point: entry_point.to_string(),
            target: self.target.clone(),
            optimization_level: slang::OptimizationLevel::Performance,
            ..Default::default()
        };

        let result = self.session.compile(&request)?;
        Ok(CompiledShader::from_result(result))
    }
}
```

### GPU Canvas Implementation

#### 1. **Canvas Trait Extension**

```rust
pub trait GPURenderer: Send + Sync {
    /// Initialize GPU resources
    fn initialize(&mut self, device: &wgpu::Device) -> Result<()>;

    /// Update audio data for visualization
    fn update_audio_data(&mut self, spectrum: &SpectrumData) -> Result<()>;

    /// Render current frame
    fn render(&mut self, view: &wgpu::TextureView, encoder: &mut wgpu::CommandEncoder) -> Result<()>;

    /// Resize render target
    fn resize(&mut self, width: u32, height: u32) -> Result<()>;
}
```

#### 2. **Shader Canvas Bridge**

```rust
pub struct ShaderCanvas {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    render_pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    shader_compiler: ShaderCompiler,
}

impl Canvas for ShaderCanvas {
    fn draw_spectrum_bars(&mut self, data: &SpectrumData) -> Result<()> {
        self.update_uniforms(data);
        self.render_frame();
        Ok(())
    }
}
```

## 🧪 Testing Strategy

### Shader Testing

```rust
#[cfg(test)]
mod shader_tests {
    use super::*;

    #[test]
    fn test_spectrum_bars_shader_compilation() {
        let compiler = ShaderCompiler::new();
        let source = include_str!("../shaders/spectrum_bars.slang");

        let result = compiler.compile_shader(source, "vertexMain");
        assert!(result.is_ok(), "Shader compilation failed: {:?}", result.err());
    }

    #[test]
    fn test_shader_uniform_binding() {
        // Test uniform buffer binding and data transfer
    }

    #[test]
    fn test_gpu_rendering_performance() {
        // Performance benchmarks for GPU rendering
    }
}
```

### Integration Testing

```rust
#[tokio::test]
async fn test_gpu_visualizer_integration() {
    let mut visualizer = GPUVisualizer::new().await?;
    let audio_data = generate_test_spectrum_data();

    visualizer.update(audio_data).await?;
    let frame = visualizer.render().await?;

    assert!(frame.is_some(), "GPU rendering failed");
}
```

## 🔧 Development Workflow

### 1. **Shader Development**

```bash
# Create new shader
touch libs/sonic-shader/src/shaders/new_effect.slang

# Compile and test shader
cargo test --package sonic-shader

# Hot reload during development
cargo run --bin shader-hot-reload
```

### 2. **GPU Visualizer Implementation**

```bash
# Create new GPU visualizer
cargo new --lib libs/sonic-visualizer/src/plugins/gpu_new_effect.rs

# Test GPU rendering
cargo test --package sonic-visualizer --test gpu_rendering

# Performance profiling
cargo bench --package sonic-visualizer
```

### 3. **Integration Testing**

```bash
# Test full GPU pipeline
cargo test --package sonic-app --test gpu_integration

# Performance validation
cargo run --bin performance-test --gpu
```

## 🚨 Critical Restrictions

### GPU Programming Constraints

- **NO CPU Fallback**: All visualizers must run on GPU
- **NO Blocking GPU Operations**: Use async GPU operations
- **NO Memory Leaks**: Proper GPU resource cleanup
- **NO Shader Compilation Errors**: Validate all shaders at build time
- **NO Excessive GPU Usage**: Monitor and limit GPU utilization

### Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum GraphicsError {
    #[error("Shader compilation failed: {0}")]
    ShaderCompilation(String),

    #[error("GPU resource allocation failed: {0}")]
    ResourceAllocation(String),

    #[error("Rendering pipeline error: {0}")]
    RenderingError(String),

    #[error("GPU memory exhausted")]
    OutOfMemory,
}
```

## 📊 Performance Monitoring

### GPU Metrics

- **Frame Time**: Target ≤ 8.3ms (120FPS)
- **GPU Memory**: Monitor usage and leaks
- **Shader Compilation Time**: Track build performance
- **Draw Call Count**: Minimize for efficiency
- **Texture Memory**: Optimize texture usage

### Profiling Tools

- **GPU-Z**: Monitor GPU utilization
- **RenderDoc**: Frame debugging
- **Intel Graphics Performance Analyzers**: Intel GPU profiling
- **NVIDIA Nsight**: NVIDIA GPU profiling
- **AMD Radeon GPU Profiler**: AMD GPU profiling

## 🔄 Migration Strategy

### From Software Rendering

1. **Phase 1**: Implement GPU canvas alongside software canvas
2. **Phase 2**: Migrate existing visualizers to GPU
3. **Phase 3**: Remove software rendering fallback
4. **Phase 4**: Optimize and add advanced effects

### Compatibility Matrix

| Platform | GPU API    | Status       | Notes            |
| -------- | ---------- | ------------ | ---------------- |
| Windows  | Vulkan     | ✅ Primary   | Best performance |
| Windows  | DirectX 12 | ✅ Secondary | Fallback option  |
| macOS    | Metal      | ✅ Primary   | Native support   |
| Linux    | Vulkan     | ✅ Primary   | Open source      |
| Web      | WGSL       | 🔄 Planned   | WebGPU support   |

## 📚 Additional Resources

### Learning Materials

- **Slang Tutorials**: https://shader-slang.org/docs/tutorials
- **GPU Programming Best Practices**: Industry standards
- **Real-time Graphics**: Modern rendering techniques
- **Audio Visualization**: Domain-specific knowledge

### Community and Support

- **Slang Community**: https://shader-slang.org/docs/community
- **GPU Programming Forums**: Technical discussions
- **Audio Visualization Community**: Domain expertise

---

**IMPORTANT**: Always reference the official Slang documentation at https://shader-slang.org/docs/ for the most up-to-date API information and best practices.
