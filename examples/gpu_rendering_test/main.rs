//! GPU Rendering Test Example
//! 
//! This example demonstrates actual GPU rendering using WebGPU for audio visualization.

use std::sync::{Arc, Mutex};
use tracing::{info, warn, error};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

use sonic_core::audio::player_manager::PlayerManager;
use sonic_shader::{
    AudioVisualizationBridge,
    VisualizationSettings,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting GPU rendering test");

    // Create event loop
    let event_loop = EventLoop::new()?;

    // Create window
    let window = WindowBuilder::new()
        .with_title("Sonic Flow GPU Rendering Test")
        .with_inner_size(winit::dpi::LogicalSize::new(800.0, 600.0))
        .build(&event_loop)?;

    // Create player manager
    let player_manager = Arc::new(Mutex::new(PlayerManager::new()?));

    // Create audio visualization bridge
    let mut bridge = AudioVisualizationBridge::new(player_manager);

    // Initialize GPU rendering
    match bridge.initialize_gpu(window).await {
        Ok(()) => {
            info!("GPU rendering initialized successfully");
            
            // Load test shaders
            load_test_shaders(&mut bridge)?;
            
            // Start visualization loop
            run_visualization_loop(event_loop, window, bridge).await?;
        }
        Err(e) => {
            error!("Failed to initialize GPU rendering: {}", e);
            warn!("Falling back to software mode");
            
            // Run in software mode
            run_software_visualization(event_loop, window, bridge).await?;
        }
    }

    Ok(())
}

/// Load test shaders for visualization
fn load_test_shaders(bridge: &mut AudioVisualizationBridge) -> Result<(), Box<dyn std::error::Error>> {
    info!("Loading test shaders");

    // Spectrum bars shader
    let spectrum_bars_shader = r#"
        struct VertexOutput {
            @builtin(position) position: vec4<f32>,
            @location(0) uv: vec2<f32>,
        };

        @vertex
        fn vertexMain(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>) -> VertexOutput {
            var output: VertexOutput;
            output.position = vec4<f32>(position, 1.0);
            output.uv = uv;
            return output;
        }

        @fragment
        fn fragmentMain(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
            let time = f32(0.0); // TODO: Pass time from uniforms
            let color = vec3<f32>(uv.x, uv.y, sin(time) * 0.5 + 0.5);
            return vec4<f32>(color, 1.0);
        }
    "#;

    bridge.load_visualization_shader(
        "spectrum_bars",
        spectrum_bars_shader,
        "vertexMain",
        "fragmentMain",
    )?;

    // Waveform shader
    let waveform_shader = r#"
        struct VertexOutput {
            @builtin(position) position: vec4<f32>,
            @location(0) uv: vec2<f32>,
        };

        @vertex
        fn vertexMain(@location(0) position: vec3<f32>, @location(1) uv: vec2<f32>) -> VertexOutput {
            var output: VertexOutput;
            output.position = vec4<f32>(position, 1.0);
            output.uv = uv;
            return output;
        }

        @fragment
        fn fragmentMain(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
            let wave = sin(uv.x * 10.0) * 0.5 + 0.5;
            let color = vec3<f32>(wave, 0.5, 1.0 - wave);
            return vec4<f32>(color, 1.0);
        }
    "#;

    bridge.load_visualization_shader(
        "waveform",
        waveform_shader,
        "vertexMain",
        "fragmentMain",
    )?;

    // Activate spectrum bars shader
    bridge.activate_shader("spectrum_bars")?;

    info!("Test shaders loaded successfully");
    Ok(())
}

/// Run the main visualization loop with GPU rendering
async fn run_visualization_loop(
    event_loop: EventLoop<()>,
    window: winit::window::Window,
    mut bridge: AudioVisualizationBridge,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting GPU visualization loop");

    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("Window close requested");
                elwt.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // Update audio data
                if let Err(e) = bridge.update_audio_data() {
                    warn!("Failed to update audio data: {}", e);
                }

                // Render frame
                if let Err(e) = bridge.render_frame() {
                    warn!("Failed to render frame: {}", e);
                }

                // Request next frame
                window.request_redraw();
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    })?;

    Ok(())
}

/// Run visualization in software mode (fallback)
async fn run_software_visualization(
    event_loop: EventLoop<()>,
    window: winit::window::Window,
    mut bridge: AudioVisualizationBridge,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting software visualization loop");

    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                info!("Window close requested");
                elwt.exit();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                // Update audio data (simulated)
                if let Err(e) = bridge.update_audio_data() {
                    warn!("Failed to update audio data: {}", e);
                }

                // In software mode, we just log the frame
                info!("Software frame rendered (frame count: {})", bridge.frame_count());

                // Request next frame
                window.request_redraw();
            }
            Event::AboutToWait => {
                window.request_redraw();
            }
            _ => {}
        }
    })?;

    Ok(())
}
