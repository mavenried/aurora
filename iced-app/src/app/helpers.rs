use iced::widget::button::Status;
use iced::widget::scrollable::{self, Rail, Scroller};
use iced::{
    Alignment, Background, Border, Color, Element,
    Length::Fill,
    Theme,
    widget::{button, container, slider, text},
};

use crate::{
    app::AuroraPlayer,
    types::{MainView, Message},
};

impl AuroraPlayer {
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
                        Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2)))
                    } else {
                        Some(Background::Color(Color::from_rgb(0.1, 0.1, 0.1)))
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

    pub fn progress_slider(&self) -> Element<'_, Message> {
        let total_secs = if let Some(song) = &self.status.current_song {
            song.duration.as_millis() as f32
        } else {
            0.
        };
        let secs = self.status.position.as_millis() as f32;
        slider(
            0.0..=total_secs,
            if self.slider_pressed {
                self.progress_slider_state
            } else {
                secs
            },
            Message::SliderChanged,
        )
        .on_release(Message::SeekCommit)
        .step(0.01)
        .into()
    }
    pub fn hide_scroll_bar(_theme: &Theme, _: scrollable::Status) -> scrollable::Style {
        scrollable::Style {
            container: container::Style {
                background: None,
                ..Default::default()
            },
            gap: None,
            horizontal_rail: Rail {
                background: None,
                border: Border::default(),
                scroller: Scroller {
                    color: Color::from_rgba(0., 0., 0., 0.),
                    border: Border::default(),
                },
            },
            vertical_rail: Rail {
                background: None,
                border: Border::default(),
                scroller: Scroller {
                    color: Color::from_rgba(0., 0., 0., 0.),
                    border: Border::default(),
                },
            },
        }
    }

    pub fn truncate(s: &str, max_len: usize) -> String {
        if s.chars().count() <= max_len {
            s.to_string()
        } else if max_len == 0 {
            String::new()
        } else if max_len == 1 {
            "…".to_string()
        } else {
            let truncated = s.chars().take(max_len - 1).collect::<String>();
            format!("{}…", truncated)
        }
    }
}
