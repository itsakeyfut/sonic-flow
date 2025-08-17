//! Advanced effects manager for audio visualization

use std::collections::HashMap;
use tracing::{debug, info};
use crate::{
    AudioVisualizationBridge,
    types::{GPURenderingError, BlendMode},
};

/// Effect types available for visualization
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EffectType {
    SpectrumBars,
    Waveform,
    ParticleSystem,
    ThreeDimensional,
    Custom(String),
}

/// Effect configuration
#[derive(Debug, Clone)]
pub struct EffectConfig {
    /// Effect type
    pub effect_type: EffectType,
    /// Effect name
    pub name: String,
    /// Shader source code
    pub shader_source: String,
    /// Vertex entry point
    pub vertex_entry: String,
    /// Fragment entry point
    pub fragment_entry: String,
    /// Effect parameters
    pub parameters: HashMap<String, f32>,
    /// Blend mode
    pub blend_mode: BlendMode,
    /// Enabled state
    pub enabled: bool,
}

impl EffectConfig {
    /// Create a new effect configuration
    pub fn new(
        effect_type: EffectType,
        name: String,
        shader_source: String,
        vertex_entry: String,
        fragment_entry: String,
    ) -> Self {
        Self {
            effect_type,
            name,
            shader_source,
            vertex_entry,
            fragment_entry,
            parameters: HashMap::new(),
            blend_mode: BlendMode::Normal,
            enabled: true,
        }
    }

    /// Set a parameter value
    pub fn set_parameter(&mut self, name: &str, value: f32) {
        self.parameters.insert(name.to_string(), value);
    }

    /// Get a parameter value
    pub fn get_parameter(&self, name: &str) -> Option<f32> {
        self.parameters.get(name).copied()
    }
}

/// Advanced effects manager
pub struct EffectsManager {
    /// Audio visualization bridge
    bridge: AudioVisualizationBridge,
    /// Registered effects
    effects: HashMap<String, EffectConfig>,
    /// Active effect
    active_effect: Option<String>,
    /// Effect transition state
    transition_state: TransitionState,
}

/// Transition state for smooth effect switching
#[derive(Debug, Clone)]
pub struct TransitionState {
    /// Currently transitioning
    pub transitioning: bool,
    /// Transition progress (0.0 to 1.0)
    pub progress: f32,
    /// Transition duration
    pub duration: f32,
    /// Source effect
    pub source_effect: Option<String>,
    /// Target effect
    pub target_effect: Option<String>,
}

impl TransitionState {
    fn new() -> Self {
        Self {
            transitioning: false,
            progress: 0.0,
            duration: 1.0,
            source_effect: None,
            target_effect: None,
        }
    }

    fn start_transition(&mut self, from: Option<String>, to: Option<String>) {
        self.transitioning = true;
        self.progress = 0.0;
        self.source_effect = from;
        self.target_effect = to;
    }

    fn update(&mut self, delta_time: f32) {
        if self.transitioning {
            self.progress += delta_time / self.duration;
            if self.progress >= 1.0 {
                self.transitioning = false;
                self.progress = 1.0;
            }
        }
    }

    fn get_transition_factor(&self) -> f32 {
        // Smooth easing function
        let t = self.progress;
        t * t * (3.0 - 2.0 * t)
    }
}

impl EffectsManager {
    /// Create a new effects manager
    pub fn new(bridge: AudioVisualizationBridge) -> Self {
        Self {
            bridge,
            effects: HashMap::new(),
            active_effect: None,
            transition_state: TransitionState::new(),
        }
    }

