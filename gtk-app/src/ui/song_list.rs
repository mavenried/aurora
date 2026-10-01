use aurora_protocol::Song;
use gtk::prelude::*;
use uuid::Uuid;

use crate::app::{send, Shared};
use crate::model::format_duration;
use crate::ui::{clear, context_menu, icon};

pub fn song_row(shared: &Shared, song: &Song, current_playlist: Option<Uuid>) -> gtk::Widget {
    let (selected, liked, is_current, art) = {
        let mut s = shared.borrow_mut();
        let selected = s.state.is_selected(&song.id);
        let liked = s.state.is_liked(&song.id);
        let is_current = s.state.current_song.as_ref().map(|c| c.id) == Some(song.id);
        let art = s.art_texture(&song.art_path);
        (selected, liked, is_current, art)
    };

    let accent_bar = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    accent_bar.add_css_class("accent-bar");
    if selected {
        accent_bar.add_css_class("selected");
    } else if is_current {
        accent_bar.add_css_class("playing");
    }
    accent_bar.set_size_request(4, 50);

    let picture = gtk::Picture::for_paintable(&art);
    picture.add_css_class("thumb");
    picture.set_overflow(gtk::Overflow::Hidden);
    picture.set_content_fit(gtk::ContentFit::Cover);
    picture.set_size_request(crate::app::ART_SIZE, crate::app::ART_SIZE);
    picture.set_hexpand(false);
    picture.set_vexpand(false);
    picture.set_halign(gtk::Align::Center);
    picture.set_valign(gtk::Align::Center);

    let title_col = gtk::Box::new(gtk::Orientation::Vertical, 3);
    title_col.set_valign(gtk::Align::Center);
    title_col.set_hexpand(true);
    let title_lbl = gtk::Label::new(Some(&song.title));
    title_lbl.add_css_class("txt1");
    title_lbl.add_css_class("title-15");
    title_lbl.set_halign(gtk::Align::Start);
    title_lbl.set_ellipsize(gtk::pango::EllipsizeMode::End);
    let artists_lbl = gtk::Label::new(Some(&song.artists.join(", ")));
    artists_lbl.add_css_class("txt2");
    artists_lbl.add_css_class("subtle-13");
    artists_lbl.set_halign(gtk::Align::Start);
    artists_lbl.set_ellipsize(gtk::pango::EllipsizeMode::End);
    title_col.append(&title_lbl);
    title_col.append(&artists_lbl);

    let heart_btn = gtk::Button::new();
    heart_btn.set_has_frame(false);
    heart_btn.add_css_class("circle-btn");
    heart_btn.add_css_class("toggle-btn");
    heart_btn.add_css_class("like-btn");
    heart_btn.set_size_request(32, 32);
    if liked {
        heart_btn.add_css_class("active");
    }
    heart_btn.set_child(Some(&icon("emblem-favorite-symbolic")));
    {
        let shared = shared.clone();
        let song_id = song.id;
        heart_btn.connect_clicked(move |_| {
            let ids = shared.borrow().state.expand_selection(song_id);
            let reqs = ids
                .into_iter()
                .map(|id| {
                    if liked {
                        aurora_protocol::Request::UnlikeSong(id)
                    } else {
                        aurora_protocol::Request::LikeSong(id)
                    }
                })
                .collect();
            crate::app::send_many(&shared, reqs);
        });
    }

    let duration_lbl = gtk::Label::new(Some(&format_duration(song.duration)));
    duration_lbl.add_css_class("txt2");
    duration_lbl.add_css_class("subtle-13");

    let content = gtk::Box::new(gtk::Orientation::Horizontal, 14);
    content.set_margin_top(8);
    content.set_margin_bottom(8);
    content.set_margin_start(12);
    content.set_margin_end(14);
    content.append(&accent_bar);
    content.append(&picture);
    content.append(&title_col);
    content.append(&heart_btn);
    content.append(&duration_lbl);

    let row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    row.add_css_class("song-row");
    row.set_margin_start(6);
    row.set_margin_end(6);
    row.set_hexpand(false);
    row.set_vexpand(false);
    row.set_valign(gtk::Align::Start);
    if selected {
        row.add_css_class("selected");
    }
    row.append(&content);

    let click = gtk::GestureClick::new();
    click.set_button(1);
    {
        let shared = shared.clone();
        let song_id = song.id;
        click.connect_pressed(move |gesture, _, _, _| {
            let ctrl = gesture
                .current_event()
                .map(|e| e.modifier_state().contains(gtk::gdk::ModifierType::CONTROL_MASK))
                .unwrap_or(false);
            if ctrl {
                let mut s = shared.borrow_mut();
                if !s.state.selected_song_ids.remove(&song_id) {
                    s.state.selected_song_ids.insert(song_id);
                }
                drop(s);
                super::search::rebuild_results(&shared);
                super::detail::rebuild(&shared);
            } else {
                send(&shared, aurora_protocol::Request::Play(song_id));
            }
        });
    }
    row.add_controller(click);

    let right_click = gtk::GestureClick::new();
    right_click.set_button(3);
    {
        let shared = shared.clone();
        let song_id = song.id;
        let row_weak = row.downgrade();
        right_click.connect_pressed(move |_, _, x, y| {
            if let Some(row) = row_weak.upgrade() {
                context_menu::show_song_menu(&shared, &row, song_id, liked, current_playlist, x, y);
            }
        });
    }
    row.add_controller(right_click);

    row.upcast()
}

pub fn populate_song_list(shared: &Shared, container: &gtk::Box, songs: &[Song], current_playlist: Option<Uuid>) {
    clear(container);
    container.set_valign(gtk::Align::Start);
    if songs.is_empty() {
        container.append(&empty_state("audio-x-generic-symbolic", "No songs here", ""));
        return;
    }
    for song in songs {
        container.append(&song_row(shared, song, current_playlist));
    }
}

pub fn empty_state(icon_name: &str, title: &str, subtitle: &str) -> gtk::Widget {
    let b = gtk::Box::new(gtk::Orientation::Vertical, 10);
    b.set_halign(gtk::Align::Center);
    b.set_valign(gtk::Align::Center);
    b.set_vexpand(true);
    b.set_hexpand(true);
    let img = icon(icon_name);
    img.set_pixel_size(48);
    img.add_css_class("txt2");
    let title_lbl = gtk::Label::new(Some(title));
    title_lbl.add_css_class("txt2");
    title_lbl.add_css_class("title-18");
    b.append(&img);
    b.append(&title_lbl);
    if !subtitle.is_empty() {
        let subtitle_lbl = gtk::Label::new(Some(subtitle));
        subtitle_lbl.add_css_class("txt2");
        subtitle_lbl.add_css_class("subtle-13");
        b.append(&subtitle_lbl);
    }
    b.upcast()
}
