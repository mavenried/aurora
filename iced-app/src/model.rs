use std::collections::HashSet;
use std::time::Duration;

use aurora_protocol::{Playlist, PlaylistMinimal, Song};
use iced::widget::image;
use uuid::Uuid;

use crate::DEFAULT_ART;

pub fn format_duration(d: Duration) -> String {
    let total = d.as_secs();
    let m = total / 60;
    let s = total % 60;
    format!("{m}:{s:02}")
}

pub fn art_handle(path: &Option<std::path::PathBuf>) -> image::Handle {
    match path {
        Some(p) => image::Handle::from_path(p),
        None => default_art(),
    }
}

pub fn default_art() -> image::Handle {
    image::Handle::from_bytes(DEFAULT_ART)
}

/// Which top-level tab is showing in the main (right-hand) view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Tab {
    #[default]
    Search,
    Library,
}

/// Which panel the Library tab is currently drilled into.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum LibraryPanel {
    #[default]
    Overview,
    Liked,
    Playlist,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    ByTitle,
    ByArtist,
}

impl SearchMode {
    pub const ALL: [SearchMode; 2] = [SearchMode::ByArtist, SearchMode::ByTitle];
}

impl std::fmt::Display for SearchMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchMode::ByTitle => write!(f, "By Title"),
            SearchMode::ByArtist => write!(f, "By Artist"),
        }
    }
}

/// What the currently-open context menu is anchored to.
#[derive(Debug, Clone, PartialEq)]
pub enum ContextMenuKind {
    Song {
        song_id: Uuid,
        liked: bool,
        /// Non-empty when viewing a specific playlist, enabling "Remove from Playlist".
        current_playlist: Option<Uuid>,
        submenu_open: bool,
    },
    Playlist {
        playlist_id: Uuid,
        title: String,
    },
    Queue {
        index: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContextMenuState {
    pub kind: ContextMenuKind,
    pub position: iced::Point,
}

#[derive(Debug, Clone, Default)]
pub struct QueueDrag {
    pub from: usize,
    pub start_y: f32,
    pub drop_target: usize,
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
