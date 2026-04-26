//! Playlist management — ordered track list with shuffle and repeat support.

use std::path::{Path, PathBuf};
use std::time::Duration;

/// Minimum information needed to represent a track in the playlist.
#[derive(Debug, Clone)]
pub struct TrackInfo {
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub duration: Option<Duration>,
}

impl TrackInfo {
    /// Build a `TrackInfo` from a path, using the file stem as the fallback title.
    pub fn from_path(path: PathBuf) -> Self {
        let title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();
        Self {
            path,
            title,
            artist: String::new(),
            duration: None,
        }
    }

    /// Overlay metadata onto a path-only `TrackInfo`.
    pub fn with_metadata(
        mut self,
        title: Option<String>,
        artist: Option<String>,
        duration: Option<Duration>,
    ) -> Self {
        if let Some(t) = title.filter(|t| !t.is_empty()) {
            self.title = t;
        }
        if let Some(a) = artist {
            self.artist = a;
        }
        self.duration = duration;
        self
    }
}

/// Repeat behaviour for the playlist.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum RepeatMode {
    #[default]
    None,
    One,
    All,
}

impl RepeatMode {
    /// Advance to the next mode in the cycle: None → One → All → None.
    pub fn cycle(self) -> Self {
        match self {
            Self::None => Self::One,
            Self::One => Self::All,
            Self::All => Self::None,
        }
    }
}

/// Ordered track list with optional shuffle and repeat.
pub struct Playlist {
    tracks: Vec<TrackInfo>,
    current_index: Option<usize>,
    /// Permuted indices used when shuffle is active.
    shuffle_order: Vec<usize>,
    /// Position within `shuffle_order` for the currently playing track.
    shuffle_pos: usize,
    pub repeat_mode: RepeatMode,
    pub shuffle_enabled: bool,
}

impl Default for Playlist {
    fn default() -> Self {
        Self::new()
    }
}

impl Playlist {
    pub fn new() -> Self {
        Self {
            tracks: Vec::new(),
            current_index: None,
            shuffle_order: Vec::new(),
            shuffle_pos: 0,
            repeat_mode: RepeatMode::None,
            shuffle_enabled: false,
        }
    }

    /// Append tracks. Shuffle order is extended if shuffle is currently active.
    pub fn add_tracks(&mut self, mut infos: Vec<TrackInfo>) {
        let start = self.tracks.len();
        self.tracks.append(&mut infos);
        if self.shuffle_enabled {
            let new_indices: Vec<usize> = (start..self.tracks.len()).collect();
            self.shuffle_order.extend(new_indices);
        }
    }

    /// Advance to the next track, applying repeat/shuffle logic.
    ///
    /// Returns `None` when the end of the playlist is reached and repeat is
    /// not set to `All`.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<&TrackInfo> {
        if self.tracks.is_empty() {
            return None;
        }
        if self.repeat_mode == RepeatMode::One {
            return self.current();
        }

        if self.shuffle_enabled {
            let next_pos = self.shuffle_pos + 1;
            if next_pos >= self.shuffle_order.len() {
                if self.repeat_mode == RepeatMode::All {
                    self.shuffle_pos = 0;
                } else {
                    self.current_index = None;
                    return None;
                }
            } else {
                self.shuffle_pos = next_pos;
            }
            let idx = self.shuffle_order[self.shuffle_pos];
            self.current_index = Some(idx);
        } else {
            let next = self.current_index.map(|i| i + 1).unwrap_or(0);
            if next >= self.tracks.len() {
                if self.repeat_mode == RepeatMode::All {
                    self.current_index = Some(0);
                } else {
                    self.current_index = None;
                    return None;
                }
            } else {
                self.current_index = Some(next);
            }
        }

