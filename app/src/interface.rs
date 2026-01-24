use std::{path::PathBuf, process::Command, sync::Arc, time::Duration, vec};

use aurora_protocol::{Request, Response, SearchType};
use slint::{ComponentHandle, Image, Model, Rgba8Pixel, SharedPixelBuffer};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        UnixStream,
        unix::{OwnedReadHalf, OwnedWriteHalf},
    },
    sync::{Mutex, mpsc::Receiver},
};
use uuid::Uuid;

use crate::{
    AuroraPlayer, DEFAULT_ART,
    types::{State, StateStruct},
};

fn hex_to_u8(hex: String) -> slint::Color {
    let hex = hex.trim_start_matches('#');

    let r = u8::from_str_radix(&hex[0..2], 16).expect("invalid red");
    let g = u8::from_str_radix(&hex[2..4], 16).expect("invalid green");
    let b = u8::from_str_radix(&hex[4..6], 16).expect("invalid blue");

    slint::Color::from_rgb_u8(r, g, b)
}

pub fn album_art_from_data(data: &[u8]) -> anyhow::Result<SharedPixelBuffer<Rgba8Pixel>> {
    let img = image::load_from_memory(data)?;
    let rgba = img.to_rgba8();
    Ok(SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(
        rgba.as_raw(),
        rgba.width(),
        rgba.height(),
    ))
}

async fn unix_sender(mut writer: OwnedWriteHalf, mut rx: Receiver<Request>) -> anyhow::Result<()> {
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

async fn unix_recver(
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
                let state_locked = state.lock().await;
                let default_art = state_locked.default_art_buffer.clone();

                let _ = app.upgrade_in_event_loop(move |aurora| {
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

                    aurora.set_AlbumArt({
                        if status
                            .current_song
                            .as_ref()
                            .is_some_and(|s| s.art_path.is_some())
                        {
                            Image::load_from_path(
                                status.current_song.unwrap().art_path.unwrap().as_path(),
                            )
                            .unwrap()
                        } else {
                            Image::from_rgba8(default_art)
                        }
                    });

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
                // if results.len() > 100 {
                //     results = results[..100].to_vec();
                // }
                let mut state = state.lock().await;
                state.search_results = results;
                state.update_search_results(app.clone()).await;
            }

            Response::PlaylistResults(result) => {
                tracing::info!("Received: PlaylistResults, len:{}", result.songs.len());

                let mut state = state.lock().await;
                state.playlist_result = Some(result);
                state.update_playlist_results(app.clone()).await;
            }
            Response::PlaylistList(plists) => {
                tracing::info!("{plists:?}");
                let mut state = state.lock().await;
                state.playlist_list_results = plists;
                state.update_search_results(app.clone()).await;
                state.update_playlists(app.clone()).await;
            }
            Response::Theme(theme) => {
                let _ = app.upgrade_in_event_loop(|aurora| {
                    aurora
                        .global::<crate::Theme>()
                        .set_acct(hex_to_u8(theme.acct));
                    aurora
                        .global::<crate::Theme>()
                        .set_btns(hex_to_u8(theme.btns));
                    aurora
                        .global::<crate::Theme>()
                        .set_srch(hex_to_u8(theme.srch));
                    aurora
                        .global::<crate::Theme>()
                        .set_txt1(hex_to_u8(theme.txt1));
                    aurora
                        .global::<crate::Theme>()
                        .set_txt2(hex_to_u8(theme.txt2));
                    aurora
                        .global::<crate::Theme>()
                        .set_bgd0(hex_to_u8(theme.bgd0));
                    aurora
                        .global::<crate::Theme>()
                        .set_bgd1(hex_to_u8(theme.bgd1));
                    aurora
                        .global::<crate::Theme>()
                        .set_bgd2(hex_to_u8(theme.bgd2));
                    aurora
                        .global::<crate::Theme>()
                        .set_bgd3(hex_to_u8(theme.bgd3));
                    aurora
                        .global::<crate::Theme>()
                        .set_bgd4(hex_to_u8(theme.bgd4));
                });
            }
            other => tracing::info!("{other:?}"),
        }
    }
}

pub async fn interface(app: slint::Weak<AuroraPlayer>) -> anyhow::Result<()> {
    let mut stream: Option<UnixStream> = None;

    while stream.is_none() {
        let path = PathBuf::from("/tmp/aurora-daemon.sock");
        if let Ok(s) = UnixStream::connect(path).await {
            tracing::info!("Connected to the daemon.");
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
        writer_tx: tx,
        queue: vec![],
        playlist_result: None,
        playlist_list_results: vec![],
        search_results: vec![],
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
                let id = Uuid::parse_str(n.as_str()).unwrap();
                let req = Request::RemoveSong(id);
                let _ = state.lock().await.writer_tx.send(req).await;
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
        aurora.on_refresh_playlists(move || {
            let state = state.clone();

            tokio::spawn(async move {
                let _ = state
                    .lock()
                    .await
                    .writer_tx
                    .send(Request::PlaylistList)
                    .await;
            });
        });

        let state = state_clone.clone();
        aurora.on_select_playlist(move |id_str| {
            tracing::info!("Called for new playlist {id_str}");
            let state = state.clone();
            if let Ok(id) = uuid::Uuid::parse_str(&id_str) {
                tokio::spawn(async move {
                    let _ = state
                        .lock()
                        .await
                        .writer_tx
                        .send(Request::PlaylistGet(id))
                        .await;
                });
            } else {
                tracing::error!("{id_str}")
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

        let state = state_clone.clone();
        aurora.on_replace_queue(move |pl| {
            let state = state.clone();
            let mut songs = vec![];
            for song in pl.songs.iter() {
                if let Ok(id) = uuid::Uuid::parse_str(song.id.as_str()) {
                    songs.push(id);
                }
            }
            tokio::spawn(async move {
                let state = state.lock().await;
                let _ = state.writer_tx.send(Request::Clear).await;
                // let _ = state.writer_tx.send(Request::Play(songs[0])).await;
                let _ = state.writer_tx.send(Request::ReplaceQueue(songs)).await;
            });
        });

        aurora.invoke_refresh_playlists();
    });

    tokio::spawn(async move {
        if let Err(err) = unix_sender(writer, rx).await {
            tracing::error!("Sender Error: {err}");
        }
    });

    tokio::spawn(async move {
        if let Err(err) = tokio::spawn(unix_recver(reader, state.clone(), app)).await {
            tracing::error!("Receiver Error: {err}");
        }
    });

    Ok(())
}
