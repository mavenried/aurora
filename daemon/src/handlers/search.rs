use crate::{helpers::send_to_client, types::*};
use aurora_protocol::{Response, SearchType, Song};

pub async fn search(
    stream: &WriteSocket,
    state: &State,
    searchtype: SearchType,
) -> anyhow::Result<()> {
    tracing::debug!("Searching for {searchtype:?}");

    let songs = {
        let mut state = state.lock().await;
        let songs = state.search(searchtype).await;
        for song in &songs {
            state.get_art(song.id);
        }
        songs
            .iter()
            .filter_map(|song| state.index.get(&song.id))
            .map(Song::from)
            .collect()
    };

    send_to_client(stream, &Response::SearchResults(songs)).await?;
    Ok(())
}
