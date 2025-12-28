use aurora_protocol::Response;

use crate::{helpers::send_to_client, types::*};

pub async fn status(stream: &WriteSocket, state: &State) -> anyhow::Result<()> {
    let mut state = state.lock().await;
    if state
        .current_song
        .as_ref()
        .is_some_and(|song| song.art_path.is_none())
    {
        let id = state.current_song.clone().unwrap().id;
        state.get_art(id);
    }
    send_to_client(stream, &Response::Status(state.to_status())).await?;
    Ok(())
}
