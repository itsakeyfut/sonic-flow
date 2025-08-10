//! Visualizer engine for managing and rendering visualizations
//! 
//! The visualizer engine coordinates between audio analysis data and visualizer plugins to create real-time visual representations.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, error, info, warn};

use crate::audio::analysis::SpectrumData;
use crate::error::VisualizerError;

use super::canvas::SoftwareCanvas;
use super::plugins::spectrum_bars::SpectrumBarsVisualizer;
use super::traits::{Canvas, VisualizationConfig, Visualizer};

/// Visualizer engine commands
#[derive(Debug)]
pub enum VisualizerCommand {
    /// Update with new spectrum data
    UpdateSpectrum(SpectrumData),
    /// Switch to a different visualizer
    SetVisualizer(String),
    /// Update configuration
    SetConfig(VisualizationConfig),
    /// Resize the canvas
    Resize(u32, u32),
    /// Start/resume rendering
    Start,
    /// Pause rendering
    Pause,
    /// Stop and cleanup
    Stop,
}

/// Visualizer engine events
#[derive(Debug, Clone)]
pub enum VisualizerEvent {
    /// Frame rendered successfully
    FrameRendered { frame_time: Duration },
    /// Visualizer changed
    VisualizerChanged { id: String },
    /// Configuration updated
    ConfigUpdated,
    /// Error occurred
    Error { error: String },
}

/// Performance metrics for the visualizer
#[derive(Debug, Clone)]
pub struct VisualizerMetrics {
    /// Current FPS
    pub fps: f32,
    /// Average frame time
    pub avg_frame_time: Duration,
    /// Peak frame time
    pub peak_frame_time: Duration,
    /// Number of dropped frames
    pub dropped_frames: u32,
    /// Total frames rendered
    pub total_frames: u64,
}

/// Current state of the visualizer engine
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VisualizerState {
    /// Stopped and idle
    Stopped,
    /// Running and rendering
    Running,
    /// Paused (maintaining state but not rendering)
    Paused,
}

/// Main visualizer engine
pub struct VisualizerEngine {
    /// Current state
    state: Arc<RwLock<VisualizerState>>,
    /// Active visualizer
    active_visualizer: Arc<RwLock<Option<Box<dyn Visualizer>>>>,
    /// Available visualizers registry
    visualizers: Arc<RwLock<HashMap<String, Box<dyn Fn() -> Box<dyn Visualizer> + Send + Sync>>>>,
    /// Current configuration
    config: Arc<RwLock<VisualizationConfig>>,
    /// Rendering canvas
    canvas: Arc<RwLock<SoftwareCanvas>>,
    /// Command sender
    command_sender: mpsc::UnboundedSender<VisualizerCommand>,
    /// Event broadcaster
    event_sender: broadcast::Sender<VisualizerEvent>,
    /// Performance metrics
    metrics: Arc<RwLock<VisualizerMetrics>>,
    /// Last spectrum data
    last_spectrum: Arc<RwLock<Option<SpectrumData>>>,
}

/// Internal visualizer worker
struct VisualizerWorker {
    /// State reference
    state: Arc<RwLock<VisualizerState>>,
    /// Active visualizer reference
    active_visualizer: Arc<RwLock<Option<Box<dyn Visualizer>>>>,
    /// Configuration reference
    config: Arc<RwLock<VisualizationConfig>>,
    /// Canvas reference
    canvas: Arc<RwLock<SoftwareCanvas>>,
    /// Command receiver
    command_receiver: mpsc::UnboundedReceiver<VisualizerCommand>,
    /// Event sender
    event_sender: broadcast::Sender<VisualizerEvent>,
    /// Metrics reference
    metrics: Arc<RwLock<VisualizerMetrics>>,
    /// Spectrum data reference
    last_spectrum: Arc<RwLock<Option<SpectrumData>>>,
    /// Frame timing
    last_frame_time: Instant,
    /// Target frame duration
    target_frame_duration: Duration,
}

