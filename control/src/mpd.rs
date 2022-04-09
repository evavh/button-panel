use std::thread;

use mpdrs::client::Client;
use mpdrs::status::State::Play;
use mpdrs::{Idle, Playlist};
use sled::IVec;

#[derive(Debug)]
pub enum AudioMode {
    Music,
    Book,
    Podcast,
    Meditation,
}

impl AudioMode {
    pub fn next(&mut self) {
        use AudioMode::*;
        *self = match self {
            Music => Book,
            Book => Podcast,
            Podcast => Meditation,
            Meditation => Music,
        }
    }

    fn to_prefix(&self) -> &str {
        match self {
            Music => "music_",
            Book => "book_",
            Podcast => "podcast_",
            Meditation => "meditation_",
        }
    }
}

pub(crate) struct Mpd {
    client: Client,
    ip: String,
    database: sled::Db,
    current_playlist: Option<String>,
}

impl Mpd {
    pub(crate) fn connect(ip: &str) -> Self {
        let client = Client::connect(ip).unwrap();
        let mut mpd = Mpd {
            client,
            ip: ip.to_owned(),
            database: sled::open("database").unwrap(),
            current_playlist: None,
        };
        mpd.playing();
        mpd
    }

    pub(crate) fn playing(&mut self) -> bool {
        let playback_state = self.client.status().unwrap().state;
        playback_state == Play
    }

    pub(crate) fn toggle(&mut self) {
        self.client.toggle_pause().unwrap();
    }

    pub(crate) fn rewind(&mut self) {
        let position: u32 = self
            .client
            .status()
            .unwrap()
            .elapsed
            .unwrap()
            .as_secs()
            .try_into()
            .unwrap();
        self.client.rewind(position.saturating_sub(15)).unwrap();
    }

    pub(crate) fn skip(&mut self) {
        let position: u32 = self
            .client
            .status()
            .unwrap()
            .elapsed
            .unwrap()
            .as_secs()
            .try_into()
            .unwrap();
        self.client.rewind(position + 15).unwrap();
    }

    pub(crate) fn next(&mut self) {
        self.client.next().unwrap();
    }

    pub(crate) fn previous(&mut self) {
        self.client.prev().unwrap();
    }

    pub(crate) fn playlists(&mut self) -> Vec<Playlist> {
        self.client.playlists().unwrap()
    }

    pub(crate) fn rescan(&mut self) {
        let mut watcher = mpdrs::Client::connect(&self.ip).unwrap();
        let thread_join_handle = thread::spawn(move || {
            watcher.wait(&[mpdrs::idle::Subsystem::Update]).unwrap();
        });
        self.client.rescan().unwrap();
        thread_join_handle.join().unwrap();
    }

    pub(crate) fn switch_to_mode(&mut self, mode: &AudioMode) {
        let playlist = self.find_mode_playlist(mode).unwrap();
        self.load_playlist(&playlist);
        if let Some((song_id, position)) = self.fetch_playlist_state(playlist) {
            self.client.seek_id(song_id, position).unwrap();
        } else {
            self.client.seek(0, 0).unwrap();
        }
    }

    fn find_mode_playlist(&mut self, mode: &AudioMode) -> Option<Playlist> {
        let playlists = dbg!(self.playlists());
        for playlist in playlists {
            if playlist.name.starts_with(mode.to_prefix()) {
                return dbg!(Some(playlist));
            }
        }
        None
    }

    pub(crate) fn store_position(&mut self) {
        let position = self.client.status().unwrap().elapsed;
        if let Some(current_playlist) = &self.current_playlist {
            if let Some(position) = position {
                let key = current_playlist.to_owned() + "_position";
                self.database
                    .insert(key.as_bytes(), &position.as_secs().to_ne_bytes())
                    .unwrap();
            }
        }
    }

    fn fetch_playlist_state(&self, playlist: Playlist) -> Option<(u32, u32)> {
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

    fn load_playlist(&mut self, playlist: &Playlist) {
        self.client.clear().unwrap();
        self.client.load(&playlist.name, ..).unwrap();
    }

    fn set_song_and_pos(&mut self, song_id: u32, position: u32) {
        self.client.seek_id(song_id, position).unwrap();
    }
}