        self.current()
    }

    /// Move to the previous track, applying repeat/shuffle logic.
    pub fn previous(&mut self) -> Option<&TrackInfo> {
        if self.tracks.is_empty() {
            return None;
        }
        if self.repeat_mode == RepeatMode::One {
            return self.current();
        }

        if self.shuffle_enabled {
            if self.shuffle_pos == 0 {
                if self.repeat_mode == RepeatMode::All {
                    self.shuffle_pos = self.shuffle_order.len().saturating_sub(1);
                }
                // else: stay at first track
            } else {
                self.shuffle_pos -= 1;
            }
            let idx = self
                .shuffle_order
                .get(self.shuffle_pos)
                .copied()
                .unwrap_or(0);
            self.current_index = Some(idx);
        } else {
            let prev = match self.current_index {
                None | Some(0) => {
                    if self.repeat_mode == RepeatMode::All {
                        self.tracks.len().saturating_sub(1)
                    } else {
                        0
                    }
                }
                Some(i) => i - 1,
            };
            self.current_index = Some(prev);
        }

        self.current()
    }

    /// Jump directly to a track by index.
    pub fn select(&mut self, index: usize) -> Option<&TrackInfo> {
        if index >= self.tracks.len() {
            return None;
        }
        self.current_index = Some(index);
        if self.shuffle_enabled
            && let Some(pos) = self.shuffle_order.iter().position(|&i| i == index)
        {
            self.shuffle_pos = pos;
        }
        self.tracks.get(index)
    }

    /// Remove the track at `index`. Adjusts `current_index` accordingly.
    pub fn remove(&mut self, index: usize) {
        if index >= self.tracks.len() {
            return;
        }
        self.tracks.remove(index);

        self.current_index = match self.current_index {
            Some(cur) if cur == index => {
                if self.tracks.is_empty() {
                    None
                } else {
                    Some(cur.min(self.tracks.len() - 1))
                }
            }
            Some(cur) if cur > index => Some(cur - 1),
            other => other,
        };

        if self.shuffle_enabled {
            self.rebuild_shuffle();
        }
    }

    /// Remove all tracks and reset state.
    pub fn clear(&mut self) {
        self.tracks.clear();
        self.current_index = None;
        self.shuffle_order.clear();
        self.shuffle_pos = 0;
    }

    /// Toggle shuffle mode on/off, rebuilding the shuffle order when enabling.
    pub fn toggle_shuffle(&mut self) {
        self.shuffle_enabled = !self.shuffle_enabled;
        if self.shuffle_enabled {
            self.rebuild_shuffle();
        }
    }

    /// Advance the repeat mode through the cycle: None → One → All → None.
    pub fn cycle_repeat(&mut self) {
        self.repeat_mode = self.repeat_mode.cycle();
    }

    /// Return the currently selected track, if any.
    pub fn current(&self) -> Option<&TrackInfo> {
        self.current_index.and_then(|i| self.tracks.get(i))
    }

    pub fn tracks(&self) -> &[TrackInfo] {
        &self.tracks
    }

    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }

    /// Sum of all known track durations.
    pub fn total_duration(&self) -> Duration {
        self.tracks
            .iter()
            .filter_map(|t| t.duration)
            .fold(Duration::ZERO, |acc, d| acc + d)
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    // ------------------------------------------------------------------
    // Internal helpers
    // ------------------------------------------------------------------

    /// Rebuild the shuffle permutation, placing the current track first.
    fn rebuild_shuffle(&mut self) {
        let n = self.tracks.len();
        self.shuffle_order = (0..n).collect();
        fisher_yates_shuffle(&mut self.shuffle_order);

        // Keep the currently playing track at position 0 of the shuffle order
        // so it does not repeat immediately in the next round.
        if let Some(cur) = self.current_index {
            if let Some(pos) = self.shuffle_order.iter().position(|&i| i == cur) {
                self.shuffle_order.swap(0, pos);
            }
            self.shuffle_pos = 0;
        }
    }
}

/// Accepted audio file extensions for folder scanning.
pub const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "wav", "ogg", "m4a", "aac"];

/// Recursively collect all audio files under `dir`.
pub fn scan_folder(dir: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    scan_folder_inner(dir, &mut results);
    results.sort();
    results
}

fn scan_folder_inner(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_folder_inner(&path, out);
        } else if let Some(ext) = path.extension().and_then(|e| e.to_str())
            && AUDIO_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str())
        {
            out.push(path);
        }
    }
}

