use aurora_protocol::Response;

use crate::{helpers::send_to_client, types::*};

pub async fn status(stream: &WriteSocket, state: &State) -> anyhow::Result<()> {
    let state = state.lock().await;
    send_to_client(stream, &Response::Status(state.to_status())).await?;
    Ok(())
}
