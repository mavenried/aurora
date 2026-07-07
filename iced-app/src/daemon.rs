use std::path::PathBuf;
use std::time::Duration;

use aurora_protocol::{Request, Response};
use iced::Subscription;
use iced::futures::channel::mpsc as iced_mpsc;
use iced::futures::sink::SinkExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc;

const SOCK_PATH: &str = "/tmp/aurora-daemon.sock";

#[derive(Debug, Clone)]
pub enum DaemonEvent {
    Connected(mpsc::Sender<Request>),
    Response(Response),
    Disconnected,
}

pub fn connection() -> Subscription<DaemonEvent> {
    Subscription::run(|| iced::stream::channel(100, run))
}

async fn run(mut output: iced_mpsc::Sender<DaemonEvent>) {
    let mut stream: Option<UnixStream> = None;
    while stream.is_none() {
        let path = PathBuf::from(SOCK_PATH);
        if let Ok(s) = UnixStream::connect(&path).await {
            tracing::info!("Connected to the daemon.");
            stream = Some(s);
        } else {
            let _ = std::process::Command::new("aurora-daemon").spawn();
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    let (reader, writer) = stream.unwrap().into_split();
    let (tx, rx) = mpsc::channel::<Request>(10);

    if output.send(DaemonEvent::Connected(tx)).await.is_err() {
        return;
    }

    let sender_task = tokio::spawn(unix_sender(writer, rx));

    if let Err(err) = unix_recver(reader, &mut output).await {
        tracing::error!("Receiver error: {err}");
    }

    sender_task.abort();
    let _ = output.send(DaemonEvent::Disconnected).await;
}

async fn unix_sender(mut writer: OwnedWriteHalf, mut rx: mpsc::Receiver<Request>) {
    loop {
        match rx.recv().await {
            Some(req) => {
                let Ok(encoded) = serde_json::to_string(&req) else {
                    continue;
                };
                let len = (encoded.len() as u32).to_be_bytes();
                if writer.write_all(&len).await.is_err()
                    || writer.write_all(encoded.as_bytes()).await.is_err()
                {
                    tracing::error!("Failed to write to daemon socket.");
                    return;
                }
                tracing::info!("Sent: {encoded}");
            }
            None => {
                tracing::info!("Writer channel closed.");
                return;
            }
        }
    }
}

async fn unix_recver(
    mut reader: OwnedReadHalf,
    output: &mut iced_mpsc::Sender<DaemonEvent>,
) -> anyhow::Result<()> {
    loop {
        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf).await?;
        let msg_len = u32::from_be_bytes(len_buf) as usize;

        let mut buf = vec![0u8; msg_len];
        reader.read_exact(&mut buf).await?;
        let res: Response = serde_json::from_slice(&buf)?;

        if output.send(DaemonEvent::Response(res)).await.is_err() {
            return Ok(());
        }
    }
}
