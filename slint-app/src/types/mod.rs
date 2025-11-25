use std::{num::NonZero, sync::Arc};

use aurora_protocol::{Request, Song};
use lru::LruCache;
use slint::{Rgba8Pixel, SharedPixelBuffer};
use tokio::sync::{mpsc::Sender, Mutex};
use uuid::Uuid;

mod helpers;
pub type State = Arc<Mutex<StateStruct>>;

pub struct ImageCache(LruCache<Uuid, SharedPixelBuffer<Rgba8Pixel>>);
impl ImageCache {
    pub fn new() -> Self {
        Self(LruCache::new(NonZero::<usize>::new(400).unwrap()))
    }
    pub fn get(&mut self, id: Uuid) -> Option<SharedPixelBuffer<Rgba8Pixel>> {
        match self.0.get(&id) {
            Some(buf) => Some(buf.clone()),
            None => None,
        }
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
    pub waiting_for_art: Vec<Uuid>,
    pub cur_idx: usize,
}
