use std::time::Duration;

use slint::ComponentHandle;
use tokio::sync::mpsc;
use tracing::{debug, error, info};

use crate::app::{Command, Event};
use crate::{AppWindow, UiState};

/// Main UI wrapper. Creates the Slint window, registers callbacks,
/// and spawns the event handler task.
pub struct Ui {
    window: AppWindow,
}

impl Ui {
    pub fn new(
        tx_cmd: mpsc::Sender<Command>,
        rx_event: mpsc::Receiver<Event>,
    ) -> anyhow::Result<Self> {
        let window = AppWindow::new()?;
        let ui = window.global::<UiState>();

        Self::register_transport_callbacks(&ui, &tx_cmd);
        Self::register_file_callbacks(&ui, &tx_cmd);
        Self::register_volume_callbacks(&ui, &tx_cmd);
        Self::register_visualizer_callbacks(&ui);
        Self::register_playlist_callbacks(&ui);
        Self::spawn_event_handler(&window, rx_event);

        info!("UI initialized");
        Ok(Self { window })
    }

    pub fn run(&self) -> anyhow::Result<()> {
        self.window.run()?;
        Ok(())
    }

    // -- Transport callbacks ----------------------------------------------

    fn register_transport_callbacks(ui: &UiState, tx_cmd: &mpsc::Sender<Command>) {
        ui.on_play_pause_clicked({
            let tx = tx_cmd.clone();
            move || {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let _ = tx.send(Command::TogglePlayback).await;
                });
            }
        });

        ui.on_stop_clicked({
            let tx = tx_cmd.clone();
            move || {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let _ = tx.send(Command::Stop).await;
                });
            }
        });

        ui.on_skip_backward({
            let tx = tx_cmd.clone();
            move || {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let _ = tx.send(Command::SkipBackward(10.0)).await;
                });
            }
        });

        ui.on_skip_forward({
            let tx = tx_cmd.clone();
            move || {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let _ = tx.send(Command::SkipForward(10.0)).await;
                });
            }
        });

        ui.on_seek(|_position| {
            debug!("Seek requested (not yet implemented)");
        });

        ui.on_next_track(|| debug!("Next track (not yet implemented)"));
        ui.on_previous_track(|| debug!("Previous track (not yet implemented)"));
    }

    // -- File callbacks ---------------------------------------------------

    fn register_file_callbacks(ui: &UiState, tx_cmd: &mpsc::Sender<Command>) {
        ui.on_load_track_clicked({
            let tx = tx_cmd.clone();
            move || {
                let path = rfd::FileDialog::new()
                    .add_filter("Audio Files", &["mp3", "flac", "wav", "ogg", "m4a", "aac"])
                    .add_filter("All Files", &["*"])
                    .set_title("Select Audio File")
                    .pick_file();

                if let Some(path) = path {
                    info!("Selected file: {}", path.display());
                    let tx = tx.clone();
                    tokio::spawn(async move {
                        let _ = tx.send(Command::LoadFile(path)).await;
                    });
                }
            }
        });
    }

    // -- Volume callbacks -------------------------------------------------

    fn register_volume_callbacks(ui: &UiState, tx_cmd: &mpsc::Sender<Command>) {
        ui.on_volume_changed({
            let tx = tx_cmd.clone();
            move |volume| {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let _ = tx.send(Command::SetVolume(volume)).await;
                });
            }
        });
    }

    // -- Visualizer callbacks (stubs) -------------------------------------

    fn register_visualizer_callbacks(ui: &UiState) {
        ui.on_visualizer_changed(|vtype| debug!("Visualizer changed: {}", vtype));
        ui.on_visualizer_sensitivity_changed(|v| debug!("Visualizer sensitivity: {:.2}", v));
        ui.on_visualizer_smoothing_changed(|v| debug!("Visualizer smoothing: {:.2}", v));
        ui.on_visualizer_preset_selected(|name| debug!("Visualizer preset: {}", name));
        ui.on_fullscreen_toggled(|| debug!("Fullscreen toggled"));
    }

    // -- Playlist callbacks (stubs) ---------------------------------------

    fn register_playlist_callbacks(ui: &UiState) {
        ui.on_shuffle_toggled(|| debug!("Shuffle toggled"));
        ui.on_repeat_toggled(|| debug!("Repeat toggled"));
        ui.on_playlist_toggle_collapsed(|| debug!("Playlist toggle collapsed"));
        ui.on_playlist_track_selected(|i| debug!("Playlist track selected: {}", i));
        ui.on_playlist_track_removed(|i| debug!("Playlist track removed: {}", i));
        ui.on_playlist_load_files(|| debug!("Playlist load files"));
        ui.on_playlist_load_folder(|| debug!("Playlist load folder"));
        ui.on_playlist_save(|| debug!("Playlist save"));
        ui.on_playlist_clear(|| debug!("Playlist clear"));
        ui.on_playlist_folder_selected(|p| debug!("Playlist folder: {}", p));
        ui.on_playlist_file_selected(|p| debug!("Playlist file: {}", p));
    }

    // -- Event handler (controller -> UI) ---------------------------------

    fn spawn_event_handler(window: &AppWindow, mut rx_event: mpsc::Receiver<Event>) {
        let weak = window.as_weak();

        tokio::spawn(async move {
            while let Some(event) = rx_event.recv().await {
                let weak = weak.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    let Some(window) = weak.upgrade() else { return };
                    let ui = window.global::<UiState>();
                    Self::handle_event(&ui, event);
                });
            }
        });
    }

    fn handle_event(ui: &UiState, event: Event) {
        match event {
            Event::PlaybackStatus {
                is_playing,
                is_paused,
                volume,
                position,
                duration,
                track_path,
                format,
            } => {
                ui.set_is_playing(is_playing);
                ui.set_is_paused(is_paused);
                ui.set_volume(volume);

                let state = if is_playing {
                    "Playing"
                } else if is_paused {
                    "Paused"
                } else {
                    "Stopped"
                };
                ui.set_playback_state(state.into());
                ui.set_position_text(format_duration(position).into());

                if let Some(dur) = duration {
                    ui.set_duration_text(format_duration(dur).into());
                    if dur.as_secs() > 0 {
                        let progress = position.as_secs_f32() / dur.as_secs_f32();
                        ui.set_progress(progress.clamp(0.0, 1.0));
                    }
                }

                match track_path {
                    Some(ref path) => {
                        let name = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Unknown");
                        ui.set_current_track(name.into());

                        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                            ui.set_file_format(ext.to_uppercase().into());
                        }
                    }
                    None => {
                        ui.set_current_track("No track loaded".into());
                        // Clear metadata when no track is loaded.
                        ui.set_track_title("".into());
                        ui.set_track_artist("".into());
                        ui.set_track_album("".into());
                        ui.set_track_year("".into());
                        ui.set_track_genre("".into());
                    }
                }

                if let Some(fmt) = format {
                    ui.set_sample_rate(format!("{} Hz", fmt.sample_rate).into());
                    ui.set_channels(fmt.channels.to_string().into());
                    ui.set_bit_depth(format!("{} bit", fmt.bit_depth).into());
                }
            }

            Event::TrackLoaded { path, metadata } => {
                info!("Track loaded: {}", path.display());
                ui.set_track_title(metadata.title.unwrap_or_default().into());
                ui.set_track_artist(metadata.artist.unwrap_or_default().into());
                ui.set_track_album(metadata.album.unwrap_or_default().into());
                ui.set_track_year(
                    metadata
                        .year
                        .map(|y| y.to_string())
                        .unwrap_or_default()
                        .into(),
                );
                ui.set_track_genre(metadata.genre.unwrap_or_default().into());
            }

            Event::TrackLoadFailed { path, error } => {
                error!("Failed to load {}: {}", path.display(), error);
                ui.set_playback_state("Error".into());
            }

            Event::SpectrumUpdated { bands, peak } => {
                let model = slint::ModelRc::new(slint::VecModel::from(bands));
                ui.set_spectrum_bands(model);
                ui.set_peak_level(peak);
            }

            Event::Error(msg) => {
                error!("Error: {}", msg);
                ui.set_playback_state("Error".into());
            }
        }
    }
}

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    format!("{:02}:{:02}", secs / 60, secs % 60)
}
