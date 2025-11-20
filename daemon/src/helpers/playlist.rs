use std::process::exit;

use crate::types::*;
use std::io::Result;
use tokio::fs;
use uuid::Uuid;

pub async fn get_playlist(id: Uuid) -> Result<Playlist> {
    let playlists_dir = dirs::config_dir()
        .unwrap_or_else(|| {
            tracing::error!("No config dir.");
            exit(1)
        })
        .join("aurora-player")
        .join("playlists");

    tokio::fs::create_dir_all(&playlists_dir).await?;

    let file_path = playlists_dir.join(format!("{}.json", id));
    let data = fs::read_to_string(file_path).await?;
    let playlist: Playlist = serde_json::from_str(&data)?;
    Ok(playlist)
}

pub async fn get_all_playlists() -> Result<Vec<PlaylistMinimal>> {
    let playlists_dir = dirs::config_dir()
        .unwrap_or_else(|| {
            tracing::error!("No config dir.");
            exit(1)
        })
        .join("aurora-player")
        .join("playlists");

    tokio::fs::create_dir_all(&playlists_dir).await?;

    let mut dir = fs::read_dir(playlists_dir).await?;
    let mut result = vec![];

    while let Some(entry) = dir.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let data = fs::read_to_string(&path).await?;
            let playlist: Playlist = serde_json::from_str(&data)?;
            result.push(PlaylistMinimal {
                id: playlist.id,
                name: playlist.title,
                len: playlist.songs.len(),
            });
        }
    }

    Ok(result)
}

pub async fn create_playlist(playlist: Playlist) -> Result<String> {
    let playlists_dir = dirs::config_dir()
        .unwrap_or_else(|| {
            tracing::error!("No config dir.");
            exit(1)
        })
        .join("aurora-player")
        .join("playlists");

    fs::create_dir_all(&playlists_dir).await?;

    let file_path = playlists_dir.join(format!("{}.json", playlist.id));
    let data = serde_json::to_string_pretty(&playlist)?;
    fs::write(&file_path, data).await?;

    Ok(file_path.to_string_lossy().to_string())
}
