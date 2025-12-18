use std::{num::NonZero, sync::Arc};

use aurora_protocol::{PlaylistMinimal, Request, Song};
use lru::LruCache;
use slint::{Rgba8Pixel, SharedPixelBuffer};
use tokio::sync::{Mutex, mpsc::Sender};
use uuid::Uuid;

mod helpers;

pub type State = Arc<Mutex<StateStruct>>;

pub enum ImageFor {
    Queue(Uuid),
    Search(Uuid),

    #[allow(unused)]
    Playlist(Uuid),
}

pub struct ImageCache(LruCache<Uuid, SharedPixelBuffer<Rgba8Pixel>>);
impl ImageCache {
    pub fn new() -> Self {
        Self(LruCache::new(NonZero::<usize>::new(500).unwrap()))
    }
    pub fn get(&mut self, id: Uuid) -> Option<SharedPixelBuffer<Rgba8Pixel>> {
        self.0.get(&id).cloned()
    }
    pub fn put(&mut self, id: Uuid, buf: SharedPixelBuffer<Rgba8Pixel>) {
        self.0.put(id, buf);
    }
}

pub struct StateStruct {
    pub default_art_buffer: SharedPixelBuffer<Rgba8Pixel>,
    pub artcache: ImageCache,
    pub writer_tx: Sender<Request>,
    pub queue: Vec<Song>,
    pub queue_waitlist: Vec<Uuid>,
    pub search_waitlist: Vec<Uuid>,
    pub playlist_waitlist: Vec<Uuid>,
    pub search_results: Vec<Song>,
    pub playlist_list_results: Vec<PlaylistMinimal>,
    pub playlist_results: Vec<Song>,
}
