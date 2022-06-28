use super::AudioMode;

#[derive(Debug)]
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

pub(crate) struct Db {
    database: sled::Db,
}

impl Db {
    pub(crate) fn open() -> Self {
        Db {
            database: sled::open("database").unwrap(),
        }
    }

    pub(crate) fn fetch_playlist_name(&self, mode: &AudioMode) -> Option<String> {
        let key = mode.to_prefix().to_owned() + "cur_playlist";
        self.database
            .get(key.as_bytes())
            .unwrap()
            .map(|data| String::from_utf8(data.to_vec()).unwrap())
    }

    pub(crate) fn store_playlist_name(&self, mode: &AudioMode, playlist_name: &str) {
        let key = mode.to_prefix().to_owned() + "cur_playlist";
        self.database
            .insert(key.as_bytes(), playlist_name.as_bytes())
            .unwrap();
    }

    pub(crate) fn store_position(&mut self, playlist_name: &str, position: Position) {
            let key = playlist_name.to_owned() + "_position";
            self.database
                .insert(key.as_bytes(), position.to_bytes())
                .unwrap();
    }

    pub(crate) fn fetch_position(&self, playlist_name: &str) -> Option<Position> {
        let key = playlist_name.to_owned() + "_position";
        self.database
            .get(key.as_bytes())
            .unwrap()
            .map(|buffer| Position::from_bytes(buffer.as_ref()))
    }
}
