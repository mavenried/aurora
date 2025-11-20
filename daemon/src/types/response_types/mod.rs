use std::time::Duration;

use serde::Serialize;

mod playlist;
mod song;

pub use playlist::*;
pub use song::*;

#[derive(Serialize)]
pub struct Status {
    pub current_song: Option<Song>,
    pub queue: Vec<Song>,
    pub current_idx: usize,
    pub is_paused: bool,
    pub position: Duration,
}

#[derive(Serialize)]
pub enum Response {
    Error { err_id: u8, err_msg: String },
    Status(Status),
    SearchResults(Vec<Song>),
    Confirm { message: String },
}
