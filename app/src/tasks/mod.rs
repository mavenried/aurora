use aurora_protocol::Response;

pub async fn handle_packet(data: Vec<u8>) -> Response {
    return serde_json::from_slice(data.as_slice()).unwrap();
}
