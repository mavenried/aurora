use aurora_protocol::Request;
use gtk::prelude::*;
use uuid::Uuid;

use crate::app::{send, Shared};
use crate::model::NavTarget;
use crate::ui::{context_menu, icon};

pub struct Built {
    /// Fixed nav rows (Search/Queue/Liked/Recent + the "PLAYLISTS" header)
    /// — never scrolls.
    pub top_nav: gtk::Widget,
    /// Just the playlist rows; the caller wraps this in a `ScrolledWindow`
    /// so only this section scrolls, independent of `top_nav`.
    pub playlists_section: gtk::Box,

    pub search_row: gtk::Box,
    pub queue_row: gtk::Box,
    pub settings_row: gtk::Box,
    pub liked_row: gtk::Box,
    pub recent_row: gtk::Box,
    pub add_btn: gtk::Button,
}

/// Pure widget construction; no `Shared` yet exists at this point (it holds
/// these widgets), so click handling is wired separately by `wire()`.
pub fn build() -> Built {
    let top_nav = gtk::Box::new(gtk::Orientation::Vertical, 2);
    top_nav.set_margin_top(6);
    top_nav.set_margin_start(6);
    top_nav.set_margin_end(6);

    let search_row = plain_row("system-search-symbolic", "Search");
    let queue_row = plain_row("view-list-symbolic", "Queue");
    let settings_row = plain_row("emblem-system-symbolic", "Settings");
    let liked_row = plain_row("emblem-favorite-symbolic", "Liked Songs");
    let recent_row = plain_row("document-open-recent-symbolic", "Recently Played");

    top_nav.append(&search_row);
    top_nav.append(&queue_row);
    top_nav.append(&settings_row);

    let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
    sep.set_margin_top(8);
    sep.set_margin_bottom(8);
    top_nav.append(&sep);

    top_nav.append(&liked_row);
    top_nav.append(&recent_row);

    let playlists_header = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    playlists_header.set_margin_top(14);
    playlists_header.set_margin_bottom(6);
    playlists_header.set_margin_start(8);
    playlists_header.set_margin_end(4);
    let hdr_lbl = gtk::Label::new(Some("PLAYLISTS"));
    hdr_lbl.add_css_class("txt2");
    hdr_lbl.add_css_class("subtle-12");
    hdr_lbl.set_halign(gtk::Align::Start);
    hdr_lbl.set_hexpand(true);
    let add_btn = gtk::Button::new();
    add_btn.set_child(Some(&icon("list-add-symbolic")));
    add_btn.add_css_class("circle-btn");
    add_btn.set_tooltip_text(Some("New Playlist"));
    playlists_header.append(&hdr_lbl);
    playlists_header.append(&add_btn);
    top_nav.append(&playlists_header);

    let playlists_section = gtk::Box::new(gtk::Orientation::Vertical, 2);
    playlists_section.set_margin_start(6);
    playlists_section.set_margin_end(6);
    playlists_section.set_margin_bottom(6);

    search_row.add_css_class("active");

    Built {
        top_nav: top_nav.upcast(),
        playlists_section,
        search_row,
        queue_row,
        settings_row,
        liked_row,
        recent_row,
        add_btn,
    }
}

/// Attaches click handling now that `shared` exists.
pub fn wire(shared: &Shared, built: &Built) {
    shared.borrow_mut().sidebar_active_row = Some(built.search_row.clone().upcast());

    wire_row(shared, &built.search_row, NavTarget::Search);
    wire_row(shared, &built.queue_row, NavTarget::Queue);
    wire_row(shared, &built.settings_row, NavTarget::Settings);
    wire_row(shared, &built.liked_row, NavTarget::Liked);
    wire_row(shared, &built.recent_row, NavTarget::Recent);

    let shared_c = shared.clone();
    let add_btn = built.add_btn.clone();
    built.add_btn.connect_clicked(move |_| {
        prompt(&shared_c, &add_btn, "New Playlist", "", move |shared, name| {
            if !name.is_empty() {
                send(shared, Request::PlaylistCreate(aurora_protocol::PlaylistIn { title: name, songs: vec![] }));
            }
        });
    });
}

pub fn rebuild_playlists(shared: &Shared) {
    let container = shared.borrow().widgets.playlists_section.clone();
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
    let playlists = shared.borrow().state.playlist_list_results.clone();
    for pl in &playlists {
        container.append(&playlist_row(shared, pl.id, &pl.name, pl.len));
    }
}

fn plain_row(icon_name: &str, label_text: &str) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    row.add_css_class("sidebar-row");
    row.set_margin_top(1);
    row.set_margin_bottom(1);

    let img = icon(icon_name);
    let lbl = gtk::Label::new(Some(label_text));
    lbl.add_css_class("txt1");
    lbl.set_halign(gtk::Align::Start);
    lbl.set_hexpand(true);
    lbl.set_ellipsize(gtk::pango::EllipsizeMode::End);

    row.append(&img);
    row.append(&lbl);
    row
}

