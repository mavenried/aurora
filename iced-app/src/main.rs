use iced::{Font, application};

use crate::app::AuroraPlayer;

mod app;
mod types;

fn main() -> iced::Result {
    let noto = include_bytes!("../assets/NotoSansJP-Regular.ttf").as_slice();
    tracing_subscriber::fmt::init();
    application("Aurora", app::AuroraPlayer::update, app::AuroraPlayer::view)
        .theme(|_player| iced::theme::Theme::Dark)
        .subscription(AuroraPlayer::subscription)
        .font(noto)
        .default_font(Font::with_name("Noto Sans JP"))
        .run_with(app::AuroraPlayer::new)
}
