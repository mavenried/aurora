use aurora_protocol::Response;

use crate::{helpers::send_to_client, types::*};

pub async fn status(stream: &WriteSocket, state: &State) {
    let state = state.lock().await;
    let _ = send_to_client(stream, &Response::Status(state.to_status()))
        .await
        .map_err(|err| tracing::error!("Error: {err}"));
}
