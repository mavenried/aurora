use aurora_protocol::{Request, SearchType};
use gtk::prelude::*;

use crate::app::{send, Shared};
use crate::model::SearchMode;
use crate::ui::song_list;

pub fn maybe_search(shared: &Shared, query: &str, force: bool) {
    let q = query.trim();
    if q.len() > 2 || (!q.is_empty() && force) {
        let mode = shared.borrow().search_mode;
        let search = match mode {
            SearchMode::ByArtist => SearchType::ByArtist(q.to_string()),
            SearchMode::ByTitle => SearchType::ByTitle(q.to_string()),
        };
        send(shared, Request::Search(search));
    } else if q.is_empty() {
        shared.borrow_mut().state.search_results.clear();
        rebuild_results(shared);
    }
}

pub fn rebuild_results(shared: &Shared) {
    let container = shared.borrow().widgets.search_results_box.clone();
    let query_empty = shared.borrow().widgets.search_entry.text().trim().is_empty();
    let songs = shared.borrow().state.search_results.clone();

    if query_empty && songs.is_empty() {
        crate::ui::clear(&container);
        container.append(&song_list::empty_state(
            "system-search-symbolic",
            "Search your library",
            "Find songs by title or artist",
        ));
        return;
    }

    song_list::populate_song_list(shared, &container, &songs, None);
}
