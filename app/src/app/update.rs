use crate::app::AuroraPlayer;
use crate::tasks::handle_packet;
use crate::types::*;
use aurora_protocol::Response;
use iced::Task;

impl AuroraPlayer {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::MainViewSelect(view) => {
                self.current_mainview = view;
            }
            Message::PlaylistSelected(_id) => (),

            Message::TcpEvent(ev) => match ev {
                TcpEvent::Connected(conn) => self.tcp_connection = Some(conn),
                TcpEvent::PacketReceived(data) => {
                    return Task::perform(handle_packet(data), Message::Response);
                }
                TcpEvent::PacketSent => (),
                TcpEvent::Error(err) => tracing::error!("{err}"),
                TcpEvent::Disconnected => (),
            },

            Message::Response(res) => match res {
                Response::Status(s) => {
                    tracing::info!("{s:?}");
                    self.status = s;
                }
                _ => (),
            },
        }
        Task::none()
    }
}