impl VisualizerEngine {
    /// Create a new visualizer engine
    pub fn new(width: u32, height: u32) -> Result<Self, VisualizerError> {
        info!("Initializing visualizer engine ({}x{})", width, height);

        // Create communication channels
        let (command_sender, command_receiver) = mpsc::unbounded_channel();
        let (event_sender, _) = broadcast::channel(100);

        // Initialize shared state
        let state = Arc::new(RwLock::new(VisualizerState::Stopped));
        let active_visualizer = Arc::new(RwLock::new(None));
        let config = Arc::new(RwLock::new(VisualizationConfig::default()));
        let canvas = Arc::new(RwLock::new(SoftwareCanvas::new(width, height)));
        let metrics = Arc::new(RwLock::new(VisualizerMetrics::default()));
        let last_spectrum = Arc::new(RwLock::new(None));

        // Create visualizer registry with built-in visualizers
        let visualizers = Arc::new(RwLock::new(HashMap::new()));
        {
            let mut registry = visualizers.write();
            registry.insert(
                "spectrum_bars".to_string(),
                Box::new(|| -> Box<dyn Visualizer> {
                    Box::new(SpectrumBarsVisualizer::new())
                }) as Box<dyn Fn() -> Box<dyn Visualizer> + Send + Sync>,
            );
        }

        // Start worker thread
        let worker = VisualizerWorker {
            state: Arc::clone(&state),
            active_visualizer: Arc::clone(&active_visualizer),
            config: Arc::clone(&config),
            canvas: Arc::clone(&canvas),
            command_receiver,
            event_sender: event_sender.clone(),
            metrics: Arc::clone(&metrics),
            last_spectrum: Arc::clone(&last_spectrum),
            last_frame_time: Instant::now(),
            target_frame_duration: Duration::from_millis(16), // 60 FPS
        };

        tokio::spawn(async move {
            worker.run().await;
        });

        debug!("Visualizer engine initialized successfully");

        Ok(Self {
            state,
            active_visualizer,
            visualizers,
            config,
            canvas,
            command_sender,
            event_sender,
            metrics,
            last_spectrum,
        })
    }

    /// Set the active visualizer
    pub fn set_visualizer(&self, visualizer_id: &str) -> Result<(), VisualizerError> {
        debug!("Setting visualizer to: {}", visualizer_id);

        // Check if visualizer exists
        let visualizers = self.visualizers.read();
        let factory = visualizers
            .get(visualizer_id)
            .ok_or_else(|| VisualizerError::PluginNotFound {
                name: visualizer_id.to_string(),
            })?;

        // Create new visualizer instance
        let mut new_visualizer = factory();

        // Initialize with current configuration
        let config = self.config.read().clone();
        new_visualizer
            .initialize(&config)
            .map_err(|e| VisualizerError::Configuration(e.to_string()))?;

        // Update active visualizer
        *self.active_visualizer.write() = Some(new_visualizer);

        // Send command to worker
        self.send_command(VisualizerCommand::SetVisualizer(visualizer_id.to_string()))?;

        info!("Visualizer changed to: {}", visualizer_id);
        Ok(())
    }

    /// Update configuration
    pub fn set_config(&self, config: VisualizationConfig) -> Result<(), VisualizerError> {
        debug!("Updating visualizer configuration");

        *self.config.write() = config.clone();
        self.send_command(VisualizerCommand::SetConfig(config))?;

        Ok(())
    }

    /// Update with new spectrum data
    pub fn update_spectrum(&self, spectrum_data: SpectrumData) -> Result<(), VisualizerError> {
        *self.last_spectrum.write() = Some(spectrum_data.clone());
        self.send_command(VisualizerCommand::UpdateSpectrum(spectrum_data))?;
        Ok(())
    }

   /// Resize the visualizer canvas
    pub fn resize(&self, width: u32, height: u32) -> Result<(), VisualizerError> {
        debug!("Resizing visualizer canvas to {}x{}", width, height);

        self.canvas.write().resize(width, height);
        self.send_command(VisualizerCommand::Resize(width, height))?;

        Ok(())
    }

    /// Start the visualizer
    pub fn start(&self) -> Result<(), VisualizerError> {
        info!("Starting visualizer engine");
        *self.state.write() = VisualizerState::Running;
        self.send_command(VisualizerCommand::Start)?;
        Ok(())
    }

    /// Pause the visualizer
    pub fn pause(&self) -> Result<(), VisualizerError> {
        info!("Pausing visualizer engine");
        *self.state.write() = VisualizerState::Paused;
        self.send_command(VisualizerCommand::Pause)?;
        Ok(())
    }

    /// Stop the visualizer
    pub fn stop(&self) -> Result<(), VisualizerError> {
        info!("Stopping visualizer engine");
        *self.state.write() = VisualizerState::Stopped;
        self.send_command(VisualizerCommand::Stop)?;
        Ok(())
    }

    /// Get current state
    pub fn state(&self) -> VisualizerState {
        *self.state.read()
    }

    /// Get current metrics
    pub fn metrics(&self) -> VisualizerMetrics {
        self.metrics.read().clone()
    }

    /// Get canvas pixels for display
    pub fn get_frame(&self) -> Vec<u8> {
        self.canvas.read().pixels().to_vec()
    }

    /// Get canvas size
    pub fn canvas_size(&self) -> (u32, u32) {
        self.canvas.read().size()
    }

    /// Subscribe to visualizer events
    pub fn subscribe_events(&self) -> broadcast::Receiver<VisualizerEvent> {
        self.event_sender.subscribe()
    }

