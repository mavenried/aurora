mod app;
mod daemon;
mod icons;
mod model;
mod theme;
mod ui;

use app::AuroraApp;

pub const DEFAULT_ART: &[u8] = include_bytes!("../../app/assets/placeholder.png");
const NOTO_SANS: &[u8] = include_bytes!("../../app/assets/NotoSans.ttf");
const JETBRAINS_MONO: &[u8] = include_bytes!("../../app/assets/JetbrainsMono.ttf");

fn main() -> iced::Result {
    tracing_subscriber::fmt::init();

    iced::application(AuroraApp::boot, AuroraApp::update, AuroraApp::view)
        .subscription(AuroraApp::subscription)
        .title(AuroraApp::title)
        .theme(AuroraApp::theme)
        .font(NOTO_SANS)
        .font(JETBRAINS_MONO)
        .default_font(theme::BODY_FONT)
        .window(iced::window::Settings {
            min_size: Some(iced::Size::new(1155.0, 650.0)),
            size: iced::Size::new(1422.0, 800.0),
            ..Default::default()
        })
        .run()
}