    /// Register a built-in effect
    pub fn register_builtin_effects(&mut self) -> Result<(), GPURenderingError> {
        // Register spectrum bars effect
        let spectrum_bars_source = include_str!("shaders/spectrum_bars.hlsl");
        let spectrum_config = EffectConfig::new(
            EffectType::SpectrumBars,
            "spectrum_bars".to_string(),
            spectrum_bars_source.to_string(),
            "vertexMain".to_string(),
            "fragmentMain".to_string(),
        );
        self.register_effect(spectrum_config)?;

        // Register waveform effect
        let waveform_source = include_str!("shaders/waveform.hlsl");
        let waveform_config = EffectConfig::new(
            EffectType::Waveform,
            "waveform".to_string(),
            waveform_source.to_string(),
            "vertexMain".to_string(),
            "fragmentMain".to_string(),
        );
        self.register_effect(waveform_config)?;

        // Register particle system effect
        let particle_source = include_str!("shaders/particle_system.hlsl");
        let particle_config = EffectConfig::new(
            EffectType::ParticleSystem,
            "particle_system".to_string(),
            particle_source.to_string(),
            "vertexMain".to_string(),
            "fragmentMain".to_string(),
        );
        self.register_effect(particle_config)?;

        // Register 3D visualizer effect
        let three_d_source = include_str!("shaders/3d_visualizer.hlsl");
        let three_d_config = EffectConfig::new(
            EffectType::ThreeDimensional,
            "3d_visualizer".to_string(),
            three_d_source.to_string(),
            "vertexMain".to_string(),
            "fragmentMain".to_string(),
        );
        self.register_effect(three_d_config)?;

        info!("Registered {} built-in effects", self.effects.len());
        Ok(())
    }

    /// Register a custom effect
    pub fn register_effect(&mut self, config: EffectConfig) -> Result<(), GPURenderingError> {
        let name = config.name.clone();
        
        // Note: We don't load the shader immediately since the GPU engine might not be initialized
        // The shader will be loaded when the effect is first activated
        
        self.effects.insert(name.clone(), config);
        info!("Registered effect: {}", name);
        
        Ok(())
    }

    /// Activate an effect
    pub fn activate_effect(&mut self, name: &str) -> Result<(), GPURenderingError> {
        if let Some(config) = self.effects.get(name) {
            if !config.enabled {
                return Err(GPURenderingError::PipelineCreation(
                    format!("Effect '{}' is disabled", name)
                ));
            }
            
            // Start transition if there's an active effect
            if let Some(current_effect) = &self.active_effect {
                if current_effect != name {
                    self.transition_state.start_transition(
                        Some(current_effect.clone()),
                        Some(name.to_string())
                    );
                }
            }
            
            // Try to load the shader if it hasn't been loaded yet
            // This will fail gracefully if the GPU engine is not initialized
            let _ = self.bridge.load_visualization_shader(
                &name,
                &config.shader_source,
                &config.vertex_entry,
                &config.fragment_entry,
            );
            
            // Try to activate the effect (this might fail if GPU is not initialized)
            let _ = self.bridge.activate_shader(name);
            self.active_effect = Some(name.to_string());
            
            // Try to set blend mode
            let _ = self.bridge.set_blend_mode(config.blend_mode);
            
            info!("Activated effect: {}", name);
        } else {
            return Err(GPURenderingError::PipelineCreation(
                format!("Effect '{}' not found", name)
            ));
        }
        
        Ok(())
    }

    /// Update effect parameters
    pub fn update_effect_parameters(&mut self, name: &str, parameters: HashMap<String, f32>) -> Result<(), GPURenderingError> {
        if let Some(config) = self.effects.get_mut(name) {
            for (param_name, value) in parameters {
                config.set_parameter(&param_name, value);
            }
            debug!("Updated parameters for effect: {}", name);
        } else {
            return Err(GPURenderingError::PipelineCreation(
                format!("Effect '{}' not found", name)
            ));
        }
        
        Ok(())
    }

    /// Enable or disable an effect
    pub fn set_effect_enabled(&mut self, name: &str, enabled: bool) -> Result<(), GPURenderingError> {
        if let Some(config) = self.effects.get_mut(name) {
            config.enabled = enabled;
            info!("{} effect: {}", if enabled { "Enabled" } else { "Disabled" }, name);
        } else {
            return Err(GPURenderingError::PipelineCreation(
                format!("Effect '{}' not found", name)
            ));
        }
        
        Ok(())
    }

    /// Get available effect names
    pub fn available_effects(&self) -> Vec<String> {
        self.effects.keys().cloned().collect()
    }

