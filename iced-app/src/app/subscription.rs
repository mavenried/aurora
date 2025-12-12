use crate::app::AuroraPlayer;
use crate::types::*;
use futures::sink::SinkExt;
use futures::stream::Stream;
use iced::Subscription;
use iced::stream;
use tokio::io::AsyncReadExt;
use tokio::net::UnixStream;

pub fn unix_connection() -> impl Stream<Item = UnixSocketEvent> + Send + 'static {
    stream::channel(100, |mut output| async move {
        let stream = match UnixStream::connect("/tmp/aurora-daemon.sock").await {
            Ok(s) => s,
            Err(e) => {
                let _ = output.send(UnixSocketEvent::Error(e.to_string())).await;
                return;
            }
        };

        let (mut reader, writer) = stream.into_split();
        let tcp_writer = SocketWriter::new(writer);

        // Notify connection
        let _ = output
            .send(UnixSocketEvent::Connected(tcp_writer.clone()))
            .await;

        loop {
            let len = match reader.read_u32().await {
                Ok(l) => l as usize,
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    let _ = output.send(UnixSocketEvent::Disconnected).await;
                    break;
                }
                Err(e) => {
                    let _ = output.send(UnixSocketEvent::Error(e.to_string())).await;
                    break;
                }
            };

            let mut buf = vec![0u8; len];
            if let Err(e) = reader.read_exact(&mut buf).await {
                let _ = output.send(UnixSocketEvent::Error(e.to_string())).await;
                break;
            }

            let _ = output.send(UnixSocketEvent::PacketReceived(buf)).await;
        }
    })
}

impl AuroraPlayer {
    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::run(unix_connection).map(Message::TcpEvent)
    }
}
