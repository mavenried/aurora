use std::process::exit;

use aurora_protocol::{Playlist, PlaylistIn, PlaylistMinimal, Song};
use std::io::Result;
use tokio::fs;
use uuid::Uuid;

use crate::types::StateStruct;

impl StateStruct {
    pub async fn get_playlist(&mut self, id: Uuid) -> Result<Playlist> {
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
        let mut playlist: Playlist = serde_json::from_str(&data)?;
        let mut out = vec![];
        for song in playlist.songs {
            self.get_art(song.id);
            if self.index.contains_key(&song.id) {
                out.push(Song::from(self.index.get(&song.id).unwrap()));
            } else {
                tracing::warn!("Index does not have {}", song.title);
            }
        }
        playlist.songs = out;
        Ok(playlist)
    }
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

pub async fn create_playlist(inp: PlaylistIn) -> Result<String> {
    let playlists_dir = dirs::config_dir()
        .unwrap_or_else(|| {
            tracing::error!("No config dir.");
            exit(1)
        })
        .join("aurora-player")
        .join("playlists");

    fs::create_dir_all(&playlists_dir).await?;

    let playlist = Playlist {
        title: inp.title.clone(),
        id: Uuid::new_v5(&Uuid::NAMESPACE_URL, inp.title.as_bytes()),
        songs: inp.songs,
    };

    let file_path = playlists_dir.join(format!("{}.json", playlist.id));
    let data = serde_json::to_string_pretty(&playlist)?;
    fs::write(&file_path, data).await?;

    Ok(file_path.to_string_lossy().to_string())
}
