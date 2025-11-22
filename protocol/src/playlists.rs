use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Song;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlaylistMinimal {
    pub id: Uuid,
    pub name: String,
    pub len: usize,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Playlist {
    pub id: Uuid,
    pub title: String,
    pub songs: Vec<Song>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlaylistIn {
    pub title: String,
    pub songs: Vec<Song>,
}
