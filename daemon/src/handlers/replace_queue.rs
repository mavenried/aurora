use crate::{
    helpers::{send_to_all, send_to_client},
    types::*,
};

use aurora_protocol::{Response, Song};
use uuid::Uuid;

pub async fn replace_queue(
    stream: &WriteSocket,
    state: &State,
    queue_in: Vec<Uuid>,
) -> anyhow::Result<()> {
    let mut state_locked = state.lock().await;
    let mut queue_out = vec![];

    for id in queue_in {
        let Some(songmeta) = state_locked.index.get(&id).cloned() else {
            drop(state_locked);
            send_to_client(
                stream,
                &Response::Error {
                    err_id: 1,
                    err_msg: format!("No such song with id {}", id),
                },
            )
            .await?;
            return Ok(());
        };
        queue_out.push(songmeta);
    }

    state_locked.queue = queue_out.into();
    if let Some(song) = state_locked.queue.front().cloned() {
        state_locked.current_song = Some(song);
        state_locked.add().await;
    } else {
        state_locked.clear().await;
    }

    let resp = Response::Queue(state_locked.queue.iter().map(Song::from).collect());
    drop(state_locked);

    send_to_all(state, &resp).await?;
    Ok(())
}
