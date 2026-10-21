use crate::{helpers::send_to_client, types::*};
use aurora_protocol::Response;

pub async fn pause(stream: &WriteSocket, state: &State) -> anyhow::Result<()> {
    let response = {
        let mut state = state.lock().await;
        state.pause().await;
        Response::Status(state.to_status())
    };

    send_to_client(stream, &response).await?;
    Ok(())
}
