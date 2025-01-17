use std::fmt;
use std::thread;

use mpdrs::error::{Error, Result};
use mpdrs::song::Range;
use mpdrs::{Playlist, Song, Status};
use tracing::{debug, instrument};

pub(super) struct MpdInterface {
    ip: String,
    client: mpdrs::Client,
}

impl fmt::Debug for MpdInterface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MpdClient").field("ip", &self.ip).finish()
    }
}

macro_rules! ok_or_reconnect_no_args {
    ($name: ident, $return_type: ty) => {
        pub(crate) fn $name(&mut self) -> Result<$return_type> {
            match self.client.$name() {
                Err(Error::Io(_)) => (),
                Err(Error::Parse(_)) => (),
                other => return other,
            };

            debug!("IOError or ParseError, reconnecting...");
            self.client = mpdrs::Client::connect(&self.ip)?;
            self.client.$name()
        }
    };
}
macro_rules! ok_or_reconnect_one_arg {
    ($name: ident, $arg: ident, $arg_type: ty, $return_type: ty) => {
        pub(crate) fn $name(
            &mut self,
            $arg: $arg_type,
        ) -> Result<$return_type> {
            match self.client.$name($arg) {
                Err(Error::Io(_)) => (),
                Err(Error::Parse(_)) => (),
                other => return other,
            };

            debug!("IOError or ParseError, reconnecting...");
            self.client = mpdrs::Client::connect(&self.ip)?;
            self.client.$name($arg)
        }
    };
}

impl MpdInterface {
    #[instrument(ret, err)]
    pub(crate) fn connect(ip: &str) -> Result<Self> {
        let client = mpdrs::Client::connect(ip)?;
        Ok(MpdInterface {
            ip: ip.to_owned(),
            client,
        })
    }

    #[instrument(ret, err)]
    pub(crate) fn rescan(&mut self) -> Result<()> {
        use mpdrs::Idle;

        let mut watcher = mpdrs::Client::connect(&self.ip)?;
        let thread_join_handle = thread::spawn(move || {
            watcher.wait(&[mpdrs::idle::Subsystem::Update])
        });
        self.client.rescan()?;
        thread_join_handle.join().unwrap()?;
        Ok(())
    }

    ok_or_reconnect_no_args! {toggle_pause, ()}
    ok_or_reconnect_no_args! {play, ()}
    ok_or_reconnect_no_args! {prev, ()}
    ok_or_reconnect_no_args! {next, ()}
    ok_or_reconnect_no_args! {status, Status}
    ok_or_reconnect_no_args! {playlists, Vec<Playlist>}
    ok_or_reconnect_no_args! {queue, Vec<Song>}
    ok_or_reconnect_no_args! {currentsong, Option<Song>}
    ok_or_reconnect_no_args! {clear, ()}

    ok_or_reconnect_one_arg! {rewind, pos, u32, ()}
    ok_or_reconnect_one_arg! {pl_remove, name, &str, ()}
    ok_or_reconnect_one_arg! {pl_clear, name, &str, ()}
    ok_or_reconnect_one_arg! {save, name, &str, ()}
    ok_or_reconnect_one_arg! {repeat, value, bool, ()}
    ok_or_reconnect_one_arg! {random, value, bool, ()}
    ok_or_reconnect_one_arg! {single, value, bool, ()}
    ok_or_reconnect_one_arg! {consume, value, bool, ()}
    ok_or_reconnect_one_arg! {playlist, name, &str, Vec<Song>}
    ok_or_reconnect_one_arg! {push, path, &str, u32}

    pub(crate) fn pause(&mut self) -> Result<()> {
        match self.client.pause(true) {
            Err(Error::Io(_)) => (),
            Err(Error::Parse(_)) => (),
            other => return other,
        };

        debug!("IOError or ParseError, reconnecting...");
        self.client = mpdrs::Client::connect(&self.ip)?;
        self.client.pause(true)
    }

    pub(crate) fn load<T: Into<Range> + std::marker::Copy>(
        &mut self,
        name: &str,
        range: T,
    ) -> Result<()> {
        match self.client.load(name, range) {
            Err(Error::Io(_) | Error::Parse(_)) => (),
            other => return other,
        };

        debug!("IOError or ParseError, reconnecting...");
        self.client = mpdrs::Client::connect(&self.ip)?;
        self.client.load(name, range)
    }

    pub(crate) fn pl_push(&mut self, pl_name: &str, song: &Song) -> Result<()> {
        match self.client.pl_push(pl_name, &song.file) {
            Err(Error::Io(_) | Error::Parse(_)) => (),
            other => return other,
        };

        debug!("IOError or ParseError, reconnecting...");
        self.client = mpdrs::Client::connect(&self.ip)?;
        self.client.pl_push(pl_name, &song.file)
    }

    pub(crate) fn seek(&mut self, place: u32, pos: u32) -> Result<()> {
        match self.client.seek(place, pos) {
            Err(Error::Io(_) | Error::Parse(_)) => (),
            other => return other,
        };

        debug!("IOError or ParseError, reconnecting...");
        self.client = mpdrs::Client::connect(&self.ip)?;
        self.client.seek(place, pos)
    }

    pub(crate) fn prioid(&mut self, id: u32, prio: u8) -> Result<()> {
        match self.client.prioid(id, prio) {
            Err(Error::Io(_) | Error::Parse(_)) => (),
            other => return other,
        };

        debug!("IOError or ParseError, reconnecting...");
        self.client = mpdrs::Client::connect(&self.ip)?;
        self.client.prioid(id, prio)
    }

    pub(crate) fn playlist_exists(&mut self, playlist_name: &str) -> bool {
        self.playlist(playlist_name).is_ok()
    }
}
