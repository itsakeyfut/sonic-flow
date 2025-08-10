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
