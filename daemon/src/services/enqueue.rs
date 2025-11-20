use actix_web::{
    HttpResponse, Responder, post,
    web::{self},
};
use uuid::Uuid;

use crate::types::*;

#[post("/add/{uuid}")]
pub async fn enqueue(state: web::Data<State>, path: web::Path<Uuid>) -> impl Responder {
    let song_uuid = path.into_inner();

    let mut state = state.lock().await;

    let song = match state.index.get(&song_uuid) {
        Some(s) => s.clone(),
        None => {
            return HttpResponse::NotFound().body("No such song with id {song_uuid}");
        }
    };

    let should_start = state.queue.is_empty();

    state.queue.push(song.clone());

    if should_start {
        state.add().await;
    }

    let message = format!("Added {} by {} to queue.", song.title, song.artists[0]);
    tracing::info!("{message}");

    HttpResponse::Ok().json(Response::Confirm { message })
}
