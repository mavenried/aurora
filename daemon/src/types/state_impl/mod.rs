use crate::types::{GetReturn, SongIndex, WriteSocket};
use aurora_protocol::{Response, Song, SongMeta, Status, Theme};
use rodio::Sink;
use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Weak};
use std::time::Duration;
use tokio::sync::Mutex;
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
    pub self_handle: Weak<Mutex<StateStruct>>,
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

    pub async fn add(&mut self) {
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

            self.get_art(song_uuid);

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

    /// Fire-and-forget: if `id`'s art hasn't been extracted yet, spawn a task to
    /// extract it and write it back into the shared index once done.
    pub fn get_art(&mut self, id: Uuid) {
        let Some(meta) = self.index.get(&id) else {
            return;
        };
        if meta.art_path.is_some() {
            return;
        }
        let path = meta.path.clone();
        let db = self.db.clone();
        let Some(state) = self.self_handle.upgrade() else {
            return;
        };
        tokio::spawn(async move {
            if let Some(art_path) = crate::helpers::extract_art(id, path, db).await {
                {
                    let mut s = state.lock().await;
                    if let Some(song) = s.index.get_mut(&id) {
                        song.art_path = Some(art_path.clone());
                    }
                }
                let _ = crate::helpers::send_to_all(&state, &Response::ArtReady { id, art_path })
                    .await;
            }
        });
    }
}
