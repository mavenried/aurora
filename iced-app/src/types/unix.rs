use std::sync::Arc;
use tokio::io::AsyncWriteExt;

use aurora_protocol::Request;
use tokio::{net::unix::OwnedWriteHalf, sync::Mutex};

#[derive(Debug, Clone)]
pub enum UnixSocketEvent {
    Connected(SocketWriter),
    PacketReceived(Vec<u8>),
    PacketSent,
    Error(String),
    Disconnected,
}

#[derive(Debug, Clone)]
pub enum UnixSocketCommand {
    Send(Vec<u8>),
    Disconnect,
}

#[derive(Debug, Clone)]
pub struct SocketWriter(Arc<Mutex<OwnedWriteHalf>>);
impl SocketWriter {
    pub fn new(writer: OwnedWriteHalf) -> Self {
        Self(Arc::new(Mutex::new(writer)))
    }

    pub async fn send(&self, data: &Request) -> Result<(), std::io::Error> {
        let encoded = serde_json::to_string(data)?;
        let len = (encoded.len() as u32).to_be_bytes();
        let mut socket_locked = self.0.lock().await;
        socket_locked.write_all(&len).await?;
        socket_locked.write_all(encoded.as_bytes()).await?;
        Ok(())
    }
}
