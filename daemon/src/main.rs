use daemonize::Daemonize;
use signal_hook::{consts::TERM_SIGNALS, iterator::Signals};
use std::{
    collections::VecDeque,
    env::args,
    fs::{File, remove_file},
    path::PathBuf,
    sync::Arc,
    vec,
};
use tokio::{net::UnixListener, sync::Mutex};

mod handlers;
mod helpers;
mod types;

mod mpris_thread;
mod watcher_thread;

use types::*;

const PIDFILE: &str = "/tmp/aurora-daemon.pid";
const SOCKFILE: &str = "/tmp/aurora-daemon.sock";
const OUTFILE: &str = "/tmp/aurora-daemon.out";

fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt::init();

    if args().nth(1).is_none() {
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

    rt.block_on(async_main())
}

async fn async_main() -> std::io::Result<()> {
    std::thread::spawn(move || {
        let mut signals = Signals::new(TERM_SIGNALS).unwrap();

        #[allow(clippy::never_loop)]
        for sig in signals.forever() {
            tracing::info!("Received signal {:?}, cleaning up SOCK file.", sig);
            remove_file(SOCKFILE).ok();
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
        current_song: None,
        queue: VecDeque::new(),
        clients: vec![],
        index,
        sink: Arc::new(sink),
        audio: None,
    }));

    let state_clone = state.clone();
    tokio::spawn(async move { watcher_thread::init(state_clone).await });
    let state_clone = state.clone();
    tokio::spawn(async move { mpris_thread::init(state_clone).await });

    let path = PathBuf::from(SOCKFILE);
    let listener = UnixListener::bind(path).unwrap_or_else(|err| {
        tracing::error!("Error: {err}");
        std::process::exit(1)
    });
    tracing::info!("{listener:?}");

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
