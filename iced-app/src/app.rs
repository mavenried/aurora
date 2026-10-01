use std::time::Duration;

use aurora_protocol::{PlaylistIn, Request, Response, SearchType};
use iced::keyboard::{self, Modifiers, key};
use iced::{Element, Event, Point, Size, Subscription, Task, Theme as IcedTheme};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::daemon::{self, DaemonEvent};
use crate::model::{ContextMenuKind, ContextMenuState, LibraryPanel, QueueDrag, SearchMode, State, Tab};
use crate::theme::Palette;
use crate::ui;

#[derive(Debug, Clone)]
pub enum Message {
    Daemon(DaemonEvent),
    Event(Event),

    TabSelected(Tab),
    OpenLibraryOverview,
    OpenLiked,
    OpenPlaylist(Uuid),

    SearchQueryChanged(String),
    SearchModeChanged(SearchMode),
    SearchSubmit,

    SongClicked(Uuid),
    SongRightClicked(Uuid, bool, Option<Uuid>),
    PlaylistRightClicked(Uuid, String),
    QueueRightClicked(usize),

    Pause,
    Next,
    Prev,
    Enqueue(Uuid),
    PlayNext(Uuid),
    LikeToggle(Uuid, bool),

    SeekChanged(u64),
    SeekReleased(u64),
    VolumeChanged(f32),
    VolumeReleased(f32),
    ShuffleToggled,
    RepeatToggled,

    QueueClicked(usize),
    QueueRemove(usize),
    QueueClear,
    QueueDragStart(usize),
    QueueMoveToFront(usize),

    CreatePlaylistToggle,
    NewPlaylistNameChanged(String),
    CreatePlaylistSubmit,
    RenamePlaylistStart(Uuid, String),
    RenamePlaylistNameChanged(String),
    RenamePlaylistSubmit,
    RenamePlaylistCancel,
    DeletePlaylist(Uuid),
    AddSongToPlaylist(Uuid, Uuid),
    RemoveSongFromPlaylist(Uuid, Uuid),
    ReplaceQueue(Vec<Uuid>),

    ToggleAddToPlaylistSubmenu,
    CloseContextMenu,

    SongHovered(Option<Uuid>),
    PlaylistHovered(Option<Uuid>),
    QueueHovered(Option<usize>),
}

pub struct AuroraApp {
    pub state: State,
    pub palette: Palette,
    pub daemon_tx: Option<mpsc::Sender<Request>>,
    pub connected: bool,

    pub tab: Tab,
    pub library_panel: LibraryPanel,

    pub search_query: String,
    pub search_mode: SearchMode,

    pub context_menu: Option<ContextMenuState>,
    pub cursor_pos: Point,
    pub window_size: Size,
    pub modifiers: Modifiers,

    pub queue_drag: Option<QueueDrag>,

    pub dragging_seek: Option<Duration>,
    pub dragging_volume: Option<f32>,

    pub creating_playlist: bool,
    pub new_playlist_name: String,
    pub renaming_playlist: Option<Uuid>,
    pub rename_playlist_name: String,

    pub hovered_song: Option<Uuid>,
    pub hovered_playlist: Option<Uuid>,
    pub hovered_queue: Option<usize>,
}

impl Default for AuroraApp {
    fn default() -> Self {
        Self {
            state: State::default(),
            palette: Palette::default(),
            daemon_tx: None,
            connected: false,
            tab: Tab::default(),
            library_panel: LibraryPanel::default(),
            search_query: String::new(),
            search_mode: SearchMode::ByTitle,
            context_menu: None,
            cursor_pos: Point::ORIGIN,
            window_size: Size::new(1422.0, 800.0),
            modifiers: Modifiers::default(),
            queue_drag: None,
            dragging_seek: None,
            dragging_volume: None,
            creating_playlist: false,
            new_playlist_name: String::new(),
            renaming_playlist: None,
            rename_playlist_name: String::new(),
            hovered_song: None,
            hovered_playlist: None,
            hovered_queue: None,
        }
    }
}

impl AuroraApp {
    pub fn boot() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    pub fn title(&self) -> String {
        "Aurora Player".to_string()
    }

