use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;
use tracing::{debug, error, info};

use sonic_core::audio::player_manager::PlayerManager;

use super::command::Command;
use super::event::{Event, FormatInfo};

/// Application controller that processes commands and sends events.
///
/// Runs as a tokio task. Owns the audio PlayerManager and bridges
/// UI commands to audio operations.
pub struct Controller {
    tx_event: mpsc::Sender<Event>,
    player: Arc<PlayerManager>,
}

impl Controller {
    pub fn new(tx_event: mpsc::Sender<Event>) -> anyhow::Result<Self> {
        let player = Arc::new(PlayerManager::new()?);
        Ok(Self { tx_event, player })
    }

    /// Main loop: process commands and send periodic status updates.
    pub async fn run(self, mut rx_cmd: mpsc::Receiver<Command>) {
        info!("Controller started");

        let mut status_interval = tokio::time::interval(Duration::from_millis(100));

        loop {
            tokio::select! {
                cmd = rx_cmd.recv() => {
                    match cmd {
                        Some(cmd) => self.handle_command(cmd).await,
                        None => break, // All senders dropped (UI closed)
                    }
                }
                _ = status_interval.tick() => {
                    self.send_status().await;
                }
            }
        }

        info!("Controller stopped");
    }

    async fn handle_command(&self, cmd: Command) {
        debug!("Processing command: {:?}", cmd);

        match cmd {
            Command::LoadFile(path) => match self.player.load_and_play(path.clone()).await {
                Ok(()) => {
                    let _ = self.tx_event.send(Event::TrackLoaded { path }).await;
                }
                Err(e) => {
                    let error = e.to_string();
                    error!("Failed to load track: {}", error);
                    let _ = self
                        .tx_event
                        .send(Event::TrackLoadFailed { path, error })
                        .await;
                }
            },
            Command::TogglePlayback => {
                let status = self.player.get_status().await;
                let result = if status.is_playing {
                    self.player.pause().await
                } else {
                    self.player.play().await
                };
                if let Err(e) = result {
                    let _ = self.tx_event.send(Event::Error(e.to_string())).await;
                }
            }
            Command::Stop => {
                if let Err(e) = self.player.stop().await {
                    let _ = self.tx_event.send(Event::Error(e.to_string())).await;
                }
            }
            Command::SetVolume(volume) => {
                if let Err(e) = self.player.set_volume(volume).await {
                    let _ = self.tx_event.send(Event::Error(e.to_string())).await;
                }
            }
            Command::Seek(position) => match self.player.seek(position).await {
                Ok(actual) => {
                    debug!("Seeked to {:?} (requested {:?})", actual, position);
                    self.send_status().await;
                }
                Err(e) => {
                    let _ = self.tx_event.send(Event::Error(e.to_string())).await;
                }
            },
            Command::SkipForward(seconds) => {
                let status = self.player.get_status().await;
                let offset = Duration::from_secs_f64(seconds);
                let target = status.position.saturating_add(offset);
                // Clamp to track duration when known.
                let target = match status.duration {
                    Some(d) => target.min(d),
                    None => target,
                };
                info!("Skip forward {}s → {:?}", seconds, target);
                match self.player.seek(target).await {
                    Ok(actual) => {
                        debug!("Skip forward seeked to {:?}", actual);
                        self.send_status().await;
                    }
                    Err(e) => {
                        let _ = self.tx_event.send(Event::Error(e.to_string())).await;
                    }
                }
            }
            Command::SkipBackward(seconds) => {
                let status = self.player.get_status().await;
                let offset = Duration::from_secs_f64(seconds);
                let target = status.position.saturating_sub(offset);
                info!("Skip backward {}s → {:?}", seconds, target);
                match self.player.seek(target).await {
                    Ok(actual) => {
                        debug!("Skip backward seeked to {:?}", actual);
                        self.send_status().await;
                    }
                    Err(e) => {
                        let _ = self.tx_event.send(Event::Error(e.to_string())).await;
                    }
                }
            }
        }
    }

    async fn send_status(&self) {
        let status = self.player.get_status().await;

        let format = status.format.as_ref().map(|f| FormatInfo {
            sample_rate: f.sample_rate,
            channels: f.channels,
            bit_depth: f.bit_depth,
        });

        let _ = self
            .tx_event
            .send(Event::PlaybackStatus {
                is_playing: status.is_playing,
                is_paused: status.is_paused,
                volume: status.volume,
                position: status.position,
                duration: status.duration,
                track_path: status.current_track,
                format,
            })
            .await;
    }
}
