use daemonize::Daemonize;
use signal_hook::{consts::TERM_SIGNALS, iterator::Signals};
use std::{
    fs::{File, remove_file},
    process::exit,
    sync::Arc,
};
use tokio::{net::TcpListener, sync::Mutex};

mod handlers;
mod helpers;
mod types;
mod watcher_thread;
use types::*;

const PIDFILE: &str = "/tmp/aurora-daemon.pid";
const OUTFILE: &str = "/tmp/aurora-daemon.out";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let stdout = File::create(OUTFILE).unwrap();
    let daemonize = Daemonize::new().pid_file(PIDFILE).stdout(stdout);

    match daemonize.start() {
        Ok(_) => tracing::info!("Daemon started successfully"),
        Err(e) => {
            tracing::error!("Error starting daemon: {}", e);
            exit(1)
        }
    }
    std::thread::spawn(move || {
        let mut signals = Signals::new(TERM_SIGNALS).unwrap();

        #[allow(clippy::never_loop)]
        for sig in signals.forever() {
            tracing::info!("Received signal {:?}, cleaning up PID file.", sig);
            remove_file(PIDFILE).ok();
            std::process::exit(0);
        }
    });
    let Ok(port) = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "4400".to_string())
        .parse::<u16>()
    else {
        tracing::error!("Could not parse port.");
        std::process::exit(1)
    };

    tracing::info!("Binding to port {port}.");

    let Ok(stream_handle) = rodio::OutputStreamBuilder::open_default_stream() else {
        tracing::error!("Could not open a rodio output stream.");
        std::process::exit(1);
    };
    let sink = rodio::Sink::connect_new(stream_handle.mixer());

    helpers::generate_index(&dirs::home_dir().unwrap().join("Music")).await?;
    let index = helpers::load_index().await?;
    let state = Arc::new(Mutex::new(StateStruct {
        current_idx: 0,
        current_song: None,
        queue: Vec::new(),
        index,
        sink: Arc::new(sink),
        audio: None,
    }));

    let state_clone = state.clone();
    tokio::spawn(async move { watcher_thread::init(state_clone).await });

    let Ok(listener) = TcpListener::bind(format!("0.0.0.0:{port}")).await else {
        tracing::error!("Could not bind to port {port}.");
        std::process::exit(1)
    };
    loop {
        let (socket, addr) = listener.accept().await?;
        tracing::info!("New client: {:?}", addr);
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) = handlers::handle_client(socket, state_clone).await {
                tracing::error!("Error with client {:?}: {:?}", addr, e);
            }
        });
    }
}
