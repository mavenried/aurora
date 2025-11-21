use crate::types::StateStruct;
use aurora_protocol::{SearchType, SongMeta};

impl StateStruct {
    pub async fn search(&self, s: SearchType) -> Vec<SongMeta> {
        let mut results = Vec::new();

        match s {
            SearchType::ByTitle(query) => {
                let q = query.to_lowercase();
                for meta in self.index.values() {
                    if meta.title.to_lowercase().contains(&q) {
                        results.push(meta.clone());
                    }
                }
            }
            SearchType::ByArtist(query) => {
                let q = query.to_lowercase();
                for meta in self.index.values() {
                    for artist in meta.artists.clone() {
                        if artist.to_lowercase().contains(&q) {
                            results.push(meta.clone());
                            break;
                        }
                    }
                }
            }
        }

        results.sort_by(|a, b| a.title.cmp(&b.title));
        results
    }
}
