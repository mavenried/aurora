use crate::types::{GetReturn, SongIndex, WriteSocket};
use aurora_protocol::{Song, SongMeta, Status, Theme};
use lofty::file::TaggedFileExt;
use lofty::read_from_path;
use rodio::Sink;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

pub struct StateStruct {
    pub current_song: Option<SongMeta>,
    pub queue: VecDeque<SongMeta>,
    pub index: SongIndex,
    pub sink: Arc<Sink>,
    pub clients: Vec<WriteSocket>,
    pub audio: Option<source::SeekableAudio>,
    pub theme: Theme,
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

    pub fn get_art(&mut self, id: Uuid) {
        tracing::info!("Getting albumart for {id}");
        let song = self.index.get_mut(&id).unwrap();
        if song.art_path.is_some() {
            return;
        }

        if let Ok(tagged_file) = read_from_path(&song.path)
            && let Some(tag) = tagged_file.primary_tag()
        {
            if !tag.pictures().is_empty() {
                let data = tag.pictures()[0].data();
                let ext = match tag.pictures()[0].mime_type() {
                    Some(lofty::picture::MimeType::Png) => "png",
                    Some(lofty::picture::MimeType::Jpeg) => "jpg",
                    Some(lofty::picture::MimeType::Tiff) => "tiff",
                    Some(lofty::picture::MimeType::Gif) => "gif",

                    _ => "bin",
                };

                let mut outpath = PathBuf::from("/tmp/aurora-player");
                std::fs::create_dir_all(&outpath).unwrap_or_else(|e| tracing::error!("{e}"));
                outpath.push(format!("{id}.{ext}"));
                tracing::debug!("Writing art to `{outpath:?}`.");
                std::fs::write(&outpath, data).unwrap_or_else(|e| tracing::error!("{e}"));
                song.art_path = Some(outpath);
                tracing::info!("{song:#?}");
            }
        }
    }
}
