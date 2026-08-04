use aurora_protocol::Request;
use gtk::glib;
use gtk::prelude::*;

use crate::app::{send, Shared};
use crate::model::format_duration;
use crate::ui::{icon, icon_sized};

pub struct Built {
    pub root: gtk::Widget,
    pub art: gtk::Picture,
    pub title_lbl: gtk::Label,
    pub artist_lbl: gtk::Label,
    pub seek_scale: gtk::Scale,
    pub volume_scale: gtk::Scale,
    pub elapsed_lbl: gtk::Label,
    pub duration_lbl: gtk::Label,
    pub prev_btn: gtk::Button,
    pub play_pause_btn: gtk::Button,
    pub next_btn: gtk::Button,
    pub shuffle_btn: gtk::Button,
    pub repeat_btn: gtk::Button,
    pub like_btn: gtk::Button,
}

fn circle_btn(icon_name: &str, diameter: i32, icon_px: i32) -> gtk::Button {
    let btn = gtk::Button::new();
    btn.set_child(Some(&icon_sized(icon_name, icon_px)));
    btn.add_css_class("circle-btn");
    btn.set_size_request(diameter, diameter);
    btn
}

pub fn build() -> Built {
    let root = gtk::Box::new(gtk::Orientation::Vertical, 3);
    root.add_css_class("panel-bg");

    // ---- one unified card: art+title/artist (left), transport (centered),
    // volume/like (right) — all on the same row, with the seek bar below. ----
    let art = gtk::Picture::new();
    art.add_css_class("thumb");
    art.set_overflow(gtk::Overflow::Hidden);
    art.set_content_fit(gtk::ContentFit::Cover);
    art.set_size_request(crate::app::ART_SIZE, crate::app::ART_SIZE);
    art.set_hexpand(false);
    art.set_vexpand(false);
    art.set_halign(gtk::Align::Center);
    art.set_valign(gtk::Align::Center);

    let title_lbl = gtk::Label::new(Some("Nothing Playing"));
    title_lbl.add_css_class("txt1");
    title_lbl.add_css_class("title-16");
    title_lbl.set_halign(gtk::Align::Start);
    title_lbl.set_ellipsize(gtk::pango::EllipsizeMode::End);
    let artist_lbl = gtk::Label::new(Some("No Artist"));
    artist_lbl.add_css_class("txt2");
    artist_lbl.add_css_class("subtle-14");
    artist_lbl.set_halign(gtk::Align::Start);
    artist_lbl.set_ellipsize(gtk::pango::EllipsizeMode::End);
    let text_col = gtk::Box::new(gtk::Orientation::Vertical, 2);
    text_col.set_valign(gtk::Align::Center);
    text_col.append(&title_lbl);
    text_col.append(&artist_lbl);

    let now_playing = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    now_playing.set_valign(gtk::Align::Center);
    now_playing.set_margin_start(16);
    now_playing.set_size_request(220, -1);
    now_playing.append(&art);
    now_playing.append(&text_col);

    let shuffle_btn = circle_btn("media-playlist-shuffle-symbolic", 30, 14);
    let prev_btn = circle_btn("media-skip-backward-symbolic", 36, 16);
    let play_pause_btn = circle_btn("media-playback-start-symbolic", 46, 20);
    play_pause_btn.add_css_class("play-btn");
    let next_btn = circle_btn("media-skip-forward-symbolic", 36, 16);
    let repeat_btn = circle_btn("media-playlist-repeat-symbolic", 30, 14);
    let transport = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    transport.set_halign(gtk::Align::Center);
    transport.set_valign(gtk::Align::Center);
    transport.set_hexpand(true);
    transport.append(&shuffle_btn);
    transport.append(&prev_btn);
    transport.append(&play_pause_btn);
    transport.append(&next_btn);
    transport.append(&repeat_btn);

    let like_btn = circle_btn("emblem-favorite-symbolic", 34, 15);
    let volume_icon = icon("audio-volume-high-symbolic");
    volume_icon.add_css_class("txt2");
    let volume_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.01);
    volume_scale.set_draw_value(false);
    volume_scale.set_size_request(70, -1);
    volume_scale.set_value(1.0);

    let right_cluster = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    right_cluster.set_halign(gtk::Align::End);
    right_cluster.set_valign(gtk::Align::Center);
    right_cluster.set_margin_end(16);
    right_cluster.set_size_request(220, -1);
    right_cluster.append(&like_btn);
    right_cluster.append(&volume_icon);
    right_cluster.append(&volume_scale);

    let controls_row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    controls_row.set_margin_top(6);
    controls_row.append(&now_playing);
    controls_row.append(&transport);
    controls_row.append(&right_cluster);

    // ---- seek row ----
    let elapsed_lbl = gtk::Label::new(Some("0:00"));
    elapsed_lbl.add_css_class("txt2");
    elapsed_lbl.add_css_class("subtle-13");
    elapsed_lbl.set_size_request(40, -1);

    let seek_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 1.0);
    seek_scale.set_draw_value(false);
    seek_scale.set_hexpand(true);

    let duration_lbl = gtk::Label::new(Some("0:00"));
    duration_lbl.add_css_class("txt2");
    duration_lbl.add_css_class("subtle-13");
    duration_lbl.set_size_request(40, -1);
    duration_lbl.set_halign(gtk::Align::End);

    let seek_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    seek_row.set_margin_start(16);
    seek_row.set_margin_end(16);
    seek_row.set_margin_bottom(5);
    seek_row.append(&elapsed_lbl);
    seek_row.append(&seek_scale);
    seek_row.append(&duration_lbl);

    root.append(&controls_row);
    root.append(&seek_row);

    Built {
        root: root.upcast(),
        art,
        title_lbl,
        artist_lbl,
        seek_scale,
        volume_scale,
        elapsed_lbl,
        duration_lbl,
        prev_btn,
        play_pause_btn,
        next_btn,
        shuffle_btn,
        repeat_btn,
        like_btn,
    }
}

