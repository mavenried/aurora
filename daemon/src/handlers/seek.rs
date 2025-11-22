use aurora_protocol::Response;

use crate::{helpers::send_to_client, types::*};
use std::time::Duration;

pub async fn seek(stream: &WriteSocket, state: &State, n: u64) -> anyhow::Result<()> {
    let mut state = state.lock().await;
    if let Some(audio) = &mut state.audio {
        let current_pos = Duration::from_secs(n);
        audio.seek(current_pos);
    }

    send_to_client(stream, &Response::Status(state.to_status())).await?;
    Ok(())
}
