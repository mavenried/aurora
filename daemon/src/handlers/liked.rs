use aurora_protocol::Response;
use uuid::Uuid;

use crate::{
    helpers::{add_liked, remove_liked, send_to_client},
    types::*,
};

pub async fn like_song(stream: &WriteSocket, state: &State, song_id: Uuid) -> anyhow::Result<()> {
    let db = {
        let mut s = state.lock().await;
        s.liked_ids.insert(song_id);
        s.db.clone()
    };
    add_liked(&db, song_id).await?;
    get_liked_songs(stream, state).await
}

pub async fn unlike_song(
    stream: &WriteSocket,
    state: &State,
    song_id: Uuid,
) -> anyhow::Result<()> {
    let db = {
        let mut s = state.lock().await;
        s.liked_ids.remove(&song_id);
        s.db.clone()
    };
    remove_liked(&db, song_id).await?;
    get_liked_songs(stream, state).await
}

pub async fn get_liked_songs(stream: &WriteSocket, state: &State) -> anyhow::Result<()> {
    let mut s = state.lock().await;
    let ids: Vec<Uuid> = s.liked_ids.iter().copied().collect();
    for id in &ids {
        s.get_art(*id);
    }
    let mut songs: Vec<aurora_protocol::Song> = ids
        .iter()
        .filter_map(|id| s.index.get(id))
        .map(|meta| {
            let mut song = aurora_protocol::Song::from(meta);
            song.liked = true;
            song
        })
        .collect();
    songs.sort_by(|a, b| a.title.cmp(&b.title));
    send_to_client(stream, &Response::LikedSongs(songs)).await?;
    Ok(())
}
