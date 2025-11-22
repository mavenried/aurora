use std::time::Duration;

use crate::types::*;
use iced::{Task, widget::image};

pub mod helpers;
pub mod subscription;
pub mod update;
pub mod view;

use aurora_protocol::{Playlist, Song, Status};

pub struct AuroraPlayer {
    current_mainview: MainView,
    queue: Vec<Song>,
    loaded_playlist: Option<Vec<Playlist>>,
    artcache: ArtCache,
    default_album_art: image::Handle,
    status: Status,
    tcp_connection: Option<TcpConnection>,
}

impl AuroraPlayer {
    pub fn new() -> (Self, Task<Message>) {
        (
            Self {
                status: Status {
                    current_song: None,
                    position: Duration::ZERO,
                    is_paused: true,
                    current_idx: 0,
                },
                tcp_connection: None,
                current_mainview: MainView::Search,
                default_album_art: image::Handle::from_bytes(
                    include_bytes!("../../assets/placeholder.jpg").as_slice(),
                ),
                queue: vec![],
                loaded_playlist: None,
                artcache: ArtCache::new(),
            },
            Task::none(),
        )
    }
}
