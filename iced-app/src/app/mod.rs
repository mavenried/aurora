use std::time::Duration;

use crate::types::*;
use iced::{Task, widget::image};

pub mod helpers;
pub mod subscription;
pub mod tasks;
pub mod update;
pub mod views;

use aurora_protocol::{Playlist, Song, Status};
use uuid::Uuid;
pub struct AuroraPlayer {
    current_mainview: MainView,
    queue: Vec<Song>,
    loaded_playlist: Option<Vec<Playlist>>,
    artcache: ArtCache,
    default_album_art: image::Handle,
    status: Status,
    tcp_connection: Option<TcpWriter>,
    progress_slider_state: f32,
    slider_pressed: bool,
    pending_art_requests: Vec<Uuid>,
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
                progress_slider_state: 0.,
                slider_pressed: false,
                pending_art_requests: vec![],
            },
            Task::none(),
        )
    }
}
