use std::time::Duration;

use crate::app::AuroraPlayer;
use crate::types::*;
use aurora_protocol::{Request, Response};
use base64::Engine;
use base64::prelude::BASE64_URL_SAFE;
use iced::Task;
use iced::widget::image;

impl AuroraPlayer {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::MainViewSelect(view) => {
                self.current_mainview = view;
            }
            Message::PlaylistSelected(_id) => (),

            Message::SliderChanged(x) => {
                self.progress_slider_state = x;
                self.slider_pressed = true
            }
            Message::SeekCommit => {
                let dur = Duration::from_millis(self.progress_slider_state as u64);
                self.slider_pressed = false;
                self.status.position = dur;
                return Task::perform(
                    Self::send(self.unix_connection.clone(), Request::Seek(dur)),
                    |_o| Message::NoOP,
                );
            }

            Message::TcpEvent(ev) => match ev {
                UnixSocketEvent::Connected(conn) => self.unix_connection = Some(conn),
                UnixSocketEvent::PacketReceived(data) => {
                    return Task::perform(Self::handle_packet(data), Message::Response);
                }
                UnixSocketEvent::PacketSent => (),
                UnixSocketEvent::Error(err) => tracing::error!("{err}"),
                UnixSocketEvent::Disconnected => (),
            },

            Message::NoOP => (),

            Message::Response(res) => match res {
                Response::Status(s) => {
                    let task = if let Some(song) = &s.current_song {
                        self.get_art(song.id)
                    } else {
                        Task::none()
                    };
                    self.status = s;
                    return task;
                }
                Response::Picture { id, data } => {
                    if let Ok(bytes) = BASE64_URL_SAFE.decode(data) {
                        let handle = image::Handle::from_bytes(bytes);
                        self.artcache.insert(id, handle);
                    }
                }
                Response::Queue(queue) => {
                    let mut tasks = vec![];
                    for song in &queue {
                        tasks.push(self.get_art(song.id))
                    }
                    self.queue = queue;
                    return Task::batch(tasks);
                }
                _ => (),
            },
        }
        Task::none()
    }
}
