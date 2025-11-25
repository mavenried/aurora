use aurora_protocol::Song;
use iced::{
    Alignment, Element,
    Length::{Fill, Fixed},
    widget::{column, container, image, lazy, row, scrollable, text},
};

use crate::{
    app::{AuroraPlayer, views::PADDING},
    types::Message,
};

impl AuroraPlayer {
    pub fn queueview(&self) -> Element<'_, Message> {
        let idx = self.status.current_idx;
        let mut display_queue: Vec<&Song> = self.queue.iter().skip(idx + 1).collect();
        display_queue.extend(self.queue.iter().take(idx));
        display_queue.reverse();

        let mut queue = column![].spacing(PADDING).width(Fill);

        for song in display_queue.iter() {
            // Use lazy widget - it only re-renders when the song ID changes
            let song_id = song.id;
            let song_clone = (*song).clone();
            let handle = self
                .artcache
                .get(&song.id)
                .unwrap_or(self.default_album_art.clone())
                .clone();

            let lazy_row = lazy(song_id, move |_| {
                Self::render_song_row_static(&song_clone, handle.clone())
            });

            queue = queue.push(lazy_row);
        }

        container(
            scrollable(queue)
                .anchor_bottom()
                .style(Self::hide_scroll_bar),
        )
        .padding(PADDING)
        .align_y(Alignment::End)
        .height(Fill)
        .style(Self::dark_box)
        .width(Fill)
        .into()
    }

    // Static version that doesn't need &self
    fn render_song_row_static(
        song: &Song,
        album_art_handle: iced::widget::image::Handle,
    ) -> Element<'static, Message> {
        let image_widget = container(image(album_art_handle).height(Fill)).padding(PADDING);

        container(row![
            image_widget,
            container(
                column![
                    text(Self::truncate(&song.title, 25)).size(24),
                    text(Self::truncate(&song.artists.join(", "), 40)).size(16)
                ]
                .spacing(PADDING)
                .align_x(Alignment::Center)
                .width(Fill)
            )
            .align_y(Alignment::Center)
            .height(Fill)
        ])
        .align_x(Alignment::Center)
        .width(Fill)
        .height(Fixed(100.))
        .into()
    }
}
