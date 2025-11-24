use crate::{
    helpers::{send_to_all, send_to_client},
    types::*,
};

use aurora_protocol::{Response, Song};

pub async fn replace_queue(
    stream: &WriteSocket,
    state: &State,
    queue_in: Vec<Song>,
) -> anyhow::Result<()> {
    let mut state_locked = state.lock().await;
    let mut queue_out = vec![];

    for song in queue_in {
        let songmeta = match state_locked.index.get(&song.id) {
            Some(s) => s.clone(),
            None => {
                send_to_client(
                    stream,
                    &Response::Error {
                        err_id: 1,
                        err_msg: format!("No such song with id {}", song.id),
                    },
                )
                .await?;
                return Ok(());
            }
        };
        queue_out.push(songmeta);
    }

    state_locked.clear().await;
    state_locked.current_idx = 1;
    state_locked.queue = queue_out;
    state_locked.prev(1).await;

    let resp = Response::Queue(state_locked.queue.iter().map(Song::from).collect());
    drop(state_locked);

    send_to_all(state, &resp).await?;
    Ok(())
}