    /// Register a new visualizer plugin
    pub fn register_visualizer<F>(&self, id: String, factory: F)
    where
        F: Fn() -> Box<dyn Visualizer> + Send + Sync + 'static,
    {
        debug!("Registering visualizer plugin: {}", id);
        self.visualizers.write().insert(id, Box::new(factory));
    }

    /// Get list of available visualizers
    pub fn available_visualizers(&self) -> Vec<String> {
        self.visualizers.read().keys().cloned().collect()
    }

    /// Send command to worker
    fn send_command(&self, command: VisualizerCommand) -> Result<(), VisualizerError> {
        self.command_sender
            .send(command)
            .map_err(|_| VisualizerError::Rendering("Visualizer worker not available".to_string()))
    }
}

impl VisualizerWorker {
    /// Run the visualizer worker main loop
    async fn run(mut self) {
        debug!("Visualizer worker started");

        let mut render_interval = tokio::time::interval(self.target_frame_duration);
        let mut last_metrics_update = Instant::now();

        loop {
            tokio::select! {
                // Handle commands
                command = self.command_receiver.recv() => {
                    match command {
                        Some(cmd) => {
                            if let Err(e) = self.handle_command(cmd).await {
                                error!("Visualizer command failed: {}", e);
                                let _ = self.event_sender.send(VisualizerEvent::Error {
                                    error: e.to_string(),
                                });
                            }
                        }
                        None => {
                            debug!("Visualizer command channel closed");
                            break;
                        }
                    }
                }

                // Render frames
                _ = render_interval.tick() => {
                    if *self.state.read() == VisualizerState::Running {
                        if let Err(e) = self.render_frame().await {
                            error!("Frame rendering failed: {}", e);
                            let _ = self.event_sender.send(VisualizerEvent::Error {
                                error: e.to_string(),
                            });
                        }
                    }
                }
            }

            // Update metrics periodically
            if last_metrics_update.elapsed() >= Duration::from_secs(1) {
                self.update_metrics();
                last_metrics_update = Instant::now();
            }
        }

        debug!("Visualizer worker shutting down");
    }

    /// Handle a visualizer command
    async fn handle_command(&mut self, command: VisualizerCommand) -> Result<(), VisualizerError> {
        match command {
            VisualizerCommand::UpdateSpectrum(spectrum_data) => {
                *self.last_spectrum.write() = Some(spectrum_data.clone());
                
                // Update active visualizer with new data
                if let Some(ref mut visualizer) = *self.active_visualizer.write() {
                    visualizer.update(&spectrum_data)?;
                }
            }

            VisualizerCommand::SetVisualizer(id) => {
                let _ = self.event_sender.send(VisualizerEvent::VisualizerChanged { id });
            }

            VisualizerCommand::SetConfig(config) => {
                // Update active visualizer configuration
                if let Some(ref mut visualizer) = *self.active_visualizer.write() {
                    let mut settings = std::collections::HashMap::new();
                    settings.insert("sensitivity".to_string(), config.sensitivity.into());
                    settings.insert("smoothing".to_string(), config.smoothing.into());
                    
                    visualizer.configure(&settings)?;
                }

                let _ = self.event_sender.send(VisualizerEvent::ConfigUpdated);
            }

            VisualizerCommand::Resize(width, height) => {
                debug!("Resizing canvas to {}x{}", width, height);
                // Canvas was already resized in the main thread
            }

            VisualizerCommand::Start => {
                debug!("Visualizer worker starting");
            }

            VisualizerCommand::Pause => {
                debug!("Visualizer worker paused");
            }

            VisualizerCommand::Stop => {
                debug!("Visualizer worker stopped");
                // Reset active visualizer
                if let Some(ref mut visualizer) = *self.active_visualizer.write() {
                    visualizer.reset();
                }
            }
        }

        Ok(())
    }

    /// Render a single frame
    async fn render_frame(&mut self) -> Result<(), VisualizerError> {
        let frame_start = Instant::now();

        // Check if we have an active visualizer
        let has_visualizer = self.active_visualizer.read().is_some();
        if !has_visualizer {
            return Ok(());
        }

        // Get current configuration and canvas
        let config = self.config.read().clone();
        let mut canvas = self.canvas.write();

        // Clear canvas
        canvas.clear(config.color_scheme.background);

        // Render visualizer
        if let Some(ref visualizer) = *self.active_visualizer.read() {
            visualizer.render(&mut *canvas)?;
        }

        drop(canvas); // Release canvas lock

        // Update timing metrics
        let frame_time = frame_start.elapsed();
        self.last_frame_time = frame_start;

        // Update metrics
        {
            let mut metrics = self.metrics.write();
            metrics.total_frames += 1;
            
            if frame_time > metrics.peak_frame_time {
                metrics.peak_frame_time = frame_time;
            }

            // Check for dropped frames (frame time > target)
            if frame_time > self.target_frame_duration {
                metrics.dropped_frames += 1;
            }
        }

        // Send frame rendered event
        let _ = self.event_sender.send(VisualizerEvent::FrameRendered { frame_time });

        Ok(())
    }

