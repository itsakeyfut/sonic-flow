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
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
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

/// Current status of the audio engine
#[derive(Debug, Clone)]
pub struct AudioEngineStatus {
    /// Current playback state
    pub state: PlaybackState,
    /// Current position in the track
    pub position: Duration,
    /// Total track duration
    pub duration: Option<Duration>,
    /// Current volume (0.0 - 1.0)
    pub volume: f32,
    /// Whether audio is muted
    pub is_muted: bool,
    /// Currently loaded track
    pub current_track: Option<TrackId>,
}

/// Main audio engine implementation
/// 
/// The AudioEngine manages audio playback using rodio for cross-platform
/// audio support. It runs on a dedicated thread to avoid blocking the UI.
pub struct AudioEngine {
    /// Command sender for communicating with the audio thread
    command_sender: mpsc::UnboundedSender<AudioCommand>,
    /// Handle to the audio processing thread
    _audio_thread: tokio::task::JoinHandle<()>,
    /// Shared status information
    status: Arc<RwLock<AudioEngineStatus>>,
    /// Track information cache
    tracks: Arc<RwLock<HashMap<TrackId, TrackInfo>>>,
}

/// Internal audio engine worker that runs on a dedicated thread
struct AudioEngineWorker {
    /// Rodio output stream
    _stream: OutputStream,
    /// Rodio stream handle for creating sinks
    stream_handle: OutputStreamHandle,
    /// Current audio sink for playback
    sink: Option<Sink>,
    /// Track information cache
    tracks: Arc<RwLock<HashMap<TrackId, TrackInfo>>>,
    /// Current engine status
    status: Arc<RwLock<AudioEngineStatus>>,
    /// Command receiver
    command_receiver: mpsc::UnboundedReceiver<AudioCommand>,
}

impl AudioEngine {
    /// Create a new audio engine instance
    /// 
    /// # Errors
    /// 
    /// Returns `AudioError` if the audio system cannot be initialized.
    pub fn new() -> Result<Self, AudioError> {
        info!("Initializing audio engine");

        // Create communication channels
        let (command_sender, command_receiver) = mpsc::unbounded_channel();

        // Initialize shared state
        let status = Arc::new(RwLock::new(AudioEngineStatus {
            state: PlaybackState::Stopped,
            position: Duration::ZERO,
            duration: None,
            volume: 0.8,
            is_muted: false,
            current_track: None,
        }));

        let tracks = Arc::new(RwLock::new(HashMap::new()));

        // Spawn the audio processing thread
        let worker_status = Arc::clone(&status);
        let worker_tracks = Arc::clone(&tracks);

        let audio_thread = tokio::spawn(async move {
            let worker = AudioEngineWorker::new(worker_status, worker_tracks, command_receiver).await;

            match worker {
                Ok(mut worker) => {
                    worker.run().await;
                }
                Err(e) => {
                    error!("Failed to initialize audio worker: {}", e);
                }
            }
        });

        debug!("Audio engine initialized successfully");

        Ok(Self {
            command_sender,
            _audio_thread: audio_thread,
            status,
            tracks,
        })
    }

    /// Send a command to the audio engine worker
    async fn send_command(&self, command: AudioCommand) -> Result<(), AudioError> {
        self.command_sender
            .send(command)
            .map_err(|_| AudioError::Device("Audio engine not available".to_string()))
    }

    /// Send a command and wait for a response
    async fn send_command_with_response<T, F>(&self, command_factory: F) -> Result<T, AudioError>
    where
        F: FnOnce(oneshot::Sender<T>) -> AudioCommand,
    {
        let (response_sender, response_receiver) = oneshot::channel();
        let command = command_factory(response_sender);

        self.send_command(command).await?;

        response_receiver
            .await
            .map_err(|_| AudioError::Device("Failed to receive response from audio engine".to_string()))
    }
}

impl AudioEngineWorker {
    /// Create a new audio engine worker
    async fn new(
        status: Arc<RwLock<AudioEngineStatus>>,
        tracks: Arc<RwLock<HashMap<TrackId, TrackInfo>>>,
        command_receiver: mpsc::UnboundedReceiver<AudioCommand>,
    ) -> Result<Self, AudioError> {
        // Initialize rodio output stream
        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| AudioError::Device(format!("Failed to initialize audio output: {}", e)))?;

        debug!("Audio output stream initialized");

