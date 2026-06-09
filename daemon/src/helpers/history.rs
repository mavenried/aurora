use std::{collections::VecDeque, process::exit};

use uuid::Uuid;

const MAX_HISTORY: usize = 30;

pub async fn load_history() -> std::io::Result<VecDeque<Uuid>> {
    let history_file = dirs::config_dir()
        .unwrap_or_else(|| {
            tracing::error!("No config dir.");
            exit(1)
        })
        .join("aurora-player")
        .join("history.json");
    let data = tokio::fs::read_to_string(history_file).await?;
    let history: VecDeque<Uuid> = serde_json::from_str(&data)?;
    Ok(history)
}

pub async fn save_history(history: &VecDeque<Uuid>) -> std::io::Result<()> {
    let configdir = dirs::config_dir()
        .unwrap_or_else(|| {
            tracing::error!("No config dir.");
            exit(1)
        })
        .join("aurora-player");
    tokio::fs::create_dir_all(&configdir).await?;
    let history_file = configdir.join("history.json");
    let data = serde_json::to_string_pretty(history)?;
    tokio::fs::write(history_file, data).await?;
    Ok(())
}

pub fn push_history(history: &mut VecDeque<Uuid>, song_id: Uuid) {
    if history.front() == Some(&song_id) {
        return;
    }
    history.retain(|id| *id != song_id);
    history.push_front(song_id);
    history.truncate(MAX_HISTORY);
}
