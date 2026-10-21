use aurora_protocol::{Request, Response, Song};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    net::unix::OwnedReadHalf,
    time::{Duration, timeout},
};

use crate::{
    handlers::status::status,
    helpers::send_to_client,
    types::{State, WriteSocket},
};

mod clear;
mod enqueue;
mod enqueue_at;
pub mod liked;
mod move_queue;
pub mod next_prev;
mod pause;
mod playlist;
mod remove_song;
mod remove_song_at;
mod replace_queue;
mod search;
mod seek;
pub mod settings;
mod status;

async fn read_request<R: AsyncRead + Unpin>(read: &mut R) -> anyhow::Result<Request> {
    read_request_with_timeout(read, Duration::from_secs(30)).await
}

async fn read_request_with_timeout(
    read: &mut (impl AsyncRead + Unpin),
    read_timeout: Duration,
) -> anyhow::Result<Request> {
    const MAX_REQUEST_SIZE: usize = 16 * 1024 * 1024;

    match timeout(read_timeout, async {
        let mut len_buf = [0u8; 4];
        read.read_exact(&mut len_buf).await?;
        let msg_len = u32::from_be_bytes(len_buf) as usize;
        tracing::debug!(frame_bytes = msg_len, "Received request frame header");
        if msg_len > MAX_REQUEST_SIZE {
            tracing::warn!(frame_bytes = msg_len, max_bytes = MAX_REQUEST_SIZE, "Rejected oversized request frame");
            anyhow::bail!("request exceeds maximum size");
        }

        let mut buf = vec![0u8; msg_len];
        read.read_exact(&mut buf).await?;
        match serde_json::from_slice(&buf) {
            Ok(request) => {
                tracing::debug!(frame_bytes = msg_len, "Decoded request frame");
                Ok(request)
            }
            Err(err) => {
                tracing::warn!(frame_bytes = msg_len, error = %err, "Rejected malformed request frame");
                Err(err.into())
            }
        }
    })
    .await
    {
        Ok(result) => result,
        Err(_) => {
            tracing::warn!(timeout_ms = read_timeout.as_millis(), "Timed out reading request frame");
            Err(anyhow::anyhow!("timed out reading request"))
        }
    }
}

fn request_name(request: &Request) -> &'static str {
    match request {
        Request::Play(_) => "Play",
        Request::Enqueue(_) => "Enqueue",
        Request::PlayNext(_) => "PlayNext",
        Request::PlaylistList => "PlaylistList",
        Request::PlaylistGet(_) => "PlaylistGet",
        Request::PlaylistCreate(_) => "PlaylistCreate",
        Request::PlaylistDelete(_) => "PlaylistDelete",
        Request::PlaylistAddSongs { .. } => "PlaylistAddSongs",
        Request::PlaylistRemoveSong { .. } => "PlaylistRemoveSong",
        Request::PlaylistRename { .. } => "PlaylistRename",
        Request::Clear => "Clear",
        Request::Next(_) => "Next",
        Request::Prev(_) => "Prev",
        Request::Pause => "Pause",
        Request::Seek(_) => "Seek",
        Request::Search(_) => "Search",
        Request::ReplaceQueue(_) => "ReplaceQueue",
        Request::RemoveSong(_) => "RemoveSong",
        Request::RemoveSongAt(_) => "RemoveSongAt",
        Request::MoveQueue { .. } => "MoveQueue",
        Request::SetVolume(_) => "SetVolume",
        Request::SetShuffle(_) => "SetShuffle",
        Request::SetRepeat(_) => "SetRepeat",
        Request::SetFollowArtColorway(_) => "SetFollowArtColorway",
        Request::GetArtistList => "GetArtistList",
        Request::GetLastPlayed => "GetLastPlayed",
        Request::LikeSong(_) => "LikeSong",
        Request::UnlikeSong(_) => "UnlikeSong",
        Request::GetLikedSongs => "GetLikedSongs",
        Request::GetHighResArt(_) => "GetHighResArt",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncWrite, AsyncWriteExt};

    async fn send_frame(stream: &mut (impl AsyncWrite + Unpin), payload: &[u8]) {
        stream
            .write_all(&(payload.len() as u32).to_be_bytes())
            .await
            .unwrap();
        stream.write_all(payload).await.unwrap();
    }

    #[tokio::test]
    async fn reads_a_valid_request_frame() {
        let (mut client, mut reader) = tokio::io::duplex(128);
        let payload = serde_json::to_vec(&Request::Clear).unwrap();
        send_frame(&mut client, &payload).await;

        assert!(matches!(
            read_request(&mut reader).await.unwrap(),
            Request::Clear
        ));
    }

    #[tokio::test]
    async fn rejects_frames_above_the_size_limit_before_reading_payload() {
        let (mut client, mut reader) = tokio::io::duplex(128);
        client
            .write_all(&((16 * 1024 * 1024 + 1) as u32).to_be_bytes())
            .await
            .unwrap();

        let error = read_request(&mut reader).await.unwrap_err();
        assert!(error.to_string().contains("maximum size"));
    }

    #[tokio::test]
    async fn rejects_invalid_json_payloads() {
        let (mut client, mut reader) = tokio::io::duplex(128);
        send_frame(&mut client, b"{").await;

        assert!(read_request(&mut reader).await.is_err());
    }

    #[tokio::test]
    async fn times_out_when_a_client_sends_no_frame() {
        let (_client, mut reader) = tokio::io::duplex(128);

        let error = read_request_with_timeout(&mut reader, Duration::from_millis(10))
            .await
            .unwrap_err();
        assert!(error.to_string().contains("timed out"));
    }

    #[tokio::test]
    async fn times_out_when_a_client_stalls_mid_frame() {
        let (mut client, mut reader) = tokio::io::duplex(128);
        client.write_all(&4_u32.to_be_bytes()).await.unwrap();

        let error = read_request_with_timeout(&mut reader, Duration::from_millis(10))
            .await
            .unwrap_err();
        assert!(error.to_string().contains("timed out"));
    }
}

