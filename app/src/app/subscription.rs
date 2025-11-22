use futures::sink::SinkExt;
use futures::stream::Stream;
use tokio::sync::mpsc;

use iced::Subscription;
use iced::stream;

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::app::AuroraPlayer;
use crate::types::Message;
use crate::types::{TcpCommand, TcpConnection, TcpEvent};

pub fn tcp_connection() -> impl Stream<Item = TcpEvent> + Send + 'static {
    stream::channel(100, |mut output| async move {
        let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel();

        let connection = TcpConnection {
            sender: Arc::new(cmd_tx),
        };

        let mut stream = match TcpStream::connect("127.0.0.1:4321").await {
            Ok(s) => {
                let _ = output.send(TcpEvent::Connected(connection.clone())).await;
                s
            }
            Err(e) => {
                let _ = output.send(TcpEvent::Error(e.to_string())).await;
                return;
            }
        };

        loop {
            tokio::select! {
                result = stream.read_u32() => {
                    let len = match result {
                        Ok(l) => l as usize,
                        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                            let _ = output.send(TcpEvent::Disconnected).await;
                            break;
                        }
                        Err(e) => {
                            let _ = output.send(TcpEvent::Error(e.to_string())).await;
                            break;
                        }
                    };

                    let mut packet = vec![0u8; len];
                    match stream.read_exact(&mut packet).await {
                        Ok(_) => {
                            let _ = output.send(TcpEvent::PacketReceived(packet)).await;
                        }
                        Err(e) => {
                            let _ = output.send(TcpEvent::Error(e.to_string())).await;
                            break;
                        }
                    }
                }

                cmd = cmd_rx.recv() => {
                    match cmd {
                        Some(TcpCommand::Send(data)) => {
                            let len = data.len() as u32;
                            if let Err(e) = stream.write_u32(len).await {
                                let _ = output.send(TcpEvent::Error(e.to_string())).await;
                                break;
                            }
                            if let Err(e) = stream.write_all(&data).await {
                                let _ = output.send(TcpEvent::Error(e.to_string())).await;
                                break;
                            }
                            if let Err(e) = stream.flush().await {
                                let _ = output.send(TcpEvent::Error(e.to_string())).await;
                                break;
                            }
                            let _ = output.send(TcpEvent::PacketSent).await;
                        }
                        Some(TcpCommand::Disconnect) => {
                            let _ = output.send(TcpEvent::Disconnected).await;
                            break;
                        }
                        None => break,
                    }
                }
            }
        }
    })
}

impl AuroraPlayer {
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::run(tcp_connection).map(Message::TcpEvent)
    }
}