    pub fn theme(&self) -> IcedTheme {
        IcedTheme::Dark
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch(vec![
            daemon::connection().map(Message::Daemon),
            iced::event::listen().map(Message::Event),
        ])
    }

    fn send(&self, req: Request) -> Task<Message> {
        let Some(tx) = self.daemon_tx.clone() else {
            return Task::none();
        };
        Task::future(async move {
            let _ = tx.send(req).await;
        })
        .discard()
    }

    fn send_many(&self, reqs: Vec<Request>) -> Task<Message> {
        let Some(tx) = self.daemon_tx.clone() else {
            return Task::none();
        };
        Task::future(async move {
            for req in reqs {
                let _ = tx.send(req).await;
            }
        })
        .discard()
    }

    fn clear_selection_task(&mut self) {
        self.state.selected_song_ids.clear();
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Daemon(event) => return self.handle_daemon_event(event),
            Message::Event(event) => return self.handle_event(event),

            Message::TabSelected(tab) => {
                self.tab = tab;
                if tab == Tab::Library {
                    return self.send_many(vec![Request::GetLikedSongs, Request::PlaylistList]);
                }
            }
            Message::OpenLibraryOverview => {
                self.library_panel = LibraryPanel::Overview;
            }
            Message::OpenLiked => {
                self.library_panel = LibraryPanel::Liked;
                return self.send(Request::GetLikedSongs);
            }
            Message::OpenPlaylist(id) => {
                self.library_panel = LibraryPanel::Playlist;
                return self.send(Request::PlaylistGet(id));
            }

            Message::SearchQueryChanged(q) => {
                self.search_query = q.clone();
                return self.maybe_search(false);
            }
            Message::SearchModeChanged(mode) => {
                self.search_mode = mode;
                return self.maybe_search(false);
            }
            Message::SearchSubmit => {
                return self.maybe_search(true);
            }

            Message::SongClicked(id) => {
                if self.modifiers.control() {
                    self.toggle_song_selection(id);
                } else {
                    return self.send(Request::Play(id));
                }
            }
            Message::SongRightClicked(id, liked, current_playlist) => {
                self.context_menu = Some(ContextMenuState {
                    kind: ContextMenuKind::Song {
                        song_id: id,
                        liked,
                        current_playlist,
                        submenu_open: false,
                    },
                    position: self.clamp_menu_pos(200.0, 260.0),
                });
            }
            Message::PlaylistRightClicked(id, title) => {
                self.context_menu = Some(ContextMenuState {
                    kind: ContextMenuKind::Playlist {
                        playlist_id: id,
                        title,
                    },
                    position: self.clamp_menu_pos(200.0, 90.0),
                });
            }
            Message::QueueRightClicked(index) => {
                self.context_menu = Some(ContextMenuState {
                    kind: ContextMenuKind::Queue { index },
                    position: self.clamp_menu_pos(160.0, 90.0),
                });
            }

            Message::Pause => return self.send(Request::Pause),
            Message::Next => return self.send(Request::Next(1)),
            Message::Prev => return self.send(Request::Prev(1)),
            Message::Enqueue(id) => {
                let ids = self.state.expand_selection(id);
                return self.send_many(ids.into_iter().map(Request::Enqueue).collect());
            }
            Message::PlayNext(id) => {
                let ids = self.state.expand_selection(id);
                return self.send_many(ids.into_iter().map(Request::PlayNext).collect());
            }
            Message::LikeToggle(id, currently_liked) => {
                let ids = self.state.expand_selection(id);
                let reqs = ids
                    .into_iter()
                    .map(|id| {
                        if currently_liked {
                            Request::UnlikeSong(id)
                        } else {
                            Request::LikeSong(id)
                        }
                    })
                    .collect();
                return self.send_many(reqs);
            }

            Message::SeekChanged(ms) => {
                self.dragging_seek = Some(Duration::from_millis(ms));
            }
            Message::SeekReleased(ms) => {
                self.dragging_seek = None;
                return self.send(Request::Seek(Duration::from_millis(ms)));
            }
            Message::VolumeChanged(v) => {
                self.dragging_volume = Some(v);
            }
            Message::VolumeReleased(v) => {
                self.dragging_volume = None;
                return self.send(Request::SetVolume(v));
            }
            Message::ShuffleToggled => {
                return self.send(Request::SetShuffle(!self.state.shuffle));
            }
            Message::RepeatToggled => {
                let next = if self.state.repeat == 0 { 1 } else { 0 };
                return self.send(Request::SetRepeat(next));
            }

            Message::QueueClicked(n) => return self.send(Request::Next(n + 1)),
            Message::QueueRemove(n) => return self.send(Request::RemoveSongAt(n)),
            Message::QueueClear => return self.send(Request::Clear),
            Message::QueueDragStart(index) => {
                self.queue_drag = Some(QueueDrag {
                    from: index,
                    start_y: self.cursor_pos.y,
                    drop_target: index,
                });
            }
            Message::QueueMoveToFront(index) => {
                self.context_menu = None;
                if index != 0 {
                    return self.send(Request::MoveQueue { from: index, to: 0 });
                }
            }

            Message::CreatePlaylistToggle => {
                self.creating_playlist = !self.creating_playlist;
                self.renaming_playlist = None;
                self.new_playlist_name.clear();
            }
            Message::NewPlaylistNameChanged(s) => self.new_playlist_name = s,
            Message::CreatePlaylistSubmit => {
                let name = self.new_playlist_name.trim().to_string();
                self.creating_playlist = false;
                self.new_playlist_name.clear();
                if !name.is_empty() {
                    return self.send(Request::PlaylistCreate(PlaylistIn {
                        title: name,
                        songs: vec![],
                    }));
                }
            }
            Message::RenamePlaylistStart(id, current_title) => {
                self.context_menu = None;
                self.creating_playlist = false;
                self.renaming_playlist = Some(id);
                self.rename_playlist_name = current_title;
            }
            Message::RenamePlaylistNameChanged(s) => self.rename_playlist_name = s,
            Message::RenamePlaylistSubmit => {
                if let Some(id) = self.renaming_playlist.take() {
                    let title = self.rename_playlist_name.trim().to_string();
                    self.rename_playlist_name.clear();
                    if !title.is_empty() {
                        return self.send(Request::PlaylistRename {
                            playlist_id: id,
                            new_title: title,
                        });
                    }
                }
            }
            Message::RenamePlaylistCancel => {
                self.renaming_playlist = None;
                self.rename_playlist_name.clear();
            }
            Message::DeletePlaylist(id) => {
                self.context_menu = None;
                return self.send(Request::PlaylistDelete(id));
            }
            Message::AddSongToPlaylist(playlist_id, song_id) => {
                self.context_menu = None;
                let song_ids = self.state.expand_selection(song_id);
                return self.send(Request::PlaylistAddSongs {
                    playlist_id,
                    song_ids,
                });
            }
            Message::RemoveSongFromPlaylist(playlist_id, song_id) => {
                self.context_menu = None;
                let ids = self.state.expand_selection(song_id);
                return self.send_many(
                    ids.into_iter()
                        .map(|song_id| Request::PlaylistRemoveSong {
                            playlist_id,
                            song_id,
                        })
                        .collect(),
                );
            }
            Message::ReplaceQueue(ids) => {
                return self.send_many(vec![Request::Clear, Request::ReplaceQueue(ids)]);
            }

            Message::ToggleAddToPlaylistSubmenu => {
                if let Some(ContextMenuKind::Song { submenu_open, .. }) = self.context_menu.as_mut().map(|m| &mut m.kind) {
                    *submenu_open = !*submenu_open;
                }
            }
            Message::CloseContextMenu => self.context_menu = None,

            Message::SongHovered(id) => self.hovered_song = id,
            Message::PlaylistHovered(id) => self.hovered_playlist = id,
            Message::QueueHovered(idx) => self.hovered_queue = idx,
        }