pub fn wire(shared: &Shared, built: &Built) {
    {
        let shared = shared.clone();
        built.play_pause_btn.connect_clicked(move |_| send(&shared, Request::Pause));
    }
    {
        let shared = shared.clone();
        built.prev_btn.connect_clicked(move |_| send(&shared, Request::Prev(1)));
    }
    {
        let shared = shared.clone();
        built.next_btn.connect_clicked(move |_| send(&shared, Request::Next(1)));
    }
    {
        let shared = shared.clone();
        built.shuffle_btn.connect_clicked(move |_| {
            let next = !shared.borrow().state.shuffle;
            send(&shared, Request::SetShuffle(next));
        });
    }
    {
        let shared = shared.clone();
        built.repeat_btn.connect_clicked(move |_| {
            let next = if shared.borrow().state.repeat == 0 { 1 } else { 0 };
            send(&shared, Request::SetRepeat(next));
        });
    }
    {
        let shared = shared.clone();
        built.like_btn.connect_clicked(move |_| {
            let Some(id) = shared.borrow().state.current_song.as_ref().map(|s| s.id) else { return };
            let liked = shared.borrow().state.is_liked(&id);
            let ids = shared.borrow().state.expand_selection(id);
            let reqs = ids
                .into_iter()
                .map(|id| if liked { Request::UnlikeSong(id) } else { Request::LikeSong(id) })
                .collect();
            crate::app::send_many(&shared, reqs);
        });
    }
    {
        let shared = shared.clone();
        built.seek_scale.connect_change_value(move |scale, _, value| {
            let ms = value.max(0.0) as u64;
            send(&shared, Request::Seek(std::time::Duration::from_millis(ms)));
            let _ = scale;
            glib::Propagation::Proceed
        });
    }
    {
        let shared = shared.clone();
        built.volume_scale.connect_change_value(move |_, _, value| {
            send(&shared, Request::SetVolume(value.clamp(0.0, 1.0) as f32));
            glib::Propagation::Proceed
        });
    }
}

/// Pushes current state into the persistent widgets without recreating them,
/// so an in-progress drag on the seek/volume sliders is never interrupted.
pub fn update(shared: &Shared) {
    let mut s = shared.borrow_mut();
    let w = &s.widgets;

    let (title, artist, art_path) = match &s.state.current_song {
        Some(song) => (song.title.clone(), song.artists.join(", "), song.art_path.clone()),
        None => ("Nothing Playing".to_string(), "No Artist".to_string(), None),
    };

    let art = w.player_art.clone();
    let title_lbl = w.player_title_lbl.clone();
    let artist_lbl = w.player_artist_lbl.clone();
    let elapsed_lbl = w.elapsed_lbl.clone();
    let duration_lbl = w.duration_lbl.clone();
    let seek_scale = w.seek_scale.clone();
    let volume_scale = w.volume_scale.clone();
    let play_pause_btn = w.play_pause_btn.clone();
    let shuffle_btn = w.shuffle_btn.clone();
    let repeat_btn = w.repeat_btn.clone();
    let like_btn = w.like_btn.clone();

    let position_ms = s.state.position.as_millis() as f64;
    let duration_ms = s.state.duration.as_millis().max(1) as f64;
    let volume = s.state.volume as f64;
    let is_paused = s.state.is_paused;
    let shuffle = s.state.shuffle;
    let repeat = s.state.repeat;
    let has_song = s.state.current_song.is_some();
    let liked = s.state.current_song.as_ref().is_some_and(|song| s.state.is_liked(&song.id));
    let art_tex = s.art_texture(&art_path);

    drop(s);

    art.set_paintable(Some(&art_tex));
    title_lbl.set_text(&title);
    artist_lbl.set_text(&artist);

    elapsed_lbl.set_text(&format_duration(std::time::Duration::from_millis(position_ms as u64)));
    duration_lbl.set_text(&format_duration(std::time::Duration::from_millis(duration_ms as u64)));
    seek_scale.set_range(0.0, duration_ms);
    seek_scale.set_value(position_ms);
    seek_scale.set_sensitive(has_song);
    volume_scale.set_value(volume);

    play_pause_btn.set_child(Some(&icon_sized(
        if is_paused { "media-playback-start-symbolic" } else { "media-playback-pause-symbolic" },
        22,
    )));

    shuffle_btn.set_css_classes(&["circle-btn", if shuffle { "active" } else { "" }]);
    repeat_btn.set_css_classes(&["circle-btn", if repeat == 1 { "active" } else { "" }]);
    like_btn.set_css_classes(&["circle-btn", if liked { "active" } else { "" }]);
}
