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

struct SettingsBuilt {
    page: gtk::Box,
    colorways: gtk::Switch,
    theme: gtk::Switch,
    compact_rows: gtk::Switch,
    smooth_scrolling: gtk::Switch,
    remember_volume: gtk::Switch,
    show_album_art: gtk::Switch,
}

fn settings_page() -> SettingsBuilt {
    let page = gtk::Box::new(gtk::Orientation::Vertical, 0);
    page.add_css_class("app-surface");
    page.set_margin_start(32);
    page.set_margin_end(32);
    page.set_margin_top(28);
    page.set_margin_bottom(28);

    let title = gtk::Label::new(Some("Settings"));
    title.add_css_class("txt1");
    title.add_css_class("title-20");
    title.set_halign(gtk::Align::Start);
    page.append(&title);

    let subtitle = gtk::Label::new(Some("Customize the look and behavior of Aurora Player."));
    subtitle.add_css_class("txt2");
    subtitle.set_halign(gtk::Align::Start);
    subtitle.set_margin_top(4);
    subtitle.set_margin_bottom(22);
    page.append(&subtitle);

    let appearance = section_label("Appearance");
    page.append(&appearance);
    let colorways = setting_switch("Album art colorways", "Tint the interface using the current album art.", false);
    let theme = setting_switch("Dark appearance", "Use Aurora's dark palette.", true);
    page.append(&colorways.0);
    page.append(&theme.0);

    let playback = section_label("Playback");
    playback.set_margin_top(24);
    page.append(&playback);
    let show_album_art = setting_switch("Show album art", "Show artwork in the player bar.", true);
    let remember_volume = setting_switch("Remember volume", "Restore the player volume when Aurora starts.", false);
    page.append(&show_album_art.0);
    page.append(&remember_volume.0);

    let interface = section_label("Interface");
    interface.set_margin_top(24);
    page.append(&interface);
    let compact_rows = setting_switch("Compact song rows", "Use less vertical space in song lists.", false);
    let smooth_scrolling = setting_switch("Smooth scrolling", "Animate scrolling in song lists.", true);
    page.append(&compact_rows.0);
    page.append(&smooth_scrolling.0);

    let reset = gtk::Button::with_label("Reset settings");
    reset.add_css_class("pill-btn");
    reset.set_halign(gtk::Align::Start);
    reset.set_margin_top(28);
    page.append(&reset);

    SettingsBuilt {
        page,
        colorways: colorways.1,
        theme: theme.1,
        compact_rows: compact_rows.1,
        smooth_scrolling: smooth_scrolling.1,
        remember_volume: remember_volume.1,
        show_album_art: show_album_art.1,
    }
}

fn section_label(text: &str) -> gtk::Label {
    let label = gtk::Label::new(Some(text));
    label.add_css_class("accent");
    label.add_css_class("title-15");
    label.set_halign(gtk::Align::Start);
    label
}

fn setting_switch(title: &str, description: &str, active: bool) -> (gtk::Box, gtk::Switch) {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 16);
    row.set_margin_top(8);
    row.set_margin_bottom(8);
    let text = gtk::Box::new(gtk::Orientation::Vertical, 2);
    text.set_hexpand(true);
    let title_label = gtk::Label::new(Some(title));
    title_label.add_css_class("txt1");
    title_label.set_halign(gtk::Align::Start);
    let description_label = gtk::Label::new(Some(description));
    description_label.add_css_class("txt2");
    description_label.add_css_class("subtle-13");
    description_label.set_halign(gtk::Align::Start);
    text.append(&title_label);
    text.append(&description_label);
    let switch = gtk::Switch::new();
    switch.set_active(active);
    switch.set_valign(gtk::Align::Center);
    row.append(&text);
    row.append(&switch);
    (row, switch)
}

fn wire_settings(shared: &Shared) {
    let colorways = shared.borrow().widgets.settings_colorways.clone();
    let shared_c = shared.clone();
    colorways.connect_active_notify(move |switch| {
        crate::app::send(&shared_c, Request::SetFollowArtColorway(switch.is_active()));
    });
}

