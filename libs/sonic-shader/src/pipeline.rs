//! GPU rendering pipeline implementation

use std::sync::Arc;

use tracing::info;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Buffer, BufferDescriptor,
    BufferUsages, ColorTargetState, ColorWrites, CommandEncoder, Device, FragmentState,
    MultisampleState, PipelineLayoutDescriptor, PrimitiveState, PrimitiveTopology, Queue,
    RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, Surface, SurfaceConfiguration,
    TextureView, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
    Adapter,
};

use crate::types::{
    AudioVisualizationUniforms, BlendMode, Color, CompiledShader, GPURenderingError,
    ShaderCanvas,
};

/// GPU rendering pipeline for audio visualization
pub struct RenderingPipeline {
    /// GPU device
    device: Arc<Device>,
    /// GPU command queue
    queue: Arc<Queue>,
    /// Render surface
    surface: Surface<'static>,
    /// Surface configuration
    surface_config: SurfaceConfiguration,
    /// Current render pipeline
    render_pipeline: Option<RenderPipeline>,
    /// Uniform buffer for shader data
    uniform_buffer: Option<Buffer>,
    /// Bind group for uniforms
    bind_group: Option<BindGroup>,
    /// Vertex buffer for geometry
    vertex_buffer: Option<Buffer>,
    /// Index buffer for geometry
    index_buffer: Option<Buffer>,
    /// Current shader
    current_shader: Option<CompiledShader>,
    /// Current blend mode
    blend_mode: BlendMode,
}

impl RenderingPipeline {
    /// Create a new rendering pipeline
    pub async fn new(
        surface: Surface<'static>, 
        device: Device, 
        queue: Queue,
        adapter: &Adapter,
    ) -> Result<Self, GPURenderingError> {
        let device = Arc::new(device);
        let queue = Arc::new(queue);

        // Get surface capabilities
        let surface_caps = surface.get_capabilities(adapter);
        
        // Choose surface format
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        // Configure surface
        let surface_config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: 800,
            height: 600,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        info!("Created rendering pipeline with surface format: {:?}", surface_format);

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            render_pipeline: None,
            uniform_buffer: None,
            bind_group: None,
            vertex_buffer: None,
            index_buffer: None,
            current_shader: None,
            blend_mode: BlendMode::Normal,
        })
    }

    /// Set the shader for rendering
    pub fn set_shader(&mut self, shader: CompiledShader) -> Result<(), GPURenderingError> {
        info!("Setting shader: {}", shader.metadata.name);

        // Create uniform buffer
        let uniform_buffer = self.create_uniform_buffer()?;

        // Create bind group
        let bind_group = self.create_bind_group(&shader, &uniform_buffer)?;

        // Create render pipeline
        let render_pipeline = self.create_render_pipeline(&shader)?;

        // Update state
        self.current_shader = Some(shader);
        self.uniform_buffer = Some(uniform_buffer);
        self.bind_group = Some(bind_group);
        self.render_pipeline = Some(render_pipeline);

        Ok(())
    }

    /// Create uniform buffer for shader data
    fn create_uniform_buffer(&self) -> Result<Buffer, GPURenderingError> {
        let uniform_data = AudioVisualizationUniforms::default();
        let uniform_buffer = self.device.create_buffer(&BufferDescriptor {
            label: Some("Audio Visualization Uniform Buffer"),
            size: std::mem::size_of::<AudioVisualizationUniforms>() as u64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Upload initial data
        self.queue.write_buffer(&uniform_buffer, 0, bytemuck::cast_slice(&[uniform_data]));

        Ok(uniform_buffer)
    }

    /// Create bind group for uniforms
    fn create_bind_group(
        &self,
        shader: &CompiledShader,
        uniform_buffer: &Buffer,
    ) -> Result<BindGroup, GPURenderingError> {
        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Audio Visualization Bind Group"),
            layout: &shader.bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: BindingResource::Buffer(uniform_buffer.as_entire_buffer_binding()),
            }],
        });

        Ok(bind_group)
    }

    /// Create render pipeline
    fn create_render_pipeline(&self, shader: &CompiledShader) -> Result<RenderPipeline, GPURenderingError> {
        let pipeline_layout = self.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Audio Visualization Pipeline Layout"),
            bind_group_layouts: &[&shader.bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = self.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Audio Visualization Pipeline"),
            layout: Some(&pipeline_layout),
            cache: None,
            vertex: VertexState {
                module: &shader.module,
                entry_point: Some(&shader.vertex_entry),
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<[f32; 2]>() as u64,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &[VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: VertexFormat::Float32x2,
                    }],
                }],
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: &shader.module,
                entry_point: Some(&shader.fragment_entry),
                targets: &[Some(ColorTargetState {
                    format: self.surface_config.format,
                    blend: Some(self.get_blend_state()),
                    write_mask: ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Ok(render_pipeline)
    }

    /// Get blend state based on current blend mode
    fn get_blend_state(&self) -> wgpu::BlendState {
        match self.blend_mode {
            BlendMode::Normal => wgpu::BlendState::REPLACE,
            BlendMode::Add => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
            },
            BlendMode::Multiply => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::Dst,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
            },
            BlendMode::Screen => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::OneMinusSrc,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
            },
            BlendMode::Overlay => wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
            },
        }
    }

    /// Update uniform buffer with new data
    pub fn update_uniforms(&self, uniforms: &AudioVisualizationUniforms) -> Result<(), GPURenderingError> {
        if let Some(uniform_buffer) = &self.uniform_buffer {
            self.queue.write_buffer(uniform_buffer, 0, bytemuck::cast_slice(&[*uniforms]));
        }
        Ok(())
    }

    /// Render a frame
    pub fn render(&mut self) -> Result<(), GPURenderingError> {
        let frame = self.surface.get_current_texture()
            .map_err(|e| GPURenderingError::Rendering(format!("Failed to get current texture: {}", e)))?;

        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Audio Visualization Command Encoder"),
        });

        // Clear the screen
        self.clear_screen(&view, &mut encoder)?;

        // Render visualization
        if let (Some(pipeline), Some(bind_group)) = (&self.render_pipeline, &self.bind_group) {
            self.render_visualization(&view, &mut encoder, pipeline, bind_group)?;
        }

        // Submit commands
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        Ok(())
    }

    /// Clear the screen with a color
    fn clear_screen(&self, view: &TextureView, encoder: &mut CommandEncoder) -> Result<(), GPURenderingError> {
        let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Clear Screen"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(Color::rgb(0.1, 0.1, 0.1).to_wgpu()),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        // Clear pass is complete when dropped
        drop(render_pass);

        Ok(())
    }

    /// Render visualization
    fn render_visualization(
        &self,
        view: &TextureView,
        encoder: &mut CommandEncoder,
        pipeline: &RenderPipeline,
        bind_group: &BindGroup,
    ) -> Result<(), GPURenderingError> {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Audio Visualization"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);

        // Render full-screen quad
        if let Some(vertex_buffer) = &self.vertex_buffer {
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..6, 0..1);
        }

        Ok(())
    }

    /// Resize the surface
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), GPURenderingError> {
        if width > 0 && height > 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
            info!("Resized surface to {}x{}", width, height);
        }
        Ok(())
    }

    /// Set blend mode
    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.blend_mode = mode;
        // Recreate pipeline with new blend state if shader is set
        if let Some(shader) = &self.current_shader {
            if let Ok(pipeline) = self.create_render_pipeline(shader) {
                self.render_pipeline = Some(pipeline);
            }
        }
    }

    /// Get surface size
    pub fn size(&self) -> (u32, u32) {
        (self.surface_config.width, self.surface_config.height)
    }

    /// Get current shader
    pub fn current_shader(&self) -> Option<&CompiledShader> {
        self.current_shader.as_ref()
    }
}

