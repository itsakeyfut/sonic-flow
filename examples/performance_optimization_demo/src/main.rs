//! Performance Optimization Demo
//!
//! This example demonstrates performance monitoring, optimization suggestions,
//! and benchmarking capabilities of the Sonic Flow framework.

use std::sync::{Arc, Mutex};
use std::time::Duration;
use anyhow::Result;
use tracing::{info, warn};
use sonic_shader::{
    EffectsManager, 
    AudioVisualizationBridge,
    PerformanceBenchmark, PerformanceThresholds, PerformanceMetrics,
    OptimizationSuggestion,
};
use sonic_core::audio::{
    player_manager::PlayerManager,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🎵 Sonic Flow - Performance Optimization Demo");
    println!("=============================================");

    // Create audio player manager
    let player_manager = Arc::new(Mutex::new(PlayerManager::new()?));
    println!("✅ Audio player manager created");

    // Create audio visualization bridge
    let bridge = AudioVisualizationBridge::new(player_manager.clone());
    println!("✅ Audio visualization bridge created");

    // Create effects manager
    let mut effects_manager = EffectsManager::new(bridge);
    println!("✅ Effects manager created");

    // Register built-in effects
    println!("\n🎨 Registering built-in effects...");
    if let Err(e) = effects_manager.register_builtin_effects() {
        warn!("Failed to register built-in effects: {}", e);
    } else {
        println!("✅ Built-in effects registered successfully");
    }

    // Print effects by performance impact
    println!("\n📊 Effects by Performance Impact:");
    let effects_by_impact = effects_manager.effects_by_performance_impact();
    for (name, config) in effects_by_impact {
        println!("   - {}: Impact {} (1-10)", name, config.performance_impact);
    }

    // Test performance monitoring
    println!("\n📈 Testing performance monitoring...");
    effects_manager.set_performance_monitoring(true);
    
    // Update effects manager for a few frames to collect metrics
    for i in 0..10 {
        if let Err(e) = effects_manager.update(0.016) { // 60 FPS delta time
            warn!("Failed to update effects manager: {}", e);
        } else {
            println!("✅ Frame {} updated", i + 1);
        }
        
        // Small delay
        tokio::time::sleep(tokio::time::Duration::from_millis(16)).await;
    }

    // Print current performance metrics
    if let Some(metrics) = effects_manager.performance_monitor().current_metrics() {
        println!("\n📊 Current Performance Metrics:");
        println!("   - Frame time: {:.2}ms", metrics.frame_time);
        println!("   - FPS: {:.1}", metrics.fps);
        println!("   - GPU memory: {:.1}MB", metrics.gpu_memory as f32 / 1024.0 / 1024.0);
        println!("   - CPU usage: {:.1}%", metrics.cpu_usage);
        println!("   - Audio processing time: {:.2}ms", metrics.audio_processing_time);
        println!("   - Draw calls: {}", metrics.draw_calls);
        println!("   - Vertex count: {}", metrics.vertex_count);
        println!("   - Fragment count: {}", metrics.fragment_count);
    }

    // Test performance statistics
    let statistics = effects_manager.performance_monitor().statistics();
    println!("\n📈 Performance Statistics:");
    println!("   - Total frames: {}", statistics.total_frames);
    println!("   - Total time: {:.2}s", statistics.total_time.as_secs_f32());
    println!("   - Average FPS: {:.1}", statistics.average_fps);
    println!("   - Min FPS: {:.1}", statistics.min_fps);
    println!("   - Max FPS: {:.1}", statistics.max_fps);
    println!("   - Average frame time: {:.2}ms", statistics.average_frame_time);
    println!("   - Average CPU usage: {:.1}%", statistics.average_cpu_usage);
    println!("   - Peak GPU memory: {:.1}MB", statistics.peak_gpu_memory as f32 / 1024.0 / 1024.0);

    // Test optimization suggestions
    println!("\n💡 Performance Optimization Suggestions:");
    let suggestions = effects_manager.performance_monitor().suggestions();
    if suggestions.is_empty() {
        println!("   - No optimization suggestions (performance is good!)");
    } else {
        for suggestion in suggestions {
            println!("   - {}", suggestion.description());
        }
    }

    // Test custom performance suggestions
    println!("\n🎯 Custom Performance Suggestions:");
    let custom_suggestions = effects_manager.get_performance_suggestions();
    if custom_suggestions.is_empty() {
        println!("   - No custom suggestions");
    } else {
        for suggestion in custom_suggestions {
            println!("   - {}", suggestion);
        }
    }

    // Test performance thresholds
    println!("\n⚙️ Testing performance thresholds...");
    let custom_thresholds = PerformanceThresholds {
        target_fps: 30.0, // Lower target for testing
        max_frame_time: 33.33, // 30 FPS = 33.33ms per frame
        max_cpu_usage: 50.0, // Lower CPU threshold
        max_gpu_memory_mb: 256, // Lower memory threshold
        max_audio_processing_time: 2.0, // Lower audio processing threshold
    };
    effects_manager.update_performance_thresholds(custom_thresholds);
    println!("✅ Updated performance thresholds");

    // Test effect switching and performance impact
    println!("\n🔄 Testing effect switching and performance impact...");
    let available_effects = effects_manager.available_effects();
    
    for effect_name in &available_effects {
        if let Err(e) = effects_manager.activate_effect(effect_name) {
            warn!("Failed to activate effect '{}': {}", effect_name, e);
        } else {
            println!("✅ Activated effect: {}", effect_name);
            
            // Update for a few frames to measure performance
            for i in 0..5 {
                if let Err(e) = effects_manager.update(0.016) {
                    warn!("Failed to update effects manager: {}", e);
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(16)).await;
            }
            
            // Print performance metrics for this effect
            if let Some(metrics) = effects_manager.performance_monitor().current_metrics() {
                println!("   - FPS: {:.1}, GPU Memory: {:.1}MB, CPU: {:.1}%", 
                    metrics.fps, 
                    metrics.gpu_memory as f32 / 1024.0 / 1024.0,
                    metrics.cpu_usage
                );
            }
        }
    }

    // Test performance benchmark
    println!("\n🏃 Running performance benchmark...");
    let mut benchmark = PerformanceBenchmark::new(Duration::from_secs(5), 60.0);
    
    let benchmark_results = benchmark.run(|| {
        // Simulate rendering a frame
        let mut metrics = PerformanceMetrics::default();
        metrics.draw_calls = 1;
        metrics.vertex_count = 4;
        metrics.fragment_count = 1920 * 1080;
        metrics.gpu_memory = 100 * 1024 * 1024; // 100 MB
        metrics.cpu_usage = 15.0;
        metrics.audio_processing_time = 2.0;
        metrics
    });
    
    benchmark_results.print_summary();

    // Test average metrics
    println!("\n📊 Average Metrics (last 10 frames):");
    if let Some(avg_metrics) = effects_manager.performance_monitor().average_metrics(10) {
        println!("   - Average FPS: {:.1}", avg_metrics.fps);
        println!("   - Average frame time: {:.2}ms", avg_metrics.frame_time);
        println!("   - Average GPU memory: {:.1}MB", avg_metrics.gpu_memory as f32 / 1024.0 / 1024.0);
        println!("   - Average CPU usage: {:.1}%", avg_metrics.cpu_usage);
    }

    // Test performance data export
    println!("\n💾 Testing performance data export...");
    let exported_data = effects_manager.performance_monitor().export_data();
    println!("   - Exported {} performance data points", exported_data.len());

    // Test performance monitoring disable/enable
    println!("\n🔧 Testing performance monitoring toggle...");
    effects_manager.set_performance_monitoring(false);
    println!("   - Performance monitoring disabled");
    
    // Update without monitoring
    for i in 0..3 {
        if let Err(e) = effects_manager.update(0.016) {
            warn!("Failed to update effects manager: {}", e);
        } else {
            println!("   - Frame {} updated (monitoring disabled)", i + 1);
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(16)).await;
    }
    
    effects_manager.set_performance_monitoring(true);
    println!("   - Performance monitoring re-enabled");

    // Print final statistics
    println!("\n📊 Final Performance Statistics:");
    let final_statistics = effects_manager.performance_monitor().statistics();
    println!("   - Total frames: {}", final_statistics.total_frames);
    println!("   - Total time: {:.2}s", final_statistics.total_time.as_secs_f32());
    println!("   - Average FPS: {:.1}", final_statistics.average_fps);
    println!("   - Peak GPU memory: {:.1}MB", final_statistics.peak_gpu_memory as f32 / 1024.0 / 1024.0);

    println!("\n🎉 Performance Optimization Demo completed!");
    println!("\n📝 Summary:");
    println!("   - Performance monitoring: Working");
    println!("   - Performance metrics: Collected");
    println!("   - Optimization suggestions: Generated");
    println!("   - Performance benchmarks: Executed");
    println!("   - Effect performance impact: Measured");
    println!("   - Performance thresholds: Configurable");
    println!("\n🚀 Ready for Phase 9: Final integration!");

    Ok(())
}
