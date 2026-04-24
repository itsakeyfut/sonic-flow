//! Audio integration bridge for GPU visualization

use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};
use sonic_core::audio::{
    analysis::{SpectrumData, SpectrumAnalyzer},
    player_manager::PlayerManager,
};

use crate::{
    ShaderEngine,
    types::GPURenderingError,
};

/// Audio visualization bridge
pub struct AudioVisualizationBridge {
    /// Audio player manager
    player_manager: Arc<Mutex<PlayerManager>>,
    /// Spectrum analyzer
    spectrum_analyzer: SpectrumAnalyzer,
    /// GPU shader engine
    shader_engine: Option<ShaderEngine>,
    /// Current audio data
    current_spectrum: Option<SpectrumData>,
    /// Visualization settings
    settings: VisualizationSettings,
}

/// Visualization settings
#[derive(Debug, Clone)]
pub struct VisualizationSettings {
    /// Update frequency (Hz)
    pub update_frequency: f32,
    /// Sensitivity multiplier
    pub sensitivity: f32,
    /// Smoothing factor (0.0 = no smoothing, 1.0 = max smoothing)
    pub smoothing: f32,
    /// Number of frequency bands to visualize
    pub band_count: usize,
    /// Enable real-time visualization
    pub real_time: bool,
}

impl Default for VisualizationSettings {
    fn default() -> Self {
        Self {
            update_frequency: 60.0,
            sensitivity: 1.0,
            smoothing: 0.3,
            band_count: 128,
            real_time: true,
        }
    }
}

impl AudioVisualizationBridge {
    /// Create a new audio visualization bridge
    pub fn new(player_manager: Arc<Mutex<PlayerManager>>) -> Self {
        let spectrum_analyzer = SpectrumAnalyzer::new(1024, 44100, 128);
        
        Self {
            player_manager,
            spectrum_analyzer,
            shader_engine: None,
            current_spectrum: None,
            settings: VisualizationSettings::default(),
        }
    }

    /// Initialize GPU shader engine
    pub async fn initialize_gpu(&mut self, window: &'static winit::window::Window) -> Result<(), GPURenderingError> {
        info!("Initializing GPU shader engine for audio visualization");
        
        let mut shader_engine = ShaderEngine::new();
        shader_engine.initialize_gpu(window).await?;
        
        self.shader_engine = Some(shader_engine);
        info!("GPU shader engine initialized successfully");
        
        Ok(())
    }

    /// Load a shader for visualization
    pub fn load_visualization_shader(
        &mut self,
        name: &str,
        source: &str,
        vertex_entry: &str,
        fragment_entry: &str,
    ) -> Result<(), GPURenderingError> {
        if let Some(shader_engine) = &mut self.shader_engine {
            shader_engine.load_shader(name, source, vertex_entry, fragment_entry)?;
            info!("Loaded visualization shader: {}", name);
        } else {
            return Err(GPURenderingError::PipelineCreation("GPU engine not initialized".to_string()));
        }
        
        Ok(())
    }

    /// Activate a visualization shader
    pub fn activate_shader(&mut self, name: &str) -> Result<(), GPURenderingError> {
        if let Some(shader_engine) = &mut self.shader_engine {
            shader_engine.activate_shader(name)?;
            info!("Activated visualization shader: {}", name);
        } else {
            return Err(GPURenderingError::PipelineCreation("GPU engine not initialized".to_string()));
        }
        
        Ok(())
    }

    /// Update audio data and process for visualization
    pub fn update_audio_data(&mut self) -> Result<(), GPURenderingError> {
        // For now, we'll simulate audio data since we don't have real audio buffer access
        // In a real implementation, you would get audio data from the player manager
        
        // Simulate spectrum data
        let simulated_spectrum = self.simulate_audio_data();
        
        // Apply smoothing if enabled
        let smoothed_spectrum = if self.settings.smoothing > 0.0 {
            self.apply_smoothing(&simulated_spectrum)
        } else {
            simulated_spectrum
        };
        
        self.current_spectrum = Some(smoothed_spectrum.clone());
        
        // Update GPU with new audio data
        if let Some(shader_engine) = &mut self.shader_engine {
            shader_engine.update_audio_data(&smoothed_spectrum)?;
            debug!("Updated GPU with audio data: {} bands", smoothed_spectrum.bands.len());
        }
        
        Ok(())
    }

    /// Simulate audio data for demo purposes
    fn simulate_audio_data(&self) -> SpectrumData {
        use std::time::Instant;
        
        let time = Instant::now().elapsed().as_secs_f32();
        let mut bands = Vec::with_capacity(self.settings.band_count);
        
        for i in 0..self.settings.band_count {
            let frequency_factor = i as f32 / self.settings.band_count as f32;
            let base_amplitude = 0.3 + 0.7 * (1.0 - frequency_factor); // More energy in lower frequencies
            let time_variation = (time * 2.0 + i as f32 * 0.1).sin() * 0.2;
            let random_factor = (time * 3.0 + i as f32 * 0.05).sin() * 0.5 + 0.5;
            
            let amplitude = (base_amplitude + time_variation) * random_factor * self.settings.sensitivity;
            bands.push(amplitude.max(0.0).min(1.0));
        }
        
        let peak_level = bands.iter().fold(0.0_f32, |max, &val| max.max(val));
        let rms_level = (bands.iter().map(|&x| x * x).sum::<f32>() / bands.len() as f32).sqrt();
        
        SpectrumData {
            bands,
            peak_level,
            rms_level,
            timestamp: Instant::now(),
        }
    }

    /// Render visualization frame
    pub fn render_frame(&mut self) -> Result<(), GPURenderingError> {
        if let Some(shader_engine) = &mut self.shader_engine {
            shader_engine.render()?;
            debug!("Rendered visualization frame");
        } else {
            return Err(GPURenderingError::Rendering("GPU engine not initialized".to_string()));
        }
        
        Ok(())
    }

