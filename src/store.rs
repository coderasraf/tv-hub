//! Per-user state kept between runs: favorites, recently watched, volume,
//! and the results of the last stream checks. One JSON file, rewritten
//! atomically on every change; it is small.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    pub favorites: Vec<String>,
    /// Most recent first.
    pub recents: Vec<String>,
    pub volume: Option<f64>,
    /// Keyed by stream URL.
    pub health: HashMap<String, Health>,
    /// Keyed by lowercase YouTube handle.
    pub youtube: HashMap<String, YtStatus>,
    /// Unix seconds of the last finished full check.
    pub last_check: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub ok: bool,
    pub checked: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct YtStatus {
    pub live: bool,
    /// False when the broadcaster blocks playback outside youtube.com.
    pub embeddable: bool,
    pub video_id: String,
    pub title: String,
    pub avatar: String,
    pub checked: u64,
}

pub const MAX_RECENTS: usize = 30;

pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub struct Store {
    path: PathBuf,
    pub state: State,
}

impl Store {
    /// A missing or unreadable file starts fresh rather than failing: none
    /// of this is worth refusing to start over.
    pub fn open(path: &Path) -> Store {
        let state = std::fs::read_to_string(path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        Store { path: path.to_path_buf(), state }
    }

    pub fn save(&self) {
        let tmp = self.path.with_extension("json.tmp");
        if let Ok(text) = serde_json::to_string(&self.state) {
            if std::fs::write(&tmp, text).is_ok() {
                let _ = std::fs::rename(&tmp, &self.path);
            }
        }
    }

    pub fn set_favorite(&mut self, id: &str, on: bool) {
        self.state.favorites.retain(|f| f != id);
        if on {
            self.state.favorites.push(id.to_string());
        }
    }

    pub fn add_recent(&mut self, id: &str) {
        self.state.recents.retain(|r| r != id);
        self.state.recents.insert(0, id.to_string());
        self.state.recents.truncate(MAX_RECENTS);
    }
}
