use aurora_protocol::Song;
use iced::{
    Alignment, Element,
    Length::{Fill, Fixed},
    widget::{column, container, image, row, scrollable, text},
};

use crate::{
    app::{AuroraPlayer, views::PADDING},
    types::Message,
};

impl AuroraPlayer {
    pub fn queueview(&self) -> Element<'_, Message> {
        let mut queue = column![].spacing(PADDING).width(Fill);

        let idx = self.status.current_idx;
        let mut display_queue: Vec<&Song> = self.queue.iter().skip(idx + 1).collect();
        display_queue.extend(self.queue.iter().take(idx));
        display_queue.reverse();

        for song in display_queue {
            let handle = self
                .artcache
                .get(&song.id)
                .unwrap_or(self.default_album_art.clone());
            let image = container(image(handle).height(Fill)).padding(PADDING);
            let info = container(row![
                image,
                container(
                    column![
                        text(Self::truncate(&song.title, 25)).size(24),
                        text(Self::truncate(&song.artists.join(", "), 40)).size(16)
                    ]
                    .spacing(PADDING)
                    .align_x(Alignment::Start)
                    .width(Fill)
                )
                .align_y(Alignment::Center)
                .height(Fill)
            ])
            .align_x(Alignment::Start)
            .width(Fill)
            .height(Fixed(100.));

            queue = queue.push(info);
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
        .height(Fill)
        .width(Fill)
        .into()
    }
}
