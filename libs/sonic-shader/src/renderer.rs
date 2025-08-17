//! GPU renderer implementation

use tracing::{debug, info};
use wgpu::{Adapter, Backends, Device, Instance, Queue, Surface};

use crate::types::{
    AudioVisualizationUniforms, BlendMode, Color, CompiledShader, GPURenderingError, Point, Rect,
    ShaderCanvas,
};
use sonic_core::audio::analysis::SpectrumData;

use super::{compiler::ShaderCompiler, pipeline::RenderingPipeline};

/// GPU renderer for audio visualization
pub struct GPURenderer {
    /// GPU instance
    instance: Instance,
    /// GPU adapter
    adapter: Adapter,
    /// GPU device
    device: Device,
    /// GPU command queue
    queue: Queue,
    /// Render surface
    surface: Surface<'static>,
    /// Rendering pipeline
    pipeline: Option<RenderingPipeline>,
    /// Shader compiler
    compiler: ShaderCompiler,
    /// Current uniforms
    uniforms: AudioVisualizationUniforms,
    /// Frame counter
    frame_count: u64,
}

impl GPURenderer {
    /// Create a new GPU renderer
    pub async fn new(window: &'static winit::window::Window) -> Result<Self, GPURenderingError> {
        info!("Initializing GPU renderer");

        // Create wgpu instance
        let instance = Instance::new(&wgpu::InstanceDescriptor {
            backends: Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            backend_options: Default::default(),
        });

        // Create surface
        let surface = instance.create_surface(window)
            .map_err(|e| GPURenderingError::SurfaceCreation(format!("Failed to create surface: {}", e)))?;

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| GPURenderingError::DeviceInit("No suitable GPU adapter found".to_string()))?;

        info!("Selected GPU adapter: {}", adapter.get_info().name);

        // Request device
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Sonic Flow GPU Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .map_err(|e| GPURenderingError::DeviceInit(format!("Failed to create device: {}", e)))?;

        info!("GPU device created successfully");

        // Create shader compiler
        let compiler = ShaderCompiler::new();

        // Create initial uniforms
        let uniforms = AudioVisualizationUniforms::default();

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            surface,
            pipeline: None,
            compiler,
            uniforms,
            frame_count: 0,
        })
    }

    /// Initialize rendering pipeline
    pub async fn initialize_pipeline(&mut self) -> Result<(), GPURenderingError> {
        info!("Initializing rendering pipeline");

        // TODO: Implement proper surface cloning when needed
        // For now, return an error indicating surface cloning is not implemented
        return Err(GPURenderingError::PipelineCreation(
            "Surface cloning not implemented yet".to_string()
        ));

        Ok(())
    }

    /// Load and compile a shader
    pub fn load_shader(
        &mut self,
        source: &str,
        vertex_entry: &str,
        fragment_entry: &str,
    ) -> Result<CompiledShader, GPURenderingError> {
        info!("Loading shader: {} -> {}", vertex_entry, fragment_entry);

        let shader = self
            .compiler
            .compile_shader(source, vertex_entry, fragment_entry, &self.device)
            .map_err(|e| GPURenderingError::PipelineCreation(format!("Shader compilation failed: {}", e)))?;

        info!("Shader compiled successfully: {}", shader.metadata.name);

        Ok(shader)
    }

    /// Set the current shader
    pub fn set_shader(&mut self, shader: CompiledShader) -> Result<(), GPURenderingError> {
        if let Some(pipeline) = &mut self.pipeline {
            pipeline.set_shader(shader)?;
            info!("Shader set successfully");
        } else {
            return Err(GPURenderingError::PipelineCreation("Pipeline not initialized".to_string()));
        }

        Ok(())
    }

    /// Update audio data for visualization
    pub fn update_audio_data(&mut self, spectrum: &SpectrumData) -> Result<(), GPURenderingError> {
        // Update uniforms with audio data
        self.uniforms.time += 1.0 / 60.0; // Assuming 60 FPS
        
        // Copy spectrum bands to uniform buffer (up to 128 bands)
        let band_count = spectrum.bands.len().min(128);
        for i in 0..band_count {
            self.uniforms.spectrum_data[i] = spectrum.bands[i];
        }
        
        // Set audio levels (using available data)
        self.uniforms.audio_levels = [
            spectrum.peak_level,  // Left channel
            spectrum.rms_level,   // Right channel
            0.0,                  // LFE (not available)
            0.0,                  // Center (not available)
        ];

        // Update GPU buffer
        if let Some(pipeline) = &self.pipeline {
            pipeline.update_uniforms(&self.uniforms)?;
        }

        Ok(())
    }

    /// Render a frame
    pub fn render(&mut self) -> Result<(), GPURenderingError> {
        if let Some(pipeline) = &mut self.pipeline {
            pipeline.render()?;
            self.frame_count += 1;
        } else {
            return Err(GPURenderingError::Rendering("Pipeline not initialized".to_string()));
        }

        Ok(())
    }

    /// Resize the render target
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), GPURenderingError> {
        if let Some(pipeline) = &mut self.pipeline {
            pipeline.resize(width, height)?;
            info!("Resized render target to {}x{}", width, height);
        }

        Ok(())
    }

    /// Set blend mode
    pub fn set_blend_mode(&mut self, mode: BlendMode) -> Result<(), GPURenderingError> {
        if let Some(pipeline) = &mut self.pipeline {
            pipeline.set_blend_mode(mode);
            info!("Set blend mode: {:?}", mode);
        }

        Ok(())
    }

    /// Get current frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Get GPU adapter info
    pub fn adapter_info(&self) -> wgpu::AdapterInfo {
        self.adapter.get_info()
    }

    /// Get surface size
    pub fn size(&self) -> Option<(u32, u32)> {
        self.pipeline.as_ref().map(|p| p.size())
    }
}

