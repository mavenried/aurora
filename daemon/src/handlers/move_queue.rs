use crate::{helpers::send_to_all, types::*};
use aurora_protocol::{Response, Song};

pub async fn move_queue(state: &State, from: usize, to: usize) -> anyhow::Result<()> {
    let mut state_locked = state.lock().await;
    let len = state_locked.queue.len();
    if from >= len || to >= len || from == to {
        return Ok(());
    }
    let item = state_locked.queue.remove(from).unwrap();
    state_locked.queue.insert(to, item);
    let resp = Response::Queue(state_locked.queue.iter().map(Song::from).collect());
    drop(state_locked);
    send_to_all(state, &resp).await?;
    Ok(())
}
