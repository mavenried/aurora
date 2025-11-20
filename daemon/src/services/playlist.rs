use actix_web::{HttpResponse, Responder, get, post, web};
use uuid::Uuid;

use crate::{
    helpers::{create_playlist, get_all_playlists, get_playlist},
    types::*,
};

#[post("/playlist/create")]
pub async fn playlist_create(item: web::Json<Playlist>) -> impl Responder {
    match create_playlist(item.into_inner()).await {
        Ok(name) => {
            let message = format!("Created playlist {name}.");
            HttpResponse::Ok().json(Response::Confirm { message })
        }
        Err(err) => HttpResponse::InternalServerError().json(Response::Error {
            err_id: 1,
            err_msg: err.to_string(),
        }),
    }
}

#[get("/playlist")]
pub async fn playlist_list() -> impl Responder {
    match get_all_playlists().await {
        Ok(list) => HttpResponse::Ok().json(list),
        Err(err) => HttpResponse::InternalServerError().json(Response::Error {
            err_id: 1,
            err_msg: err.to_string(),
        }),
    }
}

#[get("/playlist/{id}")]
pub async fn playlist_get(path: web::Path<Uuid>) -> impl Responder {
    let id = path.into_inner();
    match get_playlist(id).await {
        Ok(playlistmeta) => HttpResponse::Ok().json(playlistmeta),
        Err(err) => HttpResponse::NotFound().json(Response::Error {
            err_id: 4,
            err_msg: err.to_string(),
        }),
    }
}
