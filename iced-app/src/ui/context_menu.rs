use iced::widget::{button, column, container, text};
use iced::{Border, Color, Element, Length, Point, Shadow, Theme as IcedTheme, Vector};

use crate::app::{AuroraApp, Message};
use crate::icons;
use crate::model::{ContextMenuKind, ContextMenuState};
use crate::theme::Palette;

const ITEM_HEIGHT: f32 = 40.0;
const MENU_WIDTH: f32 = 200.0;

pub fn view<'a>(app: &'a AuroraApp, menu: &'a ContextMenuState) -> Element<'a, Message> {
    match &menu.kind {
        ContextMenuKind::Song {
            song_id,
            liked,
            current_playlist,
            submenu_open,
        } => song_menu(app, *song_id, *liked, *current_playlist, *submenu_open, menu.position),
        ContextMenuKind::Playlist { playlist_id, title } => playlist_menu(app, *playlist_id, title, menu.position),
        ContextMenuKind::Queue { index } => queue_menu(app, *index, menu.position),
    }
}

fn menu_panel<'a>(palette: Palette, width: f32, height: f32, content: Element<'a, Message>) -> Element<'a, Message> {
    container(content)
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .clip(true)
        .style(move |_theme: &IcedTheme| container::Style {
            background: Some(palette.bgd2.into()),
            border: Border {
                color: palette.bgd4,
                width: 1.0,
                radius: 8.0.into(),
            },
            shadow: Shadow {
                color: Color::from_rgba8(0, 0, 0, 0.4),
                offset: Vector::new(0.0, 4.0),
                blur_radius: 12.0,
            },
            ..container::Style::default()
        })
        .into()
}

fn menu_item<'a>(palette: Palette, label: String, enabled: bool, on_press: Message) -> Element<'a, Message> {
    let mut b = button(
        text(label)
            .size(15)
            .color(if enabled { palette.txt1 } else { palette.txt2 }),
    )
    .width(Length::Fill)
    .height(Length::Fixed(ITEM_HEIGHT))
    .padding([0, 14])
    .style(move |_theme: &IcedTheme, status| {
        let bg = match status {
            button::Status::Hovered if enabled => Some(palette.bgd3.into()),
            _ => None,
        };
        button::Style {
            background: bg,
            text_color: if enabled { palette.txt1 } else { palette.txt2 },
            border: Border::default(),
            ..button::Style::default()
        }
    });
    if enabled {
        b = b.on_press(on_press);
    }
    b.into()
}

fn song_menu<'a>(
    app: &'a AuroraApp,
    song_id: uuid::Uuid,
    liked: bool,
    current_playlist: Option<uuid::Uuid>,
    submenu_open: bool,
    pos: Point,
) -> Element<'a, Message> {
    let palette = app.palette;
    let playlists = &app.state.playlist_list_results;

    let base_height = 170.0 + if current_playlist.is_some() { ITEM_HEIGHT } else { 0.0 };
    let submenu_height = playlists.len() as f32 * ITEM_HEIGHT;
    let full_height = base_height + submenu_height;
    let expand_up = pos.y + full_height > app.window_size.height;

    let submenu = || -> Element<'a, Message> {
        column(playlists.iter().map(|p| {
            menu_item(palette, p.name.clone(), true, Message::AddSongToPlaylist(p.id, song_id))
        }))
        .into()
    };

    let mut items: Vec<Element<'a, Message>> = vec![];

    if expand_up && submenu_open {
        items.push(submenu());
    }

    items.push(menu_item(palette, "Play Next".into(), true, Message::PlayNext(song_id)));
    items.push(menu_item(palette, "Add to Queue".into(), true, Message::Enqueue(song_id)));
    items.push(menu_item(
        palette,
        if liked { "Remove from Liked Songs" } else { "Add to Liked Songs" }.into(),
        true,
        Message::LikeToggle(song_id, liked),
    ));

    let chevron = if submenu_open { icons::CHEVRON_UP } else { icons::CHEVRON_DOWN };
    items.push(menu_item(
        palette,
        format!("Add to Playlist {chevron}"),
        true,
        Message::ToggleAddToPlaylistSubmenu,
    ));

    if !expand_up && submenu_open {
        items.push(submenu());
    }

    if let Some(pl_id) = current_playlist {
        items.push(menu_item(
            palette,
            "Remove from Playlist".into(),
            true,
            Message::RemoveSongFromPlaylist(pl_id, song_id),
        ));
    }

    let menu_height = if submenu_open { full_height } else { base_height };
    let panel = menu_panel(palette, MENU_WIDTH, menu_height, column(items).into());

    let y = if expand_up {
        (pos.y - if submenu_open { submenu_height } else { 0.0 }).max(0.0)
    } else {
        pos.y
    };

    iced::widget::pin(panel).x(pos.x).y(y).into()
}

fn playlist_menu<'a>(app: &'a AuroraApp, playlist_id: uuid::Uuid, title: &str, pos: Point) -> Element<'a, Message> {
    let palette = app.palette;
    let items = column![
        menu_item(palette, "Rename".into(), true, Message::RenamePlaylistStart(playlist_id, title.to_string())),
        menu_item(palette, "Delete Playlist".into(), true, Message::DeletePlaylist(playlist_id)),
    ];
    let panel = menu_panel(palette, MENU_WIDTH, 90.0, items.into());
    iced::widget::pin(panel).x(pos.x).y(pos.y).into()
}

fn queue_menu<'a>(app: &'a AuroraApp, index: usize, pos: Point) -> Element<'a, Message> {
    let palette = app.palette;
    let items = column![
        menu_item(palette, "Move to Front".into(), true, Message::QueueMoveToFront(index)),
        menu_item(palette, "Remove".into(), true, Message::QueueRemove(index)),
    ];
    let panel = menu_panel(palette, 160.0, 90.0, items.into());
    iced::widget::pin(panel).x(pos.x).y(pos.y).into()
}
