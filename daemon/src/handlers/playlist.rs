use uuid::Uuid;

use crate::{
    helpers::{create_playlist, get_all_playlists, send_to_client},
    types::*,
};
use aurora_protocol::{PlaylistIn, Response};

pub async fn playlist_create(stream: &WriteSocket, item: PlaylistIn) -> anyhow::Result<()> {
    match create_playlist(item).await {
        Ok(name) => {
            tracing::info!("Created playlist {name}.");
            playlist_list(stream).await?;
        }
        Err(err) => {
            send_to_client(
                stream,
                &Response::Error {
                    err_id: 2,
                    err_msg: err.to_string(),
                },
            )
            .await?;
        }
    }
    Ok(())
}

pub async fn playlist_list(stream: &WriteSocket) -> anyhow::Result<()> {
    match get_all_playlists().await {
        Ok(list) => {
            send_to_client(stream, &Response::PlaylistList(list)).await?;
        }
        Err(err) => {
            send_to_client(
                stream,
                &Response::Error {
                    err_id: 2,
                    err_msg: err.to_string(),
                },
            )
            .await?;
        }
    }
    Ok(())
}

pub async fn playlist_get(stream: &WriteSocket, state: &State, id: Uuid) -> anyhow::Result<()> {
    let mut state = state.lock().await;
    match state.get_playlist(id).await {
        Ok(playlist) => {
            for song in &playlist.songs {
                state.get_art(song.id);
            }
            send_to_client(stream, &Response::PlaylistResults(playlist)).await?;
        }
        Err(err) => {
            send_to_client(
                stream,
                &Response::Error {
                    err_id: 4,
                    err_msg: err.to_string(),
                },
            )
            .await?;
        }
    }
    Ok(())
}
