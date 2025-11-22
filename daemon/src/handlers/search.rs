use crate::{helpers::send_to_client, types::*};
use aurora_protocol::{Response, SearchType, Song};

pub async fn search(
    stream: &WriteSocket,
    state: &State,
    searchtype: SearchType,
) -> anyhow::Result<()> {
    let state = state.lock().await;

    tracing::info!("Searching for {searchtype:?}");
    let songs = state
        .search(searchtype)
        .await
        .iter()
        .map(Song::from)
        .collect();
    send_to_client(stream, &Response::SearchResults(songs)).await?;
    Ok(())
}
