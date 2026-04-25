//! Thread-safe audio player manager
//!
//! This module provides a thread-safe wrapper around audio playback functionality
//! that works with Slint's single-threaded UI model.

use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info, warn};

use crate::audio::traits::{AudioFormat, AudioFormatType};
use crate::error::AudioError;
use crate::audio::player::Player;

/// Commands for the audio player manager
#[derive(Debug)]
pub enum PlayerCommand {
    /// Load and play a file
    LoadAndPlay {
        path: PathBuf,
        response: oneshot::Sender<Result<(), AudioError>>,
    },
    /// Start playback
    Play {
        response: oneshot::Sender<Result<(), AudioError>>,
    },
    /// Pause playback
    Pause {
        response: oneshot::Sender<Result<(), AudioError>>,
    },
    /// Stop playback
    Stop {
        response: oneshot::Sender<Result<(), AudioError>>,
    },
    /// Set volume (0.0 - 1.0)
    SetVolume {
        volume: f32,
        response: oneshot::Sender<Result<(), AudioError>>,
    },
    /// Seek to position
    Seek {
        position: Duration,
        response: oneshot::Sender<Result<(), AudioError>>,
    },
    /// Get current player status
    GetStatus {
        response: oneshot::Sender<PlayerStatus>,
    },
    /// Shutdown the player manager
    Shutdown,
}

/// Current status of the audio player
#[derive(Debug, Clone)]
pub struct PlayerStatus {
    /// Whether audio is currently playing
    pub is_playing: bool,
    /// Whether audio is paused
    pub is_paused: bool,
    /// Current volume (0.0 - 1.0)
    pub volume: f32,
    /// Currently loaded track path
    pub current_track: Option<PathBuf>,
    /// Current playback position
    pub position: Duration,
    /// Total duration (if known)
    pub duration: Option<Duration>,
    /// Audio format information
    pub format: Option<AudioFormat>,
}

impl Default for PlayerStatus {
    fn default() -> Self {
        Self {
            is_playing: false,
            is_paused: false,
            volume: 0.8,
            current_track: None,
            position: Duration::ZERO,
            duration: None,
            format: None,
        }
    }
}

/// Thread-safe audio player manager
pub struct PlayerManager {
    /// Command sender for communication with the player thread
    command_tx: mpsc::UnboundedSender<PlayerCommand>,
    /// Join handle for the player thread
    _player_thread: tokio::task::JoinHandle<()>,
}

impl PlayerManager {
    /// Create a new player manager
    pub fn new() -> Result<Self, AudioError> {
        info!("Initializing player manager");

        let (command_tx, command_rx) = mpsc::unbounded_channel();

        // Spawn the player thread
        let player_thread = tokio::spawn(Self::player_thread_main(command_rx));

        Ok(Self {
            command_tx,
            _player_thread: player_thread,
        })
    }

    /// Load and play an audio file
    pub async fn load_and_play(&self, path: PathBuf) -> Result<(), AudioError> {
        let (response_tx, response_rx) = oneshot::channel();
        
        self.command_tx
            .send(PlayerCommand::LoadAndPlay { path, response: response_tx })
            .map_err(|_| AudioError::InvalidState {
                from: "player_manager".to_string(),
                to: "load_and_play".to_string(),
            })?;

        response_rx.await.map_err(|_| AudioError::InvalidState {
            from: "player_manager".to_string(),
            to: "load_and_play_response".to_string(),
        })?
    }

    /// Start playback
    pub async fn play(&self) -> Result<(), AudioError> {
        let (response_tx, response_rx) = oneshot::channel();
        
        self.command_tx
            .send(PlayerCommand::Play { response: response_tx })
            .map_err(|_| AudioError::InvalidState {
                from: "player_manager".to_string(),
                to: "play".to_string(),
            })?;

        response_rx.await.map_err(|_| AudioError::InvalidState {
            from: "player_manager".to_string(),
            to: "play_response".to_string(),
        })?
    }

    /// Pause playback
    pub async fn pause(&self) -> Result<(), AudioError> {
        let (response_tx, response_rx) = oneshot::channel();
        
        self.command_tx
            .send(PlayerCommand::Pause { response: response_tx })
            .map_err(|_| AudioError::InvalidState {
                from: "player_manager".to_string(),
                to: "pause".to_string(),
            })?;

        response_rx.await.map_err(|_| AudioError::InvalidState {
            from: "player_manager".to_string(),
            to: "pause_response".to_string(),
        })?
    }

    /// Stop playback
    pub async fn stop(&self) -> Result<(), AudioError> {
        let (response_tx, response_rx) = oneshot::channel();
        
        self.command_tx
            .send(PlayerCommand::Stop { response: response_tx })
            .map_err(|_| AudioError::InvalidState {
                from: "player_manager".to_string(),
                to: "stop".to_string(),
            })?;

        response_rx.await.map_err(|_| AudioError::InvalidState {
            from: "player_manager".to_string(),
            to: "stop_response".to_string(),
        })?
    }

    /// Set volume
    pub async fn set_volume(&self, volume: f32) -> Result<(), AudioError> {
        let (response_tx, response_rx) = oneshot::channel();
        
        self.command_tx
            .send(PlayerCommand::SetVolume { volume, response: response_tx })
            .map_err(|_| AudioError::InvalidState {
                from: "player_manager".to_string(),
                to: "set_volume".to_string(),
            })?;

        response_rx.await.map_err(|_| AudioError::InvalidState {
            from: "player_manager".to_string(),
            to: "set_volume_response".to_string(),
        })?
    }

