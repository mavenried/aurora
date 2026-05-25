use crate::{helpers::send_to_all, types::*};

use aurora_protocol::{Response, Song};

pub async fn remove_song_at(state: &State, index: usize) -> anyhow::Result<()> {
    let mut state_locked = state.lock().await;
    if index < state_locked.queue.len() {
        state_locked.queue.remove(index);
        let resp = Response::Queue(state_locked.queue.iter().map(Song::from).collect());
        drop(state_locked);
        send_to_all(state, &resp).await?;
    }
    Ok(())
}
