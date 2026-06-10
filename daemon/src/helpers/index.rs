use aurora_protocol::SongMeta;
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    time::{Duration, UNIX_EPOCH},
};

use symphonia::{
    core::{
        formats::FormatOptions, io::MediaSourceStream, meta::MetadataOptions, units::TimeStamp,
    },
    default::get_probe,
};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::{
    helpers::db::{Db, to_io},
    types::SongIndex,
};

struct CachedEntry {
    id: String,
    title: String,
    artists_json: String,
    dur_ms: i64,
    mtime: i64,
    art_path: Option<String>,
}

fn read_tags(path: &PathBuf) -> Option<(String, Vec<String>, Duration)> {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!("Skipping {:?}: open error: {}", path, e);
            return None;
        }
    };

    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut probe = match get_probe().format(
        &Default::default(),
        mss,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    ) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("Skipping {:?}: probe error: {}", path, e);
            return None;
        }
    };

    let mut format = probe.format;

    let mut title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();
    let mut artist = "Unknown".to_string();
    let mut duration = Duration::ZERO;
    let mut meta_opt = format.metadata();

    if meta_opt.current().is_none() {
        if let Some(meta) = probe.metadata.get() {
            meta_opt = meta;
        }
    }
    if let Some(rev) = meta_opt.current() {
        for tag in rev.tags() {
            let key = tag.key.to_string();
            let val = tag.value.to_string();
            match key.to_lowercase().as_str() {
                "title" | "tit2" if !val.is_empty() => title = val.to_string(),
                "artist" | "tpe1" if !val.is_empty() => artist = val.to_string(),
                _ => {}
            }
        }
    }

    if let Some(track) = format.tracks().first() {
        if let (Some(tb), Some(n_frames)) =
            (track.codec_params.time_base, track.codec_params.n_frames)
        {
            let ts: TimeStamp = n_frames as TimeStamp;
            let time = tb.calc_time(ts);
            duration = Duration::from_secs(time.seconds)
                + Duration::from_millis((time.frac * 1000.) as u64);
        }
    }

    let artists = artist
        .split('/')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    Some((title, artists, duration))
}

