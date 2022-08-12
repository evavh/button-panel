use super::AudioMode;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct Position {
    pub(crate) pos_in_pl: u32,
    pub(crate) elapsed: u32,
}

impl Position {
    fn to_bytes(&self) -> Vec<u8> {
        [self.pos_in_pl.to_ne_bytes(), self.elapsed.to_ne_bytes()].concat()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        Position {
            pos_in_pl: u32::from_ne_bytes(bytes[..4].try_into().unwrap()),
            elapsed: u32::from_ne_bytes(bytes[4..].try_into().unwrap()),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Db {
    database: sled::Db,
}

impl Db {
    pub(crate) fn open(path: &str) -> Self {
        Db {
            database: sled::Config::default()
                .path(path)
                .cache_capacity(1_000_000)
                .open()
                .unwrap(),
        }
    }

    pub(crate) fn fetch_playlist_name(
        &self,
        mode: &AudioMode,
    ) -> Option<String> {
        let key = mode.to_prefix().to_owned() + "cur_playlist";
        self.database
            .get(key.as_bytes())
            .unwrap()
            .map(|data| String::from_utf8(data.to_vec()).unwrap())
    }

    pub(crate) fn store_playlist_name(
        &self,
        mode: &AudioMode,
        playlist_name: &str,
    ) {
        let key = mode.to_prefix().to_owned() + "cur_playlist";
        self.database
            .insert(key.as_bytes(), playlist_name.as_bytes())
            .unwrap();
    }

    pub(crate) fn fetch_position(
        &self,
        playlist_name: &str,
    ) -> Option<Position> {
        let key = playlist_name.to_owned() + "_position";
        self.database
            .get(key.as_bytes())
            .unwrap()
            .map(|buffer| Position::from_bytes(buffer.as_ref()))
    }

    pub(crate) fn store_position(
        &mut self,
        playlist_name: &str,
        position: &Position,
    ) {
        let key = playlist_name.to_owned() + "_position";
        self.database
            .insert(key.as_bytes(), position.to_bytes())
            .unwrap();
    }

    pub(crate) fn now_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub(crate) fn fetch_last_played(&self, playlist: &str) -> Option<u64> {
        let key = playlist.to_owned() + "last_played";
        self.database
            .get(key.as_bytes())
            .unwrap()
            .map(|bytes| u64::from_ne_bytes(bytes[..8].try_into().unwrap()))
    }

    pub(crate) fn store_last_played(&self, playlist: &str, last_played: u64) {
        let key = playlist.to_owned() + "last_played";

        self.database
            .insert(key.as_bytes(), &last_played.to_ne_bytes())
            .unwrap();
    }

    pub(crate) fn fetch_mode(&self) -> Option<AudioMode> {
        let key = "current_mode";
        self.database
            .get(key.as_bytes())
            .unwrap()
            .map(|buffer| AudioMode::from_bytes(buffer.as_ref()))
    }

    pub(crate) fn store_mode(&self, mode: &AudioMode) {
        let key = "current_mode";
        self.database
            .insert(key.as_bytes(), &mode.to_bytes())
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_and_store_last_played() {
        let db = Db::open("test_db");

        let playlist = "test_playlist_name";
        let last_played = Db::now_timestamp();

        db.store_last_played(&playlist, last_played);
        let fetched = db.fetch_last_played(&playlist).unwrap();
        assert_eq!(fetched, last_played);
    }
}
