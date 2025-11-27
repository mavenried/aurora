use super::{GetReturn, StateStruct};

impl StateStruct {
    pub async fn prev(&mut self, n: usize) -> GetReturn {
        if self.queue.is_empty() {
            return GetReturn::QueueEmpty;
        }

        let prev_idx = match self.current_song {
            None => 0,
            Some(_) => {
                (self.current_idx + self.queue.len() - (n % self.queue.len())) % self.queue.len()
            }
        };

        self.current_idx = prev_idx;
        self.current_song.replace(self.queue[prev_idx].clone());
        tracing::info!("Prev");
        GetReturn::Ok
    }

    pub async fn next(&mut self, n: usize) -> GetReturn {
        if self.queue.is_empty() {
            return GetReturn::QueueEmpty;
        }

        let next_idx = match self.current_song {
            None => 0,
            Some(_) => (self.current_idx + n) % self.queue.len(),
        };

        self.current_idx = next_idx;
        self.current_song.replace(self.queue[next_idx].clone());
        tracing::info!("Next");
        GetReturn::Ok
    }
}
