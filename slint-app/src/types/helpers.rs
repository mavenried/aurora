use std::rc::Rc;

use aurora_protocol::Request;
use slint::{Image, ModelRc, Rgba8Pixel, SharedPixelBuffer, VecModel, Weak};
use uuid::Uuid;

use crate::{types::StateStruct, AuroraPlayer, Song};

impl StateStruct {
    pub async fn get_album_art(&mut self, id: Uuid) -> SharedPixelBuffer<Rgba8Pixel> {
        match self.artcache.get(id) {
            Some(buffer) => {
                self.waiting_for_art.retain(|x| *x != id);
                buffer
            }
            None => {
                if !self.waiting_for_art.contains(&id) {
                    self.waiting_for_art.push(id);
                    let _ = self.writer_tx.send(Request::AlbumArt(id)).await;
                }
                self.default_art_buffer.clone()
            }
        }
    }
    pub async fn update_queue(&mut self, app: Weak<AuroraPlayer>) {
        tracing::info!("Redraw Queue");
        let mut img_data = vec![];

        let mut queue: Vec<aurora_protocol::Song> = self.queue.clone().into_iter().skip(self.cur_idx + 1).collect();
        queue.extend(self.queue.clone().into_iter().take(self.cur_idx));

        for song in queue.clone() {
            img_data.push(self.get_album_art(song.id).await.clone())
        }
        let _ = app.upgrade_in_event_loop(move |aurora: AuroraPlayer| {
            let mut songs = vec![];
            for (song, img) in queue.iter().zip(img_data.iter()) {
                songs.push(Song {
                    title: song.title.clone().into(),
                    artists: song.artists.join(", ").into(),
                    album_art: Image::from_rgba8(img.clone()),
                    id: song.id.to_string().into(),
                })
            }
            aurora.set_queue(ModelRc::new(Rc::new(VecModel::from(songs))));
        });
    }
}