    /// Get current player status
    pub async fn get_status(&self) -> PlayerStatus {
        let (response_tx, response_rx) = oneshot::channel();
        
        if self
            .command_tx
            .send(PlayerCommand::GetStatus { response: response_tx })
            .is_err()
        {
            warn!("Failed to send status request");
            return PlayerStatus::default();
        }

        response_rx.await.unwrap_or_default()
    }

    /// Shutdown the player manager
    pub async fn shutdown(&self) {
        if self.command_tx.send(PlayerCommand::Shutdown).is_err() {
            warn!("Failed to send shutdown command");
        }
    }

    /// Main loop for the player thread
    async fn player_thread_main(mut command_rx: mpsc::UnboundedReceiver<PlayerCommand>) {
        info!("Player thread started");
        
        let mut player: Option<Player> = None;
        let mut current_track: Option<PathBuf> = None;
        let mut current_volume = 0.8f32;

        while let Some(command) = command_rx.recv().await {
            match command {
                PlayerCommand::LoadAndPlay { path, response } => {
                    debug!("Loading and playing: {}", path.display());
                    
                    let result = match Self::load_file(&mut player, &path).await {
                        Ok(_) => {
                            current_track = Some(path.clone());
                            if let Some(p) = &mut player {
                                p.set_volume(current_volume);
                            }
                            Ok(())
                        }
                        Err(e) => {
                            error!("Failed to load file: {}", e);
                            Err(e)
                        }
                    };
                    
                    let _ = response.send(result);
                }

                PlayerCommand::Play { response } => {
                    debug!("Play command received");
                    
                    let result = if let Some(p) = &mut player {
                        p.resume();
                        Ok(())
                    } else {
                        Err(AudioError::InvalidState {
                            from: "no_player".to_string(),
                            to: "play".to_string(),
                        })
                    };
                    
                    let _ = response.send(result);
                }

                PlayerCommand::Pause { response } => {
                    debug!("Pause command received");
                    
                    let result = if let Some(p) = &mut player {
                        p.pause();
                        Ok(())
                    } else {
                        Err(AudioError::InvalidState {
                            from: "no_player".to_string(),
                            to: "pause".to_string(),
                        })
                    };
                    
                    let _ = response.send(result);
                }

                PlayerCommand::Stop { response } => {
                    debug!("Stop command received");
                    
                    let result = if let Some(p) = &mut player {
                        p.stop();
                        Ok(())
                    } else {
                        Err(AudioError::InvalidState {
                            from: "no_player".to_string(),
                            to: "stop".to_string(),
                        })
                    };
                    
                    let _ = response.send(result);
                }

                PlayerCommand::SetVolume { volume, response } => {
                    debug!("Set volume to: {:.2}", volume);
                    
                    current_volume = volume.clamp(0.0, 1.0);
                    
                    let result = if let Some(p) = &mut player {
                        p.set_volume(current_volume);
                        Ok(())
                    } else {
                        Ok(()) // Store volume for when player is created
                    };
                    
                    let _ = response.send(result);
                }

                PlayerCommand::Seek { position, response } => {
                    debug!("Seek to: {:?}", position);
                    
                    // TODO: Implement seeking when Player supports it
                    let result = Err(AudioError::UnsupportedFormat {
                        format: "seek_not_implemented".to_string(),
                    });
                    
                    let _ = response.send(result);
                }

                PlayerCommand::GetStatus { response } => {
                    let status = PlayerStatus {
                        is_playing: player.as_ref().map(|p| p.is_playing()).unwrap_or(false),
                        is_paused: player.as_ref().map(|p| !p.is_playing()).unwrap_or(true),
                        volume: current_volume,
                        current_track: current_track.clone(),
                        position: Duration::ZERO, // TODO: Get actual position
                        duration: None, // TODO: Get actual duration
                        format: None, // TODO: Get actual format
                    };
                    
                    let _ = response.send(status);
                }

                PlayerCommand::Shutdown => {
                    info!("Player thread shutting down");
                    if let Some(p) = &mut player {
                        p.stop();
                    }
                    break;
                }
            }
        }

        info!("Player thread ended");
    }

    /// Load a file into the player
    async fn load_file(player: &mut Option<Player>, path: &Path) -> Result<(), AudioError> {
        // Validate file exists and is supported
        if !path.exists() {
            return Err(AudioError::Streaming(format!(
                "File not found: {}",
                path.display()
            )));
        }

        // Check file extension
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| AudioError::UnsupportedFormat {
                format: "no_extension".to_string(),
            })?;

        let format_type = AudioFormatType::from_extension(extension);
        if !format_type.is_supported() {
            return Err(AudioError::UnsupportedFormat {
                format: extension.to_string(),
            });
        }

        // Initialize player if needed
        if player.is_none() {
            *player = Some(Player::new()?);
        }

        // Load and play the file
        if let Some(p) = player {
            p.play_file(path).await?;
        }

        Ok(())
    }
}

impl Drop for PlayerManager {
    fn drop(&mut self) {
        // Best effort to shutdown
        let _ = self.command_tx.send(PlayerCommand::Shutdown);
    }
}