        Task::none()
    }

    fn maybe_search(&self, force: bool) -> Task<Message> {
        let q = self.search_query.trim();
        if q.len() > 2 || (!q.is_empty() && force) {
            let search = match self.search_mode {
                SearchMode::ByArtist => SearchType::ByArtist(q.to_string()),
                SearchMode::ByTitle => SearchType::ByTitle(q.to_string()),
            };
            self.send(Request::Search(search))
        } else {
            Task::none()
        }
    }

    fn toggle_song_selection(&mut self, id: Uuid) {
        if !self.state.selected_song_ids.remove(&id) {
            self.state.selected_song_ids.insert(id);
        }
    }

    /// Mirrors the per-menu-type viewport clamping done in `AuroraPlayer.slint`:
    /// flip the menu to the other side if it would overflow the window edge.
    fn clamp_menu_pos(&self, menu_w: f32, menu_h: f32) -> Point {
        let mut p = self.cursor_pos;
        if p.x + menu_w > self.window_size.width {
            p.x = (self.window_size.width - menu_w).max(0.0);
        }
        if p.y + menu_h > self.window_size.height {
            p.y = (self.window_size.height - menu_h).max(0.0);
        }
        p
    }

    fn handle_event(&mut self, event: Event) -> Task<Message> {
        match event {
            Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
                self.cursor_pos = position;
                if let Some(drag) = &mut self.queue_drag {
                    let delta = drag.start_y - position.y;
                    let rows_moved = (delta / 100.0).round() as isize;
                    let target = drag.from as isize - rows_moved;
                    let max = self.state.queue.len().saturating_sub(2) as isize;
                    drag.drop_target = target.clamp(0, max.max(0)) as usize;
                }
            }
            Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
                if let Some(drag) = self.queue_drag.take()
                    && drag.drop_target != drag.from
                {
                    return self.send(Request::MoveQueue {
                        from: drag.from,
                        to: drag.drop_target,
                    });
                }
            }
            Event::Window(iced::window::Event::Resized(size)) => {
                self.window_size = size;
            }
            Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                self.modifiers = modifiers;
            }
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: key::Key::Named(key::Named::Escape),
                modifiers,
                ..
            }) => {
                if modifiers.control() {
                    self.clear_selection_task();
                }
            }
            _ => {}
        }
        Task::none()
    }

    fn handle_daemon_event(&mut self, event: DaemonEvent) -> Task<Message> {
        match event {
            DaemonEvent::Connected(tx) => {
                self.daemon_tx = Some(tx);
                self.connected = true;
                return self.send(Request::PlaylistList);
            }
            DaemonEvent::Disconnected => {
                self.connected = false;
                self.daemon_tx = None;
            }
            DaemonEvent::Response(response) => self.handle_response(response),
        }
        Task::none()
    }

    fn handle_response(&mut self, response: Response) {
        match response {
            Response::Status(status) => {
                self.state.current_song = status.current_song;
                self.state.is_paused = status.is_paused;
                self.state.position = status.position;
                self.state.volume = status.volume;
                self.state.shuffle = status.shuffle;
                self.state.repeat = status.repeat;
                self.state.duration = self
                    .state
                    .current_song
                    .as_ref()
                    .map(|s| s.duration)
                    .unwrap_or_default();
            }
            Response::Queue(queue) => self.state.queue = queue,
            Response::SearchResults(results) => self.state.search_results = results,
            Response::PlaylistResults(result) => self.state.playlist_result = Some(result),
            Response::PlaylistList(list) => self.state.playlist_list_results = list,
            Response::Theme(theme) => self.palette = theme.into(),
            Response::Volume(v) => self.state.volume = v,
            Response::ArtistList(_) => {}
            Response::LastPlayed(songs) => self.state.last_played = songs,
            Response::LikedSongs(songs) => {
                self.state.liked_song_ids = songs.iter().map(|s| s.id).collect();
                self.state.liked_songs = songs;
            }
            Response::Error { err_id, err_msg } => {
                tracing::warn!("Daemon error {err_id}: {err_msg}");
            }
            // Not wired up in the iced experiment; the Slint app is the maintained client.
            Response::ArtReady { .. } => {}
            Response::HighResArtReady { .. } => {}
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        ui::root(self)
    }
}
