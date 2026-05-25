use uuid::Uuid;

use crate::{
    helpers::{send_to_all, send_to_client},
    types::*,
};
use aurora_protocol::{Response, Song};

pub async fn enqueue_at(
    stream: &WriteSocket,
    state: &State,
    song_uuid: Uuid,
    position: Option<usize>,
) -> anyhow::Result<()> {
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

    let was_empty = state_locked.queue.is_empty();

    // Handle position: None means append to end
    if let Some(pos) = position {
        if pos == 0 {
            // Position 0 means "play now" - replace queue with just this song
            state_locked.queue.clear();
            state_locked.queue.push_back(song.clone());
            state_locked.current_song = Some(song.clone());
            state_locked.add().await;
        } else if pos == 1 {
            // Position 1 means "play next" - insert after current song (at index 0)
            // Efficient approach: pop current, push new, push current back
            if !state_locked.queue.is_empty() {
                let current = state_locked.queue.pop_front().unwrap();
                state_locked.queue.push_front(song.clone());
                state_locked.queue.push_front(current);
            } else {
                // Queue was empty, just add this song
                state_locked.queue.push_back(song.clone());
                state_locked.current_song = Some(song.clone());
                state_locked.add().await;
            }
        } else {
            // For other positions, use Vec for insertion
            let mut queue_vec: Vec<_> = state_locked.queue.drain(..).collect();
            let insert_pos = pos.min(queue_vec.len());
            queue_vec.insert(insert_pos, song.clone());
            state_locked.queue = queue_vec.into();
        }
    } else {
        // Append to end
        state_locked.queue.push_back(song.clone());
    }

    // If queue was empty and we didn't handle position 0 above, start playback
    if was_empty && position != Some(0) {
        state_locked.current_song = Some(song.clone());
        state_locked.add().await;
    }

    let message = if let Some(pos) = position {
        format!(
            "Added {} by {} to queue at position {}",
            song.title, song.artists[0], pos
        )
    } else {
        format!("Added {} by {} to queue", song.title, song.artists[0])
    };
    tracing::info!("{message}");

    let resp = Response::Queue(state_locked.queue.iter().map(Song::from).collect());
    drop(state_locked);

    send_to_all(state, &resp).await?;
    Ok(())
}