fn wire_row(shared: &Shared, row: &gtk::Box, target: NavTarget) {
    let click = gtk::GestureClick::new();
    let shared = shared.clone();
    let row_weak = row.downgrade();
    click.connect_pressed(move |_, _, _, _| {
        if let Some(row) = row_weak.upgrade() {
            activate(&shared, row.upcast(), target.clone());
        }
    });
    row.add_controller(click);
}

fn playlist_row(shared: &Shared, id: Uuid, name: &str, len: usize) -> gtk::Box {
    let row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    row.add_css_class("sidebar-row");
    row.set_margin_top(1);
    row.set_margin_bottom(1);
    row.append(&icon("view-list-symbolic"));

    let text_col = gtk::Box::new(gtk::Orientation::Vertical, 0);
    text_col.set_hexpand(true);
    let name_lbl = gtk::Label::new(Some(name));
    name_lbl.add_css_class("txt1");
    name_lbl.set_halign(gtk::Align::Start);
    name_lbl.set_ellipsize(gtk::pango::EllipsizeMode::End);
    let count_lbl = gtk::Label::new(Some(&format!("{len} songs")));
    count_lbl.add_css_class("txt2");
    count_lbl.add_css_class("subtle-12");
    count_lbl.set_halign(gtk::Align::Start);
    text_col.append(&name_lbl);
    text_col.append(&count_lbl);
    row.append(&text_col);

    wire_row(shared, &row, NavTarget::Playlist(id));

    let right_click = gtk::GestureClick::new();
    right_click.set_button(3);
    {
        let shared = shared.clone();
        let row_weak = row.downgrade();
        let name = name.to_string();
        right_click.connect_pressed(move |_, _, x, y| {
            if let Some(row) = row_weak.upgrade() {
                context_menu::show_playlist_menu(&shared, &row, id, &name, x, y);
            }
        });
    }
    row.add_controller(right_click);

    row
}

fn activate(shared: &Shared, row: gtk::Widget, target: NavTarget) {
    {
        let mut s = shared.borrow_mut();
        if let Some(prev) = s.sidebar_active_row.take() {
            prev.remove_css_class("active");
        }
        row.add_css_class("active");
        s.sidebar_active_row = Some(row);
    }
    super::set_nav(shared, target);
}

/// Popover-anchored name entry used for both playlist creation and rename,
/// with a title, roomy input, and explicit Cancel/confirm buttons.
pub fn prompt(shared: &Shared, parent: &impl IsA<gtk::Widget>, title: &str, initial: &str, on_confirm: impl Fn(&Shared, String) + 'static) {
    let title_lbl = gtk::Label::new(Some(title));
    title_lbl.add_css_class("txt1");
    title_lbl.add_css_class("title-15");
    title_lbl.set_halign(gtk::Align::Start);

    let entry = gtk::Entry::new();
    entry.add_css_class("dialog-entry");
    entry.set_placeholder_text(Some("Playlist name"));
    entry.set_text(initial);
    entry.set_width_chars(26);
    entry.set_activates_default(true);

    let cancel_btn = gtk::Button::with_label("Cancel");
    cancel_btn.add_css_class("pill-btn");
    let confirm_label = if initial.is_empty() { "Create" } else { "Rename" };
    let confirm_btn = gtk::Button::with_label(confirm_label);
    confirm_btn.add_css_class("pill-btn");
    confirm_btn.add_css_class("accent");

    let button_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    button_row.set_halign(gtk::Align::End);
    button_row.append(&cancel_btn);
    button_row.append(&confirm_btn);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content.set_margin_top(16);
    content.set_margin_bottom(16);
    content.set_margin_start(16);
    content.set_margin_end(16);
    content.append(&title_lbl);
    content.append(&entry);
    content.append(&button_row);

    let popover = gtk::Popover::new();
    popover.set_child(Some(&content));
    popover.set_parent(parent);
    popover.connect_closed(|p| p.unparent());

    let confirm: std::rc::Rc<dyn Fn()> = {
        let shared = shared.clone();
        let entry = entry.clone();
        let popover_weak = popover.downgrade();
        std::rc::Rc::new(move || {
            on_confirm(&shared, entry.text().trim().to_string());
            if let Some(p) = popover_weak.upgrade() {
                p.popdown();
            }
        })
    };
    {
        let confirm = confirm.clone();
        entry.connect_activate(move |_| confirm());
    }
    {
        let confirm = confirm.clone();
        confirm_btn.connect_clicked(move |_| confirm());
    }
    {
        let popover_weak = popover.downgrade();
        cancel_btn.connect_clicked(move |_| {
            if let Some(p) = popover_weak.upgrade() {
                p.popdown();
            }
        });
    }

    popover.popup();
    entry.grab_focus();
}
