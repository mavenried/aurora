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
    pub art_path: Option<PathBuf>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Song {
    pub id: Uuid,
    pub title: String,
    pub artists: Vec<String>,
    pub duration: Duration,
    pub art_path: Option<PathBuf>,
}

impl From<&SongMeta> for Song {
    fn from(value: &SongMeta) -> Self {
        Self {
            id: value.id,
            title: value.title.clone(),
            artists: value.artists.clone(),
            duration: value.duration,
            art_path: value.art_path.clone(),
        }
    }
}