/// Fisher-Yates shuffle using a simple time-seeded LCG (no external crate).
fn fisher_yates_shuffle(v: &mut [usize]) {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as u64 ^ (d.as_secs() << 32))
        .unwrap_or(0xdeadbeef_cafebabe);

    // Knuth multiplicative hash LCG
    let mut rng = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);

    let n = v.len();
    for i in (1..n).rev() {
        rng = rng
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let j = (rng >> 33) as usize % (i + 1);
        v.swap(i, j);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_track(n: usize) -> TrackInfo {
        TrackInfo {
            path: PathBuf::from(format!("/music/track_{n:02}.mp3")),
            title: format!("Track {n}"),
            artist: "Artist".into(),
            duration: Some(Duration::from_secs(180)),
        }
    }

    fn playlist_of(n: usize) -> Playlist {
        let mut p = Playlist::new();
        p.add_tracks((1..=n).map(make_track).collect());
        p
    }

    #[test]
    fn add_and_select() {
        let mut p = playlist_of(3);
        assert_eq!(p.len(), 3);
        let t = p.select(1).expect("index 1 should exist");
        assert_eq!(t.title, "Track 2");
        assert_eq!(p.current_index(), Some(1));
    }

    #[test]
    fn next_sequential() {
        let mut p = playlist_of(3);
        p.select(0);
        assert_eq!(p.next().map(|t| t.title.as_str()), Some("Track 2"));
        assert_eq!(p.next().map(|t| t.title.as_str()), Some("Track 3"));
        assert!(p.next().is_none()); // end of list, no repeat
    }

    #[test]
    fn next_repeat_all() {
        let mut p = playlist_of(3);
        p.repeat_mode = RepeatMode::All;
        p.select(2);
        assert_eq!(p.next().map(|t| t.title.as_str()), Some("Track 1"));
    }

    #[test]
    fn next_repeat_one() {
        let mut p = playlist_of(3);
        p.repeat_mode = RepeatMode::One;
        p.select(1);
        assert_eq!(p.next().map(|t| t.title.as_str()), Some("Track 2"));
        assert_eq!(p.next().map(|t| t.title.as_str()), Some("Track 2"));
    }

    #[test]
    fn previous_sequential() {
        let mut p = playlist_of(3);
        p.select(2);
        assert_eq!(p.previous().map(|t| t.title.as_str()), Some("Track 2"));
        assert_eq!(p.previous().map(|t| t.title.as_str()), Some("Track 1"));
        // At first track without repeat — stays at 0
        assert_eq!(p.previous().map(|t| t.title.as_str()), Some("Track 1"));
    }

    #[test]
    fn remove_current_track() {
        let mut p = playlist_of(3);
        p.select(1);
        p.remove(1);
        assert_eq!(p.len(), 2);
        // current_index clamped to new length
        assert!(p.current_index() <= Some(1));
    }

    #[test]
    fn remove_track_before_current() {
        let mut p = playlist_of(3);
        p.select(2);
        p.remove(0);
        assert_eq!(p.current_index(), Some(1)); // index shifted down
    }

    #[test]
    fn clear_resets_state() {
        let mut p = playlist_of(3);
        p.select(1);
        p.clear();
        assert!(p.is_empty());
        assert_eq!(p.current_index(), None);
    }

    #[test]
    fn total_duration() {
        let p = playlist_of(3);
        assert_eq!(p.total_duration(), Duration::from_secs(540)); // 3 * 180
    }

    #[test]
    fn shuffle_covers_all_tracks() {
        let mut p = playlist_of(5);
        p.select(0);
        p.toggle_shuffle();
        assert!(p.shuffle_enabled);
        assert_eq!(p.shuffle_order.len(), 5);

        let mut visited = std::collections::HashSet::new();
        visited.insert(p.current_index().unwrap());
        for _ in 0..4 {
            p.next();
            if let Some(idx) = p.current_index() {
                visited.insert(idx);
            }
        }
        assert_eq!(visited.len(), 5, "all 5 tracks should be visited");
    }

    #[test]
    fn scan_folder_empty_dir() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let found = scan_folder(tmp.path());
        assert!(found.is_empty());
    }

    #[test]
    fn scan_folder_finds_audio_files() {
        let tmp = tempfile::tempdir().expect("tempdir");
        // Create dummy audio files.
        for name in &["a.mp3", "b.flac", "c.txt", "d.wav"] {
            std::fs::write(tmp.path().join(name), b"").expect("write");
        }
        let found = scan_folder(tmp.path());
        assert_eq!(found.len(), 3, "should find mp3, flac, wav but not txt");
    }
}
