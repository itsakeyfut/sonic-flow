//! Core audio engine implementation
//! 
//! This module provides the main AudioEngine that handles audio playback,
//! track management, and audio processing coordination.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use parking_lot::RwLock;
use rodio::{Decoder, OutputStream, Sink};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::{AudioError, DecoderError};
use crate::{Result, TrackId};

use super::traits::{
    AudioFormat, AudioFormatType, PlaybackControl, PlaybackState, PlaybackStatus, TrackLoader,
    VolumeControl,
};

/// Track information stored in the engine
#[derive(Debug, Clone)]
pub struct TrackInfo {
    /// Unique track identifier
    pub id: TrackId,
    /// File path to the audio file
    pub path: PathBuf,
    /// Audio format information
    pub format: AudioFormat,
    /// Track duration (if available)
    pub duration: Option<Duration>,
    /// File size in bytes
    pub file_size: u64,
}

/// Audio engine commands for async communication
#[derive(Debug)]
enum AudioCommand {
    Play,
    Pause,
    Stop,
    Seek(Duration),
    SetVolume(f32),
    SetMuted(bool),
    LoadTrack(PathBuf, oneshot::Sender<Result<TrackId, AudioError>>),
    SetCurrentTrack(TrackId, oneshot::Sender<Result<(), AudioError>>),
    GetStatus(oneshot::Sender<AudioEngineStatus>),
    Shutdown,
}
