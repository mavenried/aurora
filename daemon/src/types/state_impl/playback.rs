use std::time::Duration;

use super::{GetReturn, StateStruct};

impl StateStruct {
    pub async fn prev(&mut self, n: usize) -> GetReturn {
        if self.queue.is_empty() {
            return GetReturn::QueueEmpty;
        }

        if n <= 1 {
            if let Some(audio) = &self.audio
                && audio.get_position() < Duration::from_secs(1)
            {
                let song = self.queue.pop_back().unwrap();
                self.queue.push_front(song);
            }
        } else {
            for _ in 0..n {
                let song = self.queue.pop_back().unwrap();
                self.queue.push_front(song);
            }
        }
        self.current_song.replace(self.queue[0].clone());
        tracing::info!("Prev {n}");
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
        tracing::info!("Next {n}");
        GetReturn::Ok
    }
}