/// Shader engine for managing multiple shaders
pub struct ShaderEngine {
    /// GPU renderer
    renderer: GPURenderer,
    /// Loaded shaders
    shaders: std::collections::HashMap<String, CompiledShader>,
    /// Current active shader
    active_shader: Option<String>,
}

impl ShaderEngine {
    /// Create a new shader engine
    pub async fn new(window: &'static winit::window::Window) -> Result<Self, GPURenderingError> {
        let renderer = GPURenderer::new(window).await?;
        let shaders = std::collections::HashMap::new();

        Ok(Self {
            renderer,
            shaders,
            active_shader: None,
        })
    }

    /// Initialize the engine
    pub async fn initialize(&mut self) -> Result<(), GPURenderingError> {
        self.renderer.initialize_pipeline().await?;
        info!("Shader engine initialized");
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
        let shader = self.renderer.load_shader(source, vertex_entry, fragment_entry)?;
        self.shaders.insert(name.to_string(), shader);
        info!("Loaded shader: {}", name);
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

    /// Activate a shader
    pub fn activate_shader(&mut self, name: &str) -> Result<(), GPURenderingError> {
        if let Some(shader) = self.shaders.get(name).cloned() {
            self.renderer.set_shader(shader)?;
            self.active_shader = Some(name.to_string());
            info!("Activated shader: {}", name);
        } else {
            return Err(GPURenderingError::PipelineCreation(format!("Shader not found: {}", name)));
        }

        Ok(())
    }

    /// Update audio data
    pub fn update_audio_data(&mut self, spectrum: &SpectrumData) -> Result<(), GPURenderingError> {
        self.renderer.update_audio_data(spectrum)
    }

    /// Render current frame
    pub fn render(&mut self) -> Result<(), GPURenderingError> {
        self.renderer.render()
    }

    /// Resize render target
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), GPURenderingError> {
        self.renderer.resize(width, height)
    }

    /// Set blend mode
    pub fn set_blend_mode(&mut self, mode: BlendMode) -> Result<(), GPURenderingError> {
        self.renderer.set_blend_mode(mode)
    }

    /// Get loaded shader names
    pub fn shader_names(&self) -> Vec<String> {
        self.shaders.keys().cloned().collect()
    }

    /// Get active shader name
    pub fn active_shader(&self) -> Option<&String> {
        self.active_shader.as_ref()
    }

    /// Get frame count
    pub fn frame_count(&self) -> u64 {
        self.renderer.frame_count()
    }

    /// Get GPU info
    pub fn gpu_info(&self) -> wgpu::AdapterInfo {
        self.renderer.adapter_info()
    }
}

/// Canvas implementation for GPU rendering
impl crate::types::Canvas for ShaderCanvas {
    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn set_blend_mode(&mut self, mode: BlendMode) {
        self.blend_mode = mode;
    }

    fn clear(&mut self, color: Color) -> Result<(), GPURenderingError> {
        // TODO: Implement clear operation
        debug!("Clear canvas with color: {:?}", color);
        Ok(())
    }

    fn draw_rect(&mut self, rect: Rect, color: Color) -> Result<(), GPURenderingError> {
        // TODO: Implement rectangle drawing
        debug!("Draw rectangle: {:?} with color: {:?}", rect, color);
        Ok(())
    }

    fn draw_spectrum_bars(&mut self, data: &SpectrumData) -> Result<(), GPURenderingError> {
        // TODO: Implement spectrum bars drawing
        debug!("Draw spectrum bars with {} frequency bins", data.bands.len());
        Ok(())
    }

    fn draw_waveform(&mut self, data: &[f32]) -> Result<(), GPURenderingError> {
        // TODO: Implement waveform drawing
        debug!("Draw waveform with {} samples", data.len());
        Ok(())
    }

    fn present(&mut self) -> Result<(), GPURenderingError> {
        // TODO: Implement frame presentation
        debug!("Present frame");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniforms_default() {
        let uniforms = AudioVisualizationUniforms::default();
        assert_eq!(uniforms.time, 0.0);
        assert_eq!(uniforms.sensitivity, 1.0);
        assert_eq!(uniforms.spectrum_data[0], 0.0);
    }

    #[test]
    fn test_color_conversion() {
        let color = Color::rgb(1.0, 0.5, 0.25);
        let wgpu_color = color.to_wgpu();
        assert_eq!(wgpu_color.r, 1.0);
        assert_eq!(wgpu_color.g, 0.5);
        assert_eq!(wgpu_color.b, 0.25);
        assert_eq!(wgpu_color.a, 1.0);
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
