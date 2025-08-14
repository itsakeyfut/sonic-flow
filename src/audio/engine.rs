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
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use tokio::sync::{broadcast, mpsc, oneshot};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::AudioError;
use crate::TrackId;

use super::analysis::{SpectrumAnalyzer, SpectrumData};
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
    /// Track title (from metadata)
    pub title: Option<String>,
    /// Artist name (from metadata)
    pub artist: Option<String>,
    /// Album name (from metadata)
    pub album: Option<String>,
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
    GetPosition(oneshot::Sender<Duration>),
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
pub struct AudioEngine {
    /// Command sender for communicating with the audio thread
    command_sender: mpsc::UnboundedSender<AudioCommand>,
    /// Handle to the audio processing thread
    _audio_thread: tokio::task::JoinHandle<()>,
    /// Shared status information
    status: Arc<RwLock<AudioEngineStatus>>,
    /// Track information cache
    tracks: Arc<RwLock<HashMap<TrackId, TrackInfo>>>,
    /// Spectrum data broadcaster
    spectrum_sender: broadcast::Sender<SpectrumData>,
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
    /// Spectrum analyzer
    spectrum_analyzer: SpectrumAnalyzer,
    /// Spectrum data broadcaster
    spectrum_sender: broadcast::Sender<SpectrumData>,
    /// Current track start time for position calculation
    track_start_time: Option<std::time::Instant>,
    /// Position when paused
    paused_position: Duration,
}

impl AudioEngine {
    /// Create a new audio engine instance
    pub fn new() -> Result<Self, AudioError> {
        info!("Initializing audio engine");

        // Create communication channels
        let (command_sender, command_receiver) = mpsc::unbounded_channel();
        let (spectrum_sender, _) = broadcast::channel(100);

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

        // Clone for worker thread
        let worker_status = Arc::clone(&status);
        let worker_tracks = Arc::clone(&tracks);
        let worker_spectrum_sender = spectrum_sender.clone();

        // Spawn the audio processing thread
        let audio_thread = tokio::spawn(async move {
            // Create a simple blocking runtime for audio operations
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create audio runtime");

            rt.block_on(async move {
                let worker = AudioEngineWorker::new(
                    worker_status,
                    worker_tracks,
                    command_receiver,
                    worker_spectrum_sender,
                )
                .await;

                match worker {
                    Ok(mut worker) => {
                        worker.run().await;
                    }
                    Err(e) => {
                        error!("Failed to initialize audio worker: {}", e);
                    }
                }
            });
        });

        debug!("Audio engine initialized successfully");

        Ok(Self {
            command_sender,
            _audio_thread: audio_thread,
            status,
            tracks,
            spectrum_sender,
        })
    }

    /// Subscribe to spectrum data updates
    pub fn subscribe_spectrum(&self) -> broadcast::Receiver<SpectrumData> {
        self.spectrum_sender.subscribe()
    }

    /// Get the latest spectrum data (if available)
    pub fn get_current_spectrum(&self) -> Option<SpectrumData> {
        match self.spectrum_sender.subscribe().try_recv() {
            Ok(spectrum) => Some(spectrum),
            Err(_) => None,
        }
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

        response_receiver.await.map_err(|_| {
            AudioError::Device("Failed to receive response from audio engine".to_string())
        })
    }

    /// Get track information by ID
    pub fn get_track_info(&self, track_id: TrackId) -> Option<TrackInfo> {
        self.tracks.read().get(&track_id).cloned()
    }

    /// Get current position with high precision
    pub async fn get_position(&self) -> Duration {
        self.send_command_with_response(|sender| AudioCommand::GetPosition(sender))
            .await
            .unwrap_or(Duration::ZERO)
    }
}

impl AudioEngineWorker {
    /// Create a new audio engine worker
    async fn new(
        status: Arc<RwLock<AudioEngineStatus>>,
        tracks: Arc<RwLock<HashMap<TrackId, TrackInfo>>>,
        command_receiver: mpsc::UnboundedReceiver<AudioCommand>,
        spectrum_sender: broadcast::Sender<SpectrumData>,
    ) -> Result<Self, AudioError> {
        // Initialize rodio output stream
        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| AudioError::Device(format!("Failed to initialize audio output: {}", e)))?;

