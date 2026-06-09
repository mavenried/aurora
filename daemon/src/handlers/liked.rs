use uuid::Uuid;
use aurora_protocol::Response;
use crate::{
    helpers::{save_liked, send_to_client},
    types::*,
};

pub async fn like_song(stream: &WriteSocket, state: &State, song_id: Uuid) -> anyhow::Result<()> {
    {
        let mut s = state.lock().await;
        s.liked_ids.insert(song_id);
        save_liked(&s.liked_ids).await?;
    }
    get_liked_songs(stream, state).await
}

pub async fn unlike_song(stream: &WriteSocket, state: &State, song_id: Uuid) -> anyhow::Result<()> {
    {
        let mut s = state.lock().await;
        s.liked_ids.remove(&song_id);
        save_liked(&s.liked_ids).await?;
    }
    get_liked_songs(stream, state).await
}

pub async fn get_liked_songs(stream: &WriteSocket, state: &State) -> anyhow::Result<()> {
    let s = state.lock().await;
    let mut songs: Vec<aurora_protocol::Song> = s.liked_ids.iter()
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
