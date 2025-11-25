use iced::{
    Alignment, Element,
    Length::{Fill, Fixed},
    widget::{column, container, image, row, text},
};

use crate::{
    app::{AuroraPlayer, views::PADDING},
    types::Message,
};

impl AuroraPlayer {
    pub fn now_playing(&self) -> Element<'_, Message> {
        let (current_song, current_artist, image_handle) = {
            if let Some(current_song) = self.status.current_song.clone() {
                let img = self
                    .artcache
                    .get(&current_song.id)
                    .unwrap_or(self.default_album_art.clone());
                (
                    Self::truncate(&current_song.title, 25),
                    Self::truncate(&current_song.artists.join(" ,"), 40),
                    img,
                )
            } else {
                (
                    "Nothing Playing".to_string(),
                    "No Artists".to_string(),
                    self.default_album_art.clone(),
                )
            }
        };

        let np_image = container(image(image_handle).height(Fill)).padding(PADDING);
        let np_info = container(
            column![text(current_song).size(24), text(current_artist).size(16)]
                .spacing(PADDING)
                .align_x(Alignment::Center),
        )
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .width(Fill)
        .height(Fill);
        container(row![np_image, np_info].spacing(PADDING))
            .style(Self::dark_box)
            .padding(PADDING)
            .height(Fixed(100.))
            .width(Fill)
            .into()
    }
}
