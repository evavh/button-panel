use std::thread;

use mpdrs::client::Client;
use mpdrs::status::State::Play;
use mpdrs::{Idle, Playlist};

mod db;
use db::Db;

#[derive(Default)]
struct Settings {
    repeat: bool,
    random: bool,
    single: bool,
    consume: bool,

    save_playlist: bool,
}

#[derive(Debug)]
pub enum AudioMode {
    Music,
    Book,
    Podcast,
    Meditation,
}

impl AudioMode {
    fn next(&mut self) {
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

    fn settings(&self) -> Settings {
        use AudioMode::*;
        match self {
            Music => Settings::default(),
            Book => Settings {
                consume: true,
                save_playlist: true,
                ..Settings::default()
            },
            Podcast => Settings {
                consume: true,
                save_playlist: true,
                ..Settings::default()
            },
            Meditation => Settings {
                single: true,
                ..Settings::default()
            },
        }
    }
}

pub(crate) struct AudioController {
    client: Client,
    ip: String,
    database: Db,
    pub(crate) mode: AudioMode,
}

impl AudioController {
    pub(crate) fn connect(ip: &str) -> Self {
        let client = Client::connect(ip).unwrap();
        let mut controller = AudioController {
            client,
            ip: ip.to_owned(),
            database: Db::open(),
            mode: AudioMode::Music,
        };
        controller.playing();
        controller
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
        self.client.play().unwrap();
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

    pub(crate) fn next_playlist(&mut self) {
        let current_playlist_name =
            self.database.fetch_playlist_name(&self.mode).unwrap();
        self.store_position(&current_playlist_name);
        self.save_playlist_if_necessary(&current_playlist_name);

        let new_playlist_name = if let Some(playlist_name) =
            self.next_playlist_for_mode(&current_playlist_name)
        {
            playlist_name
        } else {
            current_playlist_name
        };

        println!("Switching to playlist {}", new_playlist_name);
        self.load_playlist(&new_playlist_name);
        self.database
            .store_playlist_name(&self.mode, &new_playlist_name);

        let new_position = self.database.fetch_position(&new_playlist_name);
        self.load_position(new_position);
    }

    pub(crate) fn next_mode(&mut self) {
        let current_playlist_name =
            self.database.fetch_playlist_name(&self.mode).unwrap();
        self.store_position(&current_playlist_name);
        self.save_playlist_if_necessary(&current_playlist_name);

        self.mode.next();
        println!("Switched to mode {:?}", self.mode);

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
        self.load_position(new_position);

        self.apply_settings(self.mode.settings());

    }

    fn save_playlist_if_necessary(&mut self, playlist_name: &str) {
        if self.mode.settings().save_playlist {
            self.client.pl_remove(&playlist_name).unwrap();
            self.client.save(&playlist_name).unwrap();
        }
    }

    fn first_playlist_for_mode(&mut self) -> Option<String> {
        let playlists = self.get_playlists();
        for playlist in playlists {
            if playlist.name.starts_with(self.mode.to_prefix()) {
                return Some(playlist.name);
            }
        }
        None
    }

    fn next_playlist_for_mode(
        &mut self,
        current_playlist_name: &String,
    ) -> Option<String> {
        let playlist_names = self.get_playlists().into_iter().map(|pl| pl.name);
        let mut playlist_names = playlist_names
            .filter(|pl| pl.starts_with(self.mode.to_prefix()))
            .collect::<Vec<_>>();

        playlist_names.sort();

        let mut playlist_names = playlist_names.iter().cycle().peekable();

        while *playlist_names.peek().unwrap() != current_playlist_name {
            playlist_names.next();
        }
        playlist_names.skip(1).next().map(|s| s.to_owned())
    }

    fn store_position(&mut self, playlist_name: &str) {
        let pos_in_pl = if let Some(song) = self.client.status().unwrap().song {
            song.pos
        } else {
            0
        };

        let elapsed =
            if let Some(elapsed) = self.client.status().unwrap().elapsed {
                elapsed.as_secs().try_into().unwrap()
            } else {
                0
            };

        let position = db::Position { pos_in_pl, elapsed };
        self.database.store_position(playlist_name, position);
    }

    fn load_playlist(&mut self, playlist_name: &str) {
        self.client.clear().unwrap();
        self.client.load(playlist_name, ..).unwrap();
    }

    fn load_position(&mut self, position: Option<db::Position>) {
        if let Some(position) = position {
            self.client.queue().unwrap();
            self.client
                .seek(position.pos_in_pl, position.elapsed)
                .unwrap();
        } else {
            self.seek_to_beginning();
        }
    }

    fn seek_to_beginning(&mut self) {
        self.client.seek(0, 0).unwrap();
    }

    fn apply_settings(&mut self, audio_settings: Settings) {
        self.client.repeat(audio_settings.repeat).unwrap();
        self.client.random(audio_settings.random).unwrap();
        self.client.single(audio_settings.single).unwrap();
        self.client.consume(audio_settings.consume).unwrap();
    }
}