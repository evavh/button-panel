use std::fmt;

use mpdrs::error::Error;
use mpdrs::status::State::Play;
use mpdrs::Playlist;

mod db;
use db::Db;

mod mpdinterface;
use mpdinterface::MpdInterface;
use tracing::{debug, info, instrument};

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

#[derive(Debug, PartialEq)]
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

    #[instrument(ret)]
    fn rewind_time(last_played: u64) -> u32 {
        const MIN_REWIND: u32 = 2;

        let since_last_played = Db::now_timestamp() - last_played;
        info!("{}s since last played", since_last_played);
        let rewind_time = (0.5 * (since_last_played as f64).sqrt())
            .round()
            .clamp(0.0, 30.0) as u32;

        if rewind_time < MIN_REWIND {
            0
        } else {
            rewind_time
        }
    }

    fn store_current_pausing(&self) {
        if let Some(current_playlist) = self.db.fetch_playlist_name(&self.mode)
        {
            self.db
                .store_last_played(&current_playlist, Db::now_timestamp())
        }
    }

    fn rewind_after_pause(&mut self) {
        use AudioMode::*;
        const TO_BEGIN_MIN: u64 = 60;

        if let Some(current_playlist) = self.db.fetch_playlist_name(&self.mode)
        {
            let last_played = self.db.fetch_last_played(&current_playlist);

            match dbg!((&self.mode, last_played)) {
                (Book | Podcast, Some(last_played)) => {
                    self.rewind_by(Self::rewind_time(last_played));
                }
                (Music | Meditation, Some(last_played)) => {
                    if Db::now_timestamp() - last_played > TO_BEGIN_MIN * 60 {
                        self.seek_in_cur(0)
                    }
                }
                (_, None) => (),
            };
        }
    }

    pub(crate) fn toggle_playback(&mut self) {
        info!("Toggle playback");
        let was_playing = self.playing();

        self.client
            .toggle_pause()
            .expect("Something went wrong toggling playback");

        if was_playing {
            self.store_current_pausing()
        } else {
            self.rewind_after_pause()
        }
    }

    fn play(&mut self) {
        if !self.playing() {
            self.toggle_playback();
        }
    }

    pub(crate) fn rewind_by(&mut self, seconds: u32) {
        self.play();
        if seconds == 0 {
            debug!("0 seconds, not rewinding");
            return;
        }
        info!("Rewinding by {} seconds", seconds);

        let position: u32 = self
            .client
            .status()
            .unwrap()
            .elapsed
            .unwrap()
            .as_secs()
            .try_into()
            .unwrap();
        self.client
            .rewind(position.saturating_sub(seconds))
            .unwrap();
    }

    pub(crate) fn skip(&mut self) {
        info!("Skipping by 15 seconds");

        self.play();
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

        self.play();
        self.client.prev().unwrap();
    }

    #[instrument]
    pub(crate) fn next(&mut self) {
        info!("Next");

        self.play();

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
        self.db
            .store_last_played(&current_playlist_name, Db::now_timestamp());

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

        self.rewind_after_pause();
    }

    pub(crate) fn prev_playlist(&mut self) {
        self.play();
        self.switch_playlist(Direction::Previous);
    }

    pub(crate) fn next_playlist(&mut self) {
        self.play();
        self.switch_playlist(Direction::Next);
    }

    pub(crate) fn next_mode(&mut self) {
        self.play();

        let current_playlist_name =
            match self.db.fetch_playlist_name(&self.mode) {
                Some(playlist_name) => playlist_name,
                None => self.first_playlist_for_mode().unwrap(),
            };
        self.store_position(&current_playlist_name);
        self.save_playlist_if_necessary(&current_playlist_name);
        self.db
            .store_last_played(&current_playlist_name, Db::now_timestamp());

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
        self.rewind_after_pause();

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

    fn seek_in_cur(&mut self, elapsed: u32) {
        if let Some(song) = self.client.currentsong().unwrap() {
            if let Some(place) = song.place {
                self.seek_to(place.pos, elapsed);
            }
        }
    }

    fn apply_settings(&mut self, audio_settings: Settings) {
        self.client.repeat(audio_settings.repeat).unwrap();
        self.client.random(audio_settings.random).unwrap();
        self.client.single(audio_settings.single).unwrap();
        self.client.consume(audio_settings.consume).unwrap();
    }
}
