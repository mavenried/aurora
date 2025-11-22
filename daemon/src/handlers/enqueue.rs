use uuid::Uuid;

use crate::{
    helpers::{send_to_all, send_to_client},
    types::*,
};
use aurora_protocol::{Response, Song};

pub async fn enqueue(stream: &WriteSocket, state: &State, song_uuid: Uuid) -> anyhow::Result<()> {
    let mut state_locked = state.lock().await;

    let song = match state_locked.index.get(&song_uuid) {
        Some(s) => s.clone(),
        None => {
            send_to_client(
                stream,
                &Response::Error {
                    err_id: 1,
                    err_msg: format!("No such song with id {song_uuid}"),
                },
            )
            .await?;
            return Ok(());
        }
    };

    let should_start = state_locked.queue.is_empty();

    state_locked.queue.push(song.clone());

    if should_start {
        state_locked.add().await;
    }

    let message = format!("Added {} by {} to queue.", song.title, song.artists[0]);
    tracing::info!("{message}");
    let resp = Response::Queue(state_locked.queue.iter().map(Song::from).collect());
    drop(state_locked);

    send_to_all(state, &resp).await?;
    Ok(())
}