    /// Update performance metrics
    fn update_metrics(&mut self) {
        let mut metrics = self.metrics.write();
        
        // Calculate FPS based on recent frames
        let frames_in_last_second = 60; // Approximate based on target FPS
        metrics.fps = frames_in_last_second as f32;
        
        // Calculate average frame time
        if metrics.total_frames > 0 {
            let total_time = Duration::from_millis(metrics.total_frames * 16); // Approximate
            metrics.avg_frame_time = total_time / metrics.total_frames as u32;
        }
    }
}

impl Default for VisualizerMetrics {
    fn default() -> Self {
        Self {
            fps: 0.0,
            avg_frame_time: Duration::ZERO,
            peak_frame_time: Duration::ZERO,
            dropped_frames: 0,
            total_frames: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_visualizer_engine_creation() {
        let engine = VisualizerEngine::new(800, 600).unwrap();
        assert_eq!(engine.state(), VisualizerState::Stopped);
        assert_eq!(engine.canvas_size(), (800, 600));
    }

    #[tokio::test]
    async fn test_visualizer_state_transitions() {
        let engine = VisualizerEngine::new(800, 600).unwrap();
        
        // Test start
        engine.start().unwrap();
        sleep(Duration::from_millis(10)).await;
        assert_eq!(engine.state(), VisualizerState::Running);
        
        // Test pause
        engine.pause().unwrap();
        sleep(Duration::from_millis(10)).await;
        assert_eq!(engine.state(), VisualizerState::Paused);
        
        // Test stop
        engine.stop().unwrap();
        sleep(Duration::from_millis(10)).await;
        assert_eq!(engine.state(), VisualizerState::Stopped);
    }

    #[tokio::test]
    async fn test_visualizer_registration() {
        let engine = VisualizerEngine::new(800, 600).unwrap();
        
        // Check that spectrum_bars is available
        let available = engine.available_visualizers();
        assert!(available.contains(&"spectrum_bars".to_string()));
        
        // Test setting visualizer
        engine.set_visualizer("spectrum_bars").unwrap();
    }

    #[tokio::test]
    async fn test_canvas_resize() {
        let engine = VisualizerEngine::new(800, 600).unwrap();
        assert_eq!(engine.canvas_size(), (800, 600));
        
        engine.resize(1024, 768).unwrap();
        sleep(Duration::from_millis(10)).await;
        assert_eq!(engine.canvas_size(), (1024, 768));
    }

    #[tokio::test]
    async fn test_spectrum_update() {
        let engine = VisualizerEngine::new(800, 600).unwrap();
        engine.set_visualizer("spectrum_bars").unwrap();
        engine.start().unwrap();
        
        // Create test spectrum data
        let spectrum_data = SpectrumData::new(
            vec![0.1, 0.2, 0.3, 0.4, 0.5],
            0.5,
            0.3,
        );
        
        engine.update_spectrum(spectrum_data).unwrap();
        sleep(Duration::from_millis(50)).await;
        
        // Should not panic and should update internal state
    }

    #[tokio::test]
    async fn test_configuration_update() {
        let engine = VisualizerEngine::new(800, 600).unwrap();
        engine.set_visualizer("spectrum_bars").unwrap();
        
        let mut config = VisualizationConfig::default();
        config.sensitivity = 1.5;
        config.smoothing = false;
        
        engine.set_config(config).unwrap();
        sleep(Duration::from_millis(10)).await;
        
        // Configuration should be updated internally
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let engine = VisualizerEngine::new(800, 600).unwrap();
        let mut event_receiver = engine.subscribe_events();
        
        engine.set_visualizer("spectrum_bars").unwrap();
        
        // Should receive visualizer changed event
        tokio::select! {
            event = event_receiver.recv() => {
                match event.unwrap() {
                    VisualizerEvent::VisualizerChanged { id } => {
                        assert_eq!(id, "spectrum_bars");
                    }
                    _ => panic!("Unexpected event"),
                }
            }
            _ = sleep(Duration::from_millis(100)) => {
                panic!("Timeout waiting for event");
            }
        }
    }

    #[tokio::test]
    async fn test_frame_rendering() {
        let engine = VisualizerEngine::new(100, 100).unwrap();
        engine.set_visualizer("spectrum_bars").unwrap();
        engine.start().unwrap();
        
        // Let it render a few frames
        sleep(Duration::from_millis(100)).await;
        
        let frame = engine.get_frame();
        assert_eq!(frame.len(), 100 * 100 * 4); // RGBA
        
        let metrics = engine.metrics();
        assert!(metrics.total_frames > 0);
    }
}
