use crate::{helpers::send_to_all, types::*};
use anyhow::anyhow;
use aurora_protocol::Theme;
use notify::{EventKind, RecursiveMode, Watcher};

fn get_config() -> anyhow::Result<Theme> {
    Ok(Theme {
        bgd0: "#333333".into(),
        bgd1: "#333333".into(),
        bgd2: "#333333".into(),
        bgd3: "#333333".into(),
        bgd4: "#333333".into(),
        txt1: "#333333".into(),
        txt2: "#333333".into(),
        acct: "#333333".into(),
        srch: "#333333".into(),
        btns: "#333333".into(),
    })
}

pub async fn init(state: State) -> anyhow::Result<()> {
    let Some(mut path) = dirs::config_dir() else {
        return Err(anyhow!("could not load config dir"));
    };
    path = path.join("aurora-player/config.toml");
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(&path, RecursiveMode::NonRecursive)?;

    for event in rx {
        if let Ok(event) = event {
            if matches!(event.kind, EventKind::Modify(_)) {
                if let Ok(theme) = get_config() {
                    send_to_all(&state, &aurora_protocol::Response::Theme(theme)).await;
                }
            }
        }
    }
    Ok(())
}
