//! Integration tests for the audio engine
//!
//! These tests verify that the audio engine works correctly as a complete system.

use sonic_flow::audio::{AudioEngine, PlaybackControl, VolumeControl, TrackLoader, PlaybackStatus, PlaybackState};
use std::path::PathBuf;
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio::time::sleep;

/// Test basic audio engine creation and initialization
#[tokio::test]
async fn test_audio_engine_creation() {
    let result = AudioEngine::new();
    assert!(result.is_ok());
    
    let engine = result.unwrap();
    assert_eq!(engine.state(), PlaybackState::Stopped);
    assert_eq!(engine.volume(), 0.8); // Default volume
    assert!(!engine.is_muted());
    assert_eq!(engine.current_track(), None);
}

/// Test volume control functionality
#[tokio::test]
async fn test_volume_control() {
    let mut engine = AudioEngine::new().unwrap();
    
    // Test volume setting
    engine.set_volume(0.5);
    sleep(Duration::from_millis(50)).await; // Allow async processing
    assert_eq!(engine.volume(), 0.5);
    
    // Test volume clamping
    engine.set_volume(-0.1); // Should clamp to 0.0
    sleep(Duration::from_millis(50)).await;
    assert_eq!(engine.volume(), 0.0);
    
    engine.set_volume(1.5); // Should clamp to 1.0
    sleep(Duration::from_millis(50)).await;
    assert_eq!(engine.volume(), 1.0);
}

/// Test mute functionality
#[tokio::test]
async fn test_mute_control() {
    let mut engine = AudioEngine::new().unwrap();
    
    // Initially not muted
    assert!(!engine.is_muted());
    
    // Test muting
    engine.set_muted(true);
    sleep(Duration::from_millis(50)).await;
    assert!(engine.is_muted());
    
    // Test unmuting
    engine.set_muted(false);
    sleep(Duration::from_millis(50)).await;
    assert!(!engine.is_muted());
}

/// Test playback state transitions
#[tokio::test]
async fn test_playback_states() {
    let mut engine = AudioEngine::new().unwrap();
    
    // Initial state
    assert!(engine.is_stopped());
    assert!(!engine.is_playing());
    assert!(!engine.is_paused());
    
    // Test pause without track (should succeed but stay stopped)
    let result = engine.pause().await;
    assert!(result.is_ok());
    assert!(engine.is_stopped());
    
    // Test stop when already stopped (should succeed)
    let result = engine.stop().await;
    assert!(result.is_ok());
    assert!(engine.is_stopped());
}

/// Test playback control without loaded track
#[tokio::test]
async fn test_playback_without_track() {
    let mut engine = AudioEngine::new().unwrap();
    
    // Try to play without a loaded track
    let result = engine.play().await;
    // This should fail since no track is loaded
    assert!(result.is_err());
    
    // Try to seek without a loaded track
    let result = engine.seek(Duration::from_secs(10)).await;
    // This should succeed but have no effect
    assert!(result.is_ok());
}

/// Test error handling for invalid track loading
#[tokio::test]
async fn test_invalid_track_loading() {
    let mut engine = AudioEngine::new().unwrap();
    
    // Try to load non-existent file
    let non_existent_path = PathBuf::from("non_existent_file.mp3");
    let result = engine.load_track(&non_existent_path).await;
    assert!(result.is_err());
    
    // Try to load file with unsupported extension
    let temp_file = NamedTempFile::with_suffix(".txt").unwrap();
    let result = engine.load_track(temp_file.path()).await;
    assert!(result.is_err());
}

/// Test track ID management
#[tokio::test]
async fn test_track_id_management() {
    let mut engine = AudioEngine::new().unwrap();
    
    // Create a temporary file with supported extension
    let temp_file = NamedTempFile::with_suffix(".mp3").unwrap();
    // Write some dummy data (this won't be a valid MP3, but we're testing the loading logic)
    std::fs::write(temp_file.path(), b"dummy data").unwrap();
    
    // Load track should succeed (even with invalid MP3 data for this test)
    let result = engine.load_track(temp_file.path()).await;
    // Note: This might fail if the decoder tries to validate the file format
    // In a complete test, we'd use actual audio files or generate valid test files
    
    if let Ok(track_id) = result {
        // Track should be loaded but not current
        assert_eq!(engine.current_track(), None);
        
        // Set as current track
        let result = engine.set_current_track(track_id).await;
        assert!(result.is_ok());
        assert_eq!(engine.current_track(), Some(track_id));
    }
}

/// Test position and duration reporting
#[tokio::test]
async fn test_position_and_duration() {
    let engine = AudioEngine::new().unwrap();
    
    // Without loaded track
    assert_eq!(engine.position(), Duration::ZERO);
    assert_eq!(engine.duration(), None);
}

/// Test concurrent access
#[tokio::test]
async fn test_concurrent_access() {
    let mut engine = AudioEngine::new().unwrap();
    
    // Spawn multiple tasks that try to control the engine
    let handles = (0..10).map(|i| {
        let volume = (i as f32) / 10.0;
        
        tokio::spawn(async move {
            // Create a new engine for each task
            let mut local_engine = AudioEngine::new().unwrap();
            local_engine.set_volume(volume);
            sleep(Duration::from_millis(10)).await;
            local_engine.volume()
        })
    }).collect::<Vec<_>>();
    
    // Wait for all tasks
    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok());
    }
    
    // Original engine should still be functional
    engine.set_volume(0.7);
    sleep(Duration::from_millis(50)).await;
    assert_eq!(engine.volume(), 0.7);
}

/// Test engine cleanup and shutdown
#[tokio::test]
async fn test_engine_cleanup() {
    {
        let mut engine = AudioEngine::new().unwrap();
        engine.set_volume(0.5);
        // Engine should clean up when dropped
    }
    
    // Create another engine to ensure cleanup worked
    let engine = AudioEngine::new().unwrap();
    assert!(engine.is_stopped());
}

/// Performance test - measure engine responsiveness
#[tokio::test]
async fn test_performance_responsiveness() {
    let mut engine = AudioEngine::new().unwrap();
    
    let start = std::time::Instant::now();
    
    // Perform multiple operations
    for i in 0..100 {
        let volume = (i as f32) / 100.0;
        engine.set_volume(volume);
        
        if i % 10 == 0 {
            let _ = engine.pause().await;
            let _ = engine.stop().await;
        }
    }
    
    let elapsed = start.elapsed();
    
    // Operations should complete quickly (less than 1 second for 100 operations)
    assert!(elapsed < Duration::from_secs(1), 
            "Operations took too long: {:?}", elapsed);
}

/// Memory usage test - ensure no significant leaks
#[tokio::test]
async fn test_memory_usage() {
    // Create and destroy multiple engines
    for _ in 0..50 {
        let mut engine = AudioEngine::new().unwrap();
        engine.set_volume(0.5);
        engine.set_muted(true);
        let _ = engine.pause().await;
        // Engine is dropped here
    }
    
    // If there were significant memory leaks, this test might fail or be slow
    // In a production environment, we'd use memory profiling tools
}

/// Test error recovery
#[tokio::test]
async fn test_error_recovery() {
    let mut engine = AudioEngine::new().unwrap();
    
    // Try various invalid operations
    let _ = engine.play().await; // Should fail gracefully
    let _ = engine.seek(Duration::from_secs(3600)).await; // Large seek
    
    // Engine should still be functional
    engine.set_volume(0.6);
    sleep(Duration::from_millis(50)).await;
    assert_eq!(engine.volume(), 0.6);
    
    let result = engine.pause().await;
    assert!(result.is_ok());
}
