use crate::{helpers::send_to_all, types::State};
use aurora_protocol::{Response, Song};
use mpris_server::{Metadata, PlaybackStatus, Player, Time, TrackId};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{self, UnboundedSender};
use uuid::Uuid;

pub enum PlayerCommand {
    PlayPause,
    Play,
    Pause,
    Stop,
    Next,
    Prev,
    SeekOffset(Time),
    SeekAbsolute(Time),
    SetVolume(f64),
}

pub async fn init(state: State) -> anyhow::Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<PlayerCommand>();

    let state_for_ctrl = state.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            controller(tx, state_for_ctrl).await.ok();
        });
    });

    while let Some(cmd) = rx.recv().await {
        match cmd {
            PlayerCommand::PlayPause => {
                state.lock().await.pause().await;
            }
            PlayerCommand::Play => {
                let mut s = state.lock().await;
                if s.is_paused() {
                    s.pause().await;
                }
            }
            PlayerCommand::Pause => {
                let mut s = state.lock().await;
                if !s.is_paused() {
                    s.pause().await;
                }
            }
            PlayerCommand::Stop => {
                state.lock().await.clear().await;
            }
            PlayerCommand::Next => {
                let mut sl = state.lock().await;
                sl.next(1).await;
                sl.add().await;
                let queue = sl.queue.iter().map(Song::from).collect();
                drop(sl);
                send_to_all(&state, &Response::Queue(queue)).await?;
            }
            PlayerCommand::Prev => {
                let mut sl = state.lock().await;
                sl.prev(1).await;
                sl.add().await;
                let queue = sl.queue.iter().map(Song::from).collect();
                drop(sl);
                send_to_all(&state, &Response::Queue(queue)).await?;
            }
            PlayerCommand::SeekOffset(offset) => {
                let mut sl = state.lock().await;
                if let Some(audio) = &mut sl.audio {
                    let current_us = audio.get_position().as_micros() as i64;
                    let new_us = (current_us + offset.as_micros()).max(0) as u64;
                    audio.seek(Duration::from_micros(new_us));
                }
            }
            PlayerCommand::SeekAbsolute(pos) => {
                let mut sl = state.lock().await;
                if let Some(audio) = &mut sl.audio && !pos.is_negative() {
                    audio.seek(Duration::from_micros(pos.as_micros() as u64));
                }
            }
            PlayerCommand::SetVolume(vol) => {
                state.lock().await.sink.set_volume(vol.clamp(0.0, 1.0) as f32);
            }
        }
    }

    Ok(())
}

pub async fn controller(cmd_tx: UnboundedSender<PlayerCommand>, state: State) -> anyhow::Result<()> {
    let player = Player::builder("me.mavenried.Aurora")
        .identity("Aurora")
        .can_play(true)
        .can_pause(true)
        .can_go_next(true)
        .can_go_previous(true)
        .can_seek(true)
        .build()
        .await?;

    let tx = cmd_tx.clone();
    player.connect_play_pause(move |_| {
        let _ = tx.send(PlayerCommand::PlayPause);
    });

    let tx = cmd_tx.clone();
    player.connect_play(move |_| {
        let _ = tx.send(PlayerCommand::Play);
    });

    let tx = cmd_tx.clone();
    player.connect_pause(move |_| {
        let _ = tx.send(PlayerCommand::Pause);
    });

    let tx = cmd_tx.clone();
    player.connect_stop(move |_| {
        let _ = tx.send(PlayerCommand::Stop);
    });

    let tx = cmd_tx.clone();
    player.connect_next(move |_| {
        let _ = tx.send(PlayerCommand::Next);
    });

    let tx = cmd_tx.clone();
    player.connect_previous(move |_| {
        let _ = tx.send(PlayerCommand::Prev);
    });

    let tx = cmd_tx.clone();
    player.connect_seek(move |_, offset| {
        let _ = tx.send(PlayerCommand::SeekOffset(offset));
    });

    let tx = cmd_tx.clone();
    player.connect_set_position(move |_, _track_id, pos| {
        if !pos.is_negative() {
            let _ = tx.send(PlayerCommand::SeekAbsolute(pos));
        }
    });

    let tx = cmd_tx.clone();
    player.connect_set_volume(move |_, vol| {
        let _ = tx.send(PlayerCommand::SetVolume(vol));
    });

    let run_task = player.run();

    let sync_task = async {
        let mut last_song_id: Option<Uuid> = None;
        let mut last_paused: Option<bool> = None;
        let mut last_position = Time::ZERO;
        let mut last_tick = Instant::now();

        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;

            let sl = state.lock().await;
            let current_id = sl.current_song.as_ref().map(|s| s.id);
            let is_paused = sl.is_paused();
            let has_song = current_id.is_some();
            let position = sl
                .audio
                .as_ref()
                .map(|a| Time::from_micros(a.get_position().as_micros() as i64))
                .unwrap_or(Time::ZERO);
            let volume = sl.sink.volume() as f64;

            // Collect song metadata before dropping the lock
            let song_snapshot = sl.current_song.as_ref().map(|s| {
                (
                    s.id,
                    s.title.clone(),
                    s.artists.clone(),
                    s.duration,
                    s.art_path.clone(),
                )
            });
            drop(sl);

            // Update metadata when the current song changes
            if current_id != last_song_id {
                last_song_id = current_id;
                last_paused = None; // force status re-emit after song change

                if let Some((id, title, artists, duration, art_path)) = song_snapshot {
                    let track_id = TrackId::try_from(format!(
                        "/me/mavenried/Aurora/track/{}",
                        id.simple()
                    ))
                    .unwrap_or(TrackId::NO_TRACK);

                    let mut meta = Metadata::new();
                    meta.set_trackid(Some(track_id));
                    meta.set_title(Some(title));
                    meta.set_artist(Some(artists));
                    meta.set_length(Some(Time::from_micros(duration.as_micros() as i64)));
                    if let Some(art) = art_path {
                        meta.set_art_url(Some(format!("file://{}", art.display())));
                    }
                    let _ = player.set_metadata(meta).await;
                } else {
                    let _ = player.set_metadata(Metadata::new()).await;
                    let _ = player.set_playback_status(PlaybackStatus::Stopped).await;
                    last_paused = Some(false);
                    last_position = Time::ZERO;
                    last_tick = Instant::now();
                    continue;
                }
            }

            // Update playback status when it changes
            if has_song && last_paused != Some(is_paused) {
                last_paused = Some(is_paused);
                let status = if is_paused {
                    PlaybackStatus::Paused
                } else {
                    PlaybackStatus::Playing
                };
                let _ = player.set_playback_status(status).await;
            }

            // Keep the cached position up to date (no D-Bus signal emitted)
            player.set_position(position);

            // Emit Seeked when position jumps more than 1.5 s from where it should be
            let elapsed_us = last_tick.elapsed().as_micros() as i64;
            let expected_us = last_position.as_micros()
                + if is_paused { 0 } else { elapsed_us };
            let diff = (position.as_micros() - expected_us).unsigned_abs();
            if diff > 1_500_000 {
                let _ = player.seeked(position).await;
            }
            last_position = position;
            last_tick = Instant::now();

            // Keep MPRIS volume in sync with the sink
            let _ = player.set_volume(volume).await;
        }
    };

    tokio::select! {
        _ = run_task => {},
        _ = sync_task => {},
    }

    Ok(())
}
