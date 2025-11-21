mod interface;
mod playlists;
mod songs;
use std::time::Duration;

pub use interface::*;
pub use playlists::*;
pub use songs::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Status {
    pub current_song: Option<Song>,
    pub queue: Vec<Song>,
    pub current_idx: usize,
    pub is_paused: bool,
    pub position: Duration,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum SearchType {
    ByTitle(String),
    ByArtist(String),
}
