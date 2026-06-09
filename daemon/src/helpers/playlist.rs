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
        let data = fs::read_to_string(&file_path).await?;
        let mut playlist: Playlist = serde_json::from_str(&data)?;
        let original_len = playlist.songs.len();
        let mut out = vec![];
        for song in playlist.songs {
            self.get_art(song.id);
            if self.index.contains_key(&song.id) {
                out.push(Song::from(self.index.get(&song.id).unwrap()));
            } else {
                tracing::warn!(
                    "Removing missing song '{}' from playlist '{}'",
                    song.title,
                    playlist.title
                );
            }
        }
        playlist.songs = out;

        if playlist.songs.len() != original_len {
            let data = serde_json::to_string_pretty(&playlist)?;
            fs::write(&file_path, data).await?;
        }

        Ok(playlist)
    }

    pub async fn get_all_playlists(&mut self) -> Result<Vec<PlaylistMinimal>> {
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
                let mut art_paths = vec![];
                for song in playlist.songs.iter().take(4) {
                    self.get_art(song.id);
                    art_paths.push(
                        self.index
                            .get(&song.id)
                            .and_then(|s| s.art_path.clone())
                            .or_else(|| song.art_path.clone()),
                    );
                }
                result.push(PlaylistMinimal {
                    id: playlist.id,
                    name: playlist.title,
                    len: playlist.songs.len(),
                    art_paths,
                });
            }
        }

        Ok(result)
    }
}

pub async fn rename_playlist(playlist_id: Uuid, new_title: String) -> Result<()> {
    let playlists_dir = dirs::config_dir()
        .unwrap_or_else(|| {
            tracing::error!("No config dir.");
            exit(1)
        })
        .join("aurora-player")
        .join("playlists");

    let file_path = playlists_dir.join(format!("{}.json", playlist_id));
    let data = fs::read_to_string(&file_path).await?;
    let mut playlist: Playlist = serde_json::from_str(&data)?;
    playlist.title = new_title;
    let data = serde_json::to_string_pretty(&playlist)?;
    fs::write(&file_path, data).await?;
    Ok(())
}

pub async fn delete_playlist(playlist_id: Uuid) -> Result<()> {
    let playlists_dir = dirs::config_dir()
        .unwrap_or_else(|| {
            tracing::error!("No config dir.");
            exit(1)
        })
        .join("aurora-player")
        .join("playlists");

    let file_path = playlists_dir.join(format!("{}.json", playlist_id));
    fs::remove_file(&file_path).await?;
    Ok(())
}

pub async fn remove_song_from_playlist(playlist_id: Uuid, song_id: Uuid) -> Result<()> {
    let playlists_dir = dirs::config_dir()
        .unwrap_or_else(|| {
            tracing::error!("No config dir.");
            exit(1)
        })
        .join("aurora-player")
        .join("playlists");

    let file_path = playlists_dir.join(format!("{}.json", playlist_id));
    let data = fs::read_to_string(&file_path).await?;
    let mut playlist: Playlist = serde_json::from_str(&data)?;
    playlist.songs.retain(|s| s.id != song_id);
    let data = serde_json::to_string_pretty(&playlist)?;
    fs::write(&file_path, data).await?;
    Ok(())
}

pub async fn add_songs_to_playlist(
    state: &StateStruct,
    playlist_id: Uuid,
    song_ids: Vec<Uuid>,
) -> Result<()> {
    let playlists_dir = dirs::config_dir()
        .unwrap_or_else(|| {
            tracing::error!("No config dir.");
            exit(1)
        })
        .join("aurora-player")
        .join("playlists");

    tokio::fs::create_dir_all(&playlists_dir).await?;

    let file_path = playlists_dir.join(format!("{}.json", playlist_id));
    let data = fs::read_to_string(&file_path).await?;
    let mut playlist: Playlist = serde_json::from_str(&data)?;

    for song_id in &song_ids {
        if !playlist.songs.iter().any(|s| s.id == *song_id) {
            if let Some(song_meta) = state.index.get(song_id) {
                playlist.songs.push(Song::from(song_meta));
            }
        }
    }

    let data = serde_json::to_string_pretty(&playlist)?;
    fs::write(&file_path, data).await?;
    Ok(())
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
