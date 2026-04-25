//! Thread-safe audio player manager.
//!
//! Bridges Slint's single-threaded UI model with the audio [`Player`] that
//! runs on a dedicated tokio task. Commands are sent over an unbounded channel
//! and responses are returned via one-shot channels.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info, warn};

use crate::audio::analysis::SpectrumData;
use crate::audio::decoder::AudioFormatInfo;
use crate::audio::player::Player;
use crate::audio::traits::AudioFormat;
use crate::error::AudioError;

// ---------------------------------------------------------------------------
// Command / Status types
// ---------------------------------------------------------------------------

/// Commands sent to the player task.
#[derive(Debug)]
pub enum PlayerCommand {
    LoadAndPlay {
        path: PathBuf,
        response: oneshot::Sender<Result<(), AudioError>>,
    },
    Play {
        response: oneshot::Sender<Result<(), AudioError>>,
    },
    Pause {
        response: oneshot::Sender<Result<(), AudioError>>,
    },
    Stop {
        response: oneshot::Sender<Result<(), AudioError>>,
    },
    SetVolume {
        volume: f32,
        response: oneshot::Sender<Result<(), AudioError>>,
    },
    Seek {
        position: Duration,
        response: oneshot::Sender<Result<Duration, AudioError>>,
    },
    GetStatus {
        response: oneshot::Sender<PlayerStatus>,
    },
    GetSpectrum {
        response: oneshot::Sender<Option<SpectrumData>>,
    },
    Shutdown,
}

/// Snapshot of the audio player's current state.
#[derive(Debug, Clone)]
pub struct PlayerStatus {
    pub is_playing: bool,
    pub is_paused: bool,
    pub volume: f32,
    pub current_track: Option<PathBuf>,
    /// Current playback position (approximate, based on elapsed wall time).
    pub position: Duration,
    /// Total duration of the loaded track, if known.
    pub duration: Option<Duration>,
    /// Audio format information for the loaded track.
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

// ---------------------------------------------------------------------------
// PlayerManager
// ---------------------------------------------------------------------------

/// Thread-safe handle to the audio player task.
pub struct PlayerManager {
    command_tx: mpsc::UnboundedSender<PlayerCommand>,
    _player_thread: tokio::task::JoinHandle<()>,
}

impl PlayerManager {
    /// Spawn the audio player task and return a handle.
    pub fn new() -> Result<Self, AudioError> {
        info!("Initialising player manager");

        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let player_thread = tokio::spawn(Self::player_task(command_rx));

        Ok(Self {
            command_tx,
            _player_thread: player_thread,
        })
    }

    // ------------------------------------------------------------------
    // Public async API
    // ------------------------------------------------------------------

    pub async fn load_and_play(&self, path: PathBuf) -> Result<(), AudioError> {
        self.send_recv(|tx| PlayerCommand::LoadAndPlay { path, response: tx })
            .await
    }

    pub async fn play(&self) -> Result<(), AudioError> {
        self.send_recv(|tx| PlayerCommand::Play { response: tx })
            .await
    }

    pub async fn pause(&self) -> Result<(), AudioError> {
        self.send_recv(|tx| PlayerCommand::Pause { response: tx })
            .await
    }

    pub async fn stop(&self) -> Result<(), AudioError> {
        self.send_recv(|tx| PlayerCommand::Stop { response: tx })
            .await
    }

    pub async fn set_volume(&self, volume: f32) -> Result<(), AudioError> {
        self.send_recv(|tx| PlayerCommand::SetVolume {
            volume,
            response: tx,
        })
        .await
    }

    /// Seek to the given position. Returns the actual position seeked to.
    pub async fn seek(&self, position: Duration) -> Result<Duration, AudioError> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(PlayerCommand::Seek {
                position,
                response: tx,
            })
            .map_err(|_| self.dead_err("seek"))?;
        rx.await.map_err(|_| self.dead_err("seek_response"))?
    }

    /// Return the most recent spectrum analysis result, or `None` when no
    /// track is loaded. The data is read directly from the watch channel
    /// maintained by the `SpectrumTap` in the audio thread.
    pub async fn get_spectrum(&self) -> Option<SpectrumData> {
        let (tx, rx) = oneshot::channel();
        if self
            .command_tx
            .send(PlayerCommand::GetSpectrum { response: tx })
            .is_err()
        {
            return None;
        }
        rx.await.ok().flatten()
    }

    pub async fn get_status(&self) -> PlayerStatus {
        let (tx, rx) = oneshot::channel();
        if self
            .command_tx
            .send(PlayerCommand::GetStatus { response: tx })
            .is_err()
        {
            warn!("Player task has stopped; returning default status");
            return PlayerStatus::default();
        }
        rx.await.unwrap_or_default()
    }

    pub async fn shutdown(&self) {
        let _ = self.command_tx.send(PlayerCommand::Shutdown);
    }

    // ------------------------------------------------------------------
    // Helpers
    // ------------------------------------------------------------------

    fn dead_err(&self, op: &str) -> AudioError {
        AudioError::InvalidState {
            from: "player_manager".into(),
            to: op.into(),
        }
    }

    /// Send a command that has a `Result<T, AudioError>` response channel.
    async fn send_recv<F, T>(&self, make_cmd: F) -> Result<T, AudioError>
    where
        F: FnOnce(oneshot::Sender<Result<T, AudioError>>) -> PlayerCommand,
    {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(make_cmd(tx))
            .map_err(|_| self.dead_err("send"))?;
        rx.await.map_err(|_| self.dead_err("recv"))?
    }