        Ok(Self {
            _stream: stream,
            stream_handle,
            sink: None,
            tracks,
            status,
            command_receiver,
        })
    }

    /// Run the audio engine worker main loop
    async fn run(&mut self) {
        debug!("Audio engine worker started");

        while let Some(command) = self.command_receiver.recv().await {
            match self.handle_command(command).await {
                Ok(()) => {}
                Err(e) => {
                    error!("Audio engine command failed: {}", e);
                    // Update status to error state
                    self.status.write().state = PlaybackState::Error(e.to_string());
                }
            }
        }

        debug!("Audio engine worker shutting down");
    }

    /// Handle a single audio command
    async fn handle_command(&mut self, command: AudioCommand) -> Result<(), AudioError> {
        match command {
            AudioCommand::Play => self.handle_play().await,
            AudioCommand::Pause => self.handle_pause().await,
            AudioCommand::Stop => self.handle_stop().await,
            AudioCommand::Seek(position) => self.handle_seek(position).await,
            AudioCommand::SetVolume(volume) => self.handle_set_volume(volume).await,
            AudioCommand::SetMuted(muted) => self.handle_set_muted(muted).await,
            AudioCommand::LoadTrack(path, response) => {
                let result = self.handle_load_track(&path).await;
                let _ = response.send(result);
                Ok(())
            }
            AudioCommand::SetCurrentTrack(track_id, response) => {
                let result = self.handle_set_current_track(track_id).await;
                let _ = response.send(result);
                Ok(())
            }
            AudioCommand::GetStatus(response) => {
                let status = self.status.read().clone();
                let _ = response.send(status);
                Ok(())
            }
            AudioCommand::Shutdown => {
                if let Some(sink) = self.sink.take() {
                    sink.stop();
                }
                Ok(())
            }
        }
    }

    /// Handle play command
    async fn handle_play(&mut self) -> Result<(), AudioError> {
        if let Some(ref sink) = self.sink {
            sink.play();
            self.status.write().state = PlaybackState::Playing;
            debug!("Playback resumed");
        } else {
            // No sink available, try to create one for the current track
            let current_track = self.status.read().current_track;
            if let Some(track_id) = current_track {
                self.create_sink_for_track(track_id).await?;
                if let Some(ref sink) = self.sink {
                    sink.play();
                    self.status.write().state = PlaybackState::Playing;
                    debug!("Playback started");
                }
            } else {
                return Err(AudioError::InvalidState {
                    from: "no track loaded".to_string(),
                    to: "playing".to_string(),
                });
            }
        }
        Ok(())
    }

    /// Handle pause command
    async fn handle_pause(&mut self) -> Result<(), AudioError> {
        if let Some(ref sink) = self.sink {
            sink.pause();
            self.status.write().state = PlaybackState::Paused;
            debug!("Playback paused");
        }
        Ok(())
    }

    /// Handle stop command
    async fn handle_stop(&mut self) -> Result<(), AudioError> {
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }
        
        let mut status = self.status.write();
        status.state = PlaybackState::Stopped;
        status.position = Duration::ZERO;
        
        debug!("Playback stopped");
        Ok(())
    }

    /// Handle seek command
    async fn handle_seek(&mut self, position: Duration) -> Result<(), AudioError> {
        // Seeking in rodio requires recreating the sink
        // This is a limitation of the current implementation
        warn!("Seeking not yet implemented - requires sink recreation");
        
        // For now, just update the position in status
        // TODO: Implement proper seeking by recreating the sink at the target position
        self.status.write().position = position;
        
        Ok(())
    }

    /// Handle set volume command
    async fn handle_set_volume(&mut self, volume: f32) -> Result<(), AudioError> {
        let clamped_volume = volume.clamp(0.0, 1.0);
        
        if let Some(ref sink) = self.sink {
            sink.set_volume(clamped_volume);
        }
        
        self.status.write().volume = clamped_volume;
        debug!("Volume set to {}", clamped_volume);
        
        Ok(())
    }

    /// Handle set muted command
    async fn handle_set_muted(&mut self, muted: bool) -> Result<(), AudioError> {
        if let Some(ref sink) = self.sink {
            let volume = if muted { 0.0 } else { self.status.read().volume };
            sink.set_volume(volume);
        }
        
        self.status.write().is_muted = muted;
        debug!("Mute set to {}", muted);
        
        Ok(())
    }

    /// Handle load track command
    async fn handle_load_track(&mut self, path: &Path) -> Result<TrackId, AudioError> {
        debug!("Loading track: {}", path.display());

        // Validate file exists and is readable
        let metadata = tokio::fs::metadata(path)
            .await
            .map_err(|e| AudioError::Decoder(DecoderError::InitializationFailed {
                format: format!("File access error: {}", e),
            }))?;

        // Determine audio format from extension
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| AudioError::UnsupportedFormat {
                format: "No file extension".to_string(),
            })?;

        let format_type = AudioFormatType::from_extension(extension);
        if !format_type.is_supported() {
            return Err(AudioError::UnsupportedFormat {
                format: extension.to_string(),
            });
        }

        // Create track info
        let track_id = Uuid::new_v4();
        let track_info = TrackInfo {
            id: track_id,
            path: path.to_path_buf(),
            format: AudioFormat {
                sample_rate: 44100, // Default, will be updated when decoding
                channels: 2,        // Default, will be updated when decoding
                bit_depth: 16,      // Default, will be updated when decoding
                format_type,
            },
            duration: None, // Will be determined during decoding
            file_size: metadata.len(),
        };

        // Store track info
        self.tracks.write().insert(track_id, track_info);

        info!("Track loaded successfully: {} (ID: {})", path.display(), track_id);
        Ok(track_id)
    }
}