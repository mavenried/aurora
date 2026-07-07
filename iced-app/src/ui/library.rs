use aurora_protocol::Song;
use iced::widget::{button, column, container, row, space, text, text_input};
use iced::{Alignment, Element, Length};
use uuid::Uuid;

use crate::app::{AuroraApp, Message};
use crate::icons;
use crate::model::LibraryPanel;
use crate::ui::{bg, circle_icon_style, icon_text, pill_style, song_list};

pub fn view(app: &AuroraApp) -> Element<'_, Message> {
    match app.library_panel {
        LibraryPanel::Overview => overview(app),
        LibraryPanel::Liked => detail(app, "Liked Songs", &app.state.liked_songs, None),
        LibraryPanel::Playlist => match &app.state.playlist_result {
            Some(pl) => detail(app, &pl.title, &pl.songs, Some(pl.id)),
            None => song_list::empty_state(app, icons::NOTE_PLACEHOLDER, "Loading playlist...", ""),
        },
    }
}

fn overview(app: &AuroraApp) -> Element<'_, Message> {
    let palette = app.palette;

    let playlists_header = row![
        text("Playlists").size(16).color(palette.txt1),
        space().width(Length::Fill),
        button(icon_text(icons::PLUS, 15.0, palette.bgd0))
            .style(pill_style(palette.acct, palette.bgd0))
            .padding(8)
            .on_press(Message::CreatePlaylistToggle),
    ]
    .align_y(Alignment::Center)
    .padding(10);

    let inline_form: Element<'_, Message> = if app.creating_playlist {
        playlist_name_form(
            app,
            &app.new_playlist_name,
            Message::NewPlaylistNameChanged,
            Message::CreatePlaylistSubmit,
            Message::CreatePlaylistToggle,
        )
    } else if app.renaming_playlist.is_some() {
        playlist_name_form(
            app,
            &app.rename_playlist_name,
            Message::RenamePlaylistNameChanged,
            Message::RenamePlaylistSubmit,
            Message::RenamePlaylistCancel,
        )
    } else {
        space().height(Length::Fixed(0.0)).into()
    };

    let playlists_panel = container(column![playlists_header, inline_form, song_list::playlist_grid(app)])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(bg(palette.bgd1));

    if app.state.last_played.is_empty() {
        return row![playlists_panel].spacing(5).width(Length::Fill).height(Length::Fill).into();
    }

    let recently_played = container(column![
        row![
            icon_text(icons::CLOCK, 16.0, palette.txt2),
            text("Recently Played").size(16).color(palette.txt1),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .padding(10),
        song_list::song_list(app, &app.state.last_played, None),
    ])
    .width(Length::Fixed(330.0))
    .height(Length::Fill)
    .style(bg(palette.bgd1));

    row![recently_played, playlists_panel]
        .spacing(5)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn playlist_name_form<'a>(
    app: &'a AuroraApp,
    value: &'a str,
    on_change: impl Fn(String) -> Message + 'a,
    on_confirm: Message,
    on_cancel: Message,
) -> Element<'a, Message> {
    let palette = app.palette;
    row![
        text_input("Playlist name", value)
            .on_input(on_change)
            .on_submit(on_confirm.clone())
            .width(Length::Fill),
        button(icon_text(icons::CHECK, 16.0, palette.acct)).on_press(on_confirm),
        button(icon_text(icons::CROSS, 16.0, palette.txt2)).on_press(on_cancel),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .padding(10)
    .into()
}

fn detail<'a>(app: &'a AuroraApp, title: &'a str, songs: &'a [Song], playlist_id: Option<Uuid>) -> Element<'a, Message> {
    let palette = app.palette;

    let back = button(icon_text(icons::BACK, 18.0, palette.txt1))
        .style(circle_icon_style(false, palette.acct, palette.txt1, palette.bgd3))
        .width(Length::Fixed(36.0))
        .height(Length::Fixed(36.0))
        .on_press(Message::OpenLibraryOverview);

    let ids: Vec<Uuid> = songs.iter().map(|s| s.id).collect();
    let play_all = button(icon_text(icons::PLAY_ALL, 16.0, palette.bgd0))
        .style(pill_style(palette.acct, palette.bgd0))
        .padding(8)
        .on_press(Message::ReplaceQueue(ids));

    let header = row![
        back,
        play_all,
        text(format!("{title} · {} songs", songs.len())).size(18).color(palette.txt1),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .padding(10);

    container(column![header, song_list::song_list(app, songs, playlist_id)])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(bg(palette.bgd1))
        .into()
}
