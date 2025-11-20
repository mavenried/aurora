use crate::types::*;
use actix_web::{HttpResponse, Responder, post, web};

#[post("/next/{n}")]
pub async fn next(state: web::Data<State>, path: web::Path<usize>) -> impl Responder {
    let n = path.into_inner();
    let mut state = state.lock().await;
    state.next(n).await;
    state.add().await;
    let message = format!("Skipping {n} song(s).");
    tracing::info!("{message}");
    HttpResponse::Ok().json(Response::Confirm { message })
}

#[post("/prev/{n}")]
pub async fn prev(state: web::Data<State>, path: web::Path<usize>) -> impl Responder {
    let n = path.into_inner();
    let mut state = state.lock().await;
    state.prev(n).await;
    state.add().await;
    let message = format!("Skipping {n} song(s).");
    tracing::info!("{message}");
    HttpResponse::Ok().json(Response::Confirm { message })
}
