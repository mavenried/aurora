use crate::{helpers::send_to_all, types::*};
use anyhow::anyhow;
use aurora_protocol::Theme;
use notify::{EventKind, RecursiveMode, Watcher};

pub fn get_config() -> Theme {
    let default = Theme {
        bgd0: "#111111".into(),
        bgd1: "#282828".into(),
        bgd2: "#3c3836".into(),
        bgd3: "#504945".into(),
        bgd4: "#665c54".into(),
        txt1: "#ebdbb2".into(),
        txt2: "#bdae93".into(),
        acct: "#d3869b".into(),
        srch: "#3c3836".into(),
        btns: "#ebdbb2".into(),
    };
    if let Some(mut path) = dirs::config_dir()
        && path.join("aurora-player/config.toml").exists()
    {
        path = path.join("aurora-player/config.toml");
        let s = std::fs::read_to_string(path).unwrap();
        let parsed: Theme = toml::from_str(&s).unwrap_or(default);
        println!("{parsed:?}");
        parsed
    } else {
        default
    }
}

pub async fn init(state: State) -> anyhow::Result<()> {
    let Some(mut path) = dirs::config_dir() else {
        return Err(anyhow!("could not load config dir"));
    };
    path = path.join("aurora-player");
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(&path, RecursiveMode::Recursive)?;
    for event in rx {
        if let Ok(event) = event {
            match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                    let theme = get_config();
                    state.lock().await.theme = theme.clone();
                    send_to_all(&state, &aurora_protocol::Response::Theme(theme)).await?;
                    tracing::info!("Theme Updated!");
                }
                _ => (),
            }
        }
    }
    Ok(())
}
