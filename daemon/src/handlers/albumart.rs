use base64::{Engine, prelude::BASE64_URL_SAFE};
use lofty::{file::TaggedFileExt, read_from_path};
use uuid::Uuid;

use crate::{
    helpers::send_to_client,
    types::{State, WriteSocket},
};
use aurora_protocol::Response;

pub async fn albumart(stream: &WriteSocket, state: &State, song_uuid: Uuid) -> anyhow::Result<()> {
    let index = state.lock().await.index.clone();
    let Some(songmeta) = index.get(&song_uuid) else {
        send_to_client(
            stream,
            &Response::Error {
                err_id: 1,
                err_msg: format!("There this no song with the id `{song_uuid}`"),
            },
        )
        .await?;
        return Ok(());
    };

    let Ok(tagged_file) = read_from_path(&songmeta.path) else {
        send_to_client(
            stream,
            &Response::Error {
                err_id: 2,
                err_msg: "Could not open file metadata.".to_string(),
            },
        )
        .await?;
        return Ok(());
    };

    if let Some(tag) = tagged_file.primary_tag() {
        let picture = &tag.pictures()[0];
        let data = BASE64_URL_SAFE.encode(picture.data());
        send_to_client(
            stream,
            &Response::Picture {
                id: song_uuid,
                data,
            },
        )
        .await?;
    };

    Ok(())
}
