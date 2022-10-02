use super::{db::Position, AudioMode};
use std::time::{SystemTime, UNIX_EPOCH};

#[dbstruct::dbstruct(db=sled)]
struct PersistentData {
    mode_cur_playlist: HashMap<AudioMode, String>,
    playlist_positions: HashMap<String, Position>,
    playlist_last_played: HashMap<String, u64>,
    current_mode: Option<AudioMode>,
}

pub(crate) struct Db {
    database: PersistentData,
}

impl Db {
    pub(crate) fn open(path: &str) -> Self {
        Db {
            database: PersistentData::new(path.to_owned()+"_test").unwrap(),
        }
    }

    pub(crate) fn fetch_playlist_name(
        &self,
        mode: &AudioMode,
    ) -> Option<String> {
        self.database.mode_cur_playlist().get(mode).unwrap()
    }

    pub(crate) fn store_playlist_name(
        &self,
        mode: &AudioMode,
        playlist_name: &str,
    ) {
        self.database
            .mode_cur_playlist()
            .insert(mode, &playlist_name.to_owned())
            .unwrap();
    }

    pub(crate) fn fetch_position(
        &self,
        playlist_name: &str,
    ) -> Option<Position> {
        self.database
            .playlist_positions()
            .get(&playlist_name.to_owned())
            .unwrap()
    }

    pub(crate) fn store_position(
        &mut self,
        playlist_name: &str,
        position: &Position,
    ) {
        self.database
            .playlist_positions()
            .insert(&playlist_name.to_owned(), position)
            .unwrap();
    }

    pub(crate) fn now_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub(crate) fn fetch_last_played(&self, playlist: &str) -> Option<u64> {
        self.database
            .playlist_last_played()
            .get(&playlist.to_owned())
            .unwrap()
    }

    pub(crate) fn store_last_played(&self, playlist: &str, last_played: u64) {
        self.database
            .playlist_last_played()
            .insert(&playlist.to_owned(), &last_played)
            .unwrap();
    }

    pub(crate) fn fetch_mode(&self) -> Option<AudioMode> {
        self.database.current_mode().get().unwrap()
    }

    pub(crate) fn store_mode(&self, mode: &AudioMode) {
        self.database
            .current_mode()
            .set(mode)
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_and_store_last_played() {
        let db = Db::open("test_db2");

        let playlist = "test_playlist_name";
        let last_played = Db::now_timestamp();

        db.store_last_played(&playlist, last_played);
        let fetched = db.fetch_last_played(&playlist).unwrap();
        assert_eq!(fetched, last_played);
    }
}
