use aurora_protocol::{PlaylistMinimal, Song};
use iced::widget::{column, container, grid, image, mouse_area, row, scrollable, space, text};
use iced::{Alignment, Color, ContentFit, Element, Length};
use uuid::Uuid;

use crate::app::{AuroraApp, Message};
use crate::icons;
use crate::model::{art_handle, format_duration};
use crate::ui::{bg, icon_text};

pub fn song_row<'a>(app: &AuroraApp, song: &'a Song, current_playlist: Option<Uuid>) -> Element<'a, Message> {
    let palette = app.palette;
    let selected = app.state.is_selected(&song.id);
    let liked = app.state.is_liked(&song.id);
    let is_current = app.state.current_song.as_ref().map(|s| s.id) == Some(song.id);
    let hovered = app.hovered_song == Some(song.id);

    let art: Element<'_, Message> = image(art_handle(&song.art_path))
        .width(Length::Fixed(50.0))
        .height(Length::Fixed(50.0))
        .content_fit(ContentFit::Cover)
        .into();

    let title_col = column![
        text(song.title.clone()).size(15).color(palette.txt1),
        text(song.artists.join(", ")).size(13).color(palette.txt2),
    ]
    .spacing(2)
    .width(Length::Fill);

    let heart_glyph = if liked {
        icons::HEART_FILLED
    } else {
        icons::HEART_OUTLINE
    };
    let heart_color = if liked {
        palette.acct
    } else if hovered {
        palette.txt2
    } else {
        Color::TRANSPARENT
    };
    let heart = mouse_area(icon_text(heart_glyph, 18.0, heart_color))
        .on_press(Message::LikeToggle(song.id, liked));

    let duration = text(format_duration(song.duration)).size(13).color(palette.txt2);

    let accent_color = if selected {
        palette.acct
    } else if is_current {
        Color {
            a: 0.6,
            ..palette.acct
        }
    } else {
        Color::TRANSPARENT
    };
    let accent_bar = container(space().width(Length::Fixed(4.0)).height(Length::Fixed(50.0))).style(bg(accent_color));

    let content = row![accent_bar, art, title_col, heart, duration]
        .spacing(10)
        .align_y(Alignment::Center)
        .padding([4, 10]);

    let row_bg = if selected {
        palette.bgd4
    } else if hovered {
        palette.bgd3
    } else {
        Color::TRANSPARENT
    };

    let id = song.id;
    mouse_area(container(content).width(Length::Fill).style(bg(row_bg)))
        .on_press(Message::SongClicked(id))
        .on_right_press(Message::SongRightClicked(id, liked, current_playlist))
        .on_enter(Message::SongHovered(Some(id)))
        .on_exit(Message::SongHovered(None))
        .into()
}

pub fn song_list<'a>(app: &AuroraApp, songs: &'a [Song], current_playlist: Option<Uuid>) -> Element<'a, Message> {
    if songs.is_empty() {
        return empty_state(app, icons::NOTE_PLACEHOLDER, "No songs here", "");
    }

    let rows: Element<'_, Message> = column(songs.iter().map(|s| song_row(app, s, current_playlist)))
        .width(Length::Fill)
        .into();

    scrollable(rows).width(Length::Fill).height(Length::Fill).into()
}

pub fn empty_state<'a>(app: &AuroraApp, glyph: &'static str, title: &'static str, subtitle: &'static str) -> Element<'a, Message> {
    let palette = app.palette;
    container(
        column![
            icon_text(glyph, 48.0, palette.txt2),
            text(title).size(18).color(palette.txt2),
            text(subtitle).size(13).color(palette.txt2),
        ]
        .spacing(10)
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .align_x(Alignment::Center)
    .align_y(Alignment::Center)
    .into()
}

pub fn playlist_card<'a>(app: &'a AuroraApp, playlist: &'a PlaylistMinimal) -> Element<'a, Message> {
    card(
        app,
        playlist.id,
        &playlist.name,
        playlist.len,
        None,
        &playlist.art_paths,
    )
}

pub fn liked_songs_card(app: &AuroraApp) -> Element<'_, Message> {
    card(
        app,
        Uuid::nil(),
        "Liked Songs",
        app.state.liked_songs.len(),
        Some(icons::HEART_FILLED),
        &[],
    )
}

fn card<'a>(
    app: &'a AuroraApp,
    id: Uuid,
    title: &'a str,
    len: usize,
    icon: Option<&'static str>,
    art_paths: &'a [Option<std::path::PathBuf>],
) -> Element<'a, Message> {
    let palette = app.palette;
    let hovered = app.hovered_playlist == Some(id);
    let is_liked = icon.is_some();

    let art: Element<'_, Message> = if let Some(glyph) = icon {
        container(icon_text(glyph, 48.0, palette.acct))
            .width(Length::Fill)
            .height(Length::Fixed(140.0))
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
    } else {
        let cells: Vec<Element<'_, Message>> = art_paths
            .iter()
            .take(4)
            .map(|p| {
                if let Some(path) = p {
                    image(art_handle(&Some(path.clone())))
                        .content_fit(ContentFit::Cover)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                } else {
                    container(icon_text(icons::NOTE_PLACEHOLDER, 20.0, palette.txt2))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                        .into()
                }
            })
            .collect();
        container(grid(cells).fluid(85.0).spacing(2))
            .width(Length::Fill)
            .height(Length::Fixed(140.0))
            .clip(true)
            .into()
    };

    let content = column![
        art,
        text(title.to_string()).size(15).color(palette.txt1),
        text(format!("{len} songs")).size(12).color(palette.txt2),
    ]
    .spacing(6)
    .padding(10)
    .width(Length::Fixed(170.0));

    let bg_color = if hovered {
        palette.bgd3
    } else {
        palette.bgd2
    };

    let title_owned = title.to_string();
    let area = mouse_area(container(content).style(bg(bg_color)))
        .on_press(if is_liked {
            Message::OpenLiked
        } else {
            Message::OpenPlaylist(id)
        })
        .on_enter(Message::PlaylistHovered(Some(id)))
        .on_exit(Message::PlaylistHovered(None));

    if is_liked {
        area.into()
    } else {
        area.on_right_press(Message::PlaylistRightClicked(id, title_owned)).into()
    }
}

pub fn playlist_grid<'a>(app: &'a AuroraApp) -> Element<'a, Message> {
    let mut cells: Vec<Element<'_, Message>> = vec![liked_songs_card(app)];
    cells.extend(app.state.playlist_list_results.iter().map(|p| playlist_card(app, p)));

    scrollable(grid(cells).fluid(184.0).spacing(8)).width(Length::Fill).height(Length::Fill).into()
}
