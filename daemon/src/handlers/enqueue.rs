use uuid::Uuid;

use crate::{helpers::send_to_client, types::*};
use aurora_protocol::Response;

pub async fn enqueue(stream: &WriteSocket, state: &State, song_uuid: Uuid) {
    let mut state = state.lock().await;

    let song = match state.index.get(&song_uuid) {
        Some(s) => s.clone(),
        None => {
            let _ = send_to_client(
                stream,
                &Response::Error {
                    err_id: 1,
                    err_msg: format!("No such song with id {song_uuid}"),
                },
            )
            .await
            .map_err(|err| tracing::error!("Error: {err}"));
            return;
        }
    };

    let should_start = state.queue.is_empty();

    state.queue.push(song.clone());

    if should_start {
        state.add().await;
    }

    let message = format!("Added {} by {} to queue.", song.title, song.artists[0]);
    tracing::info!("{message}");
    let _ = send_to_client(stream, &Response::Status(state.to_status()))
        .await
        .map_err(|err| tracing::error!("Error: {err}"));
}
