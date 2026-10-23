use daemonize::Daemonize;
use signal_hook::{consts::TERM_SIGNALS, iterator::Signals};
use std::{
    collections::VecDeque,
    env::args,
    fs::{File, remove_file},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
    vec,
};
use tokio::{net::UnixListener, sync::Mutex};

mod handlers;
mod helpers;
mod types;

mod file_watcher_thread;
mod mpris_thread;
mod theme_thread;
mod watcher_thread;

use types::*;

const PIDFILE: &str = "/tmp/aurora-daemon.pid";
const SOCKFILE: &str = "/tmp/aurora-daemon.sock";
const OUTFILE: &str = "/tmp/aurora-daemon.out";
const IDLE_TIMEOUT: Duration = Duration::from_secs(5 * 60);

fn is_inactive(clients_empty: bool, queue_empty: bool, paused: bool) -> bool {
    clients_empty && (queue_empty || paused)
}

fn main() -> std::io::Result<()> {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("aurora_daemon=debug,info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    if args().nth(1).is_none() {
        let stdout = File::create(OUTFILE)?;
        let stderr = stdout.try_clone()?;
        let daemonize = Daemonize::new()
            .pid_file(PIDFILE)
            .stdout(stdout)
            .stderr(stderr);

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
    tracing::info!("Aurora daemon starting");
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

    // Rodio's default callback prints every backend error to stderr. ALSA can
    // report POLLERR continuously after the machine resumes from suspend,
    // which otherwise grows the daemon log until /tmp fills up.
    let audio_error_reported = Arc::new(AtomicBool::new(false));
    let audio_error_reported_callback = audio_error_reported.clone();
    let Ok(stream_handle) = rodio::DeviceSinkBuilder::from_default_device().and_then(|builder| {
        builder
            .with_error_callback(move |err| {
                if !audio_error_reported_callback.swap(true, Ordering::Relaxed) {
                    tracing::error!(error = %err, "Audio output stream error; suppressing repeated backend errors");
                }
            })
            .open_sink_or_fallback()
    }) else {
        tracing::error!("Could not open a rodio output stream.");
        std::process::exit(1);
    };
    let sink = rodio::Player::connect_new(stream_handle.mixer());

    let db = helpers::db::open()?;

    let music_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/root"))
        .join("Music");
    let index_started = Instant::now();
    let index = helpers::build_index(&music_dir, &db).await?;
    tracing::info!(
        songs = index.len(),
        elapsed_ms = index_started.elapsed().as_millis(),
        "Initial music index ready"
    );
    let recently_played = helpers::load_history(&db).await.unwrap_or_default();
    let liked_ids = helpers::load_liked(&db).await.unwrap_or_default();

    let state: State = Arc::new_cyclic(|self_handle| {
        Mutex::new(StateStruct {
            current_song: None,
            queue: VecDeque::new(),
            clients: vec![],
            index,
            sink: Arc::new(sink),
            audio: None,
            theme: theme_thread::get_config(),
            volume: 1.0,
            shuffle: false,
            repeat: 0,
            recently_played,
            liked_ids,
            self_handle: self_handle.clone(),
            db,
        })
    });

    let state_clone = state.clone();
    tokio::spawn(async move {
        tracing::info!("Playback watcher task starting");
        watcher_thread::init(state_clone).await;
    });

    let state_clone = state.clone();
    tokio::spawn(async move { file_watcher_thread::init(state_clone).await });

    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Err(err) = mpris_thread::init(state_clone).await {
            tracing::error!(error = %err, "MPRIS worker stopped");
        }
    });

    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Err(err) = theme_thread::init(state_clone).await {
            tracing::error!(error = %err, "Theme watcher stopped");
        }
    });

    let idle_state = state.clone();
    tokio::spawn(async move {
        let mut inactive_since = Instant::now();
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let inactive = {
                let state = idle_state.lock().await;
                is_inactive(
                    state.clients.is_empty(),
                    state.queue.is_empty(),
                    state.is_paused(),
                )
            };
            if inactive {
                if inactive_since.elapsed() >= IDLE_TIMEOUT {
                    tracing::info!(
                        "No clients or active playback for five minutes; shutting down."
                    );
                    remove_file(SOCKFILE).ok();
                    remove_file(PIDFILE).ok();
                    std::process::exit(0);
                }
            } else {
                inactive_since = Instant::now();
            }
        }
    });

    let path = PathBuf::from(SOCKFILE);
    // A stale socket file from a prior ungraceful exit (crash, SIGKILL) would
    // otherwise make bind() fail forever, since nothing else ever unlinks it.
    let _ = std::fs::remove_file(&path);
    let listener = UnixListener::bind(&path).unwrap_or_else(|err| {
        tracing::error!("Error: {err}");
        std::process::exit(1)
    });
    tracing::info!(socket = %path.display(), "Daemon listening for client connections");

    loop {
        let (socket, addr) = match listener.accept().await {
            Ok(accepted) => accepted,
            Err(err) => {
                tracing::error!(error = %err, "Accept failed; retrying");
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }
        };
        let (reader, w) = socket.into_split();
        let writer = Arc::new(Mutex::new(w));
        let client_id = Arc::as_ptr(&writer) as usize;

        {
            state.lock().await.clients.push(writer.clone());
        }

        tracing::info!(client_id, peer = ?addr, "Client connected");
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
            tracing::info!(client_id, peer = ?addr, "Client disconnected and state cleaned up");
        });
    }
}

#[cfg(test)]
mod tests {
    use super::is_inactive;

    #[test]
    fn connected_clients_keep_daemon_active() {
        assert!(!is_inactive(false, true, true));
        assert!(!is_inactive(false, false, false));
    }

    #[test]
    fn active_playback_keeps_disconnected_daemon_alive() {
        assert!(!is_inactive(true, false, false));
    }

    #[test]
    fn no_clients_and_no_playback_is_inactive() {
        assert!(is_inactive(true, true, false));
        assert!(is_inactive(true, false, true));
    }
}
