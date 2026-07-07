use crate::types::{GetReturn, SongIndex, State, WriteSocket};
use aurora_protocol::{Song, SongMeta, Status, Theme};
use rodio::Sink;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use crate::helpers::db::Db;

pub struct StateStruct {
    pub current_song: Option<SongMeta>,
    pub queue: VecDeque<SongMeta>,
    pub index: SongIndex,
    pub sink: Arc<Sink>,
    pub clients: Vec<WriteSocket>,
    pub audio: Option<source::SeekableAudio>,
    pub theme: Theme,
    pub volume: f32,
    pub shuffle: bool,
    pub repeat: u8,
    pub recently_played: VecDeque<Uuid>,
    pub liked_ids: HashSet<Uuid>,
    pub db: Db,
}

mod playback;
mod search;
mod source;

impl StateStruct {
    pub fn to_status(&self) -> Status {
        Status {
            current_song: self
                .current_song
                .clone()
                .map(|songmeta| Song::from(&songmeta)),
            is_paused: self.is_paused(),
            position: if let Some(audio) = &self.audio {
                audio.get_position()
            } else {
                Duration::ZERO
            },
            volume: self.volume,
            shuffle: self.shuffle,
            repeat: self.repeat,
        }
    }

    pub async fn add(&mut self, state: &State) {
        if let Some(song) = &self.current_song {
            let song_uuid = song.id;
            let path = self.index.get(&song_uuid).unwrap().path.clone();
            tracing::info!("Adding song_id : {song_uuid}");

            self.sink.clear();
            let sink = self.sink.clone();
            let p = path.clone();
            let audio_result = tokio::task::block_in_place(|| source::SeekableAudio::new(&p, sink));
            if let Ok(audio) = audio_result {
                self.audio = Some(audio);
            } else {
                tracing::error!("Could not load new SeekableAudio.");
            }
            self.sink.play();

            if self.index.get(&song_uuid).is_some_and(|s| s.art_path.is_none()) {
                let db = self.db.clone();
                let state = state.clone();
                tokio::spawn(async move {
                    if let Some(art_path) = crate::helpers::extract_art(song_uuid, path, db).await {
                        let mut s = state.lock().await;
                        if let Some(song) = s.index.get_mut(&song_uuid) {
                            song.art_path = Some(art_path);
                        }
                    }
                });
            }

            crate::helpers::push_history(&mut self.recently_played, song_uuid);
            let history = self.recently_played.clone();
            let db = self.db.clone();
            tokio::spawn(async move {
                if let Err(e) = crate::helpers::save_history(&db, &history).await {
                    tracing::error!("Failed to save play history: {e}");
                }
            });
        }
    }

    pub async fn pause(&mut self) {
        if self.sink.is_paused() {
            self.sink.play();
        } else {
            self.sink.pause();
        }
    }

    pub async fn clear(&mut self) {
        self.sink.clear();
        self.queue.clear();
        self.current_song = None;
        self.sink.play();
        self.audio = None;
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    pub fn pending_art(&self, ids: &[Uuid]) -> Vec<(Uuid, std::path::PathBuf, Db)> {
        ids.iter()
            .filter_map(|&id| {
                let meta = self.index.get(&id)?;
                meta.art_path
                    .is_none()
                    .then(|| (id, meta.path.clone(), self.db.clone()))
            })
            .collect()
    }
}
