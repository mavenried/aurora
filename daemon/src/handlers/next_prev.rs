use crate::{helpers::send_to_client, types::*};
use aurora_protocol::Response;

pub async fn next(stream: &WriteSocket, state: &State, n: usize) -> anyhow::Result<()> {
    let mut state = state.lock().await;
    state.next(n).await;
    state.add().await;
    let message = format!("Next {n} song(s).");
    tracing::info!("{message}");
    send_to_client(stream, &Response::Status(state.to_status())).await?;
    Ok(())
}

pub async fn prev(stream: &WriteSocket, state: &State, n: usize) -> anyhow::Result<()> {
    let mut state = state.lock().await;
    state.prev(n).await;
    state.add().await;
    let message = format!("Prev {n} song(s).");
    tracing::info!("{message}");
    send_to_client(stream, &Response::Status(state.to_status())).await?;
    Ok(())
}
