use crate::{
    app::AuroraPlayer,
    types::{Message, TcpWriter},
};
use aurora_protocol::{Request, Response};
use iced::Task;
use uuid::Uuid;

impl AuroraPlayer {
    pub async fn handle_packet(data: Vec<u8>) -> Response {
        serde_json::from_slice(data.as_slice()).unwrap()
    }

    pub async fn send(maybe_conn: Option<TcpWriter>, req: Request) {
        if let Some(conn) = maybe_conn
            && let Err(e) = conn.send(&req).await
        {
            tracing::error!("Error:{e}")
        }
    }
    pub fn get_art(&mut self, id: Uuid) -> Task<Message> {
        if self.artcache.get(&id).is_none() && !self.pending_art_requests.contains(&id) {
            Task::perform(
                Self::send(self.tcp_connection.clone(), Request::AlbumArt(id)),
                |_o| Message::NoOP,
            )
        } else {
            Task::none()
        }
    }
}
