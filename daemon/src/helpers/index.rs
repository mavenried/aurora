use std::{collections::HashMap, path::PathBuf, process::exit, time::Duration};

use crate::types::*;
use symphonia::{
    core::{
        formats::FormatOptions, io::MediaSourceStream, meta::MetadataOptions, units::TimeStamp,
    },
    default::get_probe,
};
use uuid::Uuid;
use walkdir::WalkDir;

use crate::types::SongIndex;

pub async fn generate_index(music_dir: &PathBuf) -> std::io::Result<()> {
    // collect supported audio files
    let mut songs: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(music_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            match ext.to_lowercase().as_str() {
                "mp3" | "flac" | "wav" | "ogg" | "m4a" => songs.push(path.to_path_buf()),
                _ => {}
            }
        }
    }

    tracing::info!("Found {} songs.", songs.len());

    let mut index: SongIndex = HashMap::new();

    for path in songs {
        let file = match std::fs::File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                tracing::warn!("Skipping {:?}: open error: {}", path, e);
                continue;
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
                continue;
            }
        };

        let mut format = probe.format;

        // Defaults
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
            // rev.tags() returns an iterator of tags; tag.key and tag.value are Options
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
                let time = tb.calc_time(ts); // has .seconds (u64) and .frac (f64)
                duration = Duration::from_secs(time.seconds)
                    + Duration::from_millis((time.frac * 1000.) as u64)
            }
        }

        let id = uuid::Uuid::new_v5(&Uuid::NAMESPACE_URL, path.display().to_string().as_bytes());
        let artists = artist
            .split('/')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        let songmeta = SongMeta {
            id,
            title,
            artists,
            duration: duration,
            path,
        };

        index.insert(id, songmeta);
    }

    tracing::info!("Indexed {} songs.", index.len());
    save_index(&index).await?;
    Ok(())
}

pub async fn load_index() -> std::io::Result<SongIndex> {
    let index_file = dirs::config_dir()
        .unwrap_or_else(|| {
            tracing::error!("No config dir.");
            exit(1)
        })
        .join("aurora-player")
        .join("index.json");
    let data = tokio::fs::read_to_string(index_file).await?;
    let index: SongIndex = serde_json::from_str(&data)?;
    Ok(index)
}

pub async fn save_index(index: &SongIndex) -> std::io::Result<()> {
    let configdir = dirs::config_dir()
        .unwrap_or_else(|| {
            tracing::error!("No config dir.");
            exit(1)
        })
        .join("aurora-player");
    tokio::fs::create_dir_all(&configdir).await?;
    let index_file = configdir.join("index.json");
    let data = serde_json::to_string_pretty(index)?;
    tokio::fs::write(index_file, data).await?;
    Ok(())
}
