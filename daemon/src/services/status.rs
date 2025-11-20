use actix_web::{HttpResponse, Responder, get, web};
use tokio::sync::Mutex;

use crate::types::*;

#[get("/")]
pub async fn status(s: web::Data<Mutex<StateStruct>>) -> impl Responder {
    let state = s.lock().await;
    HttpResponse::Ok().json(Response::Status(state.to_status()))
}
