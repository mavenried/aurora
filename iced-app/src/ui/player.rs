use iced::widget::{button, column, container, row, slider, space, text};
use iced::{Alignment, Element, Length};

use crate::app::{AuroraApp, Message};
use crate::icons;
use crate::model::format_duration;
use crate::ui::{bg, circle_icon_style, icon_text};

pub fn view(app: &AuroraApp) -> Element<'_, Message> {
    let palette = app.palette;

    let position = app
        .dragging_seek
        .unwrap_or(app.state.position)
        .as_millis() as f32;
    let duration_ms = app.state.duration.as_millis() as f32;
    let volume = app.dragging_volume.unwrap_or(app.state.volume);

    let is_liked = app
        .state
        .current_song
        .as_ref()
        .is_some_and(|s| app.state.is_liked(&s.id));

    let elapsed_text = text(format_duration(app.dragging_seek.unwrap_or(app.state.position)))
        .size(13)
        .color(palette.txt2)
        .width(Length::Fixed(80.0));

    let icon_button = |glyph: &'static str, size: f32, active: bool, msg: Message| {
        button(icon_text(glyph, size, if active { palette.acct } else { palette.btns }))
            .width(Length::Fixed(40.0))
            .height(Length::Fixed(40.0))
            .style(circle_icon_style(active, palette.acct, palette.btns, palette.bgd3))
            .on_press(msg)
    };

    let play_glyph = if app.state.is_paused { icons::PLAY } else { icons::PAUSE };
    let repeat_glyph = if app.state.repeat == 1 { icons::REPEAT_ON } else { icons::REPEAT_OFF };

    let transport = row![
        icon_button(icons::SHUFFLE, 18.0, app.state.shuffle, Message::ShuffleToggled),
        icon_button(icons::PREV, 20.0, false, Message::Prev),
        icon_button(play_glyph, 22.0, false, Message::Pause),
        icon_button(icons::NEXT, 20.0, false, Message::Next),
        icon_button(repeat_glyph, 18.0, app.state.repeat == 1, Message::RepeatToggled),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let heart_glyph = if is_liked { icons::HEART_FILLED } else { icons::HEART_OUTLINE };
    let like_button = button(icon_text(heart_glyph, 18.0, if is_liked { palette.acct } else { palette.btns }))
        .width(Length::Fixed(36.0))
        .height(Length::Fixed(36.0))
        .style(circle_icon_style(is_liked, palette.acct, palette.btns, palette.bgd3))
        .on_press_maybe(
            app.state
                .current_song
                .as_ref()
                .map(|s| Message::LikeToggle(s.id, is_liked)),
        );

    let volume_slider = slider(0.0..=1.0, volume, Message::VolumeChanged)
        .on_release(Message::VolumeReleased(volume))
        .width(Length::Fixed(80.0));

    let duration_text = text(format_duration(app.state.duration))
        .size(13)
        .color(palette.txt2)
        .width(Length::Fixed(80.0));

    let right_cluster = row![
        like_button,
        icon_text(icons::VOLUME, 16.0, palette.txt2),
        volume_slider,
        duration_text,
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .width(Length::Fixed(240.0));

    let controls_row = row![
        elapsed_text,
        space().width(Length::Fill),
        transport,
        space().width(Length::Fill),
        right_cluster,
    ]
    .align_y(Alignment::Center)
    .padding([0, 15]);

    let seek_range = if duration_ms > 0.0 { duration_ms } else { 1.0 };
    let seek: Element<'_, Message> = slider(0.0..=seek_range, position, |v: f32| Message::SeekChanged(v as u64))
        .on_release(Message::SeekReleased(position as u64))
        .width(Length::Fill)
        .into();

    container(
        column![controls_row, container(seek).padding([0, 10])]
            .spacing(8)
            .padding([10, 0]),
    )
    .width(Length::Fill)
    .height(Length::Fixed(100.0))
    .style(bg(palette.bgd1))
    .into()
}
