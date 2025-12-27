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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Theme {
    pub bgd0: String,
    pub bgd1: String,
    pub bgd2: String,
    pub bgd3: String,
    pub bgd4: String,
    pub txt1: String,
    pub txt2: String,
    pub acct: String,
    pub srch: String,
    pub btns: String,
}
