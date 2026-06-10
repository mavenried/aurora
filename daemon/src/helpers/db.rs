use std::{
    io,
    sync::{Arc, Mutex},
};

pub type Db = Arc<Mutex<rusqlite::Connection>>;

pub fn open() -> io::Result<Db> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No config dir"))?;
    let aurora_dir = config_dir.join("aurora-player");
    std::fs::create_dir_all(&aurora_dir)?;
    let db_path = aurora_dir.join("aurora.db");

    let conn = rusqlite::Connection::open(&db_path).map_err(to_io)?;
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
        .map_err(to_io)?;
    migrate(&conn).map_err(to_io)?;

    Ok(Arc::new(Mutex::new(conn)))
}

fn migrate(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS songs (
            id       TEXT PRIMARY KEY,
            path     TEXT NOT NULL UNIQUE,
            title    TEXT NOT NULL,
            artists  TEXT NOT NULL,
            dur_ms   INTEGER NOT NULL,
            mtime    INTEGER NOT NULL,
            art_path TEXT
        );
        CREATE TABLE IF NOT EXISTS playlists (
            id    TEXT PRIMARY KEY,
            title TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS playlist_songs (
            playlist_id TEXT NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
            song_id     TEXT NOT NULL,
            position    INTEGER NOT NULL DEFAULT 0,
            PRIMARY KEY (playlist_id, song_id)
        );
        CREATE TABLE IF NOT EXISTS liked_songs (
            song_id TEXT PRIMARY KEY
        );
        CREATE TABLE IF NOT EXISTS history (
            song_id   TEXT PRIMARY KEY,
            played_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );
        ",
    )
}

pub fn to_io<E: std::error::Error + Send + Sync + 'static>(e: E) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e)
}
