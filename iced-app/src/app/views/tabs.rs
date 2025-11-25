use iced::{
    Element,
    Length::{Fill, Fixed},
    widget::{column, container},
};

use crate::{
    app::{AuroraPlayer, views::PADDING},
    types::{MainView, Message},
};

impl AuroraPlayer {
    pub fn tabview(&self) -> Element<'_, Message> {
        let tabs = column![
            Self::new_tab_button("Search", self.current_mainview == MainView::Search),
            Self::new_tab_button("Playlists", self.current_mainview == MainView::AllPlaylist),
        ]
        .spacing(PADDING);
        container(tabs)
            .padding(PADDING)
            .height(Fill)
            .style(Self::dark_box)
            .height(Fixed(100.))
            .width(Fill)
            .into()
    }
}
