use aurora_protocol::Request;
use gtk::gdk;
use gtk::prelude::*;
use gtk::glib;
use uuid::Uuid;

use crate::app::{refresh_playlists, send, send_many, Shared};

fn menu_content() -> gtk::Box {
    let content = gtk::Box::new(gtk::Orientation::Vertical, 2);
    content.set_width_request(224);
    content.set_margin_top(4);
    content.set_margin_bottom(4);
    content.set_margin_start(4);
    content.set_margin_end(4);
    content
}

fn action_button(
    label: &str,
    close_popovers: &[gtk::Popover],
    action: impl Fn() + 'static,
) -> gtk::Button {
    let button = gtk::Button::new();
    let label_widget = gtk::Label::new(Some(label));
    label_widget.set_halign(gtk::Align::Start);
    label_widget.set_hexpand(true);
    label_widget.set_xalign(0.0);
    button.set_child(Some(&label_widget));
    button.set_has_frame(false);
    button.set_halign(gtk::Align::Fill);
    button.set_hexpand(true);
    let popovers: Vec<_> = close_popovers.iter().map(gtk::Popover::downgrade).collect();
    button.connect_clicked(move |_| {
        action();
        for popover in &popovers {
            if let Some(popover) = popover.upgrade() {
                popover.popdown();
            }
        }
    });
    button
}

fn popup_at(parent: &impl IsA<gtk::Widget>, content: &gtk::Box, popover: &gtk::Popover, x: f64, y: f64) {
    popover.add_css_class("context-menu");
    popover.set_child(Some(content));
    popover.set_parent(parent);
    popover.set_has_arrow(false);
    popover.set_halign(gtk::Align::Start);
    popover.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
    popover.connect_closed(|popover| popover.unparent());
    popover.popup();
}

pub fn show_song_menu(
    shared: &Shared,
    parent: &impl IsA<gtk::Widget>,
    song_id: Uuid,
    liked: bool,
    current_playlist: Option<Uuid>,
    x: f64,
    y: f64,
) {
    let menu = menu_content();
    let popover = gtk::Popover::new();

    let shared_c = shared.clone();
    menu.append(&action_button("Play Next", &[popover.clone()], move || {
        let ids = shared_c.borrow().state.expand_selection(song_id);
        send_many(&shared_c, ids.into_iter().map(Request::PlayNext).collect());
    }));

    let shared_c = shared.clone();
    menu.append(&action_button("Add to Queue", &[popover.clone()], move || {
        let ids = shared_c.borrow().state.expand_selection(song_id);
        send_many(&shared_c, ids.into_iter().map(Request::Enqueue).collect());
    }));

    let shared_c = shared.clone();
    let like_label = if liked { "Remove from Liked Songs" } else { "Add to Liked Songs" };
    menu.append(&action_button(like_label, &[popover.clone()], move || {
        let ids = shared_c.borrow().state.expand_selection(song_id);
        let requests = ids
            .into_iter()
            .map(|id| if liked { Request::UnlikeSong(id) } else { Request::LikeSong(id) })
            .collect();
        send_many(&shared_c, requests);
    }));

    let playlists = shared.borrow().state.playlist_list_results.clone();
    if !playlists.is_empty() {
        let submenu = gtk::Popover::new();
        submenu.add_css_class("context-menu");
        let submenu_content = menu_content();
        for playlist in playlists {
            let shared_c = shared.clone();
            let id = playlist.id;
            submenu_content.append(&action_button(
                &playlist.name,
                &[submenu.clone(), popover.clone()],
                move || {
                    let ids = shared_c.borrow().state.expand_selection(song_id);
                    send(&shared_c, Request::PlaylistAddSongs { playlist_id: id, song_ids: ids });
                    refresh_playlists(&shared_c, Some(id));
                },
            ));
        }
        submenu.set_child(Some(&submenu_content));

        let add_to_playlist = gtk::MenuButton::new();
        add_to_playlist.set_label("Add to Playlist");
        add_to_playlist.set_halign(gtk::Align::Fill);
        add_to_playlist.set_hexpand(true);
        if let Some(label) = add_to_playlist.child().and_downcast::<gtk::Label>() {
            label.set_halign(gtk::Align::Start);
            label.set_hexpand(true);
            label.set_xalign(0.0);
        }
        add_to_playlist.set_popover(Some(&submenu));
        submenu.set_position(gtk::PositionType::Right);
        menu.append(&add_to_playlist);
    }

    if let Some(playlist_id) = current_playlist {
        let shared_c = shared.clone();
        menu.append(&action_button("Remove from Playlist", &[popover.clone()], move || {
            let ids = shared_c.borrow().state.expand_selection(song_id);
            send_many(
                &shared_c,
                ids.into_iter()
                    .map(|song_id| Request::PlaylistRemoveSong { playlist_id, song_id })
                    .collect(),
            );
            refresh_playlists(&shared_c, Some(playlist_id));
        }));
    }

    popup_at(parent, &menu, &popover, x, y);
}

pub fn show_playlist_menu(
    shared: &Shared,
    parent: &impl IsA<gtk::Widget>,
    playlist_id: Uuid,
    title: &str,
    x: f64,
    y: f64,
) {
    let menu = menu_content();
    let popover = gtk::Popover::new();

    let shared_c = shared.clone();
    let parent_owned = parent.clone().upcast::<gtk::Widget>();
    let title = title.to_string();
    menu.append(&action_button("Rename", &[popover.clone()], move || {
        let shared_c = shared_c.clone();
        let parent_owned = parent_owned.clone();
        let title = title.clone();
        glib::idle_add_local_once(move || {
            super::sidebar::prompt(&shared_c, &parent_owned, "Rename Playlist", &title, move |shared, new_title| {
                if !new_title.is_empty() {
                    send(shared, Request::PlaylistRename { playlist_id, new_title });
                    refresh_playlists(shared, Some(playlist_id));
                }
            });
        });
    }));

    let shared_c = shared.clone();
    menu.append(&action_button("Delete Playlist", &[popover.clone()], move || {
        send(&shared_c, Request::PlaylistDelete(playlist_id));
        refresh_playlists(&shared_c, None);
    }));

    popup_at(parent, &menu, &popover, x, y);
}

pub fn show_queue_menu(shared: &Shared, parent: &impl IsA<gtk::Widget>, index: usize, x: f64, y: f64) {
    let menu = menu_content();
    let popover = gtk::Popover::new();

    let shared_c = shared.clone();
    menu.append(&action_button("Move to Front", &[popover.clone()], move || {
        if index != 0 {
            send(&shared_c, Request::MoveQueue { from: index, to: 0 });
        }
    }));

    let shared_c = shared.clone();
    menu.append(&action_button("Remove", &[popover.clone()], move || {
        send(&shared_c, Request::RemoveSongAt(index));
    }));

    popup_at(parent, &menu, &popover, x, y);
}
