use std::rc::Rc;

use aurora_protocol::Request;
use slint::{Image, ModelRc, Rgba8Pixel, SharedPixelBuffer, VecModel, Weak};

use crate::{AuroraPlayer, Playlist, Song, types::*};

impl StateStruct {
    pub async fn get_album_art(&mut self, req: ImageFor) -> SharedPixelBuffer<Rgba8Pixel> {
        let (id, waitlist) = match req {
            ImageFor::Queue(id) => (id, &mut self.queue_waitlist),
            ImageFor::Search(id) => (id, &mut self.search_waitlist),
            ImageFor::Playlist(id) => (id, &mut self.playlist_waitlist),
        };
        match self.artcache.get(id) {
            Some(buffer) => {
                waitlist.retain(|x| *x != id);
                buffer
            }
            None => {
                if !waitlist.contains(&id) {
                    waitlist.push(id);
                    let _ = self.writer_tx.send(Request::AlbumArt(id)).await;
                }
                self.default_art_buffer.clone()
            }
        }
    }
    pub async fn update_queue(&mut self, app: Weak<AuroraPlayer>) {
        tracing::info!("Redraw Queue");
        let mut img_data = vec![];

        let queue: Vec<aurora_protocol::Song> = self.queue.clone().into_iter().skip(1).collect();

        for song in queue.clone() {
            img_data.push(self.get_album_art(ImageFor::Queue(song.id)).await.clone())
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
    pub async fn update_playlists(&mut self, app: Weak<AuroraPlayer>) {
        tracing::info!("Redraw PL list");
        let results = self.playlist_list_results.clone();
        let _ = app.upgrade_in_event_loop(move |aurora: AuroraPlayer| {
            let mut songs = vec![];
            for playlist in results {
                songs.push(Playlist {
                    title: playlist.name.clone().into(),
                    length: playlist.len as i32,
                    id: playlist.id.to_string().into(),
                })
            }
            aurora.set_playlistsList(ModelRc::new(Rc::new(VecModel::from(songs))));
        });
    }

    pub async fn update_search_results(&mut self, app: Weak<AuroraPlayer>) {
        tracing::info!("Redraw Search Results");
        let mut img_data = vec![];

        for song in self.search_results.clone() {
            img_data.push(self.get_album_art(ImageFor::Search(song.id)).await.clone())
        }
        let results = self.search_results.clone();
        let _ = app.upgrade_in_event_loop(move |aurora: AuroraPlayer| {
            let mut songs = vec![];
            for (song, img) in results.iter().zip(img_data.iter()) {
                songs.push(Song {
                    title: song.title.clone().into(),
                    artists: song.artists.join(", ").into(),
                    album_art: Image::from_rgba8(img.clone()),
                    id: song.id.to_string().into(),
                })
            }
            aurora.set_searchResults(ModelRc::new(Rc::new(VecModel::from(songs))));
        });
    }
}
