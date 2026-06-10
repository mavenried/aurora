use aurora_protocol::{Playlist, PlaylistIn, PlaylistMinimal, Song};
use std::io::Result;
use uuid::Uuid;

use crate::{
    helpers::db::{to_io, Db},
    types::StateStruct,
};

impl StateStruct {
    pub async fn get_playlist(&mut self, id: Uuid) -> Result<Playlist> {
        let db = self.db.clone();
        let id_str = id.to_string();

        let (title, song_ids) = tokio::task::spawn_blocking(move || {
            let conn = db.lock().unwrap();
            let title: String = conn
                .query_row(
                    "SELECT title FROM playlists WHERE id = ?1",
                    rusqlite::params![id_str],
                    |row| row.get(0),
                )
                .map_err(to_io)?;

            let mut stmt = conn
                .prepare(
                    "SELECT song_id FROM playlist_songs WHERE playlist_id = ?1 ORDER BY position",
                )
                .map_err(to_io)?;
            let id_str2 = id.to_string();
            let rows = stmt
                .query_map(rusqlite::params![id_str2], |row| {
                    let s: String = row.get(0)?;
                    Ok(s)
                })
                .map_err(to_io)?;
            let mut song_ids = Vec::new();
            for row in rows {
                let s = row.map_err(to_io)?;
                if let Ok(uuid) = Uuid::parse_str(&s) {
                    song_ids.push(uuid);
                }
            }
            Ok::<_, std::io::Error>((title, song_ids))
        })
        .await
        .map_err(to_io)??;

        let mut songs = Vec::new();
        for song_id in &song_ids {
            self.get_art(*song_id);
            if let Some(meta) = self.index.get(song_id) {
                songs.push(Song::from(meta));
            } else {
                tracing::warn!("Playlist song {} not found in index, skipping", song_id);
            }
        }

        Ok(Playlist { id, title, songs })
    }

    pub async fn get_all_playlists(&mut self) -> Result<Vec<PlaylistMinimal>> {
        let db = self.db.clone();

        // Returns (id, title, song_ids_for_first_4)
        let playlists_raw: Vec<(String, String, usize, Vec<String>)> =
            tokio::task::spawn_blocking(move || {
                let conn = db.lock().unwrap();
                let mut stmt = conn
                    .prepare(
                        "SELECT p.id, p.title, COUNT(ps.song_id) \
                         FROM playlists p \
                         LEFT JOIN playlist_songs ps ON p.id = ps.playlist_id \
                         GROUP BY p.id, p.title",
                    )
                    .map_err(to_io)?;

                struct Row {
                    id: String,
                    title: String,
                    count: usize,
                }

                let rows = stmt
                    .query_map([], |row| {
                        Ok(Row {
                            id: row.get(0)?,
                            title: row.get(1)?,
                            count: row.get::<_, i64>(2)? as usize,
                        })
                    })
                    .map_err(to_io)?;

                let mut result = Vec::new();
                for row in rows {
                    let r = row.map_err(to_io)?;
                    // Fetch first 4 song_ids for art
                    let mut art_stmt = conn
                        .prepare(
                            "SELECT song_id FROM playlist_songs \
                             WHERE playlist_id = ?1 ORDER BY position LIMIT 4",
                        )
                        .map_err(to_io)?;
                    let art_rows = art_stmt
                        .query_map(rusqlite::params![r.id], |row| {
                            let s: String = row.get(0)?;
                            Ok(s)
                        })
                        .map_err(to_io)?;
                    let mut art_ids = Vec::new();
                    for art_row in art_rows {
                        art_ids.push(art_row.map_err(to_io)?);
                    }
                    result.push((r.id, r.title, r.count, art_ids));
                }
                Ok::<_, std::io::Error>(result)
            })
            .await
            .map_err(to_io)??;

        let mut result = Vec::new();
        for (id_str, title, count, art_id_strs) in playlists_raw {
            let id = match Uuid::parse_str(&id_str) {
                Ok(id) => id,
                Err(_) => continue,
            };
            let mut art_paths = Vec::new();
            for s in &art_id_strs {
                if let Ok(song_id) = Uuid::parse_str(s) {
                    self.get_art(song_id);
                    art_paths.push(
                        self.index
                            .get(&song_id)
                            .and_then(|m| m.art_path.clone()),
                    );
                }
            }
            result.push(PlaylistMinimal {
                id,
                name: title,
                len: count,
                art_paths,
            });
        }

        Ok(result)
    }
}

pub async fn create_playlist(db: &Db, inp: PlaylistIn) -> Result<()> {
    let db = db.clone();
    let id = Uuid::new_v5(&Uuid::NAMESPACE_URL, inp.title.as_bytes());
    let title = inp.title.clone();
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO playlists (id, title) VALUES (?1, ?2)",
            rusqlite::params![id.to_string(), title],
        )
        .map_err(to_io)?;
        Ok(())
    })
    .await
    .map_err(to_io)?
}

pub async fn rename_playlist(db: &Db, playlist_id: Uuid, new_title: String) -> Result<()> {
    let db = db.clone();
    let id_str = playlist_id.to_string();
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().unwrap();
        conn.execute(
            "UPDATE playlists SET title = ?1 WHERE id = ?2",
            rusqlite::params![new_title, id_str],
        )
        .map_err(to_io)?;
        Ok(())
    })
    .await
    .map_err(to_io)?
}

pub async fn delete_playlist(db: &Db, playlist_id: Uuid) -> Result<()> {
    let db = db.clone();
    let id_str = playlist_id.to_string();
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().unwrap();
        conn.execute(
            "DELETE FROM playlists WHERE id = ?1",
            rusqlite::params![id_str],
        )
        .map_err(to_io)?;
        Ok(())
    })
    .await
    .map_err(to_io)?
}

pub async fn remove_song_from_playlist(
    db: &Db,
    playlist_id: Uuid,
    song_id: Uuid,
) -> Result<()> {
    let db = db.clone();
    let playlist_id_str = playlist_id.to_string();
    let song_id_str = song_id.to_string();
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().unwrap();
        conn.execute(
            "DELETE FROM playlist_songs WHERE playlist_id = ?1 AND song_id = ?2",
            rusqlite::params![playlist_id_str, song_id_str],
        )
        .map_err(to_io)?;
        Ok(())
    })
    .await
    .map_err(to_io)?
}

pub async fn add_songs_to_playlist(
    db: &Db,
    playlist_id: Uuid,
    song_ids: Vec<Uuid>,
) -> Result<()> {
    let db = db.clone();
    let playlist_id_str = playlist_id.to_string();
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().unwrap();
        conn.execute_batch("BEGIN").map_err(to_io)?;
        for song_id in &song_ids {
            let song_id_str = song_id.to_string();
            conn.execute(
                "INSERT OR IGNORE INTO playlist_songs (playlist_id, song_id, position) \
                 VALUES (?1, ?2, COALESCE(\
                     (SELECT MAX(position)+1 FROM playlist_songs WHERE playlist_id = ?3), \
                 0))",
                rusqlite::params![playlist_id_str, song_id_str, playlist_id_str],
            )
            .map_err(to_io)?;
        }
        conn.execute_batch("COMMIT").map_err(to_io)?;
        Ok(())
    })
    .await
    .map_err(to_io)?
}
