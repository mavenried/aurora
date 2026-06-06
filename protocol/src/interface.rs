use crate::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    Play(Uuid),
    Enqueue(Uuid),
    PlayNext(Uuid),
    PlaylistList,
    PlaylistGet(Uuid),
    PlaylistCreate(PlaylistIn),
    PlaylistDelete(Uuid),
    PlaylistAddSongs { playlist_id: Uuid, song_ids: Vec<Uuid> },
    PlaylistRemoveSong { playlist_id: Uuid, song_id: Uuid },
    PlaylistRename { playlist_id: Uuid, new_title: String },
    Clear,
    Next(usize),
    Prev(usize),
    Pause,
    Seek(Duration),
    Search(SearchType),
    ReplaceQueue(Vec<Uuid>),
    RemoveSong(Uuid),
    RemoveSongAt(usize),
    MoveQueue { from: usize, to: usize },
    SetVolume(f32),
    SetShuffle(bool),
    SetRepeat(u8),
    GetArtistList,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Response {
    Error { err_id: u8, err_msg: String },
    Status(Status),
    SearchResults(Vec<Song>),
    PlaylistResults(Playlist),
    PlaylistList(Vec<PlaylistMinimal>),
    Queue(Vec<Song>),
    Theme(Theme),
    Volume(f32),
    ArtistList(Vec<String>),
}
