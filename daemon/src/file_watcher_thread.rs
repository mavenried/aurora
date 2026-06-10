use crate::{helpers, types::State};
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

pub async fn init(state: State) {
    let music_dir = dirs::home_dir().unwrap().join("Music");

    let (tx, mut rx) = mpsc::channel::<notify::Result<notify::Event>>(100);

    let mut watcher = match RecommendedWatcher::new(
        move |res| {
            let _ = tx.blocking_send(res);
        },
        Config::default(),
    ) {
        Ok(w) => w,
        Err(e) => {
            tracing::error!("File watcher init failed: {e}");
            return;
        }
    };

    if let Err(e) = watcher.watch(&music_dir, RecursiveMode::Recursive) {
        tracing::error!("File watcher watch failed for {music_dir:?}: {e}");
        return;
    }

    tracing::info!("File watcher watching {music_dir:?}");

    let mut last_event = Instant::now();
    let debounce = Duration::from_secs(2);
    let mut pending = false;

    loop {
        // Drain events with a short timeout to implement debounce
        match tokio::time::timeout(Duration::from_millis(500), rx.recv()).await {
            Ok(Some(Ok(event))) => {
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                        tracing::debug!("File watcher event: {:?}", event.kind);
                        last_event = Instant::now();
                        pending = true;
                    }
                    _ => {}
                }
            }
            Ok(Some(Err(e))) => {
                tracing::warn!("File watcher error: {e}");
            }
            Ok(None) => {
                tracing::info!("File watcher channel closed.");
                return;
            }
            Err(_) => {
                // Timeout — check if we should rescan
            }
        }

        if pending && last_event.elapsed() >= debounce {
            pending = false;
            tracing::info!("File watcher: rescanning music directory…");
            let db = state.lock().await.db.clone();
            match helpers::build_index(&music_dir, &db).await {
                Ok(new_index) => {
                    state.lock().await.index = new_index;
                    tracing::info!("File watcher: index updated.");
                }
                Err(e) => {
                    tracing::error!("File watcher: rescan error: {e}");
                }
            }
        }
    }
}
