use std::{cell::RefCell, num::NonZeroUsize};

use aurora_protocol::Response;
use iced::widget::image::Handle;
use lru::LruCache;
use uuid::Uuid;

mod tcp;
pub use tcp::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MainView {
    Playlist,
    AllPlaylist,
    Search,
}

#[derive(Debug, Clone)]
pub enum Message {
    MainViewSelect(MainView),
    PlaylistSelected(Uuid),
    TcpEvent(TcpEvent),
    Response(Response),
}

pub struct ArtCache(RefCell<LruCache<Uuid, Handle>>);
impl ArtCache {
    pub fn new() -> Self {
        Self(RefCell::new(LruCache::new(NonZeroUsize::new(200).unwrap())))
    }

    pub fn get(&self, id: &Uuid) -> Option<Handle> {
        self.0.borrow_mut().get(id).cloned()
    }

    pub fn insert(&self, id: Uuid, art: Handle) {
        self.0.borrow_mut().put(id, art);
    }
}
