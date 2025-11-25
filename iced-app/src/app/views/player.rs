use iced::{
    Element,
    Length::{Fill, Fixed},
    widget::container,
};

use crate::{app::AuroraPlayer, types::Message};

impl AuroraPlayer {
    pub fn playerview(&self) -> Element<'_, Message> {
        container(self.progress_slider())
            .width(Fill)
            .style(Self::dark_box)
            .height(Fixed(100.))
            .into()
    }
}
