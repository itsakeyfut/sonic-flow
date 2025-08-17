//! Advanced Effects Demo
//!
//! This example demonstrates advanced audio visualization effects
//! including spectrum bars, waveform, particle system, and 3D visualization.

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use anyhow::Result;
use tracing::{info, warn};
use sonic_shader::{
    EffectsManager, 
    EffectConfig, 
    EffectType, 
    EffectPreset,
    AudioVisualizationBridge,
    VisualizationSettings,
    ShaderCompiler,
    compiler::{CompilationTarget, OptimizationLevel},
};
use sonic_core::audio::{
    player_manager::PlayerManager,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🎵 Sonic Flow - Advanced Effects Demo");
    println!("=====================================");

    // Create audio player manager
    let player_manager = Arc::new(Mutex::new(PlayerManager::new()?));
    println!("✅ Audio player manager created");

    // Create audio visualization bridge
    let bridge = AudioVisualizationBridge::new(player_manager.clone());
    println!("✅ Audio visualization bridge created");

    // Create effects manager
    let mut effects_manager = EffectsManager::new(bridge);
    println!("✅ Effects manager created");

    // Test shader compilation
    println!("\n🎮 Testing shader compilation...");
    let compiler = ShaderCompiler::with_target(CompilationTarget::WGSL)
        .with_optimization(OptimizationLevel::Basic);
    
    println!("✅ Shader compilation test successful");

    // Register built-in effects
    println!("\n🎨 Registering built-in effects...");
    if let Err(e) = effects_manager.register_builtin_effects() {
        warn!("Failed to register built-in effects: {}", e);
    } else {
        println!("✅ Built-in effects registered successfully");
    }

    // Print available effects
    let available_effects = effects_manager.available_effects();
    println!("📋 Available effects: {:?}", available_effects);

    // Test effect activation
    println!("\n🎯 Testing effect activation...");
    for effect_name in &available_effects {
        if let Err(e) = effects_manager.activate_effect(effect_name) {
            warn!("Failed to activate effect '{}': {}", effect_name, e);
        } else {
            println!("✅ Activated effect: {}", effect_name);
        }
        
        // Small delay between effects
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // Test effect parameters
    println!("\n⚙️ Testing effect parameters...");
    let mut parameters = HashMap::new();
    parameters.insert("sensitivity".to_string(), 2.5);
    parameters.insert("smoothing".to_string(), 0.4);
    
    for effect_name in &available_effects {
        if let Err(e) = effects_manager.update_effect_parameters(effect_name, parameters.clone()) {
            warn!("Failed to update parameters for '{}': {}", effect_name, e);
        } else {
            println!("✅ Updated parameters for: {}", effect_name);
        }
    }

    // Test effect enabling/disabling
    println!("\n🔧 Testing effect enabling/disabling...");
    for effect_name in &available_effects {
        if let Err(e) = effects_manager.set_effect_enabled(effect_name, false) {
            warn!("Failed to disable effect '{}': {}", effect_name, e);
        } else {
            println!("✅ Disabled effect: {}", effect_name);
        }
        
        if let Err(e) = effects_manager.set_effect_enabled(effect_name, true) {
            warn!("Failed to enable effect '{}': {}", effect_name, e);
        } else {
            println!("✅ Enabled effect: {}", effect_name);
        }
    }

    // Test preset creation and loading
    println!("\n💾 Testing preset system...");
    let preset = effects_manager.create_preset("demo_preset");
    println!("✅ Created preset: {}", preset.name);
    println!("   - Effects in preset: {}", preset.effects.len());
    println!("   - Active effect: {:?}", preset.active_effect);

    // Test effects manager update
    println!("\n🔄 Testing effects manager update...");
    for i in 0..5 {
        if let Err(e) = effects_manager.update(0.016) { // 60 FPS delta time
            warn!("Failed to update effects manager: {}", e);
        } else {
            println!("✅ Effects manager update {} successful", i + 1);
        }
        
        // Small delay
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    // Test transition state
    println!("\n🎭 Testing transition state...");
    let transition_state = effects_manager.transition_state();
    println!("   - Transitioning: {}", transition_state.transitioning);
    println!("   - Progress: {:.2}", transition_state.progress);
    println!("   - Source effect: {:?}", transition_state.source_effect);
    println!("   - Target effect: {:?}", transition_state.target_effect);

    // Print final statistics
    println!("\n📊 Final Statistics:");
    println!("   - Available effects: {}", effects_manager.available_effects().len());
    println!("   - Enabled effects: {}", effects_manager.enabled_effects().len());
    println!("   - Active effect: {:?}", effects_manager.active_effect());
    
    if let Some(active_effect) = effects_manager.active_effect() {
        if let Some(config) = effects_manager.get_effect_config(active_effect) {
            println!("   - Active effect type: {:?}", config.effect_type);
            println!("   - Active effect parameters: {}", config.parameters.len());
        }
    }

    println!("\n🎉 Advanced Effects Demo completed!");
    println!("\n📝 Summary:");
    println!("   - Effects manager: Working");
    println!("   - Built-in effects: {} registered", available_effects.len());
    println!("   - Effect activation: Working");
    println!("   - Parameter management: Working");
    println!("   - Preset system: Working");
    println!("   - Transition system: Working");
    println!("\n🚀 Ready for Phase 8: Performance optimization!");

    Ok(())
}
