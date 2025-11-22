use crate::{helpers::send_to_client, types::*};
use aurora_protocol::Response;

pub async fn pause(stream: &WriteSocket, state: &State) -> anyhow::Result<()> {
    let mut state = state.lock().await;
    state.pause().await;

    send_to_client(stream, &Response::Status(state.to_status())).await?;
    Ok(())
}
