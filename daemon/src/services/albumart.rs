use actix_web::{HttpResponse, Responder, get, http::header, web};
use lofty::{file::TaggedFileExt, read_from_path};
use uuid::Uuid;

use crate::types::State;

#[get("/albumart/{song_uuid}")]
pub async fn albumart(state: web::Data<State>, path: web::Path<Uuid>) -> impl Responder {
    let song_uuid = path.into_inner();

    let index = state.lock().await.index.clone();
    let Some(songmeta) = index.get(&song_uuid) else {
        return HttpResponse::NotFound().body("No such song uuid!");
    };

    let Ok(tagged_file) = read_from_path(&songmeta.path) else {
        return HttpResponse::NotFound().body("Could not open file metadata");
    };
    if let Some(tag) = tagged_file.primary_tag() {
        let picture = tag.pictures().get(0).unwrap();

        let mime_str = picture
            .mime_type()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "application/octet-stream".to_string());

        return HttpResponse::Ok()
            .insert_header((header::CONTENT_TYPE, mime_str))
            .body(picture.data().to_vec());
    }

    HttpResponse::NotFound().body("No album art found")
}
