use aurora_protocol::SongMeta;
use std::{collections::HashMap, sync::Arc};
use tokio::{net::tcp::OwnedWriteHalf, sync::Mutex};
use uuid::Uuid;

mod state_impl;
pub use state_impl::*;

pub type SongIndex = HashMap<Uuid, SongMeta>;
pub type State = Arc<Mutex<StateStruct>>;
pub type WriteSocket = Arc<Mutex<OwnedWriteHalf>>;
pub enum GetReturn {
    Ok,
    QueueEmpty,
}
