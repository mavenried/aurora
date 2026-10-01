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

use tokio::{io::AsyncWriteExt, time::timeout};
use uuid::Uuid;

pub async fn send_to_client(socket: &WriteSocket, response: &Response) -> anyhow::Result<()> {
    let encoded = serde_json::to_string(response)?;
    let len = (encoded.len() as u32).to_be_bytes();
    timeout(std::time::Duration::from_secs(5), async {
        let mut socket_locked = socket.lock().await;
        socket_locked.write_all(&len).await?;
        socket_locked.write_all(encoded.as_bytes()).await?;
        Ok::<(), std::io::Error>(())
    })
    .await
    .map_err(|_| anyhow::anyhow!("timed out writing response to client"))??;
    Ok(())
}

pub async fn send_to_all(state: &State, response: &Response) -> anyhow::Result<()> {
    let clients: Vec<_> = state.lock().await.clients.clone();
    for client in &clients {
        let _ = send_to_client(client, response).await;
    }
    Ok(())
}

pub async fn extract_art(id: Uuid, audio_path: std::path::PathBuf, db: db::Db) -> Option<std::path::PathBuf> {
    extract_art_sized(id, audio_path, Some(db), 100, "jpg").await
}

pub async fn extract_highres_art(id: Uuid, audio_path: std::path::PathBuf) -> Option<std::path::PathBuf> {
    extract_art_sized(id, audio_path, None, 0, "png").await
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
        let cache_file = if size == 0 {
            outdir.join(format!("{id}-highres.{extension}"))
        } else {
            outdir.join(format!("{id}-{size}.{extension}"))
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
        let resized = if size == 0 {
            image
        } else {
            image::imageops::resize(&image, size, size, FilterType::Lanczos3)
        };

        std::fs::create_dir_all(&outdir).ok()?;

        let file = std::fs::File::create(&cache_file).ok()?;
        let format = if size == 0 { ImageFormat::Png } else { ImageFormat::Jpeg };
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
