pub mod context_menu;
pub mod detail;
pub mod player;
pub mod queue;
pub mod search;
pub mod sidebar;
pub mod song_list;

use adw::prelude::*;
use aurora_protocol::Request;
use gtk::gdk;

use crate::app::{send, send_many, AppState, Shared, Widgets};
use crate::model::NavTarget;

pub fn icon(name: &str) -> gtk::Image {
    icon_sized(name, 16)
}

pub fn icon_sized(name: &str, px: i32) -> gtk::Image {
    let img = gtk::Image::from_icon_name(name);
    img.set_pixel_size(px);
    img
}

/// Removes all children of a `gtk::Box` before repopulating it with rebuilt content.
pub fn clear(container: &gtk::Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}

fn pill_button(label_text: &str) -> gtk::Button {
    let btn = gtk::Button::with_label(label_text);
    btn.add_css_class("pill-btn");
    btn
}

pub fn build_root(app: &adw::Application) -> Shared {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Aurora Player")
        .default_width(1300)
        .default_height(800)
        .build();

    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_string(&crate::theme::css(&crate::theme::Palette::default()));
    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(&display, &css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    }

    // ---- Sidebar navigation pane ----
    // Only the playlists list scrolls; Search/Queue/Liked/Recent stay fixed.
    let sidebar_built = sidebar::build();
    let playlists_scroller = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .child(&sidebar_built.playlists_section)
        .build();

    let sidebar_col = gtk::Box::new(gtk::Orientation::Vertical, 0);
    sidebar_col.append(&sidebar_built.top_nav);
    sidebar_col.append(&playlists_scroller);

    let sidebar_toolbar = adw::ToolbarView::new();
    sidebar_toolbar.add_top_bar(&adw::HeaderBar::new());
    sidebar_toolbar.set_content(Some(&sidebar_col));
    let sidebar_page = adw::NavigationPage::builder().title("Aurora Player").child(&sidebar_toolbar).build();

    // ---- Search content page ----
    // GtkSearchEntry draws its own magnifying-glass/clear icons internally,
    // so it's styled directly (as a single pill) rather than wrapped in a
    // box with a second manual icon.
    let search_entry = gtk::SearchEntry::new();
    search_entry.set_hexpand(true);
    search_entry.set_placeholder_text(Some("Search..."));
    search_entry.add_css_class("search-bar");

    let search_mode_dropdown = gtk::DropDown::from_strings(&["By Title", "By Artist"]);
    search_mode_dropdown.set_size_request(140, -1);

    let search_header = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    search_header.set_margin_top(14);
    search_header.set_margin_bottom(14);
    search_header.set_margin_start(16);
    search_header.set_margin_end(16);
    search_header.append(&search_entry);
    search_header.append(&search_mode_dropdown);

    let search_results_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let search_scroller = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .child(&search_results_box)
        .build();

    let search_page = gtk::Box::new(gtk::Orientation::Vertical, 0);
    search_page.append(&search_header);
    search_page.append(&search_scroller);

    // ---- Detail content page (Liked / Recently Played / a specific Playlist) ----
    let detail_title_lbl = gtk::Label::new(None);
    detail_title_lbl.add_css_class("txt1");
    detail_title_lbl.add_css_class("title-18");
    detail_title_lbl.set_halign(gtk::Align::Start);
    detail_title_lbl.set_hexpand(true);
    detail_title_lbl.set_ellipsize(gtk::pango::EllipsizeMode::End);

    let detail_play_all_btn = gtk::Button::new();
    let play_all_content = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    play_all_content.append(&icon("media-playback-start-symbolic"));
    play_all_content.append(&gtk::Label::new(Some("Play All")));
    detail_play_all_btn.set_child(Some(&play_all_content));
    detail_play_all_btn.add_css_class("pill-btn");
    detail_play_all_btn.add_css_class("accent");

    let detail_header = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    detail_header.set_margin_top(16);
    detail_header.set_margin_bottom(14);
    detail_header.set_margin_start(16);
    detail_header.set_margin_end(16);
    detail_header.append(&detail_title_lbl);
    detail_header.append(&detail_play_all_btn);

    let detail_list_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let detail_scroller = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .child(&detail_list_box)
        .build();

    let detail_page = gtk::Box::new(gtk::Orientation::Vertical, 0);
    detail_page.append(&detail_header);
    detail_page.append(&detail_scroller);

    // ---- Queue content page ----
    let queue_header = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    queue_header.set_margin_top(16);
    queue_header.set_margin_bottom(14);
    queue_header.set_margin_start(16);
    queue_header.set_margin_end(16);
    let queue_title = gtk::Label::new(Some("Queue"));
    queue_title.add_css_class("txt1");
    queue_title.add_css_class("title-18");
    queue_title.set_halign(gtk::Align::Start);
    queue_title.set_hexpand(true);
    let queue_clear_btn = pill_button("Clear");
    queue_header.append(&queue_title);
    queue_header.append(&queue_clear_btn);

    let queue_list_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let queue_scroller = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .child(&queue_list_box)
        .build();

    let queue_page = gtk::Box::new(gtk::Orientation::Vertical, 0);
    queue_page.append(&queue_header);
    queue_page.append(&queue_scroller);

    // ---- Content stack + navigation split view ----
    let content_stack = gtk::Stack::new();
    content_stack.add_named(&search_page, Some("search"));
    content_stack.add_named(&detail_page, Some("detail"));
    content_stack.add_named(&queue_page, Some("queue"));
    content_stack.set_visible_child_name("search");
    content_stack.set_vexpand(true);

    let content_toolbar = adw::ToolbarView::new();
    content_toolbar.add_top_bar(&adw::HeaderBar::new());
    content_toolbar.set_content(Some(&content_stack));
    let content_page = adw::NavigationPage::builder().title("Search").child(&content_toolbar).build();

    let split_view = adw::NavigationSplitView::new();
    split_view.set_sidebar(Some(&sidebar_page));
    split_view.set_content(Some(&content_page));
    split_view.set_vexpand(true);
    split_view.set_min_sidebar_width(220.0);
    split_view.set_max_sidebar_width(320.0);

    // ---- Player bar (built once; only values are updated afterwards) ----
    let player_built = player::build();

    let root_col = gtk::Box::new(gtk::Orientation::Vertical, 0);
    root_col.append(&split_view);
    root_col.append(&player_built.root);

    // ---- Disconnected overlay ----
    let disconnected_overlay = gtk::Box::new(gtk::Orientation::Vertical, 0);
    disconnected_overlay.add_css_class("disconnected-scrim");
    disconnected_overlay.set_visible(false);
    disconnected_overlay.set_can_target(false);
    disconnected_overlay.set_hexpand(true);
    disconnected_overlay.set_vexpand(true);

    let card = gtk::Box::new(gtk::Orientation::Vertical, 8);
    card.add_css_class("disconnected-card");
    card.set_halign(gtk::Align::Center);
    card.set_valign(gtk::Align::Center);
    card.set_margin_top(24);
    card.set_margin_bottom(24);
    card.set_margin_start(48);
    card.set_margin_end(48);
    let title = gtk::Label::new(Some("Disconnected from daemon"));
    title.add_css_class("txt1");
    title.add_css_class("title-20");
    let subtitle = gtk::Label::new(Some("Attempting to reconnect..."));
    subtitle.add_css_class("txt2");
    card.append(&title);
    card.append(&subtitle);

    let overlay_center = gtk::Box::new(gtk::Orientation::Vertical, 0);
    overlay_center.set_halign(gtk::Align::Center);
    overlay_center.set_valign(gtk::Align::Center);
    overlay_center.set_hexpand(true);
    overlay_center.set_vexpand(true);
    overlay_center.append(&card);
    disconnected_overlay.append(&overlay_center);

    let overlay = gtk::Overlay::new();
    overlay.set_child(Some(&root_col));
    overlay.add_overlay(&disconnected_overlay);

    window.set_content(Some(&overlay));

    let widgets = Widgets {
        window: window.clone(),
        css_provider,
        content_page: content_page.clone(),
        content_stack: content_stack.clone(),
        playlists_section: sidebar_built.playlists_section.clone(),
        search_entry: search_entry.clone(),
        search_results_box,
        detail_title_lbl,
        detail_list_box,
        queue_list_box,
        queue_scroller,
        player_art: player_built.art.clone(),
        player_title_lbl: player_built.title_lbl.clone(),
        player_artist_lbl: player_built.artist_lbl.clone(),
        seek_scale: player_built.seek_scale.clone(),
        volume_scale: player_built.volume_scale.clone(),
        elapsed_lbl: player_built.elapsed_lbl.clone(),
        duration_lbl: player_built.duration_lbl.clone(),
        play_pause_btn: player_built.play_pause_btn.clone(),
        shuffle_btn: player_built.shuffle_btn.clone(),
        repeat_btn: player_built.repeat_btn.clone(),
        like_btn: player_built.like_btn.clone(),
        disconnected_overlay,
    };

    let shared = AppState::new(widgets);

    // ---- Wiring (now that `shared` exists) ----
    sidebar::wire(&shared, &sidebar_built);
    player::wire(&shared, &player_built);

    {
        let shared = shared.clone();
        queue_clear_btn.connect_clicked(move |_| send(&shared, Request::Clear));
    }
    {
        let shared = shared.clone();
        detail_play_all_btn.connect_clicked(move |_| {
            let ids = detail::current_song_ids(&shared);
            send_many(&shared, vec![Request::Clear, Request::ReplaceQueue(ids)]);
        });
    }
    {
        let shared = shared.clone();
        search_entry.connect_search_changed(move |entry| search::maybe_search(&shared, &entry.text(), false));
    }
    {
        let shared = shared.clone();
        search_entry.connect_activate(move |entry| search::maybe_search(&shared, &entry.text(), true));
    }
    {
        let shared = shared.clone();
        search_mode_dropdown.connect_selected_notify(move |dd| {
            shared.borrow_mut().search_mode = if dd.selected() == 1 {
                crate::model::SearchMode::ByArtist
            } else {
                crate::model::SearchMode::ByTitle
            };
            let text = shared.borrow().widgets.search_entry.text();
            search::maybe_search(&shared, &text, false);
        });
    }

    shared
}

