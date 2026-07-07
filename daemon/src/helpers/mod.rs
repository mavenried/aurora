pub mod db;
mod history;
mod index;
mod liked_store;
mod playlist;
use crate::types::*;
use anyhow::Ok;
use aurora_protocol::Response;
pub use history::*;
pub use index::*;
pub use liked_store::*;
pub use playlist::*;

use tokio::io::AsyncWriteExt;
use uuid::Uuid;

pub async fn send_to_client(socket: &WriteSocket, response: &Response) -> anyhow::Result<()> {
    let encoded = serde_json::to_string(response)?;
    let len = (encoded.len() as u32).to_be_bytes();
    let mut socket_locked = socket.lock().await;
    socket_locked.write_all(&len).await?;
    socket_locked.write_all(encoded.as_bytes()).await?;
    Ok(())
}

pub async fn send_to_all(state: &State, response: &Response) -> anyhow::Result<()> {
    let clients: Vec<_> = state.lock().await.clients.clone();
    for client in &clients {
        let _ = send_to_client(client, response).await;
    }
    Ok(())
}

pub fn trigger_art_for(state: State, pending: Vec<(Uuid, std::path::PathBuf, db::Db)>) {
    for (id, path, db) in pending {
        let state = state.clone();
        tokio::spawn(async move {
            if let Some(art_path) = extract_art(id, path, db).await {
                if let Some(song) = state.lock().await.index.get_mut(&id) {
                    song.art_path = Some(art_path);
                }
            }
        });
    }
}

pub async fn extract_art(id: Uuid, audio_path: std::path::PathBuf, db: db::Db) -> Option<std::path::PathBuf> {
    tokio::task::spawn_blocking(move || {
        use image::{ImageFormat, ImageReader, imageops::FilterType};
        use lofty::file::TaggedFileExt;
        use std::io::Cursor;

        let outdir = dirs::cache_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp/"))
            .join("aurora-player");
        let cache_file = outdir.join(format!("{id}.jpg"));

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
        let resized = image::imageops::resize(&image, 100, 100, FilterType::Nearest);

        std::fs::create_dir_all(&outdir).ok()?;

        let file = std::fs::File::create(&cache_file).ok()?;
        resized
            .write_to(&mut std::io::BufWriter::new(file), ImageFormat::Jpeg)
            .ok()?;

        tracing::debug!("Wrote album art to {cache_file:?}");

        let id_str = id.to_string();
        let art_str = cache_file.to_string_lossy().to_string();
        if let Ok(conn) = db.lock() {
            conn.execute(
                "UPDATE songs SET art_path = ?1 WHERE id = ?2",
                rusqlite::params![art_str, id_str],
            )
            .ok();
        }

        Some(cache_file)
    })
    .await
    .ok()
    .flatten()
}
