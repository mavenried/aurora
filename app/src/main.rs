slint::include_modules!();

mod interface;
mod types;

const DEFAULT_ART: &[u8] = include_bytes!("../../assets/placeholder.png");

#[tokio::main]
async fn main() -> Result<(), slint::PlatformError> {
    tracing_subscriber::fmt::init();
    let aurora_player = AuroraPlayer::new()?;

    if let Err(err) = interface::interface(aurora_player.as_weak()).await {
        tracing::error!("Could not initialize interface: {err}");
        std::process::exit(1);
    }

    aurora_player.run()
}
