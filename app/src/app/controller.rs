use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use sonic_core::audio::player_manager::PlayerManager;
use sonic_core::audio::playlist::{Playlist, TrackInfo, scan_folder};
use sonic_core::{DEFAULT_BAND_COUNT, MetadataExtractor};

use super::command::Command;
use super::event::{Event, FormatInfo, TrackSummary};

/// Application controller — processes UI commands and sends status/spectrum
/// events back to the UI.
///
/// Owns the `Playlist` and the `PlayerManager`. Runs as a tokio task.
pub struct Controller {
    tx_event: mpsc::Sender<Event>,
    player: Arc<PlayerManager>,
    playlist: Playlist,
}

impl Controller {
    pub fn new(tx_event: mpsc::Sender<Event>) -> anyhow::Result<Self> {
        let player = Arc::new(PlayerManager::new()?);
        Ok(Self {
            tx_event,
            player,
            playlist: Playlist::new(),
        })
    }

    /// Main loop: process commands and send periodic status/spectrum updates.
    pub async fn run(mut self, mut rx_cmd: mpsc::Receiver<Command>) {
        info!("Controller started");

        let mut status_interval = tokio::time::interval(Duration::from_millis(100));
        let mut spectrum_interval = tokio::time::interval(Duration::from_millis(16));
        // Track whether we were playing on the last status tick — used to
        // detect natural track-end (sink drained) vs explicit stop/pause.
        let mut was_playing = false;

        loop {
            tokio::select! {
                cmd = rx_cmd.recv() => {
                    match cmd {
                        Some(cmd) => self.handle_command(cmd).await,
                        None => break,
                    }
                }
                _ = status_interval.tick() => {
                    let status = self.player.get_status().await;

                    // Natural track-end detection: was playing, now stopped
                    // (not paused), and the player still has a track loaded
                    // (i.e., not an explicit Stop that clears current_path).
                    let track_ended = was_playing
                        && !status.is_playing
                        && !status.is_paused
                        && status.current_track.is_some();

                    if track_ended {
                        info!("Track ended naturally — advancing playlist");
                        self.auto_advance().await;
                    }

                    was_playing = status.is_playing;
                    self.emit_status(&status).await;
                }
                _ = spectrum_interval.tick() => {
                    self.send_spectrum().await;
                }
            }
        }

        info!("Controller stopped");
    }

    // ------------------------------------------------------------------
    // Command dispatch
    // ------------------------------------------------------------------

    async fn handle_command(&mut self, cmd: Command) {
        debug!("Processing command: {:?}", cmd);

        match cmd {
            Command::LoadFile(path) => {
                self.load_and_emit(path).await;
            }

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
                }
                Err(e) => {
                    let _ = self.tx_event.send(Event::Error(e.to_string())).await;
                }
            },

            Command::SkipForward(seconds) => {
                let status = self.player.get_status().await;
                let target = {
                    let offset = Duration::from_secs_f64(seconds);
                    let t = status.position.saturating_add(offset);
                    status.duration.map_or(t, |d| t.min(d))
                };
                info!("Skip forward {}s → {:?}", seconds, target);
                if let Err(e) = self.player.seek(target).await {
                    let _ = self.tx_event.send(Event::Error(e.to_string())).await;
                }
            }

            Command::SkipBackward(seconds) => {
                let status = self.player.get_status().await;
                let target = status
                    .position
                    .saturating_sub(Duration::from_secs_f64(seconds));
                info!("Skip backward {}s → {:?}", seconds, target);
                if let Err(e) = self.player.seek(target).await {
                    let _ = self.tx_event.send(Event::Error(e.to_string())).await;
                }
            }

            // -- Playlist commands -----------------------------------------
            Command::AddTracks(paths) => {
                let infos = extract_track_infos(paths).await;
                self.playlist.add_tracks(infos);
                self.emit_playlist_updated().await;

                // Auto-start if nothing is playing and this is the first batch.
                if !self.player.get_status().await.is_playing
                    && self.playlist.current_index().is_none()
                {
                    self.playlist.select(0);
                    if let Some(path) = self.playlist.current().map(|t| t.path.clone()) {
                        self.load_and_emit(path).await;
                    }
                    self.emit_playlist_updated().await;
                }
            }

            Command::AddFolder(folder) => {
                let paths = tokio::task::spawn_blocking(move || scan_folder(&folder))
                    .await
                    .unwrap_or_default();
                if paths.is_empty() {
                    warn!("No audio files found in selected folder");
                    return;
                }
                let infos = extract_track_infos(paths).await;
                self.playlist.add_tracks(infos);
                self.emit_playlist_updated().await;

                // Auto-start if idle.
                if !self.player.get_status().await.is_playing
                    && self.playlist.current_index().is_none()
                {
                    self.playlist.select(0);
                    if let Some(path) = self.playlist.current().map(|t| t.path.clone()) {
                        self.load_and_emit(path).await;
                    }
                    self.emit_playlist_updated().await;
                }
            }