/// Switches the content pane to the given destination, fetching fresh data
/// for it and (re)populating the reused pages as needed.
pub fn set_nav(shared: &Shared, target: NavTarget) {
    shared.borrow_mut().nav = target.clone();
    let (stack, content_page) = {
        let s = shared.borrow();
        (s.widgets.content_stack.clone(), s.widgets.content_page.clone())
    };

    match &target {
        NavTarget::Search => {
            stack.set_visible_child_name("search");
            content_page.set_title("Search");
        }
        NavTarget::Queue => {
            stack.set_visible_child_name("queue");
            content_page.set_title("Queue");
        }
        NavTarget::Liked => {
            stack.set_visible_child_name("detail");
            content_page.set_title("Liked Songs");
            send(shared, Request::GetLikedSongs);
            detail::rebuild(shared);
        }
        NavTarget::Recent => {
            stack.set_visible_child_name("detail");
            content_page.set_title("Recently Played");
            send(shared, Request::GetLastPlayed);
            detail::rebuild(shared);
        }
        NavTarget::Playlist(id) => {
            stack.set_visible_child_name("detail");
            content_page.set_title("Playlist");
            shared.borrow_mut().state.playlist_result = None;
            send(shared, Request::PlaylistGet(*id));
            detail::rebuild(shared);
        }
    }
}

pub fn update_disconnected_overlay(shared: &Shared) {
    let s = shared.borrow();
    s.widgets.disconnected_overlay.set_visible(!s.connected);
}
