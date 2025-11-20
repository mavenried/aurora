use actix_web::{HttpResponse, Responder, post, web};

use crate::types::*;

#[post("/pause")]
pub async fn pause(state: web::Data<State>) -> impl Responder {
    let mut state = state.lock().await;
    state.pause().await;
    let is_paused = state.is_paused();

    HttpResponse::Ok().json(Response::Confirm {
        message: format!(
            "The player is now {}.",
            if is_paused { "paused." } else { "playing." }
        ),
    })
}
