use std::{collections::HashSet, sync::Arc};

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
    pub selected_song_ids: HashSet<String>,
    pub current_song_id: String,
    pub artist_list: Vec<String>,
    pub artist_songs: Vec<Song>,
    pub pending_artist_search: bool,
}
