use crate::types::State;
use mpris_server::Player;
use tokio::sync::mpsc::{self, UnboundedSender};

pub enum PlayerCommand {
    PlayPause,
    Next,
    Prev,
}

pub async fn init(state: State) -> anyhow::Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel::<PlayerCommand>();

    tracing::info!("Watcher thread started.");
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            controller(tx).await.ok();
        });
    });

    while let Some(cmd) = rx.recv().await {
        match cmd {
            PlayerCommand::PlayPause => {
                state.lock().await.pause().await;
            }
            PlayerCommand::Next => {
                let mut state_locked = state.lock().await;
                state_locked.next(1).await;
                state_locked.add().await;
            }
            PlayerCommand::Prev => {
                let mut state_locked = state.lock().await;
                state_locked.prev(1).await;
                state_locked.add().await;
            }
        }
    }

    Ok(())
}

pub async fn controller(ctx: UnboundedSender<PlayerCommand>) -> anyhow::Result<()> {
    let player = Player::builder("me.mavenried.Aurora")
        .can_play(true)
        .can_pause(true)
        .can_go_next(true)
        .can_go_previous(true)
        .build()
        .await?;

    let tx = ctx.clone();
    player.connect_next(move |_player| {
        let _ = tx.send(PlayerCommand::Next);
    });

    let tx = ctx.clone();
    player.connect_previous(move |_player| {
        let _ = tx.send(PlayerCommand::Prev);
    });

    let tx = ctx.clone();
    player.connect_play_pause(move |_player| {
        let _ = tx.send(PlayerCommand::PlayPause);
    });

    player.run().await;
    Ok(())
}
