use crate::{
    helpers::{send_to_all, send_to_client},
    types::*,
};
use aurora_protocol::{Response, Song};

pub async fn next(stream: &WriteSocket, state: &State, n: usize) -> anyhow::Result<()> {
    let mut state_locked = state.lock().await;
    state_locked.next(n).await;
    state_locked.add().await;
    let message = format!("Next {n} song(s).");
    tracing::info!("{message}");
    send_to_client(stream, &Response::Status(state_locked.to_status())).await?;

    let queue = state_locked.queue.iter().map(Song::from).collect();
    drop(state_locked);
    let _ = send_to_all(state, &Response::Queue(queue)).await;

    Ok(())
}

pub async fn prev(stream: &WriteSocket, state: &State, n: usize) -> anyhow::Result<()> {
    let mut state_locked = state.lock().await;
    state_locked.prev(n).await;
    state_locked.add().await;
    let message = format!("Prev {n} song(s).");
    tracing::info!("{message}");
    send_to_client(stream, &Response::Status(state_locked.to_status())).await?;

    let queue = state_locked.queue.iter().map(Song::from).collect();
    drop(state_locked);
    let _ = send_to_all(state, &Response::Queue(queue)).await;
    Ok(())
}
