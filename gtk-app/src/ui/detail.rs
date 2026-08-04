use aurora_protocol::Song;
use uuid::Uuid;

use crate::app::Shared;
use crate::model::NavTarget;
use crate::ui::song_list;

/// Rebuilds the reused "detail" content page (title + play-all + song list)
/// for whichever of Liked / Recently Played / a specific Playlist is active.
/// A no-op when the current nav target isn't a detail page (Search/Queue).
pub fn rebuild(shared: &Shared) {
    let nav = shared.borrow().nav.clone();

    let (title, songs, playlist_id): (String, Vec<Song>, Option<Uuid>) = match &nav {
        NavTarget::Liked => ("Liked Songs".to_string(), shared.borrow().state.liked_songs.clone(), None),
        NavTarget::Recent => ("Recently Played".to_string(), shared.borrow().state.last_played.clone(), None),
        NavTarget::Playlist(id) => {
            let result = shared.borrow().state.playlist_result.clone();
            match result {
                Some(pl) if pl.id == *id => (pl.title.clone(), pl.songs.clone(), Some(*id)),
                _ => ("Loading playlist...".to_string(), vec![], Some(*id)),
            }
        }
        NavTarget::Search | NavTarget::Queue => return,
    };

    let (title_lbl, list) = {
        let s = shared.borrow();
        (s.widgets.detail_title_lbl.clone(), s.widgets.detail_list_box.clone())
    };
    title_lbl.set_text(&format!("{title} · {} songs", songs.len()));
    song_list::populate_song_list(shared, &list, &songs, playlist_id);
}

/// Song ids for whatever the detail page is currently showing, used by the
/// persistent "Play All" button so it never needs its click handler rewired.
pub fn current_song_ids(shared: &Shared) -> Vec<Uuid> {
    let s = shared.borrow();
    match &s.nav {
        NavTarget::Liked => s.state.liked_songs.iter().map(|s| s.id).collect(),
        NavTarget::Recent => s.state.last_played.iter().map(|s| s.id).collect(),
        NavTarget::Playlist(id) => s
            .state
            .playlist_result
            .as_ref()
            .filter(|pl| pl.id == *id)
            .map(|pl| pl.songs.iter().map(|s| s.id).collect())
            .unwrap_or_default(),
        NavTarget::Search | NavTarget::Queue => vec![],
    }
}
