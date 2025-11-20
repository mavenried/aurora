use crate::types::SongMeta;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
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
