use std::thread;

use mpdrs::client::Client;
use mpdrs::status::State::Play;
use mpdrs::{Idle, Playlist};

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

    pub(crate) fn rescan(&mut self) {
        let mut watcher = mpdrs::Client::connect(&self.ip).unwrap();
        let thread_join_handle = thread::spawn(move || {
            watcher.wait(&[mpdrs::idle::Subsystem::Update]).unwrap();
        });
        self.client.rescan().unwrap();
        thread_join_handle.join().unwrap();
    }

    fn playing(&mut self) -> bool {
        let playback_state = self.client.status().unwrap().state;
        playback_state == Play
    }

    fn get_playlists(&mut self) -> Vec<Playlist> {
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
        self.client.play().unwrap();
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
        self.client.play().unwrap();
        self.client.rewind(position + 15).unwrap();
    }

    pub(crate) fn previous(&mut self) {
        self.client.play().unwrap();
        self.client.prev().unwrap();
    }

    pub(crate) fn next(&mut self) {
        self.client.play().unwrap();
        self.client.next().unwrap();
    }

    pub(crate) fn prev_playlist(&self) {
        todo!()
    }

    pub(crate) fn next_playlist(&self) {
        todo!()
    }

    pub(crate) fn next_mode(&mut self) {
        self.store_position();
        self.mode.next();

        let new_playlist_name = self.database.fetch_playlist_name(&self.mode);
        let new_playlist_name = if let Some(playlist_name) = new_playlist_name {
            playlist_name
        } else {
            let playlist_name = self.first_playlist_for_mode().unwrap();
            self.database
                .store_playlist_name(&self.mode, &playlist_name);
            playlist_name
        };
        self.load_playlist(&new_playlist_name);

        let new_position = self.database.fetch_position(&new_playlist_name);
        if let Some(new_position) = new_position {
            self.load_position(new_position);
        } else {
            self.seek_to_beginning();
        }

        dbg!(&self.mode);
    }

    fn first_playlist_for_mode(&mut self) -> Option<String> {
        let playlists = self.get_playlists();
        for playlist in playlists {
            if playlist.name.starts_with(self.mode.to_prefix()) {
                return dbg!(Some(playlist.name));
            }
        }
        None
    }
    fn store_position(&mut self) {
        let position = db::Position {
            pos_in_pl: self.client.status().unwrap().song.unwrap().pos,
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

    fn load_playlist(&mut self, playlist_name: &str) {
        self.client.clear().unwrap();
        self.client.load(playlist_name, ..).unwrap();
    }

    fn load_position(&mut self, position: db::Position) {
        self.client.queue().unwrap();
        self.client
            .seek(position.pos_in_pl, position.elapsed)
            .unwrap();
    }

    fn seek_to_beginning(&mut self) {
        self.client.seek(0, 0).unwrap();
    }
}
