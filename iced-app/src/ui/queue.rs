use aurora_protocol::Song;
use iced::widget::{button, column, container, image, mouse_area, row, scrollable, space, text};
use iced::{Alignment, Color, ContentFit, Element, Length};

use crate::app::{AuroraApp, Message};
use crate::icons;
use crate::model::art_handle;
use crate::ui::{bg, icon_text, pill_style, song_list::empty_state};

const ROW_HEIGHT: f32 = 97.0;

pub fn view(app: &AuroraApp) -> Element<'_, Message> {
    let palette = app.palette;

    let header = row![
        text("Queue").size(18).color(palette.txt1),
        space().width(Length::Fill),
        button(text("Clear").size(13))
            .padding([6, 14])
            .on_press(Message::QueueClear)
            .style(pill_style(palette.bgd3, palette.txt1)),
    ]
    .align_y(Alignment::Center)
    .padding(10);

    let display: Vec<&Song> = app.state.queue.iter().skip(1).collect();

    let body: Element<'_, Message> = if display.is_empty() {
        empty_state(app, icons::NOTE_PLACEHOLDER, "Queue is empty", "Add songs to get started")
    } else {
        let rows = display.into_iter().enumerate().map(|(i, song)| queue_row(app, i, song));
        scrollable(column(rows)).height(Length::Fill).width(Length::Fill).into()
    };

    container(column![header, body])
        .width(Length::Fixed(400.0))
        .height(Length::Fill)
        .style(bg(palette.bgd1))
        .into()
}

fn queue_row<'a>(app: &'a AuroraApp, index: usize, song: &'a Song) -> Element<'a, Message> {
    let palette = app.palette;
    let hovered = app.hovered_queue == Some(index);
    let dragging = app.queue_drag.as_ref().is_some_and(|d| d.from == index);
    let is_drop_target = app.queue_drag.as_ref().is_some_and(|d| d.drop_target == index);

    let handle_color = if dragging { palette.acct } else { palette.txt2 };
    let drag_handle = mouse_area(
        container(icon_text(icons::DRAG_HANDLE, 18.0, handle_color))
            .width(Length::Fixed(40.0))
            .align_x(Alignment::Center),
    )
    .on_press(Message::QueueDragStart(index));

    let art = image(art_handle(&song.art_path))
        .width(Length::Fixed(60.0))
        .height(Length::Fixed(60.0))
        .content_fit(ContentFit::Cover);

    let title_col = column![
        text(song.title.clone()).size(14).color(palette.txt1),
        text(song.artists.join(", ")).size(12).color(palette.txt2),
    ]
    .width(Length::Fill);

    let content = row![drag_handle, art, title_col]
        .spacing(10)
        .align_y(Alignment::Center)
        .padding([4, 10]);

    let drop_indicator = container(space().width(Length::Fill).height(Length::Fixed(3.0)))
        .style(bg(if is_drop_target { palette.acct } else { Color::TRANSPARENT }));

    let row_bg = if dragging {
        Color { a: 0.3, ..palette.bgd2 }
    } else if hovered {
        palette.bgd3
    } else {
        palette.bgd2
    };

    let clickable = mouse_area(
        container(content)
            .width(Length::Fill)
            .height(Length::Fixed(ROW_HEIGHT))
            .style(bg(row_bg)),
    )
    .on_press(Message::QueueClicked(index))
    .on_right_press(Message::QueueRightClicked(index))
    .on_enter(Message::QueueHovered(Some(index)))
    .on_exit(Message::QueueHovered(None));

    column![drop_indicator, clickable].into()
}