fn now_playing_page() -> (gtk::Box, gtk::Picture, gtk::Label, gtk::Label, gtk::Scale, gtk::Button) {
    let page = gtk::Box::new(gtk::Orientation::Vertical, 0);
    page.add_css_class("app-surface");
    page.set_margin_start(32);
    page.set_margin_end(32);
    page.set_margin_top(20);
    page.set_margin_bottom(28);

    let back = gtk::Button::from_icon_name("go-previous-symbolic");
    back.add_css_class("circle-btn");
    back.set_halign(gtk::Align::Start);
    back.set_tooltip_text(Some("Back"));
    page.append(&back);

    let art = gtk::Picture::new();
    art.add_css_class("thumb");
    art.set_content_fit(gtk::ContentFit::Cover);
    art.set_hexpand(true);
    art.set_vexpand(true);
    art.set_halign(gtk::Align::Fill);
    art.set_valign(gtk::Align::Fill);

    let art_frame = gtk::AspectFrame::new(0.5, 0.5, 1.0, true);
    art_frame.set_size_request(360, 360);
    art_frame.set_halign(gtk::Align::Center);
    art_frame.set_valign(gtk::Align::Center);
    art_frame.set_margin_top(12);
    art_frame.set_margin_bottom(22);
    art_frame.set_child(Some(&art));
    page.append(&art_frame);

    let title = gtk::Label::new(Some("Nothing Playing"));
    title.add_css_class("txt1");
    title.add_css_class("title-20");
    title.set_halign(gtk::Align::Center);
    title.set_ellipsize(gtk::pango::EllipsizeMode::End);
    page.append(&title);
    let artist = gtk::Label::new(Some("No Artist"));
    artist.add_css_class("txt2");
    artist.add_css_class("subtle-14");
    artist.set_halign(gtk::Align::Center);
    page.append(&artist);

    let seek = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 1.0);
    seek.set_draw_value(false);
    seek.set_hexpand(true);
    seek.set_margin_top(24);
    page.append(&seek);
    (page, art, title, artist, seek, back)
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
    sidebar_col.add_css_class("app-surface");
    sidebar_col.append(&sidebar_built.top_nav);
    sidebar_col.append(&playlists_scroller);

    let sidebar_header = adw::HeaderBar::new();
    let sidebar_title_label = gtk::Label::new(Some("Aurora Player"));
    sidebar_title_label.add_css_class("txt1");
    sidebar_title_label.add_css_class("title-15");
    sidebar_header.set_title_widget(Some(&sidebar_title_label));

    let sidebar_toolbar = adw::ToolbarView::new();
    sidebar_toolbar.add_top_bar(&sidebar_header);
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
    search_page.add_css_class("app-surface");
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
    detail_page.add_css_class("app-surface");
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
    queue_page.add_css_class("app-surface");
    queue_page.append(&queue_header);
    queue_page.append(&queue_scroller);

    let settings = settings_page();
    let settings_page = settings.page.clone();
    let (now_playing_page, expanded_art, expanded_title_lbl, expanded_artist_lbl, expanded_seek_scale, now_playing_back) =
        now_playing_page();

    // ---- Content stack + navigation split view ----
    let content_stack = gtk::Stack::new();
    content_stack.add_named(&search_page, Some("search"));
    content_stack.add_named(&detail_page, Some("detail"));
    content_stack.add_named(&queue_page, Some("queue"));
    content_stack.add_named(&settings_page, Some("settings"));
    content_stack.add_named(&now_playing_page, Some("now-playing"));
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
        settings_colorways: settings.colorways.clone(),
        settings_theme: settings.theme.clone(),
        settings_compact_rows: settings.compact_rows.clone(),
        settings_smooth_scrolling: settings.smooth_scrolling.clone(),
        settings_remember_volume: settings.remember_volume.clone(),
        settings_show_album_art: settings.show_album_art.clone(),
        expanded_art,
        expanded_title_lbl,
        expanded_artist_lbl,
        expanded_seek_scale,
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
    wire_settings(&shared);
    {
        let stack = content_stack.clone();
        let click = gtk::GestureClick::new();
        click.set_button(1);
        let shared_c = shared.clone();
        click.connect_pressed(move |_, _, _, _| {
            stack.set_visible_child_name("now-playing");
            if let Some(id) = shared_c.borrow().state.current_song.as_ref().map(|song| song.id) {
                send(&shared_c, Request::GetHighResArt(id));
            }
        });
        player_built.art.add_controller(click);
    }
    {
        let shared_c = shared.clone();
        now_playing_back.connect_clicked(move |_| {
            let target = shared_c.borrow().nav.clone();
            set_nav(&shared_c, target);
        });
    }
    {
        let shared_c = shared.clone();
        let expanded_seek = shared.borrow().widgets.expanded_seek_scale.clone();
        expanded_seek.connect_change_value(move |_, _, value| {
            send(&shared_c, Request::Seek(std::time::Duration::from_millis(value.max(0.0) as u64)));
            gtk::glib::Propagation::Proceed
        });
    }

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
        NavTarget::Settings => {
            stack.set_visible_child_name("settings");
            content_page.set_title("Settings");
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
