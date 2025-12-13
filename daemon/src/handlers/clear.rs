use crate::{helpers::send_to_all, types::*};
use aurora_protocol::{Response, Song};

pub async fn clear(state: &State) -> anyhow::Result<()> {
    let mut state_locked = state.lock().await;
    state_locked.clear().await;
    let message = String::from("Queue Cleared.");
    tracing::info!("{message}");

    let resp = Response::Queue(state_locked.queue.iter().map(Song::from).collect());
    drop(state_locked);
    send_to_all(state, &resp).await?;

    Ok(())
}
