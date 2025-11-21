use crate::types::*;
use std::time::Duration;
use tokio::time::sleep;

pub async fn init(state: State) {
    tracing::info!("Watcher thread started.");
    loop {
        sleep(Duration::from_millis(100)).await;

        let mut state = state.lock().await;
        if state.sink.empty() {
            state.next(1).await;
            state.add().await;
        }
    }
}
