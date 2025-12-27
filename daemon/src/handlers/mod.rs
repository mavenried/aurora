use anyhow::Ok;
use aurora_protocol::{Request, Response, Song};
use std::time::Duration;
use tokio::{io::AsyncReadExt, net::unix::OwnedReadHalf};

use crate::{
    handlers::status::status,
    helpers::send_to_client,
    types::{State, WriteSocket},
};

mod albumart;
mod clear;
mod enqueue;
pub mod next_prev;
mod pause;
mod playlist;
mod replace_queue;
mod search;
mod seek;
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
            Request::PlaylistList => playlist::playlist_list(&writer).await,
            Request::PlaylistGet(pl_uuid) => playlist::playlist_get(&writer, pl_uuid).await,
            Request::Search(st) => search::search(&writer, &state, st).await,
            Request::Clear => clear::clear(&state).await,
            Request::PlaylistCreate(pl_in) => playlist::playlist_create(&writer, pl_in).await,
            Request::AlbumArt(song_uuid) => albumart::albumart(&writer, &state, song_uuid).await,
            Request::Next(n) => next_prev::next(&writer, &state, n).await,
            Request::Prev(n) => next_prev::prev(&writer, &state, n).await,
            Request::Pause => pause::pause(&writer, &state).await,
            Request::Seek(n) => seek::seek(&writer, &state, n).await,
            Request::ReplaceQueue(queue) => {
                replace_queue::replace_queue(&writer, &state, queue).await
            }
        } {
            tracing::error!("Err: {err}");
        }
    }
}
