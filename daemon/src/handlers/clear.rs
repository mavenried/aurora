use crate::{helpers::send_to_client, types::*};
use aurora_protocol::Response;

pub async fn clear(stream: &WriteSocket, state: &State) -> anyhow::Result<()> {
    let mut state = state.lock().await;
    state.clear().await;
    let message = String::from("Queue Cleared.");
    tracing::info!("{message}");
    send_to_client(stream, &Response::Status(state.to_status())).await?;
    Ok(())
}
