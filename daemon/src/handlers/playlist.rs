use uuid::Uuid;

use crate::{
    helpers::{add_songs_to_playlist, create_playlist, delete_playlist, get_all_playlists, remove_song_from_playlist, rename_playlist, send_to_client},
    types::*,
};
use aurora_protocol::{PlaylistIn, Response};

pub async fn playlist_rename(stream: &WriteSocket, playlist_id: Uuid, new_title: String) -> anyhow::Result<()> {
    match rename_playlist(playlist_id, new_title).await {
        Ok(_) => playlist_list(stream).await,
        Err(err) => {
            send_to_client(stream, &Response::Error { err_id: 8, err_msg: err.to_string() }).await?;
            Ok(())
        }
    }
}

pub async fn playlist_delete(stream: &WriteSocket, playlist_id: Uuid) -> anyhow::Result<()> {
    match delete_playlist(playlist_id).await {
        Ok(_) => playlist_list(stream).await,
        Err(err) => {
            send_to_client(stream, &Response::Error { err_id: 7, err_msg: err.to_string() }).await?;
            Ok(())
        }
    }
}

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

pub async fn playlist_remove_song(
    stream: &WriteSocket,
    state: &State,
    playlist_id: Uuid,
    song_id: Uuid,
) -> anyhow::Result<()> {
    match remove_song_from_playlist(playlist_id, song_id).await {
        Ok(_) => {
            playlist_get(stream, state, playlist_id).await?;
        }
        Err(err) => {
            send_to_client(
                stream,
                &Response::Error {
                    err_id: 6,
                    err_msg: err.to_string(),
                },
            )
            .await?;
        }
    }
    Ok(())
}

pub async fn playlist_add_songs(
    stream: &WriteSocket,
    state: &State,
    playlist_id: Uuid,
    song_ids: Vec<Uuid>,
) -> anyhow::Result<()> {
    let state_locked = state.lock().await;
    match add_songs_to_playlist(&state_locked, playlist_id, song_ids).await {
        Ok(_) => {
            playlist_list(stream).await?;
        }
        Err(err) => {
            send_to_client(
                stream,
                &Response::Error {
                    err_id: 5,
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
