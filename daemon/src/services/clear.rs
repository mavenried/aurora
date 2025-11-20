use crate::types::*;
use actix_web::{HttpResponse, Responder, post, web};

#[post("/clear")]
pub async fn clear(state: web::Data<State>) -> impl Responder {
    state.lock().await.clear().await;
    let message = String::from("Queue Cleared.");
    tracing::info!("{message}");
    HttpResponse::Ok().json(Response::Confirm { message })
}