pub async fn build_index(music_dir: &PathBuf, db: &Db) -> std::io::Result<SongIndex> {
    // Step 1: Walk music_dir, collect (path, mtime) for audio files
    let mut fs_files: Vec<(PathBuf, u64)> = Vec::new();
    for entry in WalkDir::new(music_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            match ext.to_lowercase().as_str() {
                "mp3" | "flac" | "wav" | "ogg" | "m4a" => {
                    let mtime = entry
                        .metadata()
                        .ok()
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    fs_files.push((path.to_path_buf(), mtime));
                }
                _ => {}
            }
        }
    }

    let total_files = fs_files.len();

    // Step 2: Load entire songs table from DB into path-keyed HashMap
    let db_clone = db.clone();
    let cache: HashMap<String, CachedEntry> = tokio::task::spawn_blocking(move || {
        let conn = db_clone.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, path, title, artists, dur_ms, mtime, art_path FROM songs")
            .map_err(to_io)?;

        let mut map: HashMap<String, CachedEntry> = HashMap::new();
        let rows_iter = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let path: String = row.get(1)?;
                let title: String = row.get(2)?;
                let artists_json: String = row.get(3)?;
                let dur_ms: i64 = row.get(4)?;
                let mtime: i64 = row.get(5)?;
                let art_path: Option<String> = row.get(6)?;
                Ok((
                    path,
                    CachedEntry {
                        id,
                        title,
                        artists_json,
                        dur_ms,
                        mtime,
                        art_path,
                    },
                ))
            })
            .map_err(to_io)?;

        for row in rows_iter {
            let (path, entry) = row.map_err(to_io)?;
            map.insert(path, entry);
        }
        Ok::<_, std::io::Error>(map)
    })
    .await
    .map_err(to_io)??;

    let cached_count = cache.len();

    // Build filesystem path set for stale detection
    let fs_path_set: HashSet<String> = fs_files
        .iter()
        .map(|(p, _)| p.display().to_string())
        .collect();

    // Step 3: For each file, decide cache hit or rescan
    struct UpsertEntry {
        id: String,
        path: String,
        title: String,
        artists_json: String,
        dur_ms: i64,
        mtime: i64,
        art_path: Option<String>,
    }

    let mut index: SongIndex = HashMap::new();
    let mut to_upsert: Vec<UpsertEntry> = Vec::new();

    for (path, file_mtime) in &fs_files {
        let path_str = path.display().to_string();

        if let Some(cached) = cache.get(&path_str) {
            if cached.mtime == *file_mtime as i64 {
                // Cache hit — build SongMeta from cache without symphonia
                if let Ok(id) = Uuid::parse_str(&cached.id) {
                    let artists: Vec<String> =
                        serde_json::from_str(&cached.artists_json).unwrap_or_default();
                    let duration = Duration::from_millis(cached.dur_ms as u64);
                    let art_path = cached.art_path.as_ref().map(PathBuf::from);
                    let songmeta = SongMeta {
                        id,
                        title: cached.title.clone(),
                        artists,
                        duration,
                        path: path.clone(),
                        art_path,
                    };
                    index.insert(id, songmeta);
                }
                continue;
            }
        }

        // Cache miss or mtime changed — read tags via symphonia
        let id = Uuid::new_v5(&Uuid::NAMESPACE_URL, path_str.as_bytes());
        // Preserve existing art_path if the song was previously indexed
        let existing_art = cache.get(&path_str).and_then(|c| c.art_path.clone());

        if let Some((title, artists, duration)) = read_tags(path) {
            let artists_json = serde_json::to_string(&artists).unwrap_or_else(|_| "[]".to_string());
            let dur_ms = duration.as_millis() as i64;

            to_upsert.push(UpsertEntry {
                id: id.to_string(),
                path: path_str,
                title: title.clone(),
                artists_json: artists_json.clone(),
                dur_ms,
                mtime: *file_mtime as i64,
                art_path: existing_art.clone(),
            });

            let songmeta = SongMeta {
                id,
                title,
                artists,
                duration,
                path: path.clone(),
                art_path: existing_art.map(PathBuf::from),
            };
            index.insert(id, songmeta);
        }
    }

    let rescanned = to_upsert.len();
    tracing::info!(
        "Found {} files, {} cached, {} rescanned",
        total_files,
        cached_count,
        rescanned
    );

    // Collect stale paths (in DB but no longer on disk)
    let stale_paths: Vec<String> = cache
        .keys()
        .filter(|p| !fs_path_set.contains(*p))
        .cloned()
        .collect();

    // Step 4: Upsert changed/new, delete stale — all in one transaction
    let db_clone = db.clone();
    tokio::task::spawn_blocking(move || {
        let conn = db_clone.lock().unwrap();
        conn.execute_batch("BEGIN").map_err(to_io)?;

        for entry in &to_upsert {
            conn.execute(
                "INSERT OR REPLACE INTO songs \
                 (id, path, title, artists, dur_ms, mtime, art_path) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    entry.id,
                    entry.path,
                    entry.title,
                    entry.artists_json,
                    entry.dur_ms,
                    entry.mtime,
                    entry.art_path,
                ],
            )
            .map_err(to_io)?;
        }

        for stale_path in &stale_paths {
            conn.execute(
                "DELETE FROM songs WHERE path = ?1",
                rusqlite::params![stale_path],
            )
            .map_err(to_io)?;
        }

        conn.execute_batch("COMMIT").map_err(to_io)?;
        Ok::<_, std::io::Error>(())
    })
    .await
    .map_err(to_io)??;

    tracing::info!("Indexed {} songs.", index.len());
    Ok(index)
}
