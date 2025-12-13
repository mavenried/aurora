mod interface;
mod playlists;
mod songs;
use std::time::Duration;

pub use interface::*;
pub use playlists::*;
pub use songs::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Status {
    pub current_song: Option<Song>,
    pub is_paused: bool,
    pub position: Duration,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum SearchType {
    ByTitle(String),
    ByArtist(String),
}