    /// Apply smoothing to spectrum data
    fn apply_smoothing(&self, spectrum: &SpectrumData) -> SpectrumData {
        let smoothing_factor = self.settings.smoothing;
        let mut smoothed_bands = Vec::with_capacity(spectrum.bands.len());
        
        for (i, &current_band) in spectrum.bands.iter().enumerate() {
            let smoothed_value = if let Some(prev_spectrum) = &self.current_spectrum {
                if i < prev_spectrum.bands.len() {
                    // Apply exponential smoothing
                    current_band * (1.0 - smoothing_factor) + prev_spectrum.bands[i] * smoothing_factor
                } else {
                    current_band
                }
            } else {
                current_band
            };
            
            smoothed_bands.push(smoothed_value);
        }
        
        SpectrumData {
            bands: smoothed_bands,
            peak_level: spectrum.peak_level,
            rms_level: spectrum.rms_level,
            timestamp: spectrum.timestamp,
        }
    }

    /// Get current spectrum data
    pub fn current_spectrum(&self) -> Option<&SpectrumData> {
        self.current_spectrum.as_ref()
    }

    /// Update visualization settings
    pub fn update_settings(&mut self, settings: VisualizationSettings) {
        self.settings = settings;
        info!("Updated visualization settings: {:?}", self.settings);
    }

    /// Get current settings
    pub fn settings(&self) -> &VisualizationSettings {
        &self.settings
    }

    /// Resize visualization
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), GPURenderingError> {
        if let Some(shader_engine) = &mut self.shader_engine {
            shader_engine.resize(width, height)?;
            info!("Resized visualization to {}x{}", width, height);
        }
        
        Ok(())
    }

    /// Set blend mode
    pub fn set_blend_mode(&mut self, mode: crate::types::BlendMode) -> Result<(), GPURenderingError> {
        if let Some(shader_engine) = &mut self.shader_engine {
            shader_engine.set_blend_mode(mode)?;
        }
        
        Ok(())
    }

    /// Get available shader names
    pub fn available_shaders(&self) -> Vec<String> {
        if let Some(shader_engine) = &self.shader_engine {
            shader_engine.shader_names()
        } else {
            Vec::new()
        }
    }

    /// Get active shader name
    pub fn active_shader(&self) -> Option<String> {
        if let Some(shader_engine) = &self.shader_engine {
            shader_engine.active_shader_name().cloned()
        } else {
            None
        }
    }

    /// Get frame count
    pub fn frame_count(&self) -> u64 {
        if let Some(shader_engine) = &self.shader_engine {
            shader_engine.frame_count()
        } else {
            0
        }
    }

    /// Get GPU info
    pub fn gpu_info(&self) -> Option<wgpu::AdapterInfo> {
        if let Some(shader_engine) = &self.shader_engine {
            shader_engine.gpu_info()
        } else {
            None
        }
    }
}

/// Real-time visualization loop
pub struct VisualizationLoop {
    /// Audio visualization bridge
    bridge: AudioVisualizationBridge,
    /// Running state
    running: bool,
    /// Frame rate limiter
    frame_limiter: FrameRateLimiter,
}

/// Frame rate limiter for consistent visualization
struct FrameRateLimiter {
    target_fps: f32,
    last_frame_time: std::time::Instant,
}

impl FrameRateLimiter {
    fn new(target_fps: f32) -> Self {
        Self {
            target_fps,
            last_frame_time: std::time::Instant::now(),
        }
    }

    fn wait_for_next_frame(&mut self) {
        let frame_duration = std::time::Duration::from_secs_f32(1.0 / self.target_fps);
        let elapsed = self.last_frame_time.elapsed();
        
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
        
        self.last_frame_time = std::time::Instant::now();
    }
}

impl VisualizationLoop {
    /// Create a new visualization loop
    pub fn new(bridge: AudioVisualizationBridge) -> Self {
        let frame_limiter = FrameRateLimiter::new(60.0);
        
        Self {
            bridge,
            running: false,
            frame_limiter,
        }
    }

    /// Start the visualization loop
    pub fn start(&mut self) {
        self.running = true;
        info!("Started visualization loop");
        
        while self.running {
            // Update audio data
            if let Err(e) = self.bridge.update_audio_data() {
                warn!("Failed to update audio data: {}", e);
            }
            
            // Render frame
            if let Err(e) = self.bridge.render_frame() {
                warn!("Failed to render frame: {}", e);
            }
            
            // Limit frame rate
            self.frame_limiter.wait_for_next_frame();
        }
    }

    /// Stop the visualization loop
    pub fn stop(&mut self) {
        self.running = false;
        info!("Stopped visualization loop");
    }

    /// Check if loop is running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Get bridge reference
    pub fn bridge(&self) -> &AudioVisualizationBridge {
        &self.bridge
    }

    /// Get mutable bridge reference
    pub fn bridge_mut(&mut self) -> &mut AudioVisualizationBridge {
        &mut self.bridge
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visualization_settings_default() {
        let settings = VisualizationSettings::default();
        assert_eq!(settings.update_frequency, 60.0);
        assert_eq!(settings.sensitivity, 1.0);
        assert_eq!(settings.smoothing, 0.3);
        assert_eq!(settings.band_count, 128);
        assert!(settings.real_time);
    }

    #[test]
    fn test_frame_rate_limiter() {
        let mut limiter = FrameRateLimiter::new(60.0);
        let start = std::time::Instant::now();
        
        limiter.wait_for_next_frame();
        
        let elapsed = start.elapsed();
        assert!(elapsed >= std::time::Duration::from_secs_f32(1.0 / 60.0));
    }
}
