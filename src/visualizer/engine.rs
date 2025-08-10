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