use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::Song;

#[derive(Serialize, Deserialize, Clone)]
pub struct PlaylistMinimal {
    pub id: Uuid,
    pub name: String,
    pub len: usize,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct Playlist {
    pub id: Uuid,
    pub title: String,
    pub songs: Vec<Song>,
}
