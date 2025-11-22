use std::sync::Arc;

use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum TcpEvent {
    Connected(TcpConnection),
    PacketReceived(Vec<u8>),
    PacketSent,
    Error(String),
    Disconnected,
}

#[derive(Debug, Clone)]
pub enum TcpCommand {
    Send(Vec<u8>),
    Disconnect,
}
#[derive(Debug, Clone)]
pub struct TcpConnection {
    pub sender: Arc<mpsc::UnboundedSender<TcpCommand>>,
}

impl TcpConnection {
    pub fn send_packet(&self, data: Vec<u8>) {
        let _ = self.sender.send(TcpCommand::Send(data));
    }

    pub fn disconnect(&self) {
        let _ = self.sender.send(TcpCommand::Disconnect);
    }
}
