pub mod db;
mod history;
mod index;
mod liked_store;
mod playlist;
use crate::types::*;
use aurora_protocol::Response;
pub use history::*;
pub use index::*;
pub use liked_store::*;
pub use playlist::*;

use futures::future::join_all;
use std::sync::Arc;
use std::time::Instant;
use tokio::{io::AsyncWriteExt, time::timeout};
use uuid::Uuid;

fn response_name(response: &Response) -> &'static str {
    match response {
        Response::Error { .. } => "Error",
        Response::Status(_) => "Status",
        Response::SearchResults(_) => "SearchResults",
        Response::PlaylistResults(_) => "PlaylistResults",
        Response::PlaylistList(_) => "PlaylistList",
        Response::Queue(_) => "Queue",
        Response::Theme(_) => "Theme",
        Response::Volume(_) => "Volume",
        Response::ArtistList(_) => "ArtistList",
        Response::LastPlayed(_) => "LastPlayed",
        Response::LikedSongs(_) => "LikedSongs",
        Response::ArtReady { .. } => "ArtReady",
        Response::HighResArtReady { .. } => "HighResArtReady",
    }
}

pub async fn send_to_client(socket: &WriteSocket, response: &Response) -> anyhow::Result<()> {
    let encoded = serde_json::to_string(response)?;
    let len = (encoded.len() as u32).to_be_bytes();
    let response_type = response_name(response);
    let client_id = Arc::as_ptr(socket) as usize;
    let started = Instant::now();
    tracing::debug!(
        client_id,
        response = response_type,
        bytes = encoded.len(),
        "Writing response frame"
    );
    match timeout(std::time::Duration::from_secs(5), async {
        let mut socket_locked = socket.lock().await;
        socket_locked.write_all(&len).await?;
        socket_locked.write_all(encoded.as_bytes()).await?;
        Ok::<(), std::io::Error>(())
    })
    .await
    {
        Ok(Ok(())) => {
            tracing::debug!(
                response = response_type,
                client_id,
                bytes = encoded.len(),
                elapsed_ms = started.elapsed().as_millis(),
                "Response frame sent"
            );
            Ok(())
        }
        Ok(Err(err)) => {
            tracing::warn!(client_id, response = response_type, error = %err, "Socket write failed");
            Err(err.into())
        }
        Err(_) => {
            tracing::warn!(
                response = response_type,
                client_id,
                bytes = encoded.len(),
                elapsed_ms = started.elapsed().as_millis(),
                "Timed out writing response frame"
            );
            Err(anyhow::anyhow!("timed out writing response to client"))
        }
    }
}

pub async fn send_to_all(state: &State, response: &Response) -> anyhow::Result<()> {
    let clients: Vec<_> = state.lock().await.clients.clone();
    let response_type = response_name(response);
    tracing::debug!(
        response = response_type,
        clients = clients.len(),
        "Broadcast started"
    );
    let results = join_all(
        clients
            .iter()
            .map(|client| send_to_client(client, response)),
    )
    .await;
    let failed: Vec<_> = clients
        .iter()
        .zip(results)
        .filter_map(|(client, result)| result.is_err().then(|| client.clone()))
        .collect();

    if !failed.is_empty() {
        tracing::warn!(
            response = response_type,
            failed_clients = failed.len(),
            "Removing clients with failed broadcast writes"
        );
        state
            .lock()
            .await
            .clients
            .retain(|client| !failed.iter().any(|failed| Arc::ptr_eq(client, failed)));
    }
    tracing::debug!(
        response = response_type,
        clients = clients.len(),
        failed_clients = failed.len(),
        "Broadcast completed"
    );
    Ok(())
}

pub async fn extract_art(
    id: Uuid,
    audio_path: std::path::PathBuf,
    db: db::Db,
) -> Option<std::path::PathBuf> {
    extract_art_sized(id, audio_path, Some(db), 100, "jpg").await
}

pub async fn extract_highres_art(
    id: Uuid,
    audio_path: std::path::PathBuf,
) -> Option<std::path::PathBuf> {
    extract_art_sized(id, audio_path, None, 1000, "png").await
}

async fn extract_art_sized(
    id: Uuid,
    audio_path: std::path::PathBuf,
    db: Option<db::Db>,
    size: u32,
    extension: &'static str,
) -> Option<std::path::PathBuf> {
    tokio::task::spawn_blocking(move || {
        use image::{ImageFormat, ImageReader, imageops::FilterType};
        use lofty::file::TaggedFileExt;
        use std::io::Cursor;

        let outdir = dirs::cache_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp/"))
            .join("aurora-player");
        let cache_file = if db.is_none() {
            outdir.join(format!("{id}-highres.{extension}"))
        } else {
            outdir.join(format!("{id}-lowres.{extension}"))
        };

        if std::fs::exists(&cache_file).unwrap_or(false) {
            return Some(cache_file);
        }

        tracing::info!("Extracting album art for {id}");

        let tagged_file = lofty::read_from_path(&audio_path).ok()?;
        let tag = tagged_file.primary_tag()?;
        let pic = tag.pictures().first()?;

        let image = ImageReader::new(Cursor::new(pic.data()))
            .with_guessed_format()
            .ok()?
            .decode()
            .ok()?
            .into_rgb8();
        let resized = if db.is_none() {
            image::imageops::thumbnail(&image, size, size)
        } else {
            image::imageops::resize(&image, size, size, FilterType::Lanczos3)
        };

        std::fs::create_dir_all(&outdir).ok()?;

        let file = std::fs::File::create(&cache_file).ok()?;
        let format = if db.is_none() {
            ImageFormat::Png
        } else {
            ImageFormat::Jpeg
        };
        resized
            .write_to(&mut std::io::BufWriter::new(file), format)
            .ok()?;

        tracing::debug!("Wrote album art to {cache_file:?}");

        if size == 100 {
            let id_str = id.to_string();
            let art_str = cache_file.to_string_lossy().to_string();
            if let Some(db) = db {
                if let Ok(conn) = db.lock() {
                    conn.execute(
                        "UPDATE songs SET art_path = ?1 WHERE id = ?2",
                        rusqlite::params![art_str, id_str],
                    )
                    .ok();
                }
            }
        }

        Some(cache_file)
    })
    .await
    .ok()
    .flatten()
}
