use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

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
    // Three columns, Spotify-style: art+title (left, spans the full bar
    // height) | transport+seek stacked (center) | volume/like (right).
    let root = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    root.add_css_class("panel-bg");
    // `now_playing`/`art` below set vexpand(true) so the art can stretch to
    // the bar's own height; without capping it here that would otherwise
    // propagate up and make the whole bar compete with the split_view for
    // vertical space in the window (ballooning to roughly half the screen).
    root.set_vexpand(false);

    // The picture is capped at a fixed size (never bigger than 100px) and
    // centered in its wrapper; the wrapper — a plain Box, whose margin/
    // alignment handling is more predictable than relying on the Picture's
    // own sizing interacting with its aspect-ratio/content-fit logic —
    // still stretches to the bar's full height (vexpand+valign(Fill)), so
    // any extra height beyond the art's fixed size becomes symmetric
    // top/bottom padding automatically instead of the art growing to fill it.
    const PLAYER_ART_SIZE: i32 = 84;
    let art = gtk::Picture::new();
    art.add_css_class("thumb");
    art.set_overflow(gtk::Overflow::Hidden);
    art.set_content_fit(gtk::ContentFit::Cover);
    art.set_size_request(PLAYER_ART_SIZE, PLAYER_ART_SIZE);
    art.set_hexpand(false);
    art.set_vexpand(false);
    art.set_halign(gtk::Align::Center);
    art.set_valign(gtk::Align::Center);

    let art_wrapper = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    art_wrapper.set_size_request(PLAYER_ART_SIZE, -1);
    art_wrapper.set_hexpand(false);
    art_wrapper.set_vexpand(true);
    art_wrapper.set_valign(gtk::Align::Fill);
    art_wrapper.append(&art);

    let title_lbl = gtk::Label::new(Some("Nothing Playing"));
    title_lbl.add_css_class("txt1");
    title_lbl.add_css_class("title-16");
    title_lbl.set_halign(gtk::Align::Start);
    title_lbl.set_hexpand(true);
    title_lbl.set_max_width_chars(16);
    title_lbl.set_ellipsize(gtk::pango::EllipsizeMode::End);
    let artist_lbl = gtk::Label::new(Some("No Artist"));
    artist_lbl.add_css_class("txt2");
    artist_lbl.add_css_class("subtle-14");
    artist_lbl.set_halign(gtk::Align::Start);
    artist_lbl.set_hexpand(true);
    artist_lbl.set_max_width_chars(16);
    artist_lbl.set_ellipsize(gtk::pango::EllipsizeMode::End);
    let text_col = gtk::Box::new(gtk::Orientation::Vertical, 2);
    text_col.set_size_request(116, -1);
    text_col.set_hexpand(false);
    text_col.set_valign(gtk::Align::Center);
    text_col.append(&title_lbl);
    text_col.append(&artist_lbl);

    let now_playing = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    now_playing.set_vexpand(true);
    now_playing.set_valign(gtk::Align::Fill);
    now_playing.set_margin_start(8);
    now_playing.set_size_request(220, -1);
    now_playing.append(&art_wrapper);
    now_playing.append(&text_col);

    let shuffle_btn = circle_btn("media-playlist-shuffle-symbolic", 30, 14);
    shuffle_btn.add_css_class("toggle-btn");
    let prev_btn = circle_btn("media-skip-backward-symbolic", 36, 16);
    let play_pause_btn = circle_btn("media-playback-start-symbolic", 46, 20);
    play_pause_btn.add_css_class("play-btn");
    let next_btn = circle_btn("media-skip-forward-symbolic", 36, 16);
    let repeat_btn = circle_btn("media-playlist-repeat-symbolic", 30, 14);
    repeat_btn.add_css_class("toggle-btn");
    let transport = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    transport.set_halign(gtk::Align::Center);
    transport.append(&shuffle_btn);
    transport.append(&prev_btn);
    transport.append(&play_pause_btn);
    transport.append(&next_btn);
    transport.append(&repeat_btn);

    // ---- seek cluster: fixed-width and centered, not stretched edge to edge ----
    let elapsed_lbl = gtk::Label::new(Some("0:00"));
    elapsed_lbl.add_css_class("txt2");
    elapsed_lbl.add_css_class("subtle-13");
    elapsed_lbl.set_size_request(36, -1);

    let seek_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 1.0);
    seek_scale.set_draw_value(false);
    seek_scale.set_hexpand(false);
    seek_scale.set_size_request(480, -1);

    let duration_lbl = gtk::Label::new(Some("0:00"));
    duration_lbl.add_css_class("txt2");
    duration_lbl.add_css_class("subtle-13");
    duration_lbl.set_size_request(36, -1);
    duration_lbl.set_halign(gtk::Align::End);

    let seek_cluster = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    seek_cluster.set_halign(gtk::Align::Center);
    seek_cluster.append(&elapsed_lbl);
    seek_cluster.append(&seek_scale);
    seek_cluster.append(&duration_lbl);

    let center_col = gtk::Box::new(gtk::Orientation::Vertical, 4);
    center_col.set_hexpand(true);
    center_col.set_vexpand(true);
    center_col.set_valign(gtk::Align::Center);
    center_col.set_margin_top(10);
    center_col.set_margin_bottom(10);
    center_col.append(&transport);
    center_col.append(&seek_cluster);

    let like_btn = circle_btn("emblem-favorite-symbolic", 34, 15);
    like_btn.add_css_class("toggle-btn");
    like_btn.add_css_class("like-btn");
    let volume_icon = icon("audio-volume-high-symbolic");
    volume_icon.add_css_class("txt2");
    let volume_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.01);
    volume_scale.set_draw_value(false);
    volume_scale.set_size_request(140, -1);
    volume_scale.set_value(1.0);

    // No fixed size_request here: it previously reserved more width than
    // the content needed, leaving dead space to the left of the icons
    // despite halign(End) anchoring the cluster to the right edge.
    let right_cluster = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    right_cluster.set_halign(gtk::Align::End);
    right_cluster.set_valign(gtk::Align::Center);
    right_cluster.set_margin_end(16);
    right_cluster.append(&like_btn);
    right_cluster.append(&volume_icon);
    right_cluster.append(&volume_scale);

    root.append(&now_playing);
    root.append(&center_col);
    root.append(&right_cluster);

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
        // GtkScale's internal slider-drag gesture claims the pointer
        // sequence exclusively, so an externally attached GestureClick
        // never sees a `released` signal on it — there's no reliable
        // "pointer came up" event to hook here. Instead: every change-value
        // (fired continuously while dragging, or once for a click-to-jump)
        // records the pending position for immediate visual feedback and
        // (re)schedules a short debounce timer that actually sends the
        // Seek. Each new change-value cancels the previous timer, so the
        // request only goes out once the value has stopped moving for
        // 250ms — a reliable proxy for "released" that doesn't fight GTK's
        // gesture ownership.
        let shared = shared.clone();
        let pending_commit: Rc<Cell<Option<glib::SourceId>>> = Rc::new(Cell::new(None));
        built.seek_scale.connect_change_value(move |_, _, value| {
            let ms = value.max(0.0) as u64;
            shared.borrow_mut().seek_override_ms = Some(ms);
            let elapsed_lbl = shared.borrow().widgets.elapsed_lbl.clone();
            elapsed_lbl.set_text(&format_duration(Duration::from_millis(ms)));

            if let Some(id) = pending_commit.take() {
                id.remove();
            }
            let shared = shared.clone();
            let pending_commit_inner = pending_commit.clone();
            let id = glib::timeout_add_local(Duration::from_millis(250), move || {
                // Extracted into its own statement, not the scrutinee of the
                // `if let` directly below: a temporary `RefMut` from
                // `borrow_mut()` used as an `if let` scrutinee stays alive
                // for the whole if-let body in Rust, which would still be
                // holding this borrow when `send()` below tries its own
                // `shared.borrow()` — panicking with "already mutably
                // borrowed".
                let pending_ms = shared.borrow_mut().seek_override_ms.take();
                if let Some(ms) = pending_ms {
                    send(&shared, Request::Seek(Duration::from_millis(ms)));
                }
                pending_commit_inner.set(None);
                glib::ControlFlow::Break
            });
            pending_commit.set(Some(id));

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
    let expanded_art = w.expanded_art.clone();
    let expanded_title_lbl = w.expanded_title_lbl.clone();
    let expanded_artist_lbl = w.expanded_artist_lbl.clone();
    let expanded_seek_scale = w.expanded_seek_scale.clone();
    let elapsed_lbl = w.elapsed_lbl.clone();
    let duration_lbl = w.duration_lbl.clone();
    let seek_scale = w.seek_scale.clone();
    let volume_scale = w.volume_scale.clone();
    let play_pause_btn = w.play_pause_btn.clone();
    let shuffle_btn = w.shuffle_btn.clone();
    let repeat_btn = w.repeat_btn.clone();
    let like_btn = w.like_btn.clone();

    // While the user is dragging (or has clicked to jump) the seek slider,
    // show that pending position instead of the daemon's real position, so
    // incoming Status ticks don't yank the slider back mid-drag.
    let seek_override_ms = s.seek_override_ms;
    let position_ms = seek_override_ms.map(|ms| ms as f64).unwrap_or(s.state.position.as_millis() as f64);
    let duration_ms = s.state.duration.as_millis().max(1) as f64;
    let volume = s.state.volume as f64;
    let is_paused = s.state.is_paused;
    let shuffle = s.state.shuffle;
    let repeat = s.state.repeat;
    let has_song = s.state.current_song.is_some();
    let liked = s.state.current_song.as_ref().is_some_and(|song| s.state.is_liked(&song.id));
    let art_tex = s.art_texture(&art_path);
    let expanded_art_path = s
        .state
        .current_song
        .as_ref()
        .and_then(|song| s.highres_art.get(&song.id).cloned())
        .or_else(|| art_path.clone());
    let expanded_art_tex = s.art_texture_sized(&expanded_art_path, 720);

    drop(s);

    art.set_paintable(Some(&art_tex));
    expanded_art.set_paintable(Some(&expanded_art_tex));
    title_lbl.set_text(&title);
    artist_lbl.set_text(&artist);
    expanded_title_lbl.set_text(&title);
    expanded_artist_lbl.set_text(&artist);

    elapsed_lbl.set_text(&format_duration(std::time::Duration::from_millis(position_ms as u64)));
    duration_lbl.set_text(&format_duration(std::time::Duration::from_millis(duration_ms as u64)));
    seek_scale.set_range(0.0, duration_ms);
    expanded_seek_scale.set_range(0.0, duration_ms);
    if seek_override_ms.is_none() {
        seek_scale.set_value(position_ms);
        expanded_seek_scale.set_value(position_ms);
    }
    seek_scale.set_sensitive(has_song);
    volume_scale.set_value(volume);

    play_pause_btn.set_child(Some(&icon_sized(
        if is_paused { "media-playback-start-symbolic" } else { "media-playback-pause-symbolic" },
        22,
    )));

    shuffle_btn.set_css_classes(&["circle-btn", "toggle-btn", if shuffle { "active" } else { "" }]);
    repeat_btn.set_css_classes(&["circle-btn", "toggle-btn", if repeat == 1 { "active" } else { "" }]);
    like_btn.set_css_classes(&["circle-btn", "toggle-btn", "like-btn", if liked { "active" } else { "" }]);
}

pub fn replace_expanded_art(shared: &Shared, id: uuid::Uuid, path: &std::path::Path) {
    let mut s = shared.borrow_mut();
    let is_current = s.state.current_song.as_ref().is_some_and(|song| song.id == id);
    if !is_current {
        return;
    }

    let texture = s.art_texture_sized(&Some(path.to_path_buf()), 720);
    s.widgets.expanded_art.set_paintable(Some(&texture));
}
