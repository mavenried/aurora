use iced::widget::{button, row, text};
use iced::{Alignment, Element, Length};

use crate::app::{AuroraApp, Message};
use crate::icons;
use crate::model::Tab;
use crate::ui::{icon_text, pill_style};

pub fn view(app: &AuroraApp) -> Element<'_, Message> {
    let palette = app.palette;

    let tab_button = |glyph: &'static str, label: &'static str, tab: Tab| {
        let active = app.tab == tab;
        let bg_color = if active { palette.bgd1 } else { palette.bgd2 };
        let fg = if active { palette.acct } else { palette.txt1 };
        button(
            row![icon_text(glyph, 15.0, fg), text(label).size(15).color(fg)]
                .spacing(8)
                .align_y(Alignment::Center),
        )
        .padding([10, 20])
        .style(pill_style(bg_color, fg))
        .on_press(Message::TabSelected(tab))
    };

    row![
        tab_button(icons::SEARCH, "Search", Tab::Search),
        tab_button(icons::LIBRARY, "Library", Tab::Library),
    ]
    .spacing(8)
    .height(Length::Fixed(60.0))
    .padding(8)
    .into()
}
