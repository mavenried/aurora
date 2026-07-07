use iced::widget::{container, pick_list, row, text_input};
use iced::{Alignment, Element, Length};

use crate::app::{AuroraApp, Message};
use crate::icons;
use crate::model::SearchMode;
use crate::ui::{bg, icon_text, song_list};

pub fn view(app: &AuroraApp) -> Element<'_, Message> {
    let palette = app.palette;

    let search_bar = container(
        row![
            icon_text(icons::SEARCH, 18.0, palette.srch),
            text_input("Search...", &app.search_query)
                .on_input(Message::SearchQueryChanged)
                .on_submit(Message::SearchSubmit)
                .size(16)
                .width(Length::Fill),
        ]
        .spacing(10)
        .align_y(Alignment::Center)
        .padding(12),
    )
    .style(bg(palette.bgd1))
    .width(Length::Fill);

    let mode_picker = pick_list(SearchMode::ALL, Some(app.search_mode), Message::SearchModeChanged).width(Length::Fixed(160.0));

    let header = row![search_bar, mode_picker].spacing(10).padding(10).align_y(Alignment::Center);

    let body: Element<'_, Message> = if app.search_query.trim().is_empty() && app.state.search_results.is_empty() {
        song_list::empty_state(app, icons::SEARCH_BIG, "Search your library", "Find songs by title or artist")
    } else {
        song_list::song_list(app, &app.state.search_results, None)
    };

    container(iced::widget::column![header, body])
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
