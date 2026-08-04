use std::collections::HashSet;
use std::time::Duration;

use aurora_protocol::{Playlist, PlaylistMinimal, Song};
use uuid::Uuid;

pub fn format_duration(d: Duration) -> String {
    let total = d.as_secs();
    let m = total / 60;
    let s = total % 60;
    format!("{m}:{s:02}")
}

/// Which destination is selected in the sidebar / shown in the content pane.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum NavTarget {
    #[default]
    Search,
    Queue,
    Liked,
    Recent,
    Playlist(Uuid),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    ByTitle,
    ByArtist,
}

#[derive(Debug, Default, Clone)]
pub struct State {
    pub queue: Vec<Song>,
    pub search_results: Vec<Song>,
    pub playlist_list_results: Vec<PlaylistMinimal>,
    pub playlist_result: Option<Playlist>,
    pub selected_song_ids: HashSet<Uuid>,
    pub liked_song_ids: HashSet<Uuid>,
    pub liked_songs: Vec<Song>,
    pub last_played: Vec<Song>,

    pub current_song: Option<Song>,
    pub is_paused: bool,
    pub position: Duration,
    pub duration: Duration,
    pub volume: f32,
    pub shuffle: bool,
    pub repeat: u8,
}

impl State {
    pub fn is_liked(&self, id: &Uuid) -> bool {
        self.liked_song_ids.contains(id)
    }

    pub fn is_selected(&self, id: &Uuid) -> bool {
        self.selected_song_ids.contains(id)
    }

    /// Expands a clicked song id into the full selection set if it's part of
    /// a multi-select, mirroring the Slint app's selection-aware callbacks.
    pub fn expand_selection(&self, clicked: Uuid) -> Vec<Uuid> {
        if self.selected_song_ids.contains(&clicked) {
            self.selected_song_ids.iter().copied().collect()
        } else {
            vec![clicked]
        }
    }
}
