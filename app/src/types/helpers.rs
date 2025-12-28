use std::rc::Rc;

use slint::{Image, ModelRc, VecModel, Weak};

use crate::{AuroraPlayer, Playlist, PlaylistMinimal, Song, types::*};

impl StateStruct {
    pub async fn update_queue(&mut self, app: Weak<AuroraPlayer>) {
        tracing::info!("Redraw Queue");

        let queue: Vec<aurora_protocol::Song> = self.queue.clone().into_iter().skip(1).collect();
        let default_art = self.default_art_buffer.clone();
        let _ = app.upgrade_in_event_loop(move |aurora: AuroraPlayer| {
            let mut songs = vec![];
            for song in queue.iter() {
                songs.push(Song {
                    title: song.title.clone().into(),
                    artists: song.artists.join(", ").into(),
                    album_art: {
                        if song.art_path.is_some() {
                            Image::load_from_path(song.art_path.clone().unwrap().as_path()).unwrap()
                        } else {
                            Image::from_rgba8(default_art.clone())
                        }
                    },
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
                songs.push(PlaylistMinimal {
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

        let default_art = self.default_art_buffer.clone();
        let results = self.search_results.clone();
        let _ = app.upgrade_in_event_loop(move |aurora: AuroraPlayer| {
            let mut songs = vec![];
            for song in results.iter() {
                songs.push(Song {
                    title: song.title.clone().into(),

                    album_art: {
                        if song.art_path.is_some() {
                            Image::load_from_path(song.art_path.clone().unwrap().as_path()).unwrap()
                        } else {
                            Image::from_rgba8(default_art.clone())
                        }
                    },
                    artists: song.artists.join(", ").into(),
                    id: song.id.to_string().into(),
                })
            }
            aurora.set_searchResults(ModelRc::new(Rc::new(VecModel::from(songs))));
        });
    }

    pub async fn update_playlist_results(&mut self, app: Weak<AuroraPlayer>) {
        tracing::info!("Redraw Playlist Content Results");
        let result = self.playlist_result.clone().unwrap();
        let default_art = self.default_art_buffer.clone();
        let results = result.songs;

        let _ = app.upgrade_in_event_loop(move |aurora: AuroraPlayer| {
            let mut songs = vec![];
            for song in results.iter() {
                songs.push(Song {
                    title: song.title.clone().into(),

                    album_art: {
                        if song.art_path.is_some() {
                            Image::load_from_path(song.art_path.clone().unwrap().as_path()).unwrap()
                        } else {
                            Image::from_rgba8(default_art.clone())
                        }
                    },
                    artists: song.artists.join(", ").into(),
                    id: song.id.to_string().into(),
                })
            }
            let slint_playlist = Playlist {
                title: result.title.into(),
                id: result.id.to_string().into(),
                songs: ModelRc::new(Rc::new(VecModel::from(songs))),
            };
            aurora.set_playlistResult(slint_playlist);
        });
    }
}
