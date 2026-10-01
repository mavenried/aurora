use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use aurora_protocol::{Request, Response};
use gtk::gdk;
use gtk::gdk_pixbuf::Pixbuf;
use gtk::glib;

use crate::daemon::DaemonEvent;
use crate::model::{NavTarget, SearchMode, State};
use crate::theme::Palette;
use crate::{ui, DEFAULT_ART};

/// Every piece of album art in the app (song rows, queue rows, the player
/// bar) renders at this one size. Textures are decoded pre-scaled to it
/// (see `art_texture`/`scaled_texture_from_bytes`) rather than relying on
/// GtkPicture's `size_request`, which only sets a *minimum* — a
/// large source image (or the bundled placeholder) would otherwise inflate
/// the widget's natural size and, via GTK's expand propagation, stretch
/// whatever row/panel contains it.
pub const ART_SIZE: i32 = 64;

/// Persistent widget handles. Most containers here have their *children*
/// cleared and rebuilt on relevant state changes; widgets that hold user
/// input focus or drag state (search entry, seek/volume scales) are built
/// once and only ever have their *values* updated, never recreated, so
/// typing and dragging are never interrupted mid-gesture.
pub struct Widgets {
    pub window: adw::ApplicationWindow,
    pub css_provider: gtk::CssProvider,

    pub content_page: adw::NavigationPage,
    pub content_stack: gtk::Stack,

    pub playlists_section: gtk::Box,

    pub search_entry: gtk::SearchEntry,
    pub search_results_box: gtk::Box,

    pub detail_title_lbl: gtk::Label,
    pub detail_list_box: gtk::Box,

    pub queue_list_box: gtk::Box,
    pub queue_scroller: gtk::ScrolledWindow,

    pub player_art: gtk::Picture,
    pub player_title_lbl: gtk::Label,
    pub player_artist_lbl: gtk::Label,

    pub seek_scale: gtk::Scale,
    pub volume_scale: gtk::Scale,
    pub elapsed_lbl: gtk::Label,
    pub duration_lbl: gtk::Label,
    pub play_pause_btn: gtk::Button,
    pub shuffle_btn: gtk::Button,
    pub repeat_btn: gtk::Button,
    pub like_btn: gtk::Button,

    pub disconnected_overlay: gtk::Box,
}

pub struct AppState {
    pub state: State,
    pub palette: Palette,
    pub req_tx: Option<async_channel::Sender<Request>>,
    pub connected: bool,

    pub nav: NavTarget,
    pub search_mode: SearchMode,

    /// Set while the user is dragging the seek slider (or has clicked to
    /// jump it) and not yet released it. `player::update` shows this value
    /// instead of the daemon's real playback position so incoming Status
    /// ticks don't yank the slider back mid-drag; the actual `Seek` request
    /// is only sent once the pointer is released (see `player::wire`).
    pub seek_override_ms: Option<u64>,

    pub sidebar_active_row: Option<gtk::Widget>,

    pub art_cache: HashMap<PathBuf, gdk::Texture>,
    pub default_art: gdk::Texture,

    pub widgets: Widgets,
}

pub type Shared = Rc<RefCell<AppState>>;

/// Scales a `Pixbuf` to a `size`x`size` square using "cover" semantics
/// (scale up to fully cover the square, then crop the center) so both the
/// aspect ratio is preserved and the result exactly matches `ART_SIZE` —
/// unlike GtkPicture's `content_fit: Cover`, which only affects painting,
/// not the widget's own size negotiation.
fn cover_pixbuf(pixbuf: &Pixbuf, size: i32) -> Pixbuf {
    let (w, h) = (pixbuf.width(), pixbuf.height());
    if w <= 0 || h <= 0 {
        return pixbuf.clone();
    }
    let scale = (size as f64 / w as f64).max(size as f64 / h as f64);
    let sw = ((w as f64 * scale).round() as i32).max(1);
    let sh = ((h as f64 * scale).round() as i32).max(1);
    let Some(scaled) = pixbuf.scale_simple(sw, sh, gtk::gdk_pixbuf::InterpType::Bilinear) else {
        return pixbuf.clone();
    };
    let x = ((sw - size).max(0)) / 2;
    let y = ((sh - size).max(0)) / 2;
    scaled.new_subpixbuf(x, y, size.min(sw), size.min(sh))
}

fn scaled_texture_from_file(path: &std::path::Path, size: i32) -> Option<gdk::Texture> {
    let pixbuf = Pixbuf::from_file(path).ok()?;
    Some(gdk::Texture::for_pixbuf(&cover_pixbuf(&pixbuf, size)))
}

fn scaled_texture_from_bytes(bytes: &'static [u8], size: i32) -> Option<gdk::Texture> {
    let stream = gtk::gio::MemoryInputStream::from_bytes(&glib::Bytes::from_static(bytes));
    let pixbuf = Pixbuf::from_stream(&stream, gtk::gio::Cancellable::NONE).ok()?;
    Some(gdk::Texture::for_pixbuf(&cover_pixbuf(&pixbuf, size)))
}

impl AppState {
    pub fn new(widgets: Widgets) -> Shared {
        let default_art = scaled_texture_from_bytes(DEFAULT_ART, ART_SIZE).expect("failed to decode bundled placeholder art");

        Rc::new(RefCell::new(AppState {
            state: State::default(),
            palette: Palette::default(),
            req_tx: None,
            connected: false,
            nav: NavTarget::default(),
            search_mode: SearchMode::ByTitle,
            seek_override_ms: None,
            sidebar_active_row: None,
            art_cache: HashMap::new(),
            default_art,
            widgets,
        }))
    }

