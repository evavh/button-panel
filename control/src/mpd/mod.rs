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
        mpd.is_playing();
        mpd
    }

    pub(crate) fn rescan(&mut self) {
        let mut watcher = mpdrs::Client::connect(&self.ip).unwrap();
        let thread_join_handle = thread::spawn(move || {
            watcher.wait(&[mpdrs::idle::Subsystem::Update]).unwrap();
        });
        self.client.rescan().unwrap();
        thread_join_handle.join().unwrap();
    }

    pub(crate) fn is_playing(&mut self) -> bool {
        let playback_state = self.client.status().unwrap().state;
        playback_state == Play
    }

    pub(crate) fn get_playlists(&mut self) -> Vec<Playlist> {
        self.client.playlists().unwrap()
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

    pub(crate) fn previous(&mut self) {
        self.client.prev().unwrap();
    }

    pub(crate) fn next(&mut self) {
        self.client.next().unwrap();
    }

    pub(crate) fn prev_playlist(&self) {
        todo!()
    }

    pub(crate) fn next_playlist(&self) {
        todo!()
    }

    pub(crate) fn next_mode(&self) {
        /* self.store_position();
        self.mode.next();
        self.load_mode_playlist_pos();
        dbg!(&mpd.mode); */
        todo!();
    }

    fn load_mode_playlist_pos(&mut self) {
        let playlist_name = self.playlist_name_for_mode().unwrap();
        self.load_playlist(&playlist_name);
        if let Some(position) = self.database.fetch_position(playlist_name) {
            self.client
                .seek_id(position.song_id, position.elapsed)
                .unwrap();
            dbg!("Loaded {}", position);
        } else {
            self.client.seek(0, 0).unwrap();
            dbg!("No position found in db");
        }
    }

    fn store_position(&mut self) {
        let position = db::Position {
            song_id: self.client.status().unwrap().song.unwrap().id,
            elapsed: self
                .client
                .status()
                .unwrap()
                .elapsed
                .unwrap()
                .as_secs()
                .try_into()
                .unwrap(),
        };
        self.database.store_position(&self.mode, position);
    }

    fn playlist_name_for_mode(&mut self) -> Option<String> {
        let playlists = self.get_playlists();
        for playlist in playlists {
            if playlist.name.starts_with(self.mode.to_prefix()) {
                return dbg!(Some(playlist.name));
            }
        }
        None
    }

    fn load_playlist(&mut self, playlist_name: &String) {
        self.client.clear().unwrap();
        self.client.load(&playlist_name, ..).unwrap();
    }
}
