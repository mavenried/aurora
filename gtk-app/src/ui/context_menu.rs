use aurora_protocol::Request;
use gtk::gdk;
use gtk::prelude::*;
use gtk::{gio, glib};
use uuid::Uuid;

use crate::app::{send, send_many, Shared};

fn popup_at(parent: &impl IsA<gtk::Widget>, menu: &gio::Menu, actions: &gio::SimpleActionGroup, prefix: &str, x: f64, y: f64) {
    parent.insert_action_group(prefix, Some(actions));
    let popover = gtk::PopoverMenu::from_model(Some(menu));
    popover.set_parent(parent);
    popover.set_has_arrow(false);
    popover.set_halign(gtk::Align::Start);
    popover.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
    popover.connect_closed(|p| p.unparent());
    popover.popup();
}

pub fn show_song_menu(shared: &Shared, parent: &impl IsA<gtk::Widget>, song_id: Uuid, liked: bool, current_playlist: Option<Uuid>, x: f64, y: f64) {
    let menu = gio::Menu::new();
    menu.append(Some("Play Next"), Some("song.play-next"));
    menu.append(Some("Add to Queue"), Some("song.enqueue"));
    menu.append(
        Some(if liked { "Remove from Liked Songs" } else { "Add to Liked Songs" }),
        Some("song.like-toggle"),
    );

    let playlists = shared.borrow().state.playlist_list_results.clone();
    if !playlists.is_empty() {
        let submenu = gio::Menu::new();
        for pl in &playlists {
            submenu.append(Some(&pl.name), Some(&format!("song.add-to-playlist::'{}'", pl.id)));
        }
        menu.append_submenu(Some("Add to Playlist"), &submenu);
    }

    if current_playlist.is_some() {
        menu.append(Some("Remove from Playlist"), Some("song.remove-from-playlist"));
    }

    let actions = gio::SimpleActionGroup::new();

    let act = gio::SimpleAction::new("play-next", None);
    let shared_c = shared.clone();
    act.connect_activate(move |_, _| {
        let ids = shared_c.borrow().state.expand_selection(song_id);
        send_many(&shared_c, ids.into_iter().map(Request::PlayNext).collect());
    });
    actions.add_action(&act);

    let act = gio::SimpleAction::new("enqueue", None);
    let shared_c = shared.clone();
    act.connect_activate(move |_, _| {
        let ids = shared_c.borrow().state.expand_selection(song_id);
        send_many(&shared_c, ids.into_iter().map(Request::Enqueue).collect());
    });
    actions.add_action(&act);

    let act = gio::SimpleAction::new("like-toggle", None);
    let shared_c = shared.clone();
    act.connect_activate(move |_, _| {
        let ids = shared_c.borrow().state.expand_selection(song_id);
        let reqs = ids
            .into_iter()
            .map(|id| if liked { Request::UnlikeSong(id) } else { Request::LikeSong(id) })
            .collect();
        send_many(&shared_c, reqs);
    });
    actions.add_action(&act);

    let act = gio::SimpleAction::new("add-to-playlist", Some(glib::VariantTy::STRING));
    let shared_c = shared.clone();
    act.connect_activate(move |_, param| {
        let Some(param) = param else { return };
        let Some(id_str) = param.str() else { return };
        let Ok(playlist_id) = Uuid::parse_str(id_str) else { return };
        let ids = shared_c.borrow().state.expand_selection(song_id);
        send(&shared_c, Request::PlaylistAddSongs { playlist_id, song_ids: ids });
    });
    actions.add_action(&act);

    if let Some(pl_id) = current_playlist {
        let act = gio::SimpleAction::new("remove-from-playlist", None);
        let shared_c = shared.clone();
        act.connect_activate(move |_, _| {
            let ids = shared_c.borrow().state.expand_selection(song_id);
            send_many(
                &shared_c,
                ids.into_iter()
                    .map(|song_id| Request::PlaylistRemoveSong { playlist_id: pl_id, song_id })
                    .collect(),
            );
        });
        actions.add_action(&act);
    }

    popup_at(parent, &menu, &actions, "song", x, y);
}

pub fn show_playlist_menu(shared: &Shared, parent: &impl IsA<gtk::Widget>, playlist_id: Uuid, title: &str, x: f64, y: f64) {
    let menu = gio::Menu::new();
    menu.append(Some("Rename"), Some("playlist.rename"));
    menu.append(Some("Delete Playlist"), Some("playlist.delete"));

    let actions = gio::SimpleActionGroup::new();

    let act = gio::SimpleAction::new("rename", None);
    let shared_c = shared.clone();
    let parent_owned = parent.clone().upcast::<gtk::Widget>();
    let title = title.to_string();
    act.connect_activate(move |_, _| {
        super::sidebar::prompt(&shared_c, &parent_owned, "Rename Playlist", &title, move |shared, new_title| {
            if !new_title.is_empty() {
                send(shared, Request::PlaylistRename { playlist_id, new_title });
            }
        });
    });
    actions.add_action(&act);

    let act = gio::SimpleAction::new("delete", None);
    let shared_c = shared.clone();
    act.connect_activate(move |_, _| {
        send(&shared_c, Request::PlaylistDelete(playlist_id));
    });
    actions.add_action(&act);

    popup_at(parent, &menu, &actions, "playlist", x, y);
}

pub fn show_queue_menu(shared: &Shared, parent: &impl IsA<gtk::Widget>, index: usize, x: f64, y: f64) {
    let menu = gio::Menu::new();
    menu.append(Some("Move to Front"), Some("queue.move-to-front"));
    menu.append(Some("Remove"), Some("queue.remove"));

    let actions = gio::SimpleActionGroup::new();

    let act = gio::SimpleAction::new("move-to-front", None);
    let shared_c = shared.clone();
    act.connect_activate(move |_, _| {
        if index != 0 {
            send(&shared_c, Request::MoveQueue { from: index, to: 0 });
        }
    });
    actions.add_action(&act);

    let act = gio::SimpleAction::new("remove", None);
    let shared_c = shared.clone();
    act.connect_activate(move |_, _| {
        send(&shared_c, Request::RemoveSongAt(index));
    });
    actions.add_action(&act);

    popup_at(parent, &menu, &actions, "queue", x, y);
}
