use crate::{app::AuroraPlayer, types::*};
use iced::{
    Background, Color, Element,
    Length::{Fill, FillPortion, Fixed},
    widget::{column, container, row},
};

mod center;
mod now_playing;
mod player;
mod sidebar;
mod tabs;
const PADDING: u16 = 5;

impl AuroraPlayer {
    pub fn view(&self) -> Element<'_, Message> {
        let sidebar = self.queueview();
        let mainview = self.mainview();
        let now_playing = self.now_playing();
        let playerview = self.playerview();
        let tabview = self.tabview();

        container(
            row![
                column![tabview, sidebar, now_playing]
                    .spacing(PADDING)
                    .width(Fixed(450.)),
                column![row![mainview,].spacing(PADDING).height(Fill), playerview]
                    .spacing(PADDING)
                    .width(FillPortion(4))
            ]
            .spacing(PADDING),
        )
        .style(|theme| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.2, 0.2, 0.2))),
            ..container::rounded_box(theme)
        })
        .padding(PADDING)
        .into()
    }
}
