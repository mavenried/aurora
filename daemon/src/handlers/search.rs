use crate::{helpers::send_to_client, types::*};
use aurora_protocol::{Response, SearchType, Song};

pub async fn search(stream: &WriteSocket, state: &State, searchtype: SearchType) {
    let state = state.lock().await;

    tracing::info!("Searching for {searchtype:?}");
    let songs = state
        .search(searchtype)
        .await
        .iter()
        .map(Song::from)
        .collect();
    let _ = send_to_client(stream, &Response::SearchResults(songs))
        .await
        .map_err(|err| tracing::error!("Error: {err}"));
}