    pub fn art_texture(&mut self, path: &Option<PathBuf>) -> gdk::Texture {
        let Some(path) = path else {
            return self.default_art.clone();
        };
        if let Some(tex) = self.art_cache.get(path) {
            return tex.clone();
        }
        let tex = scaled_texture_from_file(path, ART_SIZE).unwrap_or_else(|| self.default_art.clone());
        self.art_cache.insert(path.clone(), tex.clone());
        tex
    }
}

pub fn send(shared: &Shared, req: Request) {
    let tx = shared.borrow().req_tx.clone();
    let Some(tx) = tx else { return };
    let _ = tx.send_blocking(req);
}

pub fn send_many(shared: &Shared, reqs: Vec<Request>) {
    let tx = shared.borrow().req_tx.clone();
    let Some(tx) = tx else { return };
    for req in reqs {
        let _ = tx.send_blocking(req);
    }
}

pub fn handle_daemon_event(shared: &Shared, event: DaemonEvent) {
    match event {
        DaemonEvent::Connected => {
            shared.borrow_mut().connected = true;
            ui::update_disconnected_overlay(shared);
            send_many(shared, vec![Request::PlaylistList, Request::GetLikedSongs, Request::GetLastPlayed]);
        }
        DaemonEvent::Disconnected => {
            shared.borrow_mut().connected = false;
            ui::update_disconnected_overlay(shared);
        }
        DaemonEvent::Response(response) => handle_response(shared, response),
    }
}

fn handle_response(shared: &Shared, response: Response) {
    match response {
        Response::Status(status) => {
            let song_changed = {
                let mut s = shared.borrow_mut();
                let prev_id = s.state.current_song.as_ref().map(|song| song.id);
                let new_id = status.current_song.as_ref().map(|song| song.id);
                s.state.duration = status.current_song.as_ref().map(|s| s.duration).unwrap_or_default();
                s.state.current_song = status.current_song;
                s.state.is_paused = status.is_paused;
                s.state.position = status.position;
                s.state.volume = status.volume;
                s.state.shuffle = status.shuffle;
                s.state.repeat = status.repeat;
                prev_id != new_id
            };
            ui::player::update(shared);
            // The accent-bar highlighting the playing row is baked in at
            // build time, so as the track advances naturally (not via a
            // user action that already rebuilds these lists) the lists
            // need an explicit nudge — but only on an actual track change,
            // not every position tick, since Status arrives ~every second.
            if song_changed {
                ui::queue::rebuild(shared);
                ui::search::rebuild_results(shared);
                ui::detail::rebuild(shared);
            }
        }
        Response::Queue(queue) => {
            shared.borrow_mut().state.queue = queue;
            ui::queue::rebuild(shared);
        }
        Response::SearchResults(results) => {
            shared.borrow_mut().state.search_results = results;
            ui::search::rebuild_results(shared);
        }
        Response::PlaylistResults(result) => {
            shared.borrow_mut().state.playlist_result = Some(result);
            ui::detail::rebuild(shared);
        }
        Response::PlaylistList(list) => {
            shared.borrow_mut().state.playlist_list_results = list;
            ui::sidebar::rebuild_playlists(shared);
        }
        Response::Theme(theme) => {
            let palette: Palette = theme.into();
            shared.borrow_mut().palette = palette.clone();
            let css_provider = shared.borrow().widgets.css_provider.clone();
            css_provider.load_from_string(&crate::theme::css(&palette));
        }
        Response::Volume(v) => {
            shared.borrow_mut().state.volume = v;
            ui::player::update(shared);
        }
        Response::ArtistList(_) => {}
        Response::LastPlayed(songs) => {
            shared.borrow_mut().state.last_played = songs;
            ui::detail::rebuild(shared);
        }
        Response::LikedSongs(songs) => {
            {
                let mut s = shared.borrow_mut();
                s.state.liked_song_ids = songs.iter().map(|s| s.id).collect();
                s.state.liked_songs = songs;
            }
            ui::search::rebuild_results(shared);
            ui::detail::rebuild(shared);
            ui::queue::rebuild(shared);
            ui::player::update(shared);
        }
        Response::Error { err_id, err_msg } => {
            tracing::warn!("Daemon error {err_id}: {err_msg}");
        }
        Response::ArtReady { id, art_path } => {
            let touched = {
                let mut s = shared.borrow_mut();
                let mut touched = false;
                let state = &mut s.state;
                for list in [
                    &mut state.queue,
                    &mut state.search_results,
                    &mut state.last_played,
                    &mut state.liked_songs,
                ] {
                    for song in list.iter_mut() {
                        if song.id == id {
                            song.art_path = Some(art_path.clone());
                            touched = true;
                        }
                    }
                }
                if let Some(pl) = s.state.playlist_result.as_mut() {
                    for song in pl.songs.iter_mut() {
                        if song.id == id {
                            song.art_path = Some(art_path.clone());
                            touched = true;
                        }
                    }
                }
                touched
            };
            if touched {
                ui::queue::rebuild(shared);
                ui::search::rebuild_results(shared);
                ui::detail::rebuild(shared);
                ui::player::update(shared);
            }
        }
    }
}
