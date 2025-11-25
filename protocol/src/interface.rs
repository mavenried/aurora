use crate::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
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
    Seek(Duration),
    Search(SearchType),
    ReplaceQueue(Vec<Song>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Response {
    Error { err_id: u8, err_msg: String },
    Status(Status),
    SearchResults(Vec<Song>),
    Picture { id: Uuid, data: String },
    PlaylistResults(Playlist),
    PlaylistList(Vec<PlaylistMinimal>),
    Queue(Vec<Song>),
}
