use iced::application;

use crate::app::AuroraPlayer;

mod app;
mod tasks;
mod types;

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();
    application("Aurora", app::AuroraPlayer::update, app::AuroraPlayer::view)
        .theme(|_player| iced::theme::Theme::Dark)
        .subscription(AuroraPlayer::subscription)
        .run_with(app::AuroraPlayer::new)
}