    /// Get enabled effect names
    pub fn enabled_effects(&self) -> Vec<String> {
        self.effects
            .iter()
            .filter(|(_, config)| config.enabled)
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Get active effect name
    pub fn active_effect(&self) -> Option<&String> {
        self.active_effect.as_ref()
    }

    /// Get effect configuration
    pub fn get_effect_config(&self, name: &str) -> Option<&EffectConfig> {
        self.effects.get(name)
    }

    /// Update the effects manager
    pub fn update(&mut self, delta_time: f32) -> Result<(), GPURenderingError> {
        // Update transition state
        self.transition_state.update(delta_time);
        
        // Update audio data
        self.bridge.update_audio_data()?;
        
        // Render frame
        self.bridge.render_frame()?;
        
        Ok(())
    }

    /// Get bridge reference
    pub fn bridge(&self) -> &AudioVisualizationBridge {
        &self.bridge
    }

    /// Get mutable bridge reference
    pub fn bridge_mut(&mut self) -> &mut AudioVisualizationBridge {
        &mut self.bridge
    }

    /// Get transition state
    pub fn transition_state(&self) -> &TransitionState {
        &self.transition_state
    }

    /// Create a preset configuration
    pub fn create_preset(&self, name: &str) -> EffectPreset {
        let mut preset = EffectPreset::new(name.to_string());
        
        for (effect_name, config) in &self.effects {
            preset.add_effect_config(effect_name.clone(), config.clone());
        }
        
        if let Some(active_effect) = &self.active_effect {
            preset.set_active_effect(active_effect.clone());
        }
        
        preset
    }

    /// Load a preset configuration
    pub fn load_preset(&mut self, preset: &EffectPreset) -> Result<(), GPURenderingError> {
        // Load effect configurations
        for (effect_name, config) in &preset.effects {
            if let Some(existing_config) = self.effects.get_mut(effect_name) {
                existing_config.parameters = config.parameters.clone();
                existing_config.blend_mode = config.blend_mode;
                existing_config.enabled = config.enabled;
            }
        }
        
        // Activate the preset's active effect
        if let Some(active_effect) = &preset.active_effect {
            self.activate_effect(active_effect)?;
        }
        
        info!("Loaded preset: {}", preset.name);
        Ok(())
    }
}

/// Effect preset for saving and loading configurations
#[derive(Debug, Clone)]
pub struct EffectPreset {
    /// Preset name
    pub name: String,
    /// Effect configurations
    pub effects: HashMap<String, EffectConfig>,
    /// Active effect
    pub active_effect: Option<String>,
}

impl EffectPreset {
    /// Create a new preset
    pub fn new(name: String) -> Self {
        Self {
            name,
            effects: HashMap::new(),
            active_effect: None,
        }
    }

    /// Add an effect configuration
    pub fn add_effect_config(&mut self, name: String, config: EffectConfig) {
        self.effects.insert(name, config);
    }

    /// Set the active effect
    pub fn set_active_effect(&mut self, name: String) {
        self.active_effect = Some(name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_config_creation() {
        let config = EffectConfig::new(
            EffectType::SpectrumBars,
            "test_effect".to_string(),
            "shader_source".to_string(),
            "vertexMain".to_string(),
            "fragmentMain".to_string(),
        );
        
        assert_eq!(config.name, "test_effect");
        assert_eq!(config.effect_type, EffectType::SpectrumBars);
        assert!(config.enabled);
    }

    #[test]
    fn test_effect_config_parameters() {
        let mut config = EffectConfig::new(
            EffectType::Waveform,
            "test_effect".to_string(),
            "shader_source".to_string(),
            "vertexMain".to_string(),
            "fragmentMain".to_string(),
        );
        
        config.set_parameter("sensitivity", 2.0);
        assert_eq!(config.get_parameter("sensitivity"), Some(2.0));
        assert_eq!(config.get_parameter("nonexistent"), None);
    }

    #[test]
    fn test_transition_state() {
        let mut state = TransitionState::new();
        assert!(!state.transitioning);
        
        state.start_transition(Some("effect1".to_string()), Some("effect2".to_string()));
        assert!(state.transitioning);
        assert_eq!(state.progress, 0.0);
        
        state.update(0.5);
        assert!(state.progress > 0.0);
    }
}
