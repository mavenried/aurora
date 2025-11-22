use super::helpers::*;
use crate::{app::AuroraPlayer, types::*};
use iced::{
    Alignment, Background, Color, Element,
    Length::{Fill, FillPortion, Fixed},
    widget::{column, container, image, row, text},
};

const SPACING: u16 = 5;
const PADDING: u16 = 5;

impl AuroraPlayer {
    pub fn view(&self) -> Element<'_, Message> {
        let queue = column![].spacing(SPACING).width(Fill);
        let tabs = column![
            new_tab_button("Search", self.current_mainview == MainView::Search),
            new_tab_button("Playlists", self.current_mainview == MainView::AllPlaylist),
        ]
        .spacing(SPACING);
        let (current_song, current_artist, image_handle) = {
            if let Some(current_song) = self.status.current_song.clone() {
                let img = self
                    .artcache
                    .get(&current_song.id)
                    .unwrap_or(self.default_album_art.clone());
                (current_song.title, current_song.artists.join(" ,"), img)
            } else {
                (
                    "Nothing Playing".to_string(),
                    "".to_string(),
                    self.default_album_art.clone(),
                )
            }
        };

        let np_image = container(image(image_handle).height(Fill)).padding(PADDING);
        let np_info = container(
            column![text(current_song).size(24), text(current_artist).size(16)]
                .spacing(SPACING)
                .align_x(Alignment::Center),
        )
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .width(Fill)
        .height(Fill);
        let now_playing = container(row![np_image, np_info].spacing(SPACING))
            .style(dark_box)
            .padding(PADDING);
        let tabbar = container(tabs)
            .padding(PADDING)
            .height(Fill)
            .style(dark_box);
        let sidebar = container(queue)
            .padding(PADDING)
            .height(Fill)
            .style(dark_box);
        let mainview = container(match self.current_mainview {
            MainView::Playlist => "PlaylistView",
            MainView::AllPlaylist => "AllPlaylistView",
            MainView::Search => "SearchView",
        })
        .width(Fill)
        .height(Fill)
        .style(dark_box);
        let playerview = container("Playerview")
            .width(Fill)
            .height(Fill)
            .style(dark_box);
        container(
            row![
                column![
                    tabbar.height(Fixed(100.)).width(Fill),
                    sidebar.height(Fill).width(Fill),
                    now_playing.height(Fixed(100.)).width(Fill)
                ]
                .spacing(SPACING)
                .width(Fixed(450.)),
                column![
                    row![mainview.width(Fill),].spacing(SPACING).height(Fill),
                    playerview.height(Fixed(100.))
                ]
                .spacing(SPACING)
                .width(FillPortion(4))
            ]
            .spacing(SPACING),
        )
        .style(|theme| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
            ..container::rounded_box(theme)
        })
        .padding(PADDING)
        .into()
    }
}