impl ShaderCanvas {
    /// Create a new shader canvas
    pub async fn new(
        surface: Surface<'static>, 
        device: Device, 
        queue: Queue,
        adapter: &Adapter,
    ) -> Result<Self, GPURenderingError> {
        let pipeline = RenderingPipeline::new(surface, device, queue, adapter).await?;
        
        Ok(Self {
            width: pipeline.size().0,
            height: pipeline.size().1,
            blend_mode: BlendMode::Normal,
            device: pipeline.device.clone(),
            queue: pipeline.queue.clone(),
            surface: pipeline.surface,
            render_pipeline: pipeline.render_pipeline,
            uniform_buffer: pipeline.uniform_buffer,
            bind_group: pipeline.bind_group,
        })
    }

    /// Set blend mode
    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.blend_mode = mode;
    }

    /// Get canvas size
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blend_state_creation() {
        let pipeline = RenderingPipeline {
            device: Arc::new(unsafe { std::mem::zeroed() }),
            queue: Arc::new(unsafe { std::mem::zeroed() }),
            surface: unsafe { std::mem::zeroed() },
            surface_config: unsafe { std::mem::zeroed() },
            render_pipeline: None,
            uniform_buffer: None,
            bind_group: None,
            vertex_buffer: None,
            index_buffer: None,
            current_shader: None,
            blend_mode: BlendMode::Normal,
        };

        let blend_state = pipeline.get_blend_state();
        assert!(matches!(blend_state, wgpu::BlendState::REPLACE));
    }
}
