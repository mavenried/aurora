use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, time::Duration};
use tokio::sync::Mutex;
use uuid::Uuid;

mod response_types;
pub use response_types::*;
mod state_impl;
pub use state_impl::*;

#[derive(Clone, Deserialize, Serialize)]
pub struct SongMeta {
    pub id: Uuid,
    pub title: String,
    pub artists: Vec<String>,
    pub duration: Duration,
    pub path: PathBuf,
}

pub type SongIndex = HashMap<Uuid, SongMeta>;
pub type State = Mutex<StateStruct>;

pub enum GetReturn {
    Ok,
    QueueEmpty,
}
pub enum SearchType {
    ByTitle(String),
    ByArtist(String),
}
