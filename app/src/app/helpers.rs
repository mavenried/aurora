use iced::{
    Alignment, Background, Border, Color, Element,
    Length::Fill,
    Theme,
    widget::{button, button::Status, container, text},
};

use crate::types::{MainView, Message};

pub fn dark_box(theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.1))),
        border: Border {
            radius: 10.into(),
            ..Default::default()
        },
        ..container::rounded_box(theme)
    }
}
pub fn new_tab_button(label: &str, active: bool) -> Element<'_, Message> {
    button(
        text(label)
            .size(20)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .width(Fill),
    )
    .width(Fill)
    .height(Fill)
    .style(move |_theme, status| button::Style {
        background: match status {
            Status::Active => {
                if active {
                    Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.1)))
                } else {
                    Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2)))
                }
            }
            Status::Hovered => Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
            Status::Disabled => Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
            Status::Pressed => Some(Background::Color(Color::from_rgb(0.3, 0.3, 0.3))),
        },
        text_color: Color::from_rgb(0.8, 0.8, 0.8),
        border: Border {
            radius: 10.into(),
            ..Default::default()
        },
        ..Default::default()
    })
    .on_press(if label == "Playlists" {
        Message::MainViewSelect(MainView::AllPlaylist)
    } else {
        Message::MainViewSelect(MainView::Search)
    })
    .into()
}
