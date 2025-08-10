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
                Box::new(|| Box::new(SpectrumBarsVisualizer::new()))
                    as Box<dyn Fn() -> Box<dyn Visualizer> + Send + Sync>,
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
}
