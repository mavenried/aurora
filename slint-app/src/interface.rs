use std::{process::Command, sync::Arc, time::Duration};

use aurora_protocol::{Request, Response, SearchType};
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
    AuroraPlayer, DEFAULT_ART,
    types::{ImageCache, ImageFor, State, StateStruct},
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
                tracing::info!("Sent: {encoded}");
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
                    state_locked.get_album_art(ImageFor::Queue(s.id)).await
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
                    aurora.set_Title(
                        status
                            .current_song
                            .as_ref()
                            .map(|s| s.title.as_str())
                            .unwrap_or("Nothing Playing")
                            .into(),
                    );

                    aurora.set_Artists(
                        status
                            .current_song
                            .as_ref()
                            .map(|s| s.artists.join(", "))
                            .unwrap_or_else(|| "No Artist".into())
                            .into(),
                    );

                    aurora.set_duration(
                        status
                            .current_song
                            .as_ref()
                            .map(|s| s.duration.as_millis() as i32)
                            .unwrap_or(0),
                    );

                    aurora.set_AlbumArt(album_art);

                    aurora.set_is_paused(status.is_paused);

                    aurora.set_position(status.position.as_millis() as i32);
                });
            }

            Response::Queue(queue) => {
                tracing::info!("Received: Queue");
                let mut state = state.lock().await;
                state.queue = queue;
                state.update_queue(app.clone()).await;
            }
            Response::SearchResults(mut results) => {
                tracing::info!("Received: SearchResults, len:{}", results.len());
                if results.len() > 300 {
                    results = results[..300].to_vec();
                }
                let mut state = state.lock().await;
                state.search_results = results;
                state.update_search_results(app.clone()).await;
            }
            Response::Picture { id, data } => {
                tracing::info!("Received: Picture");
                let decoded = BASE64_URL_SAFE.decode(data).unwrap();
                let buf = album_art_from_data(decoded.as_slice()).unwrap();

                let mut state = state.lock().await;
                state.artcache.put(id, buf);
                if state.search_waitlist.contains(&id) {
                    state.update_search_results(app.clone()).await;
                } else if state.queue_waitlist.contains(&id) {
                    state.update_queue(app.clone()).await;
                } else if state.playlist_waitlist.contains(&id) {
                }
            }
            _ => (),
        }
    }
}

pub async fn interface(app: slint::Weak<AuroraPlayer>) -> anyhow::Result<()> {
    let mut stream: Option<TcpStream> = None;
    
    while stream.is_none() {
        if let Ok(s) = TcpStream::connect("0.0.0.0:4321").await{
            tracing::info!("Connected at 0.0.0.0:4321");
            stream = Some(s);
        } else {
            Command::new("aurora-daemon").spawn()?;
            std::thread::sleep(Duration::from_secs(1));
        }
    }

    let (reader, writer) = stream.unwrap().into_split();
    let (tx, rx) = tokio::sync::mpsc::channel::<Request>(10);
    let state = Arc::new(Mutex::new(StateStruct {
        default_art_buffer: album_art_from_data(DEFAULT_ART).unwrap(),
        artcache: ImageCache::new(),
        writer_tx: tx,
        queue: vec![],
        search_results: vec![],
        queue_waitlist: vec![],
        search_waitlist: vec![],
        playlist_waitlist: vec![],
        cur_idx: 0,
    }));

    let state_clone = state.clone();
    let _ = app.upgrade_in_event_loop(move |aurora: AuroraPlayer| {
        let state = state_clone.clone();
        aurora.on_queue_click(move |n| {
            let state = state.clone();
            tokio::spawn(async move {
                let _ = state
                    .lock()
                    .await
                    .writer_tx
                    .send(Request::Next((n + 1) as usize))
                    .await;
            });
        });

        let state = state_clone.clone();
        aurora.on_next(move || {
            let state = state.clone();
            tokio::spawn(async move {
                let _ = state.lock().await.writer_tx.send(Request::Next(1)).await;
            });
        });
        let state = state_clone.clone();
        aurora.on_prev(move || {
            let state = state.clone();
            tokio::spawn(async move {
                let _ = state.lock().await.writer_tx.send(Request::Prev(1)).await;
            });
        });
        let state = state_clone.clone();
        aurora.on_pause(move || {
            let state = state.clone();
            tokio::spawn(async move {
                let _ = state.lock().await.writer_tx.send(Request::Pause).await;
            });
        });
        let state = state_clone.clone();
        aurora.on_queue_clear(move || {
            let state = state.clone();
            tokio::spawn(async move {
                let _ = state.lock().await.writer_tx.send(Request::Clear).await;
            });
        });
        let state = state_clone.clone();
        aurora.on_queue_remove(move |n| {
            let state = state.clone();
            tokio::spawn(async move {
                let state = state.lock().await;
                let mut queue = state.queue.clone();
                queue.remove((state.cur_idx + n as usize + 1) % queue.len());
                let req = Request::ReplaceQueue(queue);
                let _ = state.writer_tx.send(req).await;
            });
        });

        let state = state_clone.clone();
        aurora.on_search(move |q, m| {
            let state = state.clone();
            if !q.is_empty() {
                let query = if m == "By Artist" {
                    Request::Search(SearchType::ByArtist(q.to_string()))
                } else {
                    Request::Search(SearchType::ByTitle(q.to_string()))
                };
                tokio::spawn(async move {
                    let _ = state.lock().await.writer_tx.send(query).await;
                });
            }
        });

        let state = state_clone.clone();
        aurora.on_add(move |id| {
            let state = state.clone();
            let id = uuid::Uuid::parse_str(&id).unwrap();
            tokio::spawn(async move {
                let _ = state.lock().await.writer_tx.send(Request::Play(id)).await;
            });
        });

        let state = state_clone.clone();
        aurora.on_seek(move |value| {
            let state = state.clone();
            tokio::spawn(async move {
                let _ = state
                    .lock()
                    .await
                    .writer_tx
                    .send(Request::Seek(Duration::from_millis(value as u64)))
                    .await;
            });
        });
    });

    tokio::spawn(async move {
        if let Err(err) = tcp_sender(writer, rx).await {
            tracing::error!("Sender Error: {err}");
        }
    });

    tokio::spawn(async move {
        if let Err(err) = tokio::spawn(tcp_recver(reader, state.clone(), app)).await {
            tracing::error!("Receiver Error: {err}");
        }
    });

    Ok(())
}
