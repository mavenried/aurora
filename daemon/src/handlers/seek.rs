use crate::{handlers::status::status, types::*};
use std::time::Duration;

pub async fn seek(stream: &WriteSocket, state: &State, n: Duration) -> anyhow::Result<()> {
    let mut state_locked = state.lock().await;
    if let Some(audio) = &mut state_locked.audio {
        let current_pos = n;
        audio.seek(current_pos);
    }
    drop(state_locked);
    status(stream, state).await?;

    Ok(())
}
