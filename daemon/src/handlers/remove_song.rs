use crate::{helpers::send_to_all, types::*};

use aurora_protocol::{Response, Song};
use uuid::Uuid;

pub async fn remove_song(state: &State, song_id: Uuid) -> anyhow::Result<()> {
    let mut state_locked = state.lock().await;
    state_locked.queue = state_locked
        .queue
        .clone()
        .iter()
        .filter(|song| song.id != song_id)
        .cloned()
        .collect();

    let resp = Response::Queue(state_locked.queue.iter().map(Song::from).collect());
    drop(state_locked);

    send_to_all(state, &resp).await?;
    Ok(())
}
