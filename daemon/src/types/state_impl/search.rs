use super::{SearchType, SongMeta, StateStruct};
impl StateStruct {
    pub async fn search(&self, s: SearchType) -> Vec<SongMeta> {
        let mut results = Vec::new();

        match s {
            SearchType::ByTitle(query) => {
                let q = query.to_lowercase();
                for (_id, meta) in &self.index {
                    if meta.title.to_lowercase().contains(&q) {
                        results.push(meta.clone());
                    }
                }
            }
            SearchType::ByArtist(query) => {
                let q = query.to_lowercase();
                for (_id, meta) in &self.index {
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
