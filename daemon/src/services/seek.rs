use crate::types::*;
use actix_web::{HttpResponse, Responder, post, web};
use std::time::Duration;

#[post("/seek/{n}")]
pub async fn seek(state: web::Data<State>, path: web::Path<u64>) -> impl Responder {
    let n = path.into_inner();
    let mut state = state.lock().await;
    if let Some(audio) = &mut state.audio {
        let current_pos = Duration::from_secs(n);
        audio.seek(current_pos);
    }

    HttpResponse::Ok().json(Response::Confirm {
        message: format!("Seeking {n} sec(s).",),
    })
}
