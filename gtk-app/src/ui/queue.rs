use aurora_protocol::{Request, Song};
use gtk::prelude::*;

use crate::app::{send, Shared};
use crate::model::format_duration;
use crate::ui::{clear, context_menu, song_list};

pub fn rebuild(shared: &Shared) {
    let container = shared.borrow().widgets.queue_list_box.clone();
    clear(&container);
    container.set_valign(gtk::Align::Start);

    let display: Vec<Song> = {
        let s = shared.borrow();
        s.state.queue.iter().skip(1).cloned().collect()
    };

    if display.is_empty() {
        container.append(&song_list::empty_state("audio-x-generic-symbolic", "Queue is empty", "Add songs to get started"));
        return;
    }

    for (index, song) in display.iter().enumerate() {
        container.append(&queue_row(shared, index, song));
    }
}

fn queue_row(shared: &Shared, index: usize, song: &Song) -> gtk::Widget {
    let art = shared.borrow_mut().art_texture(&song.art_path);

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
    title_col.set_hexpand(true);
    title_col.set_valign(gtk::Align::Center);
    let title_lbl = gtk::Label::new(Some(&song.title));
    title_lbl.add_css_class("txt1");
    title_lbl.set_halign(gtk::Align::Start);
    title_lbl.set_ellipsize(gtk::pango::EllipsizeMode::End);
    let artists_lbl = gtk::Label::new(Some(&song.artists.join(", ")));
    artists_lbl.add_css_class("txt2");
    artists_lbl.add_css_class("subtle-13");
    artists_lbl.set_halign(gtk::Align::Start);
    artists_lbl.set_ellipsize(gtk::pango::EllipsizeMode::End);
    title_col.append(&title_lbl);
    title_col.append(&artists_lbl);

    let duration_lbl = gtk::Label::new(Some(&format_duration(song.duration)));
    duration_lbl.add_css_class("txt2");
    duration_lbl.add_css_class("subtle-13");

    let row = gtk::Box::new(gtk::Orientation::Horizontal, 14);
    row.add_css_class("queue-row");
    row.set_size_request(-1, 68);
    row.set_margin_start(12);
    row.set_margin_end(14);
    row.set_margin_top(3);
    row.set_margin_bottom(3);
    row.set_hexpand(false);
    row.set_vexpand(false);
    row.set_valign(gtk::Align::Start);
    row.append(&picture);
    row.append(&title_col);
    row.append(&duration_lbl);

    let click = gtk::GestureClick::new();
    click.set_button(1);
    {
        let shared = shared.clone();
        click.connect_pressed(move |_, _, _, _| send(&shared, Request::Next(index + 1)));
    }
    row.add_controller(click);

    let right_click = gtk::GestureClick::new();
    right_click.set_button(3);
    {
        let shared = shared.clone();
        let row_weak = row.downgrade();
        right_click.connect_pressed(move |_, _, x, y| {
            if let Some(row) = row_weak.upgrade() {
                context_menu::show_queue_menu(&shared, &row, index, x, y);
            }
        });
    }
    row.add_controller(right_click);

    row.upcast()
}
