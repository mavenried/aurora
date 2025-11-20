use actix_web::{HttpResponse, Responder, get, web};

use crate::types::*;

#[get("/search/{mode}/{query}")]
pub async fn search(state: web::Data<State>, path: web::Path<(String, String)>) -> impl Responder {
    let state = state.lock().await;
    let (mode, query) = path.into_inner();
    tracing::info!("Searching for {mode} {query}");
    let searchtype = if mode == "artist" {
        SearchType::ByArtist(query)
    } else if mode == "title" {
        SearchType::ByTitle(query)
    } else {
        return HttpResponse::NotFound().body("Invalid search mode, must be `artist` or `title`");
    };
    let songs = state
        .search(searchtype)
        .await
        .iter()
        .map(|songmeta| Song::from(songmeta))
        .collect();
    HttpResponse::Ok().json(Response::SearchResults(songs))
}
