use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Play(Uuid),
    PlaylistList,
    PlaylistGet(Uuid),
    PlaylistCreate(PlaylistIn),
    AlbumArt(Uuid),
    Clear,
    Next(usize),
    Prev(usize),
    Pause,
    Seek(u64),
    Search(SearchType),
}

pub use crate::*;

#[derive(Serialize, Deserialize)]
pub enum Response {
    Error { err_id: u8, err_msg: String },
    Status(Status),
    SearchResults(Vec<Song>),
    Picture(String),
    PlaylistResults(Playlist),
    PlaylistList(Vec<PlaylistMinimal>),
}
