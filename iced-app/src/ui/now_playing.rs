use iced::widget::{column, container, image, row, text};
use iced::{Alignment, ContentFit, Element, Length};

use crate::app::{AuroraApp, Message};
use crate::icons;
use crate::model::art_handle;
use crate::ui::{bg, icon_text};

pub fn mini_card(app: &AuroraApp) -> Element<'_, Message> {
    let palette = app.palette;

    let (art_el, title, artists): (Element<'_, Message>, String, String) = match &app.state.current_song {
        Some(song) => (
            image(art_handle(&song.art_path))
                .width(Length::Fixed(80.0))
                .height(Length::Fixed(80.0))
                .content_fit(ContentFit::Cover)
                .into(),
            song.title.clone(),
            song.artists.join(", "),
        ),
        None => (
            container(icon_text(icons::NOTE_PLACEHOLDER, 28.0, palette.txt2))
                .width(Length::Fixed(80.0))
                .height(Length::Fixed(80.0))
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into(),
            "Nothing Playing".to_string(),
            "No Artist".to_string(),
        ),
    };

    let text_col = column![
        text(title).size(15).color(palette.txt1),
        text(artists).size(13).color(palette.txt2),
    ]
    .spacing(4)
    .width(Length::Fill);

    container(
        row![art_el, text_col]
            .spacing(12)
            .align_y(Alignment::Center)
            .padding(10),
    )
    .width(Length::Fixed(400.0))
    .height(Length::Fixed(100.0))
    .style(bg(palette.bgd1))
    .into()
}
