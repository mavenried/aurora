use iced::{Element, Length::Fill, widget::container};

use crate::{
    app::AuroraPlayer,
    types::{MainView, Message},
};

impl AuroraPlayer {
    pub fn mainview(&self) -> Element<'_, Message> {
        container(match self.current_mainview {
            MainView::Playlist => "PlaylistView",
            MainView::AllPlaylist => "AllPlaylistView",
            MainView::Search => "SearchView",
        })
        .width(Fill)
        .height(Fill)
        .style(Self::dark_box)
        .into()
    }
}
