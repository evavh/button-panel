use std::thread;

use mpdrs::client::Client;
use mpdrs::status::State::Play;
use mpdrs::{Idle, Playlist};
use sled::IVec;

mod db;
use db::Db;

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
        use AudioMode::*;
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
    database: Db,
    pub(crate) mode: AudioMode,
}

impl Mpd {
    pub(crate) fn connect(ip: &str) -> Self {
        let client = Client::connect(ip).unwrap();
        let mut mpd = Mpd {
            client,
            ip: ip.to_owned(),
            database: Db::open(),
            mode: AudioMode::Music,
        };
        mpd.playing();
        mpd
    }

    pub(crate) fn playing(&mut self) -> bool {
        let playback_state = self.client.status().unwrap().state;
        playback_state == Play
    }

    pub(crate) fn toggle_playback(&mut self) {
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

    pub(crate) fn switch_to_mode(&mut self) {
        let playlist = self.find_mode_playlist().unwrap();
        self.load_playlist(&playlist);
        if let Some((song_id, position)) = self.database.fetch_playlist_state(playlist) {
            self.client.seek_id(song_id, position).unwrap();
        } else {
            self.client.seek(0, 0).unwrap();
        }
    }

    pub(crate) fn store_position(&mut self) {
        let position = self.client.status().unwrap().elapsed;
        self.database.store_position(&self.mode, position);
    }

    fn find_mode_playlist(&mut self) -> Option<Playlist> {
        let playlists = dbg!(self.playlists());
        for playlist in playlists {
            if playlist.name.starts_with(self.mode.to_prefix()) {
                return dbg!(Some(playlist));
            }
        }
        None
    }

    fn load_playlist(&mut self, playlist: &Playlist) {
        self.client.clear().unwrap();
        self.client.load(&playlist.name, ..).unwrap();
    }

    fn set_song_and_pos(&mut self, song_id: u32, position: u32) {
        self.client.seek_id(song_id, position).unwrap();
    }
}
