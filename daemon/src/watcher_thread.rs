use crate::{helpers::send_to_all, types::*};
use aurora_protocol::{Response, Song};
use std::time::Duration;
use tokio::time::sleep;

pub async fn init(state: State) {
    tracing::info!("Watcher thread started.");
    loop {
        sleep(Duration::from_millis(100)).await;
        let mut state_locked = state.lock().await;
        if state_locked.sink.empty() && !state_locked.queue.is_empty() {
            state_locked.next(1).await;
            state_locked.add().await;

            let queue = state_locked.queue.clone().iter().map(Song::from).collect();
            drop(state_locked);
            let _ = send_to_all(&state, &Response::Queue(queue)).await;
            tracing::debug!("Auto Next")
        }
    }
}