pub async fn handle_client(
    mut reader: OwnedReadHalf,
    writer: WriteSocket,
    state: State,
) -> anyhow::Result<()> {
    let status_notifier = {
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

            let _ = send_to_client(&writer, &Response::Volume(state.lock().await.volume)).await;

            let _ = settings::get_last_played(&writer, &state).await;
            let _ = liked::get_liked_songs(&writer, &state).await;
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                if status(&writer, &state).await.is_err() {
                    tracing::error!("Status notifier failed for a client");
                    break;
                }
            }
        })
    };
    let result = async {
        loop {
            tracing::trace!("Waiting for client request frame");
            let request = match read_request(&mut reader).await {
                Ok(request) => request,
                Err(err) => {
                    tracing::debug!(error = %err, "Stopping client request loop");
                    return Err(err);
                }
            };
            let request_type = request_name(&request);
            tracing::debug!(request = request_type, "Dispatching client request");
            tracing::trace!(request = ?request, "Client request details");
            let started = std::time::Instant::now();
            let result = match request {
                Request::Play(song_uuid) => enqueue::enqueue(&writer, &state, song_uuid).await,
                Request::Enqueue(song_uuid) => {
                    enqueue_at::enqueue_at(&writer, &state, song_uuid, None).await
                }
                Request::PlayNext(song_uuid) => {
                    enqueue_at::enqueue_at(&writer, &state, song_uuid, Some(1)).await
                }
                Request::PlaylistList => playlist::playlist_list(&writer, &state).await,
                Request::PlaylistGet(pl_uuid) => {
                    playlist::playlist_get(&writer, &state, pl_uuid).await
                }
                Request::Search(st) => search::search(&writer, &state, st).await,
                Request::Clear => clear::clear(&state).await,
                Request::PlaylistCreate(pl_in) => {
                    playlist::playlist_create(&writer, &state, pl_in).await
                }
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
                Request::PlaylistAddSongs {
                    playlist_id,
                    song_ids,
                } => playlist::playlist_add_songs(&writer, &state, playlist_id, song_ids).await,
                Request::PlaylistRemoveSong {
                    playlist_id,
                    song_id,
                } => playlist::playlist_remove_song(&writer, &state, playlist_id, song_id).await,
                Request::PlaylistRename {
                    playlist_id,
                    new_title,
                } => playlist::playlist_rename(&writer, &state, playlist_id, new_title).await,
                Request::SetVolume(v) => settings::set_volume(&writer, &state, v).await,
                Request::SetShuffle(b) => settings::set_shuffle(&writer, &state, b).await,
                Request::SetRepeat(r) => settings::set_repeat(&writer, &state, r).await,
                Request::SetFollowArtColorway(enabled) => {
                    settings::set_follow_art_colorway(&state, enabled).await
                }
                Request::GetArtistList => settings::get_artist_list(&writer, &state).await,
                Request::GetLastPlayed => settings::get_last_played(&writer, &state).await,
                Request::LikeSong(id) => liked::like_song(&writer, &state, id).await,
                Request::UnlikeSong(id) => liked::unlike_song(&writer, &state, id).await,
                Request::GetLikedSongs => liked::get_liked_songs(&writer, &state).await,
                Request::GetHighResArt(id) => settings::get_highres_art(&writer, &state, id).await,
            };
            match result {
                Ok(()) => tracing::debug!(request = request_type, elapsed_ms = started.elapsed().as_millis(), "Client request handled"),
                Err(err) => tracing::error!(request = request_type, elapsed_ms = started.elapsed().as_millis(), error = %err, "Client request failed"),
            }
        }
    }
    .await;
    status_notifier.abort();
    result
}
