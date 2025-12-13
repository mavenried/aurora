use crate::types::{GetReturn, SongIndex, WriteSocket};
use aurora_protocol::{Song, SongMeta, Status};
use rodio::Sink;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

pub struct StateStruct {
    pub current_song: Option<SongMeta>,
    pub queue: VecDeque<SongMeta>,
    pub index: SongIndex,
    pub sink: Arc<Sink>,
    pub clients: Vec<WriteSocket>,
    pub audio: Option<source::SeekableAudio>,
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
        }
    }
    pub async fn add(&mut self) {
        if let Some(song) = &self.current_song {
            let song_uuid = song.id;
            let song = self.index.get(&song_uuid).unwrap();
            tracing::info!("Adding song_id : {song_uuid}");

            self.sink.clear();
            if let Ok(audio) = source::SeekableAudio::new(&song.path, self.sink.clone()) {
                self.audio = Some(audio)
            } else {
                tracing::error!("Could not load new SeekableAudio.");
            };

            self.sink.play();
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
}