            Command::NextTrack => {
                let next = self.playlist.next().map(|t| t.path.clone());
                if let Some(path) = next {
                    self.load_and_emit(path).await;
                    self.emit_playlist_updated().await;
                }
            }

            Command::PreviousTrack => {
                let prev = self.playlist.previous().map(|t| t.path.clone());
                if let Some(path) = prev {
                    self.load_and_emit(path).await;
                    self.emit_playlist_updated().await;
                }
            }

            Command::SelectTrack(index) => {
                let selected = self.playlist.select(index).map(|t| t.path.clone());
                if let Some(path) = selected {
                    self.load_and_emit(path).await;
                    self.emit_playlist_updated().await;
                }
            }

            Command::RemoveTrack(index) => {
                self.playlist.remove(index);
                self.emit_playlist_updated().await;
            }

            Command::ClearPlaylist => {
                self.playlist.clear();
                let _ = self.player.stop().await;
                self.emit_playlist_updated().await;
            }

            Command::ToggleShuffle => {
                self.playlist.toggle_shuffle();
                self.emit_playlist_updated().await;
            }

            Command::CycleRepeat => {
                self.playlist.cycle_repeat();
                self.emit_playlist_updated().await;
            }
        }
    }

    // ------------------------------------------------------------------
    // Internal helpers
    // ------------------------------------------------------------------

    /// Called when the sink drains naturally — advance the playlist and load.
    async fn auto_advance(&mut self) {
        let next_path = self.playlist.next().map(|t| t.path.clone());
        match next_path {
            Some(path) => {
                self.load_and_emit(path).await;
                self.emit_playlist_updated().await;
            }
            None => {
                // End of playlist — stop and clear the player's current_track
                // so subsequent status polls show idle state correctly.
                if let Err(e) = self.player.stop().await {
                    error!("Error stopping after playlist end: {e}");
                }
            }
        }
    }

    /// Load a file, start playback, extract metadata, and emit events.
    async fn load_and_emit(&self, path: PathBuf) {
        match self.player.load_and_play(path.clone()).await {
            Ok(()) => {
                let meta_path = path.clone();
                let metadata = tokio::task::spawn_blocking(move || {
                    MetadataExtractor::extract_with_fallback(&meta_path)
                })
                .await
                .unwrap_or_default();

                let _ = self
                    .tx_event
                    .send(Event::TrackLoaded {
                        path,
                        metadata: Box::new(metadata),
                    })
                    .await;
            }
            Err(e) => {
                let error = e.to_string();
                error!("Failed to load track '{}': {error}", path.display());
                let _ = self
                    .tx_event
                    .send(Event::TrackLoadFailed { path, error })
                    .await;
            }
        }
    }

    async fn emit_playlist_updated(&self) {
        let tracks: Vec<TrackSummary> = self
            .playlist
            .tracks()
            .iter()
            .map(|t| TrackSummary {
                title: t.title.clone(),
                artist: t.artist.clone(),
                duration: t.duration,
            })
            .collect();

        let _ = self
            .tx_event
            .send(Event::PlaylistUpdated {
                current_index: self.playlist.current_index(),
                total_duration: self.playlist.total_duration(),
                tracks,
            })
            .await;
    }

    async fn send_spectrum(&self) {
        let stale_threshold = Duration::from_millis(200);

        match self.player.get_spectrum().await {
            Some(data) if data.is_recent(stale_threshold) => {
                let _ = self
                    .tx_event
                    .send(Event::SpectrumUpdated {
                        bands: data.bands,
                        peak: data.peak_level,
                    })
                    .await;
            }
            Some(data) => {
                let n = data.bands.len();
                let _ = self
                    .tx_event
                    .send(Event::SpectrumUpdated {
                        bands: vec![0.0; n],
                        peak: 0.0,
                    })
                    .await;
            }
            None => {
                let _ = self
                    .tx_event
                    .send(Event::SpectrumUpdated {
                        bands: vec![0.0; DEFAULT_BAND_COUNT],
                        peak: 0.0,
                    })
                    .await;
            }
        }
    }

    async fn emit_status(&self, status: &sonic_core::PlayerStatus) {
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
                track_path: status.current_track.clone(),
                format,
            })
            .await;
    }
}

// ------------------------------------------------------------------
// Free functions
// ------------------------------------------------------------------

/// Extract metadata for a batch of paths in a blocking task.
async fn extract_track_infos(paths: Vec<PathBuf>) -> Vec<TrackInfo> {
    tokio::task::spawn_blocking(move || {
        paths
            .into_iter()
            .map(|path| {
                let meta = MetadataExtractor::extract_with_fallback(&path);
                TrackInfo::from_path(path).with_metadata(meta.title, meta.artist, meta.duration)
            })
            .collect()
    })
    .await
    .unwrap_or_default()
}
