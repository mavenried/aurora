pub mod context_menu;
pub mod library;
pub mod now_playing;
pub mod player;
pub mod queue;
pub mod search;
pub mod song_list;
pub mod tabs;

use iced::widget::{button, column, container, mouse_area, row, space, stack, text};
use iced::{Border, Color, Element, Length, Theme as IcedTheme};

use crate::app::{AuroraApp, Message};
use crate::icons;
use crate::model::Tab;

pub fn bg(color: Color) -> impl Fn(&IcedTheme) -> container::Style {
    move |_theme| container::Style::default().background(color)
}

pub fn rounded_bg(color: Color, radius: f32) -> impl Fn(&IcedTheme) -> container::Style {
    move |_theme| {
        container::Style::default()
            .background(color)
            .border(iced::Border {
                radius: radius.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            })
    }
}

pub fn icon_text<'a>(glyph: &'a str, size: f32, color: Color) -> Element<'a, Message> {
    text(glyph)
        .font(icons::ICON_FONT)
        .size(size)
        .color(color)
        .into()
}

/// A pill-shaped, filled button (used for "Clear", "Play All", confirm/cancel, tabs...).
pub fn pill_style(bg_color: Color, fg: Color) -> impl Fn(&IcedTheme, button::Status) -> button::Style {
    move |_theme, status| {
        let bg_color = match status {
            button::Status::Hovered | button::Status::Pressed => Color {
                a: (bg_color.a + 0.15).min(1.0),
                ..bg_color
            },
            _ => bg_color,
        };
        button::Style {
            background: Some(bg_color.into()),
            text_color: fg,
            border: Border::default().rounded(999.0),
            ..button::Style::default()
        }
    }
}

/// A circular, transparent-by-default icon button (used for transport controls).
pub fn circle_icon_style(active: bool, accent: Color, txt: Color, hover_bg: Color) -> impl Fn(&IcedTheme, button::Status) -> button::Style {
    move |_theme, status| {
        let background = match status {
            button::Status::Hovered | button::Status::Pressed => Some(hover_bg.into()),
            _ => None,
        };
        button::Style {
            background,
            text_color: if active { accent } else { txt },
            border: Border::default().rounded(999.0),
            ..button::Style::default()
        }
    }
}

pub fn root(app: &AuroraApp) -> Element<'_, Message> {
    let left = column![queue::view(app), now_playing::mini_card(app)]
        .width(Length::Fixed(400.0))
        .spacing(5);

    let main_view: Element<'_, Message> = match app.tab {
        Tab::Search => search::view(app),
        Tab::Library => library::view(app),
    };

    let right = column![tabs::view(app), main_view, player::view(app)]
        .spacing(5)
        .width(Length::Fill)
        .height(Length::Fill);

    let base = container(row![left, right].spacing(5).padding(5))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(bg(app.palette.bgd0));

    let mut layers: Vec<Element<'_, Message>> = vec![base.into()];

    if let Some(menu) = &app.context_menu {
        layers.push(
            mouse_area(space().width(Length::Fill).height(Length::Fill))
                .on_press(Message::CloseContextMenu)
                .on_right_press(Message::CloseContextMenu)
                .into(),
        );
        layers.push(context_menu::view(app, menu));
    }

    if !app.connected {
        layers.push(disconnected_overlay(app));
    }

    stack(layers).width(Length::Fill).height(Length::Fill).into()
}

fn disconnected_overlay(app: &AuroraApp) -> Element<'_, Message> {
    let card = container(
        column![
            text("Disconnected from daemon")
                .size(20)
                .color(app.palette.txt1),
            text("Attempting to reconnect...")
                .size(14)
                .color(app.palette.txt2),
        ]
        .spacing(8)
        .align_x(iced::Alignment::Center),
    )
    .padding(24)
    .width(Length::Fixed(420.0))
    .height(Length::Fixed(160.0))
    .align_x(iced::Alignment::Center)
    .align_y(iced::Alignment::Center)
    .style(rounded_bg(app.palette.bgd1, 12.0));

    container(card)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::Alignment::Center)
        .align_y(iced::Alignment::Center)
        .style(bg(Color::from_rgba8(0, 0, 0, 0.8)))
        .into()
}
