use aurora_protocol::Response;

use crate::{
    helpers::send_to_client,
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
