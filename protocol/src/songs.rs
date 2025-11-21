use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};
use uuid::Uuid;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SongMeta {
    pub id: Uuid,
    pub title: String,
    pub artists: Vec<String>,
    pub duration: Duration,
    pub path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Song {
    pub id: Uuid,
    pub title: String,
    pub artists: Vec<String>,
    pub duration: Duration,
}

impl From<&SongMeta> for Song {
    fn from(value: &SongMeta) -> Self {
        Self {
            id: value.id,
            title: value.title.clone(),
            artists: value.artists.clone(),
            duration: value.duration,
        }
    }
}
