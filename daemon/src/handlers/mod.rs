use anyhow::Ok;
use aurora_protocol::{Request, Response, Song};
use std::time::Duration;
use tokio::{io::AsyncReadExt, net::unix::OwnedReadHalf};

use crate::{
    handlers::status::status,
    helpers::send_to_client,
    types::{State, WriteSocket},
};

mod clear;
mod enqueue;
pub mod liked;
mod move_queue;
mod enqueue_at;
pub mod next_prev;
mod pause;
mod playlist;
mod remove_song;
mod remove_song_at;
mod replace_queue;
mod search;
mod seek;
mod settings;
mod status;

async fn read_request(read: &mut OwnedReadHalf) -> anyhow::Result<Request> {
    let mut len_buf = [0u8; 4];
    read.read_exact(&mut len_buf).await?;
    let msg_len = u32::from_be_bytes(len_buf) as usize;

    let mut buf = vec![0u8; msg_len];
    read.read_exact(&mut buf).await?;
    let req: Request = serde_json::from_slice(&buf)?;
    Ok(req)
}

pub async fn handle_client(
    mut reader: OwnedReadHalf,
    writer: WriteSocket,
    state: State,
) -> anyhow::Result<()> {
    {
        let writer = writer.clone();
        let state = state.clone();
        tokio::spawn(async move {
            let _ = send_to_client(
                &writer,
                &Response::Queue(state.lock().await.queue.iter().map(Song::from).collect()),
            )
            .await;

            let _ =
                send_to_client(&writer, &Response::Theme(state.lock().await.theme.clone())).await;

            let _ =
                send_to_client(&writer, &Response::Volume(state.lock().await.volume)).await;

            let _ = settings::get_last_played(&writer, &state).await;
            let _ = liked::get_liked_songs(&writer, &state).await;
            loop {
                std::thread::sleep(Duration::from_millis(200));
                if status(&writer, &state).await.is_err() {
                    tracing::error!("Status notifier failed for a client");
                    break;
                }
            }
        });
    }
    loop {
        let request = read_request(&mut reader).await?;
        tracing::debug!("{request:?}");
        if let Err(err) = match request {
            Request::Play(song_uuid) => enqueue::enqueue(&writer, &state, song_uuid).await,
            Request::Enqueue(song_uuid) => {
                enqueue_at::enqueue_at(&writer, &state, song_uuid, None).await
            }
            Request::PlayNext(song_uuid) => {
                enqueue_at::enqueue_at(&writer, &state, song_uuid, Some(1)).await
            }
            Request::PlaylistList => playlist::playlist_list(&writer, &state).await,
            Request::PlaylistGet(pl_uuid) => playlist::playlist_get(&writer, &state, pl_uuid).await,
            Request::Search(st) => search::search(&writer, &state, st).await,
            Request::Clear => clear::clear(&state).await,
            Request::PlaylistCreate(pl_in) => playlist::playlist_create(&writer, &state, pl_in).await,
            Request::Next(n) => next_prev::next(&writer, &state, n).await,
            Request::Prev(n) => next_prev::prev(&writer, &state, n).await,
            Request::Pause => pause::pause(&writer, &state).await,
            Request::Seek(n) => seek::seek(&writer, &state, n).await,
            Request::ReplaceQueue(queue) => {
                replace_queue::replace_queue(&writer, &state, queue).await
            }
            Request::RemoveSong(id) => remove_song::remove_song(&state, id).await,
            Request::RemoveSongAt(index) => remove_song_at::remove_song_at(&state, index).await,
            Request::MoveQueue { from, to } => move_queue::move_queue(&state, from, to).await,
            Request::PlaylistDelete(id) => playlist::playlist_delete(&writer, &state, id).await,
            Request::PlaylistAddSongs { playlist_id, song_ids } => {
                playlist::playlist_add_songs(&writer, &state, playlist_id, song_ids).await
            }
            Request::PlaylistRemoveSong { playlist_id, song_id } => {
                playlist::playlist_remove_song(&writer, &state, playlist_id, song_id).await
            }
            Request::PlaylistRename { playlist_id, new_title } => {
                playlist::playlist_rename(&writer, &state, playlist_id, new_title).await
            }
            Request::SetVolume(v) => settings::set_volume(&writer, &state, v).await,
            Request::SetShuffle(b) => settings::set_shuffle(&writer, &state, b).await,
            Request::SetRepeat(r) => settings::set_repeat(&writer, &state, r).await,
            Request::GetArtistList => settings::get_artist_list(&writer, &state).await,
            Request::GetLastPlayed => settings::get_last_played(&writer, &state).await,
            Request::LikeSong(id) => liked::like_song(&writer, &state, id).await,
            Request::UnlikeSong(id) => liked::unlike_song(&writer, &state, id).await,
            Request::GetLikedSongs => liked::get_liked_songs(&writer, &state).await,
        } {
            tracing::error!("Err: {err}");
        }
    }
}
