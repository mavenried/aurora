use crate::{helpers::send_to_client, types::*};
use aurora_protocol::Response;

pub async fn pause(stream: &WriteSocket, state: &State) {
    let mut state = state.lock().await;
    state.pause().await;

    let _ = send_to_client(stream, &Response::Status(state.to_status()))
        .await
        .map_err(|err| tracing::error!("Error: {err}"));
}
