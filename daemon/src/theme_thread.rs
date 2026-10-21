use crate::{helpers::send_to_all, types::*};
use anyhow::anyhow;
use aurora_protocol::Theme;
use notify::{EventKind, RecursiveMode, Watcher};
use tokio::sync::mpsc;

pub fn get_config() -> Theme {
    let default = default_config();
    if let Some(mut path) = dirs::config_dir()
        && path.join("aurora-player/config.toml").exists()
    {
        path = path.join("aurora-player/config.toml");
        match std::fs::read_to_string(path) {
            Ok(contents) => parse_config(&contents),
            Err(err) => {
                tracing::warn!("Could not read theme config: {err}");
                default
            }
        }
    } else {
        default
    }
}

fn default_config() -> Theme {
    Theme {
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
        follow_art_colorway: false,
    }
}

fn parse_config(contents: &str) -> Theme {
    match toml::from_str(contents) {
        Ok(theme) => theme,
        Err(err) => {
            tracing::warn!("Could not parse theme config: {err}");
            default_config()
        }
    }
}

pub fn save_config(theme: &Theme) -> anyhow::Result<()> {
    let Some(mut path) = dirs::config_dir() else {
        return Err(anyhow!("could not load config dir"));
    };
    path = path.join("aurora-player");
    std::fs::create_dir_all(&path)?;
    std::fs::write(path.join("config.toml"), toml::to_string_pretty(theme)?)?;
    Ok(())
}

pub async fn init(state: State) -> anyhow::Result<()> {
    let Some(mut path) = dirs::config_dir() else {
        return Err(anyhow!("could not load config dir"));
    };
    path = path.join("aurora-player");
    std::fs::create_dir_all(&path)?;
    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut watcher = notify::recommended_watcher(move |event| {
        let _ = tx.send(event);
    })?;
    watcher.watch(&path, RecursiveMode::Recursive)?;
    while let Some(event) = rx.recv().await {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_theme_config_falls_back_to_defaults() {
        let theme = parse_config("this is not valid TOML =");
        assert_eq!(theme.bgd0, "#111111");
        assert!(!theme.follow_art_colorway);
    }

    #[test]
    fn theme_config_reads_optional_follow_art_setting() {
        let theme = parse_config(
            r##"
bgd0 = "#000000"
bgd1 = "#111111"
bgd2 = "#222222"
bgd3 = "#333333"
bgd4 = "#444444"
txt1 = "#ffffff"
txt2 = "#eeeeee"
acct = "#ff0000"
srch = "#555555"
btns = "#ffffff"
follow_art_colorway = true
"##,
        );
        assert_eq!(theme.bgd0, "#000000");
        assert!(theme.follow_art_colorway);
    }
}
