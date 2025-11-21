mod index;
mod playlist;
use crate::types::*;
use aurora_protocol::Response;
pub use index::*;
pub use playlist::*;

use tokio::io::AsyncWriteExt;

pub async fn send_to_client(socket: &WriteSocket, response: &Response) -> anyhow::Result<()> {
    let encoded = serde_json::to_string(response)?;
    let len = (encoded.len() as u32).to_be_bytes();
    let mut socket_locked = socket.lock().await;
    socket_locked.write_all(&len).await?;
    socket_locked.write_all(encoded.as_bytes()).await?;
    Ok(())
}
