use std::sync::Arc;

use aurora_protocol::{Playlist, PlaylistMinimal, Request, Song};
use slint::{Rgba8Pixel, SharedPixelBuffer};
use tokio::sync::{Mutex, mpsc::Sender};

mod helpers;

pub type State = Arc<Mutex<StateStruct>>;
pub struct StateStruct {
    pub default_art_buffer: SharedPixelBuffer<Rgba8Pixel>,
    pub writer_tx: Sender<Request>,
    pub queue: Vec<Song>,
    pub search_results: Vec<Song>,
    pub playlist_list_results: Vec<PlaylistMinimal>,
    pub playlist_result: Option<Playlist>,
}
