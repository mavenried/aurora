use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct SongMeta {
    pub id: Uuid,
    pub title: String,
    pub artists: Vec<String>,
    pub duration: u32, // in seconds
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Status {
    pub current_song: Option<SongMeta>,
    pub queue: Vec<SongMeta>,
    pub current_idx: usize,
    pub is_paused: bool,
}
