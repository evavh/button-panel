use std::fmt;

use mpdrs::error::Error;
use mpdrs::status::State::Play;
use mpdrs::Playlist;

mod db;
use db::Db;

mod mpdinterface;
use mpdinterface::MpdInterface;
use tracing::{info, instrument};

#[derive(Debug)]
enum Direction {
    Next,
    Previous,
}

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
    client: MpdInterface,
    db: Db,
    pub(crate) mode: AudioMode,
}

impl fmt::Debug for AudioController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AudioController")
            .field("mode", &self.mode)
            .finish()
    }
}

impl AudioController {
    pub(crate) fn new(ip: &str) -> Self {
        let client = MpdInterface::connect(ip).unwrap();
        let mut controller = AudioController {
            client,
            db: Db::open("database"),
            mode: AudioMode::Music,
        };
        controller.playing();
        controller
    }

    pub(crate) fn rescan(&mut self) {
        info!("Rescanning mpd library");
        self.client.rescan().unwrap();
    }

    fn playing(&mut self) -> bool {
        let playback_state = self.client.status().unwrap().state;
        playback_state == Play
    }

    fn get_playlists(&mut self) -> Vec<Playlist> {
        self.client.playlists().unwrap()
    }

    pub(crate) fn toggle_playback(&mut self) {
        info!("Toggle playback");
        self.client
            .toggle_pause()
            .expect("Something went wrong toggling playback");
    }

    pub(crate) fn rewind(&mut self) {
        info!("Rewinding by 15 seconds");

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
        self.client.play().unwrap();
        self.client.rewind(position.saturating_sub(15)).unwrap();
    }

    pub(crate) fn skip(&mut self) {
        info!("Skipping by 15 seconds");

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
        info!("Going to previous track");

        self.client.play().unwrap();
        self.client.prev().unwrap();
    }

    #[instrument]
    pub(crate) fn next(&mut self) {
        info!("Next");

        self.client.play().unwrap();

        match self.client.next() {
            Ok(_) => (),
            Err(Error::Server(server_error)) => {
                if server_error.detail != "Not playing" {
                    panic!("Unexpected ServerError: {server_error}")
                }
            }
            Err(other_error) => panic!("Unexpected error: {other_error}"),
        };
    }

    fn apply_shuffle(&mut self, playlist_name: &str) {
        if playlist_name.ends_with("_shuf") {
            self.client.random(true).unwrap();
        } else {
            let random = self.mode.settings().random;
            self.client.random(random).unwrap();
        }
    }

    #[instrument]
    fn switch_playlist(&mut self, direction: Direction) {
        let current_playlist_name =
            match self.db.fetch_playlist_name(&self.mode) {
                Some(playlist_name) => playlist_name,
                None => self.first_playlist_for_mode().unwrap(),
            };
        self.store_position(&current_playlist_name);
        self.save_playlist_if_necessary(&current_playlist_name);

        let new_playlist_name = if let Some(playlist_name) =
            self.playlist_for_mode(direction, &current_playlist_name)
        {
            playlist_name
        } else {
            current_playlist_name
        };

        info!("Switching to playlist {}", new_playlist_name);
        self.load_playlist(&new_playlist_name);
        self.db.store_playlist_name(&self.mode, &new_playlist_name);
        self.apply_shuffle(&new_playlist_name);

        let new_position = self.db.fetch_position(&new_playlist_name);
        self.load_position(new_position);
    }

    pub(crate) fn prev_playlist(&mut self) {
        self.client.play().unwrap();
        self.switch_playlist(Direction::Previous);
    }

    pub(crate) fn next_playlist(&mut self) {
        self.client.play().unwrap();
        self.switch_playlist(Direction::Next);
    }

    pub(crate) fn next_mode(&mut self) {
        self.client.play().unwrap();

        let current_playlist_name =
            match self.db.fetch_playlist_name(&self.mode) {
                Some(playlist_name) => playlist_name,
                None => self.first_playlist_for_mode().unwrap(),
            };
        self.store_position(&current_playlist_name);
        self.save_playlist_if_necessary(&current_playlist_name);

        self.mode.next();
        info!("Switching to mode {:?}", self.mode);

        let new_playlist_name = self.db.fetch_playlist_name(&self.mode);
        let new_playlist_name = if let Some(playlist_name) = new_playlist_name {
            playlist_name
        } else {
            let playlist_name = self.first_playlist_for_mode().unwrap();
            self.db.store_playlist_name(&self.mode, &playlist_name);
            playlist_name
        };
        self.load_playlist(&new_playlist_name);

        let new_position = self.db.fetch_position(&new_playlist_name);
        self.load_position(new_position);

        self.apply_settings(self.mode.settings());
        self.apply_shuffle(&new_playlist_name);
    }

    fn save_playlist_if_necessary(&mut self, playlist_name: &str) {
        if self.mode.settings().save_playlist {
            self.client.pl_remove(playlist_name).unwrap();
            self.client.save(playlist_name).unwrap();
        }
    }

    #[instrument(ret)]
    fn first_playlist_for_mode(&mut self) -> Option<String> {
        let playlists = self.get_playlists();
        for playlist in playlists {
            if playlist.name.starts_with(self.mode.to_prefix()) {
                return Some(playlist.name);
            }
        }
        None
    }

    #[instrument(ret)]
    fn playlist_for_mode(
        &mut self,
        direction: Direction,
        current_playlist_name: &String,
    ) -> Option<String> {
        let playlist_names = self.get_playlists().into_iter().map(|pl| pl.name);
        let mut playlist_names = playlist_names
            .filter(|pl| pl.starts_with(self.mode.to_prefix()))
            .collect::<Vec<_>>();

        playlist_names.sort();

        if let Direction::Previous = direction {
            playlist_names.reverse()
        }
        let mut playlist_names = playlist_names.iter().cycle().peekable();

        while *playlist_names.peek().unwrap() != current_playlist_name {
            playlist_names.next();
        }
        playlist_names.nth(1).map(|s| s.to_owned())
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
        self.db.store_position(playlist_name, position);
    }

    fn load_playlist(&mut self, playlist_name: &str) {
        self.client.clear().unwrap();
        self.client.load(playlist_name, ..).unwrap();
    }

    fn load_position(&mut self, position: Option<db::Position>) {
        if let Some(position) = position {
            self.client.queue().unwrap();
            self.seek_to(position.pos_in_pl, position.elapsed);
        } else {
            self.seek_to(0, 0);
        }
    }

    fn seek_to(&mut self, pos_in_pl: u32, elapsed: u32) {
        match self.client.seek(pos_in_pl, elapsed) {
            Ok(_) => (),
            Err(Error::Server(server_error)) => {
                if server_error.detail != "Bad song index" {
                    panic!("Unexpected ServerError: {server_error}")
                }
            }
            Err(other_error) => panic!("Unexpected error: {other_error}"),
        }
    }

    fn apply_settings(&mut self, audio_settings: Settings) {
        self.client.repeat(audio_settings.repeat).unwrap();
        self.client.random(audio_settings.random).unwrap();
        self.client.single(audio_settings.single).unwrap();
        self.client.consume(audio_settings.consume).unwrap();
    }
}