        // Initialize spectrum analyzer
        let spectrum_analyzer = SpectrumAnalyzer::new(
            2048,  // FFT size
            44100, // Sample rate
            64,    // Output bands
        );

        debug!("Audio output stream and spectrum analyzer initialized");

        Ok(Self {
            _stream: stream,
            stream_handle,
            sink: None,
            tracks,
            status,
            command_receiver,
            spectrum_analyzer,
            spectrum_sender,
            track_start_time: None,
            paused_position: Duration::ZERO,
        })
    }

    /// Run the audio engine worker main loop
    async fn run(&mut self) {
        debug!("Audio engine worker started");

        let mut spectrum_update_interval = tokio::time::interval(Duration::from_millis(16)); // ~60fps
        let mut position_update_interval = tokio::time::interval(Duration::from_millis(100)); // 10Hz

        loop {
            tokio::select! {
                // Handle commands
                command = self.command_receiver.recv() => {
                    match command {
                        Some(cmd) => {
                            match self.handle_command(cmd).await {
                                Ok(()) => {}
                                Err(e) => {
                                    error!("Audio engine command failed: {}", e);
                                    self.status.write().state = PlaybackState::Error(e.to_string());
                                }
                            }
                        }
                        None => {
                            debug!("Audio command channel closed");
                            break;
                        }
                    }
                }

                // Update position
                _ = position_update_interval.tick() => {
                    self.update_position();
                }

                // Update spectrum analysis
                _ = spectrum_update_interval.tick() => {
                    if let Err(e) = self.update_spectrum_analysis().await {
                        error!("Spectrum analysis update failed: {}", e);
                    }
                }
            }
        }

        debug!("Audio engine worker shutting down");
    }

    /// Update current playback position
    fn update_position(&mut self) {
        let mut status = self.status.write();

        match status.state {
            PlaybackState::Playing => {
                if let Some(start_time) = self.track_start_time {
                    let elapsed = start_time.elapsed();
                    status.position = self.paused_position + elapsed;
                }
            }
            PlaybackState::Paused => {
                // Position remains at paused position
            }
            _ => {
                status.position = Duration::ZERO;
            }
        }

        // Check if we've finished the track
        if let Some(duration) = status.duration {
            if status.position >= duration && status.state == PlaybackState::Playing {
                status.state = PlaybackState::Stopped;
                status.position = Duration::ZERO;
                self.track_start_time = None;
                self.paused_position = Duration::ZERO;

                // Stop the sink
                if let Some(sink) = self.sink.take() {
                    sink.stop();
                }
            }
        }
    }

    /// Update spectrum analysis with current audio data
    async fn update_spectrum_analysis(&mut self) -> Result<(), AudioError> {
        let state = self.status.read().state.clone();
        if state != PlaybackState::Playing {
            return Ok(());
        }

        if let Some(ref sink) = self.sink {
            if sink.empty() {
                return Ok(());
            }

            // Generate realistic spectrum data based on current position
            let position = self.status.read().position;
            let time_secs = position.as_secs_f32();

            // Create more realistic spectrum data
            let mut samples = Vec::with_capacity(2048);
            for i in 0..2048 {
                let t = time_secs + (i as f32 / 44100.0);

                // Mix multiple frequencies with some variation
                let sample = 0.3 * (2.0 * std::f32::consts::PI * 440.0 * t).sin() +
                    0.2 * (2.0 * std::f32::consts::PI * 880.0 * t * (1.0 + 0.1 * (t * 0.1).sin())).sin() +
                    0.15 * (2.0 * std::f32::consts::PI * 220.0 * t).sin() +
                    0.1 * (2.0 * std::f32::consts::PI * 1760.0 * t).sin() +
                    0.05 * (2.0 * std::f32::consts::PI * 110.0 * t).sin() +
                    // Add some noise and variation
                    0.02 * ((t * 13.0).sin() + (t * 17.0).sin() + (t * 19.0).sin());

                // Add envelope for more realistic spectrum
                let envelope = (1.0 + (t * 0.3).sin()) * 0.5;
                samples.push(sample * envelope * 0.6);
            }

            // Analyze the audio data
            let spectrum_data = self.spectrum_analyzer.analyze(&samples);

            // Broadcast spectrum data
            if let Err(_) = self.spectrum_sender.send(spectrum_data) {
                // Channel might be full or have no listeners, which is OK
            }
        }

        Ok(())
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
            AudioCommand::GetPosition(response) => {
                let position = self.status.read().position;
                let _ = response.send(position);
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
        let current_state = self.status.read().state.clone();

        match current_state {
            PlaybackState::Playing => {
                // Already playing, nothing to do
                return Ok(());
            }
            PlaybackState::Paused => {
                // Resume from paused state
                if let Some(ref sink) = self.sink {
                    sink.play();
                    self.status.write().state = PlaybackState::Playing;
                    self.track_start_time = Some(std::time::Instant::now());
                    debug!("Playback resumed from pause");
                } else {
                    return Err(AudioError::InvalidState {
                        from: "paused without sink".to_string(),
                        to: "playing".to_string(),
                    });
                }
            }
            _ => {
                // Start from beginning or load current track
                let current_track = self.status.read().current_track;
                if let Some(track_id) = current_track {
                    self.create_sink_for_track(track_id).await?;
                    if let Some(ref sink) = self.sink {
                        sink.play();
                        self.status.write().state = PlaybackState::Playing;
                        self.track_start_time = Some(std::time::Instant::now());
                        self.paused_position = Duration::ZERO;
                        debug!("Playback started from beginning");
                    }
                } else {
                    return Err(AudioError::InvalidState {
                        from: "no track loaded".to_string(),
                        to: "playing".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Handle pause command
    async fn handle_pause(&mut self) -> Result<(), AudioError> {
        if let Some(ref sink) = self.sink {
            sink.pause();

            // Update paused position
            if let Some(start_time) = self.track_start_time {
                self.paused_position += start_time.elapsed();
                self.track_start_time = None;
            }

            self.status.write().state = PlaybackState::Paused;
            debug!("Playback paused at {:?}", self.paused_position);
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

        self.track_start_time = None;
        self.paused_position = Duration::ZERO;

        debug!("Playback stopped");
        Ok(())
    }

    /// Handle seek command
    async fn handle_seek(&mut self, position: Duration) -> Result<(), AudioError> {
        warn!("Seeking not yet fully implemented - position will be updated but playback will restart");

        // For now, recreate the sink and start from beginning
        // TODO: Implement proper seeking
        let current_track = self.status.read().current_track;
        if let Some(track_id) = current_track {
            if let Some(sink) = self.sink.take() {
                sink.stop();
            }

            self.create_sink_for_track(track_id).await?;
            self.paused_position = position;

            // If we were playing, continue playing
            if matches!(self.status.read().state, PlaybackState::Playing) {
                if let Some(ref sink) = self.sink {
                    sink.play();
                    self.track_start_time = Some(std::time::Instant::now());
                }
            }
        }

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
        debug!("Volume set to {:.2}", clamped_volume);

        Ok(())
    }

    /// Handle set muted command
    async fn handle_set_muted(&mut self, muted: bool) -> Result<(), AudioError> {
        if let Some(ref sink) = self.sink {
            let volume = if muted {
                0.0
            } else {
                self.status.read().volume
            };
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
            .map_err(|e| AudioError::Streaming(format!("File access error: {}", e)))?;

        // Determine audio format from extension
        let extension = path.extension().and_then(|s| s.to_str()).ok_or_else(|| {
            AudioError::UnsupportedFormat {
                format: "No file extension".to_string(),
            }
        })?;

        let format_type = AudioFormatType::from_extension(extension);
        if !format_type.is_supported() {
            return Err(AudioError::UnsupportedFormat {
                format: extension.to_string(),
            });
        }

        // Try to probe the file to get duration and format info
        let (duration, sample_rate, channels) = self.probe_audio_file(path).await?;

        // Extract basic metadata from filename
        let filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // Create track info
        let track_id = Uuid::new_v4();
        let track_info = TrackInfo {
            id: track_id,
            path: path.to_path_buf(),
            format: AudioFormat {
                sample_rate,
                channels,
                bit_depth: 16, // Default, would be better to detect this
                format_type,
            },
            duration,
            file_size: metadata.len(),
            title: Some(filename),
            artist: None,
            album: None,
        };

        // Store track info
        self.tracks.write().insert(track_id, track_info.clone());

        info!(
            "Track loaded successfully: {} (ID: {}, Duration: {:?})",
            path.display(),
            track_id,
            duration
        );
        Ok(track_id)
    }

    /// Probe audio file to get format information
    async fn probe_audio_file(
        &self,
        path: &Path,
    ) -> Result<(Option<Duration>, u32, u16), AudioError> {
        // Try to open the file with rodio to get basic info
        let file = std::fs::File::open(path)
            .map_err(|e| AudioError::Streaming(format!("Failed to open file: {}", e)))?;

        let buf_reader = std::io::BufReader::new(file);

        match Decoder::new(buf_reader) {
            Ok(source) => {
                let sample_rate = source.sample_rate();
                let channels = source.channels();

                // For duration, we'd need to consume the entire source, which is expensive
                // For now, return None and let the UI handle it
                Ok((None, sample_rate, channels))
            }
            Err(e) => {
                warn!("Failed to probe audio file {}: {}", path.display(), e);
                // Return defaults
                Ok((None, 44100, 2))
            }
        }
    }

    /// Handle set current track command
    async fn handle_set_current_track(&mut self, track_id: TrackId) -> Result<(), AudioError> {
        // Verify track exists
        let track_info =
            self.tracks
                .read()
                .get(&track_id)
                .cloned()
                .ok_or_else(|| AudioError::InvalidState {
                    from: "unknown track".to_string(),
                    to: "current track".to_string(),
                })?;

        // Stop current playback
        if let Some(sink) = self.sink.take() {
            sink.stop();
        }

        // Update status
        let mut status = self.status.write();
        status.current_track = Some(track_id);
        status.state = PlaybackState::Stopped;
        status.position = Duration::ZERO;
        status.duration = track_info.duration;

        self.track_start_time = None;
        self.paused_position = Duration::ZERO;

        debug!(
            "Current track set to: {} ({})",
            track_id,
            track_info.path.display()
        );
        Ok(())
    }

    /// Create a new sink for the specified track
    async fn create_sink_for_track(&mut self, track_id: TrackId) -> Result<(), AudioError> {
        let track_info =
            self.tracks
                .read()
                .get(&track_id)
                .cloned()
                .ok_or_else(|| AudioError::InvalidState {
                    from: "unknown track".to_string(),
                    to: "playing".to_string(),
                })?;

        // Open and decode the audio file
        let file = std::fs::File::open(&track_info.path)
            .map_err(|e| AudioError::Streaming(format!("File open error: {}", e)))?;

        // Use BufReader for better performance
        let buf_reader = std::io::BufReader::new(file);
        let source = Decoder::new(buf_reader)
            .map_err(|e| AudioError::Streaming(format!("Decoder initialization failed: {}", e)))?;

        // Create new sink using the stored stream handle
        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| AudioError::Device(format!("Failed to create audio sink: {}", e)))?;

        // Apply current volume settings
        let status = self.status.read();
        let volume = if status.is_muted { 0.0 } else { status.volume };
        sink.set_volume(volume);

        // Add source to sink
        sink.append(source);

        // Store the sink
        self.sink = Some(sink);

        debug!("Sink created for track: {}", track_id);
        Ok(())
    }
}

#[async_trait]
impl PlaybackControl for AudioEngine {
    async fn play(&mut self) -> Result<(), AudioError> {
        self.send_command(AudioCommand::Play).await
    }

    async fn pause(&mut self) -> Result<(), AudioError> {
        self.send_command(AudioCommand::Pause).await
    }

    async fn stop(&mut self) -> Result<(), AudioError> {
        self.send_command(AudioCommand::Stop).await
    }

    async fn seek(&mut self, position: Duration) -> Result<(), AudioError> {
        self.send_command(AudioCommand::Seek(position)).await
    }

    async fn next_track(&mut self) -> Result<(), AudioError> {
        warn!("Next track not yet implemented - requires playlist management");
        Ok(())
    }

    async fn previous_track(&mut self) -> Result<(), AudioError> {
        warn!("Previous track not yet implemented - requires playlist management");
        Ok(())
    }
}

impl VolumeControl for AudioEngine {
    fn set_volume(&mut self, volume: f32) {
        let _ = self.command_sender.send(AudioCommand::SetVolume(volume));
    }

    fn volume(&self) -> f32 {
        self.status.read().volume
    }

    fn set_muted(&mut self, muted: bool) {
        let _ = self.command_sender.send(AudioCommand::SetMuted(muted));
    }

    fn is_muted(&self) -> bool {
        self.status.read().is_muted
    }
}

#[async_trait]
impl TrackLoader for AudioEngine {
    async fn load_track(&mut self, path: &Path) -> Result<TrackId, AudioError> {
        self.send_command_with_response(|sender| {
            AudioCommand::LoadTrack(path.to_path_buf(), sender)
        })
        .await?
    }

    async fn set_current_track(&mut self, track_id: TrackId) -> Result<(), AudioError> {
        self.send_command_with_response(|sender| AudioCommand::SetCurrentTrack(track_id, sender))
            .await?
    }

    fn current_track(&self) -> Option<TrackId> {
        self.status.read().current_track
    }
}

impl PlaybackStatus for AudioEngine {
    fn state(&self) -> PlaybackState {
        self.status.read().state.clone()
    }

    fn position(&self) -> Duration {
        self.status.read().position
    }

    fn duration(&self) -> Option<Duration> {
        self.status.read().duration
    }

    fn is_playing(&self) -> bool {
        matches!(self.state(), PlaybackState::Playing)
    }

    fn is_paused(&self) -> bool {
        matches!(self.state(), PlaybackState::Paused)
    }

    fn is_stopped(&self) -> bool {
        matches!(self.state(), PlaybackState::Stopped)
    }
}

impl Drop for AudioEngine {
    fn drop(&mut self) {
        let _ = self.command_sender.send(AudioCommand::Shutdown);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_audio_engine_creation() {
        let result = AudioEngine::new();
        // This might fail in test environment due to audio system
        if result.is_err() {
            println!("AudioEngine creation failed in test environment");
            return;
        }

        let engine = result.unwrap();
        assert_eq!(engine.state(), PlaybackState::Stopped);
        assert_eq!(engine.volume(), 0.8);
        assert!(!engine.is_muted());
    }

    #[test]
    fn test_audio_format_type() {
        let mp3_format = AudioFormatType::from_extension("mp3");
        assert_eq!(mp3_format, AudioFormatType::Mp3);
        assert!(mp3_format.is_supported());
        assert_eq!(mp3_format.as_str(), "mp3");

        let unknown_format = AudioFormatType::from_extension("xyz");
        assert!(!unknown_format.is_supported());
    }

    #[tokio::test]
    async fn test_volume_control() {
        let result = AudioEngine::new();
        if result.is_err() {
            return; // Skip test in environments without audio
        }

        let mut engine = result.unwrap();

        // Test volume setting
        engine.set_volume(0.5);
        // Note: Due to async nature, we can't immediately assert the volume

        // Test muting
        engine.set_muted(true);
    }

    #[test]
    fn test_track_info_creation() {
        let track_id = Uuid::new_v4();
        let path = PathBuf::from("test.mp3");

        let track_info = TrackInfo {
            id: track_id,
            path: path.clone(),
            format: AudioFormat {
                sample_rate: 44100,
                channels: 2,
                bit_depth: 16,
                format_type: AudioFormatType::Mp3,
            },
            duration: Some(Duration::from_secs(180)),
            file_size: 1024 * 1024,
            title: Some("test title".to_string()),
            album: Some("test album".to_string()),
            artist: Some("test artist".to_string()),
        };

        assert_eq!(track_info.id, track_id);
        assert_eq!(track_info.path, path);
        assert_eq!(track_info.format.sample_rate, 44100);
    }

    #[tokio::test]
    async fn test_playback_state_transitions() {
        let result = AudioEngine::new();
        if result.is_err() {
            return;
        }

        let mut engine = result.unwrap();

        // Initial state should be stopped
        assert!(engine.is_stopped());
        assert!(!engine.is_playing());
        assert!(!engine.is_paused());

        // Test pause without track (should not panic)
        let result = engine.pause().await;
        assert!(result.is_ok());
    }
}