    // ------------------------------------------------------------------
    // Player task (runs on a tokio thread)
    // ------------------------------------------------------------------

    async fn player_task(mut rx: mpsc::UnboundedReceiver<PlayerCommand>) {
        info!("Player task started");

        // All mutable state lives here — no locking required.
        let mut player: Option<Player> = None;
        let mut current_track: Option<PathBuf> = None;
        let mut current_volume = 0.8_f32;
        let mut current_format: Option<AudioFormatInfo> = None;

        // Position tracking: position = seek_base + elapsed since play_start.
        let mut seek_base = Duration::ZERO;
        let mut play_start: Option<Instant> = None;

        while let Some(cmd) = rx.recv().await {
            match cmd {
                PlayerCommand::LoadAndPlay { path, response } => {
                    debug!("LoadAndPlay: {}", path.display());

                    let result = Self::ensure_player(&mut player).and_then(|p| p.play_file(&path));

                    let result = match result {
                        Ok(info) => {
                            current_track = Some(path.clone());
                            current_format = Some(info);
                            seek_base = Duration::ZERO;
                            play_start = Some(Instant::now());
                            if let Some(p) = &mut player {
                                p.set_volume(current_volume);
                            }
                            Ok(())
                        }
                        Err(e) => {
                            error!("Failed to load '{}': {e}", path.display());
                            Err(e)
                        }
                    };

                    let _ = response.send(result);
                }

                PlayerCommand::Play { response } => {
                    let result = if let Some(p) = &mut player {
                        p.resume();
                        play_start = Some(Instant::now());
                        Ok(())
                    } else {
                        Err(AudioError::InvalidState {
                            from: "idle".into(),
                            to: "play".into(),
                        })
                    };
                    let _ = response.send(result);
                }

                PlayerCommand::Pause { response } => {
                    let result = if let Some(p) = &mut player {
                        // Snapshot position before pausing.
                        if let Some(start) = play_start.take() {
                            seek_base += start.elapsed();
                        }
                        p.pause();
                        Ok(())
                    } else {
                        Err(AudioError::InvalidState {
                            from: "idle".into(),
                            to: "pause".into(),
                        })
                    };
                    let _ = response.send(result);
                }

                PlayerCommand::Stop { response } => {
                    if let Some(p) = &mut player {
                        p.stop();
                    }
                    current_track = None;
                    current_format = None;
                    seek_base = Duration::ZERO;
                    play_start = None;
                    let _ = response.send(Ok(()));
                }

                PlayerCommand::SetVolume { volume, response } => {
                    current_volume = volume.clamp(0.0, 1.0);
                    if let Some(p) = &mut player {
                        p.set_volume(current_volume);
                    }
                    let _ = response.send(Ok(()));
                }

                PlayerCommand::Seek { position, response } => {
                    let result = if let Some(p) = &mut player {
                        match p.seek(position) {
                            Ok(actual) => {
                                seek_base = actual;
                                play_start = if p.is_paused() {
                                    None
                                } else {
                                    Some(Instant::now())
                                };
                                Ok(actual)
                            }
                            Err(e) => Err(e),
                        }
                    } else {
                        Err(AudioError::InvalidState {
                            from: "idle".into(),
                            to: "seek".into(),
                        })
                    };
                    let _ = response.send(result);
                }

                PlayerCommand::GetStatus { response } => {
                    let is_playing = player.as_ref().map(|p| p.is_playing()).unwrap_or(false);
                    let is_paused = player.as_ref().map(|p| p.is_paused()).unwrap_or(false);

                    // Compute approximate playback position from wall time.
                    let position = if is_playing {
                        if let Some(start) = play_start {
                            seek_base + start.elapsed()
                        } else {
                            seek_base
                        }
                    } else {
                        seek_base
                    };

                    // Clamp to known duration to avoid overshooting.
                    let duration = current_format.as_ref().and_then(|f| f.duration);
                    let position = duration.map_or(position, |d| position.min(d));

                    let format = current_format.as_ref().map(|f| AudioFormat {
                        sample_rate: f.sample_rate,
                        channels: f.channels,
                        bit_depth: f.bit_depth.unwrap_or(16) as u16,
                        format_type: f.format_type.clone(),
                    });

                    let _ = response.send(PlayerStatus {
                        is_playing,
                        is_paused,
                        volume: current_volume,
                        current_track: current_track.clone(),
                        position,
                        duration,
                        format,
                    });
                }

                PlayerCommand::GetSpectrum { response } => {
                    let data = player
                        .as_ref()
                        .and_then(|p| p.spectrum_rx())
                        .map(|rx| (*rx.borrow()).clone());
                    let _ = response.send(data);
                }

                PlayerCommand::Shutdown => {
                    info!("Player task shutting down");
                    if let Some(p) = &mut player {
                        p.stop();
                    }
                    break;
                }
            }
        }

        info!("Player task ended");
    }

    /// Initialise the player lazily — creates it on first use.
    fn ensure_player(player: &mut Option<Player>) -> Result<&mut Player, AudioError> {
        if player.is_none() {
            *player = Some(Player::new()?);
        }
        Ok(player.as_mut().expect("just initialised"))
    }
}

impl Drop for PlayerManager {
    fn drop(&mut self) {
        let _ = self.command_tx.send(PlayerCommand::Shutdown);
    }
}
