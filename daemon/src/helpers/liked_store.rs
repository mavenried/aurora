use std::collections::HashSet;
use uuid::Uuid;

use crate::helpers::db::{to_io, Db};

pub async fn load_liked(db: &Db) -> std::io::Result<HashSet<Uuid>> {
    let db = db.clone();
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT song_id FROM liked_songs")
            .map_err(to_io)?;
        let rows = stmt
            .query_map([], |row| {
                let s: String = row.get(0)?;
                Ok(s)
            })
            .map_err(to_io)?;
        let mut set = HashSet::new();
        for row in rows {
            let s = row.map_err(to_io)?;
            if let Ok(id) = Uuid::parse_str(&s) {
                set.insert(id);
            }
        }
        Ok(set)
    })
    .await
    .map_err(to_io)?
}

pub async fn add_liked(db: &Db, id: Uuid) -> std::io::Result<()> {
    let db = db.clone();
    let id_str = id.to_string();
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO liked_songs (song_id) VALUES (?1)",
            rusqlite::params![id_str],
        )
        .map_err(to_io)?;
        Ok(())
    })
    .await
    .map_err(to_io)?
}

pub async fn remove_liked(db: &Db, id: Uuid) -> std::io::Result<()> {
    let db = db.clone();
    let id_str = id.to_string();
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().unwrap();
        conn.execute(
            "DELETE FROM liked_songs WHERE song_id = ?1",
            rusqlite::params![id_str],
        )
        .map_err(to_io)?;
        Ok(())
    })
    .await
    .map_err(to_io)?
}
