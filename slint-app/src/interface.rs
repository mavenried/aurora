use std::sync::Arc;

use aurora_protocol::{Request, Response};
use base64::{Engine, prelude::BASE64_URL_SAFE};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        TcpStream,
        tcp::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::{Mutex, mpsc::Receiver},
};

use crate::{
    AuroraPlayer, CurrentSong, DEFAULT_ART,
    types::{ImageCache, State, StateStruct},
};

pub fn album_art_from_data(data: &[u8]) -> anyhow::Result<SharedPixelBuffer<Rgba8Pixel>> {
    let img = image::load_from_memory(data)?;
    let rgba = img.to_rgba8();
    Ok(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        rgba.as_raw(),
        rgba.width(),
        rgba.height(),
    ))
}

async fn tcp_sender(mut writer: OwnedWriteHalf, mut rx: Receiver<Request>) -> anyhow::Result<()> {
    loop {
        match rx.recv().await {
            Some(req) => {
                let encoded = serde_json::to_string(&req)?;
                let len = (encoded.len() as u32).to_be_bytes();
                writer.write_all(&len).await?;
                writer.write_all(encoded.as_bytes()).await?;
            }
            None => {
                tracing::info!("Writer channel closed.");
                return Err(anyhow::anyhow!("Writer channel closed."));
            }
        }
    }
}

async fn tcp_recver(
    mut reader: OwnedReadHalf,
    state: State,
    app: slint::Weak<AuroraPlayer>,
) -> anyhow::Result<()> {
    loop {
        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf).await?;
        let msg_len = u32::from_be_bytes(len_buf) as usize;

        let mut buf = vec![0u8; msg_len];
        reader.read_exact(&mut buf).await?;
        let res: Response = serde_json::from_slice(&buf)?;

        match res {
            Response::Status(status) => {
                let mut state_locked = state.lock().await;
                let buffer = if let Some(s) = &status.current_song {
                    state_locked.get_album_art(s.id).await
                } else {
                    state_locked.default_art_buffer.clone()
                };

                if status.current_idx != state_locked.cur_idx {
                    state_locked.cur_idx = status.current_idx;
                    state_locked.update_queue(app.clone()).await;
                }

                drop(state_locked);

                let _ = app.upgrade_in_event_loop(move |aurora| {
                    let album_art = Image::from_rgba8(buffer);
                    aurora.set_currentSong(CurrentSong {
                        title: status
                            .current_song
                            .as_ref()
                            .map(|s| s.title.as_str())
                            .unwrap_or("Nothing Playing")
                            .into(),

                        artists: status
                            .current_song
                            .as_ref()
                            .map(|s| s.artists.join(", "))
                            .unwrap_or_else(|| "No Artist".into())
                            .into(),

                        song_uuid: status
                            .current_song
                            .as_ref()
                            .map(|s| s.id.to_string())
                            .unwrap_or_else(|| "None".into())
                            .into(),

                        duration: status
                            .current_song
                            .as_ref()
                            .map(|s| s.duration.as_millis() as i32)
                            .unwrap_or(0),

                        album_art,

                        is_paused: status.is_paused,

                        position: status.position.as_millis() as i32,
                    });
                });
            }

            Response::Queue(queue) => {
                let mut state = state.lock().await;
                state.queue = queue;
                state.update_queue(app.clone()).await;
            }
            Response::Picture { id, data } => {
                let decoded = BASE64_URL_SAFE.decode(data).unwrap();
                let buf = album_art_from_data(decoded.as_slice()).unwrap();

                let mut state = state.lock().await;
                state.artcache.put(id, buf);
                state.update_queue(app.clone()).await;
            }
            _ => (),
        }
    }
}

pub async fn interface(app: slint::Weak<AuroraPlayer>) -> anyhow::Result<()> {
    let stream = TcpStream::connect("0.0.0.0:4321").await?;
    tracing::info!("Connected at 0.0.0.0:4321");

    let (reader, writer) = stream.into_split();
    let (tx, rx) = tokio::sync::mpsc::channel::<Request>(10);
    let state = Arc::new(Mutex::new(StateStruct {
        default_art_buffer: album_art_from_data(DEFAULT_ART).unwrap(),
        artcache: ImageCache::new(),
        writer_tx: tx,
        queue: vec![],
        waiting_for_art: vec![],
        cur_idx: 0,
    }));

    let state_clone = state.clone();
    let _ = app.upgrade().unwrap().on_queue_click(move |n| {
        tracing::info!("{n}");
        let state = state_clone.clone();
        tokio::spawn(async move {
            let _ = state
                .lock()
                .await
                .writer_tx
                .send(Request::Next((n + 1) as usize))
                .await;
        });
    });

    tokio::spawn(tcp_sender(writer, rx));
    tokio::spawn(tcp_recver(reader, state.clone(), app));
    Ok(())
}
