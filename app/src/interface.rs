use std::{collections::HashSet, path::PathBuf, process::Command, sync::Arc, time::Duration, vec};

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
                let mut state_locked = state.lock().await;
                let default_art = state_locked.default_art_buffer.clone();

                let current_id = status
                    .current_song
                    .as_ref()
                    .map(|s| s.id.to_string())
                    .unwrap_or_default();
                let song_changed = current_id != state_locked.current_song_id;
                state_locked.current_song_id = current_id.clone();

                let new_art_path = status.current_song.as_ref().and_then(|s| s.art_path.clone());
                let art_changed = new_art_path != state_locked.current_art_path;
                if art_changed {
                    state_locked.current_art_path = new_art_path.clone();
                }

                let shuffle = status.shuffle;
                let repeat = status.repeat;
                let volume = status.volume;
                let is_liked = state_locked.liked_song_ids.contains(&current_id);

                let position_ms = status.position.as_millis() as i32;

                drop(state_locked);

                // Load art on the async thread (not the UI thread) and only when it changes.
                // slint::Image is not Send, so we decode to SharedPixelBuffer here and
                // create the Image inside upgrade_in_event_loop.
                let maybe_art: Option<SharedPixelBuffer<Rgba8Pixel>> = if art_changed {
                    let buf = new_art_path
                        .as_ref()
                        .and_then(|p| std::fs::read(p).ok())
                        .and_then(|data| album_art_from_data(&data).ok())
                        .unwrap_or(default_art);
                    Some(buf)
                } else {
                    None
                };

                let _ = app.upgrade_in_event_loop(move |aurora| {
                    aurora.set_is_paused(status.is_paused);
                    aurora.set_position(position_ms);
                    aurora.set_shuffle(shuffle);
                    aurora.set_repeat_mode(repeat as i32);
                    aurora.set_volume(volume);
                    aurora.set_current_song_liked(is_liked);

                    // Song metadata only changes when the song changes — avoid triggering
                    // reactive re-evaluations across SongList/PlaylistCard bindings every tick.
                    if song_changed {
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
                        aurora.set_current_playing_id(current_id.into());
                    }

                    if let Some(buf) = maybe_art {
                        aurora.set_has_art(new_art_path.is_some());
                        aurora.set_AlbumArt(Image::from_rgba8(buf));
                    }
                });
            }

            Response::Queue(queue) => {
                tracing::info!("Received: Queue");
                let mut state = state.lock().await;
                state.queue = queue;
                state.update_queue(app.clone()).await;
            }
            Response::SearchResults(results) => {
                tracing::info!("Received: SearchResults, len:{}", results.len());
                let mut state = state.lock().await;
                if state.pending_artist_search {
                    state.pending_artist_search = false;
                    state.artist_songs = results;
                    state.update_artist_songs(app.clone()).await;
                } else {
                    state.search_results = results;
                    state.update_search_results(app.clone()).await;
                }
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
            Response::Volume(volume) => {
                let _ = app.upgrade_in_event_loop(move |aurora| {
                    aurora.set_volume(volume);
                });
            }
            Response::ArtistList(list) => {
                tracing::info!("Received: ArtistList, len:{}", list.len());
                let mut state_locked = state.lock().await;
                state_locked.artist_list = list.clone();
                drop(state_locked);
                let _ = app.upgrade_in_event_loop(move |aurora| {
                    use std::rc::Rc;
                    use slint::{ModelRc, VecModel};
                    let items: Vec<slint::SharedString> =
                        list.iter().map(|s| s.as_str().into()).collect();
                    aurora.set_artist_list(ModelRc::new(Rc::new(VecModel::from(items))));
                });
            }
            Response::LastPlayed(songs) => {
                tracing::info!("Received: LastPlayed, len:{}", songs.len());
                let mut state_locked = state.lock().await;
                state_locked.last_played = songs;
                state_locked.update_last_played(app.clone()).await;
            }
            Response::LikedSongs(songs) => {
                tracing::info!("Received: LikedSongs, len:{}", songs.len());
                let mut state = state.lock().await;
                state.liked_song_ids = songs.iter().map(|s| s.id.to_string()).collect();
                state.liked_songs = songs;
                state.update_liked_songs(app.clone()).await;
                state.update_search_results(app.clone()).await;
                state.update_artist_songs(app.clone()).await;
                if state.playlist_result.is_some() {
                    state.update_playlist_results(app.clone()).await;
                }
                state.update_last_played(app.clone()).await;
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
        selected_song_ids: HashSet::new(),
        liked_song_ids: HashSet::new(),
        current_song_id: String::new(),
        artist_list: vec![],
        artist_songs: vec![],
        last_played: vec![],
        liked_songs: vec![],
        pending_artist_search: false,
        current_art_path: None,
    }));

    let state_clone = state.clone();
    let app_for_sel = app.clone();
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
                let req = Request::RemoveSongAt(n as usize);
                let _ = state.lock().await.writer_tx.send(req).await;
            });
        });

        let state = state_clone.clone();
        aurora.on_queue_move(move |from, to| {
            let state = state.clone();
            tokio::spawn(async move {
                let req = Request::MoveQueue { from: from as usize, to: to as usize };
                let _ = state.lock().await.writer_tx.send(req).await;
            });
        });

        let state = state_clone.clone();
        aurora.on_search(move |q, m, f| {
            let state = state.clone();

            if q.trim().len() > 2 || (!q.trim().is_empty() && f) {
                let query = if m == "By Artist" {
                    Request::Search(SearchType::ByArtist(q.trim().to_string()))
                } else {
                    Request::Search(SearchType::ByTitle(q.trim().to_string()))
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
            let Ok(id) = uuid::Uuid::parse_str(&id) else { return };
            tokio::spawn(async move {
                let _ = state.lock().await.writer_tx.send(Request::Play(id)).await;
            });
        });

        let state = state_clone.clone();
        aurora.on_enqueue(move |id| {
            let state = state.clone();
            let id_str = id.to_string();
            tokio::spawn(async move {
                let (ids, writer_tx) = {
                    let state_locked = state.lock().await;
                    let ids: Vec<uuid::Uuid> = if state_locked.selected_song_ids.contains(&id_str) {
                        state_locked.selected_song_ids.iter()
                            .filter_map(|s| uuid::Uuid::parse_str(s).ok())
                            .collect()
                    } else {
                        uuid::Uuid::parse_str(&id_str).map(|id| vec![id]).unwrap_or_default()
                    };
                    (ids, state_locked.writer_tx.clone())
                };
                for song_id in ids {
                    let _ = writer_tx.send(Request::Enqueue(song_id)).await;
                }
            });
        });

        let state = state_clone.clone();
        aurora.on_play_next(move |id| {
            let state = state.clone();
            let id_str = id.to_string();
            tokio::spawn(async move {
                let (ids, writer_tx) = {
                    let state_locked = state.lock().await;
                    let ids: Vec<uuid::Uuid> = if state_locked.selected_song_ids.contains(&id_str) {
                        state_locked.selected_song_ids.iter()
                            .filter_map(|s| uuid::Uuid::parse_str(s).ok())
                            .collect()
                    } else {
                        uuid::Uuid::parse_str(&id_str).map(|id| vec![id]).unwrap_or_default()
                    };
                    (ids, state_locked.writer_tx.clone())
                };
                for song_id in ids {
                    let _ = writer_tx.send(Request::PlayNext(song_id)).await;
                }
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

                let _ = state.writer_tx.send(Request::ReplaceQueue(songs)).await;
            });
        });

        let state = state_clone.clone();
        let app_clr = app_for_sel.clone();
        aurora.on_clear_song_selection(move || {
            let state = state.clone();
            let app = app_clr.clone();
            tokio::spawn(async move {
                let mut state = state.lock().await;
                state.selected_song_ids.clear();
                state.update_search_results(app.clone()).await;
                state.update_artist_songs(app.clone()).await;
                if state.playlist_result.is_some() {
                    state.update_playlist_results(app.clone()).await;
                }
            });
        });

        let state = state_clone.clone();
        let app_sel = app_for_sel.clone();
        aurora.on_toggle_song_selection(move |id| {
            let state = state.clone();
            let app = app_sel.clone();
            let id_str = id.to_string();
            tokio::spawn(async move {
                let mut state = state.lock().await;
                if state.selected_song_ids.contains(id_str.as_str()) {
                    state.selected_song_ids.remove(id_str.as_str());
                } else {
                    state.selected_song_ids.insert(id_str);
                }
                state.update_search_results(app.clone()).await;
                if state.playlist_result.is_some() {
                    state.update_playlist_results(app.clone()).await;
                }
            });
        });

        let state = state_clone.clone();
        aurora.on_create_playlist(move |name| {
            let state = state.clone();
            let name_str = name.trim().to_string();
            if !name_str.is_empty() {
                tokio::spawn(async move {
                    let _ = state
                        .lock()
                        .await
                        .writer_tx
                        .send(Request::PlaylistCreate(aurora_protocol::PlaylistIn {
                            title: name_str,
                            songs: vec![],
                        }))
                        .await;
                });
            }
        });

        let state = state_clone.clone();
        aurora.on_rename_playlist(move |playlist_id_str, new_title| {
            let state = state.clone();
            let new_title = new_title.trim().to_string();
            if !new_title.is_empty() {
                tokio::spawn(async move {
                    if let Ok(playlist_id) = uuid::Uuid::parse_str(&playlist_id_str) {
                        let _ = state
                            .lock()
                            .await
                            .writer_tx
                            .send(Request::PlaylistRename { playlist_id, new_title })
                            .await;
                    }
                });
            }
        });

        let state = state_clone.clone();
        aurora.on_delete_playlist(move |playlist_id_str| {
            let state = state.clone();
            tokio::spawn(async move {
                if let Ok(id) = uuid::Uuid::parse_str(&playlist_id_str) {
                    let _ = state
                        .lock()
                        .await
                        .writer_tx
                        .send(Request::PlaylistDelete(id))
                        .await;
                }
            });
        });

        let state = state_clone.clone();
        aurora.on_remove_song_from_playlist(move |playlist_id_str, song_id_str| {
            let state = state.clone();
            tokio::spawn(async move {
                let (ids, writer_tx) = {
                    let state_locked = state.lock().await;
                    let ids: Vec<String> = if state_locked.selected_song_ids.contains(song_id_str.as_str()) {
                        state_locked.selected_song_ids.iter().cloned().collect()
                    } else {
                        vec![song_id_str.to_string()]
                    };
                    (ids, state_locked.writer_tx.clone())
                };
                if let Ok(playlist_id) = uuid::Uuid::parse_str(&playlist_id_str) {
                    for id_str in &ids {
                        if let Ok(song_id) = uuid::Uuid::parse_str(id_str) {
                            let _ = writer_tx.send(Request::PlaylistRemoveSong { playlist_id, song_id }).await;
                        }
                    }
                }
            });
        });

        let state = state_clone.clone();
        aurora.on_add_songs_to_playlist(move |playlist_id_str, primary_song_id| {
            let state = state.clone();
            tokio::spawn(async move {
                let (song_ids, writer_tx) = {
                    let state_locked = state.lock().await;
                    let song_ids: Vec<uuid::Uuid> = if state_locked.selected_song_ids.contains(primary_song_id.as_str()) {
                        state_locked
                            .selected_song_ids
                            .iter()
                            .filter_map(|id| uuid::Uuid::parse_str(id).ok())
                            .collect()
                    } else {
                        uuid::Uuid::parse_str(&primary_song_id)
                            .map(|id| vec![id])
                            .unwrap_or_default()
                    };
                    (song_ids, state_locked.writer_tx.clone())
                };
                if let Ok(playlist_id) = uuid::Uuid::parse_str(&playlist_id_str) {
                    let _ = writer_tx
                        .send(Request::PlaylistAddSongs { playlist_id, song_ids })
                        .await;
                }
            });
        });

        let state = state_clone.clone();
        aurora.on_set_shuffle(move |b| {
            let state = state.clone();
            tokio::spawn(async move {
                let _ = state.lock().await.writer_tx.send(Request::SetShuffle(b)).await;
            });
        });

        let state = state_clone.clone();
        aurora.on_set_repeat(move |r| {
            let state = state.clone();
            tokio::spawn(async move {
                let _ = state.lock().await.writer_tx.send(Request::SetRepeat(r as u8)).await;
            });
        });

        let state = state_clone.clone();
        aurora.on_set_volume(move |v| {
            let state = state.clone();
            tokio::spawn(async move {
                let _ = state.lock().await.writer_tx.send(Request::SetVolume(v)).await;
            });
        });

        let state = state_clone.clone();
        aurora.on_get_artist_list(move || {
            let state = state.clone();
            tokio::spawn(async move {
                let _ = state.lock().await.writer_tx.send(Request::GetArtistList).await;
            });
        });

        let state = state_clone.clone();
        aurora.on_like_song(move |id| {
            let state = state.clone();
            let id_str = id.to_string();
            tokio::spawn(async move {
                let (ids, writer_tx) = {
                    let state_locked = state.lock().await;
                    let ids: Vec<uuid::Uuid> = if state_locked.selected_song_ids.contains(&id_str) {
                        state_locked.selected_song_ids.iter()
                            .filter_map(|s| uuid::Uuid::parse_str(s).ok())
                            .collect()
                    } else {
                        uuid::Uuid::parse_str(&id_str).map(|u| vec![u]).unwrap_or_default()
                    };
                    (ids, state_locked.writer_tx.clone())
                };
                for song_id in ids {
                    let _ = writer_tx.send(Request::LikeSong(song_id)).await;
                }
            });
        });

        let state = state_clone.clone();
        aurora.on_unlike_song(move |id| {
            let state = state.clone();
            let id_str = id.to_string();
            tokio::spawn(async move {
                let (ids, writer_tx) = {
                    let state_locked = state.lock().await;
                    let ids: Vec<uuid::Uuid> = if state_locked.selected_song_ids.contains(&id_str) {
                        state_locked.selected_song_ids.iter()
                            .filter_map(|s| uuid::Uuid::parse_str(s).ok())
                            .collect()
                    } else {
                        uuid::Uuid::parse_str(&id_str).map(|u| vec![u]).unwrap_or_default()
                    };
                    (ids, state_locked.writer_tx.clone())
                };
                for song_id in ids {
                    let _ = writer_tx.send(Request::UnlikeSong(song_id)).await;
                }
            });
        });

        let state = state_clone.clone();
        aurora.on_get_liked_songs(move || {
            let state = state.clone();
            tokio::spawn(async move {
                let _ = state.lock().await.writer_tx.send(Request::GetLikedSongs).await;
            });
        });

        let state = state_clone.clone();
        aurora.on_get_last_played(move || {
            let state = state.clone();
            tokio::spawn(async move {
                let _ = state.lock().await.writer_tx.send(Request::GetLastPlayed).await;
            });
        });

        let state = state_clone.clone();
        aurora.on_get_songs_by_artist(move |artist| {
            let state = state.clone();
            let artist_str = artist.to_string();
            tokio::spawn(async move {
                let mut state_locked = state.lock().await;
                state_locked.pending_artist_search = true;
                let _ = state_locked
                    .writer_tx
                    .send(Request::Search(aurora_protocol::SearchType::ByArtist(artist_str)))
                    .await;
            });
        });

        aurora.invoke_refresh_playlists();
    });

    let app_sender = app.clone();
    tokio::spawn(async move {
        if let Err(err) = unix_sender(writer, rx).await {
            tracing::error!("Sender Error: {err}");
            let _ = app_sender.upgrade_in_event_loop(|aurora| aurora.set_disconnected(true));
        }
    });

    tokio::spawn(async move {
        if let Err(err) = tokio::spawn(unix_recver(reader, state.clone(), app.clone())).await {
            tracing::error!("Receiver Error: {err}");
            let _ = app.upgrade_in_event_loop(|aurora| aurora.set_disconnected(true));
        }
    });

    Ok(())
}
