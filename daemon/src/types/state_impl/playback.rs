use std::time::Duration;

use rand::Rng;
use std::collections::VecDeque;

use super::{GetReturn, StateStruct};
use aurora_protocol::SongMeta;

fn rotate_next(queue: &mut VecDeque<SongMeta>, n: usize) {
    let steps = n % queue.len().max(1);
    for _ in 0..steps {
        if let Some(song) = queue.pop_front() {
            queue.push_back(song);
        }
    }
}

fn rotate_prev(queue: &mut VecDeque<SongMeta>, n: usize) {
    let steps = n % queue.len().max(1);
    for _ in 0..steps {
        if let Some(song) = queue.pop_back() {
            queue.push_front(song);
        }
    }
}

impl StateStruct {
    pub async fn prev(&mut self, n: usize) -> GetReturn {
        let len = self.queue.len();
        if len == 0 {
            return GetReturn::QueueEmpty;
        }

        if n == 0 {
            return GetReturn::Ok;
        }

        if n == 1 {
            if let Some(audio) = &self.audio
                && audio.get_position() < Duration::from_secs(1)
            {
                let song = self.queue.pop_back().unwrap();
                self.queue.push_front(song);
            }
        } else {
            rotate_prev(&mut self.queue, n);
        }
        self.current_song.replace(self.queue[0].clone());
        tracing::info!("Prev {n}");
        GetReturn::Ok
    }

    pub async fn next(&mut self, n: usize) -> GetReturn {
        let len = self.queue.len();
        if len == 0 {
            return GetReturn::QueueEmpty;
        }

        if n == 0 {
            return GetReturn::Ok;
        }

        if self.repeat == 1 {
            self.current_song.replace(self.queue[0].clone());
            tracing::info!("Next (repeat-one)");
            return GetReturn::Ok;
        }

        if self.shuffle && self.queue.len() > 1 {
            let song = self.queue.pop_front().unwrap();
            self.queue.push_back(song);

            let idx = rand::thread_rng().gen_range(0..self.queue.len() - 1);
            self.queue.swap(0, idx);
            self.current_song.replace(self.queue[0].clone());
            tracing::info!("Next (shuffle)");
            return GetReturn::Ok;
        }

        rotate_next(&mut self.queue, n);

        self.current_song.replace(self.queue[0].clone());
        tracing::info!("Next {n}");
        GetReturn::Ok
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn queue(size: usize) -> VecDeque<SongMeta> {
        (0..size)
            .map(|index| SongMeta {
                id: Uuid::from_u128(index as u128 + 1),
                title: format!("track-{index}"),
                artists: vec!["artist".into()],
                duration: Duration::from_secs(10),
                path: format!("track-{index}.mp3").into(),
                art_path: None,
            })
            .collect()
    }

    fn ids(queue: &VecDeque<SongMeta>) -> Vec<u128> {
        queue.iter().map(|song| song.id.as_u128()).collect()
    }

    #[test]
    fn next_rotation_moves_front_to_back() {
        let mut queue = queue(3);
        rotate_next(&mut queue, 1);
        assert_eq!(ids(&queue), vec![2, 3, 1]);
    }

    #[test]
    fn previous_rotation_moves_back_to_front() {
        let mut queue = queue(3);
        rotate_prev(&mut queue, 1);
        assert_eq!(ids(&queue), vec![3, 1, 2]);
    }

    #[test]
    fn rotations_wrap_large_step_counts() {
        let mut queue = queue(3);
        rotate_next(&mut queue, usize::MAX - 1);
        assert_eq!(ids(&queue), vec![3, 1, 2]);
        rotate_prev(&mut queue, usize::MAX - 1);
        assert_eq!(ids(&queue), vec![1, 2, 3]);
    }

    #[test]
    fn empty_and_single_item_queues_are_safe() {
        let mut empty = queue(0);
        rotate_next(&mut empty, usize::MAX - 1);
        rotate_prev(&mut empty, usize::MAX - 1);
        assert!(empty.is_empty());

        let mut one = queue(1);
        rotate_next(&mut one, usize::MAX - 1);
        rotate_prev(&mut one, usize::MAX - 1);
        assert_eq!(ids(&one), vec![1]);
    }

    #[test]
    fn zero_steps_leave_queue_unchanged() {
        let mut queue = queue(3);
        rotate_next(&mut queue, 0);
        rotate_prev(&mut queue, 0);
        assert_eq!(ids(&queue), vec![1, 2, 3]);
    }
}
