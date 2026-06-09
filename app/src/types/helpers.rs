use std::rc::Rc;

use slint::{Image, ModelRc, VecModel, Weak};

use crate::{AuroraPlayer, Playlist, PlaylistMinimal, Song, types::*};

fn format_duration(d: std::time::Duration) -> slint::SharedString {
    let total = d.as_secs();
    let m = total / 60;
    let s = total % 60;
    format!("{}:{:02}", m, s).into()
}

impl StateStruct {
    pub async fn update_queue(&mut self, app: Weak<AuroraPlayer>) {
        tracing::info!("Redraw Queue");

        let queue: Vec<aurora_protocol::Song> = self.queue.clone().into_iter().skip(1).collect();
        let default_art = self.default_art_buffer.clone();
        let liked = self.liked_song_ids.clone();
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
                    selected: false,
                    duration: format_duration(song.duration),
                    liked: liked.contains(&song.id.to_string()),
                })
            }
            aurora.set_queue(ModelRc::new(Rc::new(VecModel::from(songs))));
        });
    }

    pub async fn update_playlists(&mut self, app: Weak<AuroraPlayer>) {
        tracing::info!("Redraw PL list");
        let results = self.playlist_list_results.clone();
        let default_art = self.default_art_buffer.clone();
        let _ = app.upgrade_in_event_loop(move |aurora: AuroraPlayer| {
            let mut songs = vec![];
            for playlist in results {
                let mut album_arts = vec![];
                for i in 0..4 {
                    album_arts.push(
                        playlist
                            .art_paths
                            .get(i)
                            .and_then(|p| p.as_ref())
                            .and_then(|path| Image::load_from_path(path.as_path()).ok())
                            .unwrap_or_else(|| Image::from_rgba8(default_art.clone())),
                    );
                }
                songs.push(PlaylistMinimal {
                    title: playlist.name.clone().into(),
                    length: playlist.len as i32,
                    id: playlist.id.to_string().into(),
                    album_arts: ModelRc::new(Rc::new(VecModel::from(album_arts))),
                })
            }
            aurora.set_playlistsList(ModelRc::new(Rc::new(VecModel::from(songs))));
        });
    }

    pub async fn update_search_results(&mut self, app: Weak<AuroraPlayer>) {
        tracing::info!("Redraw Search Results");

        let selected = self.selected_song_ids.clone();
        let liked = self.liked_song_ids.clone();
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
                    selected: selected.contains(&song.id.to_string()),
                    duration: format_duration(song.duration),
                    liked: liked.contains(&song.id.to_string()),
                })
            }
            aurora.set_searchResults(ModelRc::new(Rc::new(VecModel::from(songs))));
        });
    }

    pub async fn update_artist_songs(&mut self, app: Weak<AuroraPlayer>) {
        tracing::info!("Redraw Artist Songs");
        let selected = self.selected_song_ids.clone();
        let liked = self.liked_song_ids.clone();
        let default_art = self.default_art_buffer.clone();
        let results = self.artist_songs.clone();
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
                    selected: selected.contains(&song.id.to_string()),
                    duration: format_duration(song.duration),
                    liked: liked.contains(&song.id.to_string()),
                })
            }
            aurora.set_artist_songs(ModelRc::new(Rc::new(VecModel::from(songs))));
        });
    }

    pub async fn update_last_played(&mut self, app: Weak<AuroraPlayer>) {
        tracing::info!("Redraw Last Played");
        let liked = self.liked_song_ids.clone();
        let default_art = self.default_art_buffer.clone();
        let results = self.last_played.clone();
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
                    selected: false,
                    duration: format_duration(song.duration),
                    liked: liked.contains(&song.id.to_string()),
                })
            }
            aurora.set_last_played(ModelRc::new(Rc::new(VecModel::from(songs))));
        });
    }

    pub async fn update_liked_songs(&mut self, app: Weak<AuroraPlayer>) {
        tracing::info!("Redraw Liked Songs");
        let selected = self.selected_song_ids.clone();
        let liked = self.liked_song_ids.clone();
        let default_art = self.default_art_buffer.clone();
        let results = self.liked_songs.clone();
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
                    selected: selected.contains(&song.id.to_string()),
                    duration: format_duration(song.duration),
                    liked: liked.contains(&song.id.to_string()),
                })
            }
            aurora.set_liked_songs(ModelRc::new(Rc::new(VecModel::from(songs))));
        });
    }

    pub async fn update_playlist_results(&mut self, app: Weak<AuroraPlayer>) {
        tracing::info!("Redraw Playlist Content Results");
        let result = self.playlist_result.clone().unwrap();
        let selected = self.selected_song_ids.clone();
        let liked = self.liked_song_ids.clone();
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
                    selected: selected.contains(&song.id.to_string()),
                    duration: format_duration(song.duration),
                    liked: liked.contains(&song.id.to_string()),
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
