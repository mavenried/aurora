use aurora_protocol::Request;
use std::{sync::Arc, time::Duration};
use tokio::{
    io::AsyncReadExt,
    net::{TcpStream, tcp::OwnedReadHalf},
    sync::Mutex,
};

use crate::{handlers::status::status, types::State};

mod albumart;
mod clear;
mod enqueue;
mod next_prev;
mod pause;
mod playlist;
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

pub async fn handle_client(stream: TcpStream, state: State) -> anyhow::Result<()> {
    let (mut reader, w) = stream.into_split();
    let writer = Arc::new(Mutex::new(w));

    {
        state.lock().await.clients.push(writer.clone());
    }

    {
        let writer = writer.clone();
        let state = state.clone();
        tokio::spawn(async move {
            loop {
                std::thread::sleep(Duration::from_millis(500));
                if status(&writer, &state).await.is_err() {
                    break;
                }
            }
        });
    }

    loop {
        let request = read_request(&mut reader).await?;
        match request {
            Request::Play(song_uuid) => enqueue::enqueue(&writer, &state, song_uuid).await?,
            Request::PlaylistList => playlist::playlist_list(&writer).await?,
            Request::PlaylistGet(pl_uuid) => playlist::playlist_get(&writer, pl_uuid).await?,
            Request::Search(st) => search::search(&writer, &state, st).await?,
            Request::Clear => clear::clear(&writer, &state).await?,
            Request::PlaylistCreate(pl_in) => playlist::playlist_create(&writer, pl_in).await?,
            Request::AlbumArt(song_uuid) => albumart::albumart(&writer, &state, song_uuid).await?,
            Request::Next(n) => next_prev::next(&writer, &state, n).await?,
            Request::Prev(n) => next_prev::prev(&writer, &state, n).await?,
            Request::Pause => pause::pause(&writer, &state).await?,
            Request::Seek(n) => seek::seek(&writer, &state, n).await?,
            Request::ReplaceQueue(queue) => {
                clear::clear(&writer, &state).await?;
                for song in queue {
                    enqueue::enqueue(&writer, &state, song.id).await?;
                }
            }
        }
    }
}
