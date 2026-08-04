use std::path::PathBuf;
use std::time::Duration;

use aurora_protocol::{Request, Response};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};

const SOCK_PATH: &str = "/tmp/aurora-daemon.sock";

#[derive(Debug, Clone)]
pub enum DaemonEvent {
    Connected,
    Response(Response),
    Disconnected,
}

/// Spawns a background OS thread that owns a tokio runtime and the unix
/// socket connection to `aurora-daemon`. Requests are pushed in from the GTK
/// main thread via `req_tx`; responses arrive on `evt_rx`, polled from a
/// `glib::spawn_future_local` task on the main thread. `async_channel` is
/// used (rather than `tokio::sync::mpsc`) because it is executor-agnostic and
/// its senders can be used synchronously from GTK callbacks via `send_blocking`.
pub fn start() -> (async_channel::Sender<Request>, async_channel::Receiver<DaemonEvent>) {
    let (req_tx, req_rx) = async_channel::unbounded::<Request>();
    let (evt_tx, evt_rx) = async_channel::unbounded::<DaemonEvent>();

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("failed to start tokio runtime");
        rt.block_on(run(req_rx, evt_tx));
    });

    (req_tx, evt_rx)
}

async fn run(req_rx: async_channel::Receiver<Request>, evt_tx: async_channel::Sender<DaemonEvent>) {
    loop {
        let stream = connect().await;
        let (reader, writer) = stream.into_split();

        if evt_tx.send(DaemonEvent::Connected).await.is_err() {
            return;
        }

        let sender_task = tokio::spawn(unix_sender(writer, req_rx.clone()));

        if let Err(err) = unix_recver(reader, &evt_tx).await {
            tracing::error!("Receiver error: {err}");
        }

        sender_task.abort();
        if evt_tx.send(DaemonEvent::Disconnected).await.is_err() {
            return;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn connect() -> UnixStream {
    let mut spawned_daemon = false;
    loop {
        let path = PathBuf::from(SOCK_PATH);
        if let Ok(s) = UnixStream::connect(&path).await {
            tracing::info!("Connected to the daemon.");
            return s;
        }
        if !spawned_daemon {
            let _ = std::process::Command::new("aurora-daemon").spawn();
            spawned_daemon = true;
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn unix_sender(mut writer: OwnedWriteHalf, rx: async_channel::Receiver<Request>) {
    loop {
        match rx.recv().await {
            Ok(req) => {
                let Ok(encoded) = serde_json::to_string(&req) else {
                    continue;
                };
                let len = (encoded.len() as u32).to_be_bytes();
                if writer.write_all(&len).await.is_err() || writer.write_all(encoded.as_bytes()).await.is_err() {
                    tracing::error!("Failed to write to daemon socket.");
                    return;
                }
                tracing::info!("Sent: {encoded}");
            }
            Err(_) => {
                tracing::info!("Request channel closed.");
                return;
            }
        }
    }
}

async fn unix_recver(mut reader: OwnedReadHalf, evt_tx: &async_channel::Sender<DaemonEvent>) -> anyhow::Result<()> {
    loop {
        let mut len_buf = [0u8; 4];
        reader.read_exact(&mut len_buf).await?;
        let msg_len = u32::from_be_bytes(len_buf) as usize;

        let mut buf = vec![0u8; msg_len];
        reader.read_exact(&mut buf).await?;
        let res: Response = serde_json::from_slice(&buf)?;

        if evt_tx.send(DaemonEvent::Response(res)).await.is_err() {
            return Ok(());
        }
    }
}
