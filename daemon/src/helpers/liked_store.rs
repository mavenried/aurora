use std::collections::HashSet;
use uuid::Uuid;

pub async fn load_liked() -> std::io::Result<HashSet<Uuid>> {
    let file = dirs::config_dir()
        .unwrap()
        .join("aurora-player")
        .join("liked.json");
    if !tokio::fs::try_exists(&file).await.unwrap_or(false) {
        return Ok(HashSet::new());
    }
    let data = tokio::fs::read_to_string(&file).await?;
    let ids: Vec<Uuid> = serde_json::from_str(&data).unwrap_or_default();
    Ok(ids.into_iter().collect())
}

pub async fn save_liked(ids: &HashSet<Uuid>) -> std::io::Result<()> {
    let configdir = dirs::config_dir().unwrap().join("aurora-player");
    tokio::fs::create_dir_all(&configdir).await?;
    let mut ids_vec: Vec<&Uuid> = ids.iter().collect();
    ids_vec.sort();
    let data = serde_json::to_string_pretty(&ids_vec)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    tokio::fs::write(configdir.join("liked.json"), data).await?;
    Ok(())
}
