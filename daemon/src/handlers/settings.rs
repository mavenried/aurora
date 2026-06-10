use aurora_protocol::{Response, Song};
use uuid::Uuid;

use crate::{
    helpers::{send_to_all, send_to_client},
    types::{State, WriteSocket},
};

pub async fn set_volume(writer: &WriteSocket, state: &State, volume: f32) -> anyhow::Result<()> {
    let mut state_locked = state.lock().await;
    state_locked.volume = volume;
    state_locked.sink.set_volume(volume);
    drop(state_locked);
    send_to_client(writer, &Response::Status(state.lock().await.to_status())).await
}

pub async fn set_shuffle(writer: &WriteSocket, state: &State, shuffle: bool) -> anyhow::Result<()> {
    state.lock().await.shuffle = shuffle;
    send_to_client(writer, &Response::Status(state.lock().await.to_status())).await
}

pub async fn set_repeat(writer: &WriteSocket, state: &State, repeat: u8) -> anyhow::Result<()> {
    state.lock().await.repeat = repeat;
    send_to_client(writer, &Response::Status(state.lock().await.to_status())).await
}

pub async fn get_artist_list(writer: &WriteSocket, state: &State) -> anyhow::Result<()> {
    let state_locked = state.lock().await;
    let mut artists: Vec<String> = state_locked
        .index
        .values()
        .flat_map(|s| s.artists.iter().cloned())
        .collect::<std::collections::HashSet<String>>()
        .into_iter()
        .collect();
    artists.sort();
    drop(state_locked);
    send_to_client(writer, &Response::ArtistList(artists)).await
}

pub async fn broadcast_last_played(state: &State) -> anyhow::Result<()> {
    let songs = {
        let mut s = state.lock().await;
        let missing: Vec<Uuid> = s
            .recently_played
            .iter()
            .filter(|id| !s.index.contains_key(id))
            .cloned()
            .collect();
        if !missing.is_empty() {
            s.recently_played.retain(|id| !missing.contains(id));
        }
        let ids: Vec<Uuid> = s.recently_played.iter().cloned().collect();
        let mut result = vec![];
        for id in ids {
            s.get_art(id);
            if let Some(meta) = s.index.get(&id) {
                result.push(Song::from(meta));
            }
        }
        result
    };
    send_to_all(state, &Response::LastPlayed(songs)).await
}

pub async fn get_last_played(writer: &WriteSocket, state: &State) -> anyhow::Result<()> {
    let mut state_locked = state.lock().await;

    let missing: Vec<Uuid> = state_locked
        .recently_played
        .iter()
        .filter(|id| !state_locked.index.contains_key(id))
        .cloned()
        .collect();

    if !missing.is_empty() {
        state_locked
            .recently_played
            .retain(|id| !missing.contains(id));
        let history = state_locked.recently_played.clone();
        let db = state_locked.db.clone();
        tokio::spawn(async move {
            if let Err(e) = crate::helpers::save_history(&db, &history).await {
                tracing::error!("Failed to save play history: {e}");
            }
        });
    }

    let ids: Vec<Uuid> = state_locked.recently_played.iter().cloned().collect();
    let mut songs = vec![];
    for id in ids {
        state_locked.get_art(id);
        if let Some(meta) = state_locked.index.get(&id) {
            songs.push(Song::from(meta));
        }
    }

    drop(state_locked);
    send_to_client(writer, &Response::LastPlayed(songs)).await
}
