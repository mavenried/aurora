use crate::types::{GetReturn, SongIndex, WriteSocket};
use aurora_protocol::{Song, SongMeta, Status, Theme};
use image::{DynamicImage, ImageError};
use image::{ImageReader, imageops::FilterType};
use lofty::file::TaggedFileExt;
use lofty::read_from_path;
use rodio::Sink;
use std::collections::VecDeque;
use std::io::Cursor;
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

        let song = match self.index.get_mut(&id) {
            Some(s) => s,
            None => return,
        };

        if song.art_path.is_some() {
            return;
        }

        let Ok(tagged_file) = read_from_path(&song.path) else {
            return;
        };
        let Some(tag) = tagged_file.primary_tag() else {
            return;
        };
        let Some(pic) = tag.pictures().first() else {
            return;
        };

        let image;
        if let Ok(img) = ImageReader::new(Cursor::new(pic.data())).with_guessed_format()
            && let Ok(img) = img.decode()
        {
            image = img.into_rgb8();
        } else {
            tracing::error!("Failed to decode album art");
            return;
        }

        let resized = image::imageops::resize(&image, 100, 100, FilterType::Nearest);

        let mut outpath = PathBuf::from("/tmp/aurora-player");
        if let Err(e) = std::fs::create_dir_all(&outpath) {
            tracing::error!("{e}");
            return;
        }

        outpath.push(format!("{id}.jpg"));
        tracing::debug!("Writing resized art to `{outpath:?}`");

        let file = match std::fs::File::create(&outpath) {
            Ok(f) => f,
            Err(e) => {
                tracing::error!("{e}");
                return;
            }
        };

        if let Err(e) =
            resized.write_to(&mut std::io::BufWriter::new(file), image::ImageFormat::Jpeg)
        {
            tracing::error!("Failed to write JPEG: {e}");
            return;
        }

        song.art_path = Some(outpath);
    }
}
