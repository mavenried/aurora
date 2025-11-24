use daemonize::Daemonize;
use signal_hook::{consts::TERM_SIGNALS, iterator::Signals};
use std::{
    env::args,
    fs::{File, remove_file},
    sync::Arc,
    vec,
};
use tokio::{net::TcpListener, sync::Mutex};

mod handlers;
mod helpers;
mod types;
mod watcher_thread;
use types::*;

const PIDFILE: &str = "/tmp/aurora-daemon.pid";
const OUTFILE: &str = "/tmp/aurora-daemon.out";

fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    let Ok(port) = args()
        .nth(1)
        .unwrap_or_else(|| "4321".to_string())
        .parse::<u16>()
    else {
        tracing::error!("could not parse port.");
        std::process::exit(1)
    };

    if args().nth(2).is_none() {
        let daemonize = Daemonize::new()
            .pid_file(PIDFILE)
            .stdout(File::create(OUTFILE).unwrap());

        daemonize.start().unwrap_or_else(|err| {
            tracing::error!("Error {err}");
            std::process::exit(1)
        })
    }

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    rt.block_on(async_main(port))
}

async fn async_main(port: u16) -> std::io::Result<()> {
    tracing::info!("Binding to port {port}.");

    std::thread::spawn(move || {
        let mut signals = Signals::new(TERM_SIGNALS).unwrap();

        #[allow(clippy::never_loop)]
        for sig in signals.forever() {
            tracing::info!("Received signal {:?}, cleaning up PID file.", sig);
            remove_file(PIDFILE).ok();
            std::process::exit(0);
        }
    });
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
        clients: vec![],
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
        let (reader, w) = socket.into_split();
        let writer = Arc::new(Mutex::new(w));

        {
            state.lock().await.clients.push(writer.clone());
        }

        tracing::info!("New client: {:?}", addr);
        let state_clone = state.clone();
        tokio::spawn(async move {
            if let Err(e) =
                handlers::handle_client(reader, writer.clone(), state_clone.clone()).await
            {
                tracing::error!("Error with client {:?}: {:?}", addr, e);
            }

            let mut state_guard = state_clone.lock().await;
            if let Some(pos) = state_guard
                .clients
                .iter()
                .position(|c| Arc::ptr_eq(c, &writer))
            {
                state_guard.clients.swap_remove(pos);
            }
        });
    }
}
