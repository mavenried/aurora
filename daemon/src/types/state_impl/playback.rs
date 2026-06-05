use std::time::Duration;

use rand::Rng;

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

        // Repeat-one: don't advance, just replay current song
        if self.repeat == 1 {
            self.current_song.replace(self.queue[0].clone());
            tracing::info!("Next (repeat-one)");
            return GetReturn::Ok;
        }

        // Shuffle: move current song to back, then randomly pick the next one
        if self.shuffle && self.queue.len() > 1 {
            let song = self.queue.pop_front().unwrap();
            self.queue.push_back(song);
            // Pick from everything except the song we just sent to back
            let idx = rand::thread_rng().gen_range(0..self.queue.len() - 1);
            self.queue.swap(0, idx);
            self.current_song.replace(self.queue[0].clone());
            tracing::info!("Next (shuffle)");
            return GetReturn::Ok;
        }

        // Normal advance
        for _ in 0..n {
            let song = self.queue.pop_front().unwrap();
            self.queue.push_back(song);
        }

        self.current_song.replace(self.queue[0].clone());
        tracing::info!("Next {n}");
        GetReturn::Ok
    }
}
