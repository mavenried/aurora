use std::collections::VecDeque;
use uuid::Uuid;

use crate::helpers::db::{Db, to_io};

const MAX_HISTORY: usize = 30;

pub async fn load_history(db: &Db) -> std::io::Result<VecDeque<Uuid>> {
    let db = db.clone();
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT song_id FROM history ORDER BY played_at DESC LIMIT 30")
            .map_err(to_io)?;
        let rows = stmt
            .query_map([], |row| {
                let s: String = row.get(0)?;
                Ok(s)
            })
            .map_err(to_io)?;
        let mut deque = VecDeque::new();
        for row in rows {
            let s = row.map_err(to_io)?;
            if let Ok(id) = Uuid::parse_str(&s) {
                deque.push_back(id);
            }
        }
        Ok(deque)
    })
    .await
    .map_err(to_io)?
}

pub async fn save_history(db: &Db, history: &VecDeque<Uuid>) -> std::io::Result<()> {
    let db = db.clone();
    let history = history.clone();
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().unwrap();
        conn.execute_batch("BEGIN").map_err(to_io)?;
        conn.execute("DELETE FROM history", []).map_err(to_io)?;
        for (i, id) in history.iter().enumerate() {
            let played_at = i64::MAX - i as i64;
            conn.execute(
                "INSERT INTO history (song_id, played_at) VALUES (?1, ?2)",
                rusqlite::params![id.to_string(), played_at],
            )
            .map_err(to_io)?;
        }
        conn.execute_batch("COMMIT").map_err(to_io)?;
        Ok(())
    })
    .await
    .map_err(to_io)?
}

pub fn push_history(history: &mut VecDeque<Uuid>, song_id: Uuid) {
    if history.front() == Some(&song_id) {
        return;
    }
    history.retain(|id| *id != song_id);
    history.push_front(song_id);
    history.truncate(MAX_HISTORY);
}
