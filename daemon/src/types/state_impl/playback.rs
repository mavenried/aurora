use super::{GetReturn, StateStruct};

impl StateStruct {
    pub async fn prev(&mut self, n: usize) -> GetReturn {
        if self.queue.is_empty() {
            return GetReturn::QueueEmpty;
        }
        for _ in 0..n {
            let song = self.queue.pop_back().unwrap();
            self.queue.push_front(song);
        }

        self.current_song.replace(self.queue[0].clone());
        tracing::info!("Prev");
        GetReturn::Ok
    }

    pub async fn next(&mut self, n: usize) -> GetReturn {
        if self.queue.is_empty() {
            return GetReturn::QueueEmpty;
        }

        for _ in 0..n {
            let song = self.queue.pop_front().unwrap();
            self.queue.push_back(song);
        }

        self.current_song.replace(self.queue[0].clone());
        tracing::info!("Next");
        GetReturn::Ok
    }
}
