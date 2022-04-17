use mpdrs::Playlist;
use sled::IVec;
use std::time::Duration;

use super::AudioMode;

pub(crate) struct Db {
    database: sled::Db,
}

impl Db {
    pub(crate) fn open() -> Self {
        Db {
            database: sled::open("database").unwrap(),
        }
    }

    fn current_playlist_name(&self, mode: &AudioMode) -> Option<String> {
        let key = mode.to_prefix().to_owned() + "cur_playlist";
        if let Some(data) = self.database.get(key.as_bytes()).unwrap() {
            Some(String::from_utf8(data.to_vec()).unwrap())
        } else {
            None
        }
    }

    fn store_current_playlist(&self, mode: AudioMode, playlist_name: String) {
        let key = mode.to_prefix().to_owned() + "cur_playlist";
        self.database
            .insert(key.as_bytes(), playlist_name.as_bytes())
            .unwrap();
    }

    pub(crate) fn store_position(&mut self, mode: &AudioMode, position: Option<Duration>) {
        if let Some(current_playlist_name) = self.current_playlist_name(mode) {
            if let Some(position) = position {
                let key = current_playlist_name.to_owned() + "_position";
                self.database
                    .insert(key.as_bytes(), &position.as_secs().to_ne_bytes())
                    .unwrap();
            }
        }
    }

    pub(crate) fn fetch_playlist_state(&self, playlist: Playlist) -> Option<(u32, u32)> {
        fn to_u32(buffer: IVec) -> u32 {
            u32::from_ne_bytes(buffer.as_ref().try_into().unwrap())
        }

        let key = playlist.name.clone() + "song_id";
        let song_id = self.database.get(key.as_bytes()).unwrap();
        let key = playlist.name + "position";
        let position = self.database.get(key.as_bytes()).unwrap();
        match (song_id, position) {
            (Some(id), Some(pos)) => Some((to_u32(id), to_u32(pos))),
            (Some(id), None) => Some((to_u32(id), 0)),
            (None, _) => None,
        }
    }
}
