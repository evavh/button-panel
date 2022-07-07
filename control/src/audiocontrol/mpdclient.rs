use std::fmt;
use std::thread;

use mpdrs::error::{Error, Result};
use mpdrs::song::Range;
use mpdrs::{Playlist, Song, Status};
use tracing::{debug, instrument};

pub(crate) struct MpdClient {
    ip: String,
    connection: mpdrs::Client,
}

impl fmt::Debug for MpdClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MpdClient").field("ip", &self.ip).finish()
    }
}

macro_rules! ok_or_reconnect_no_args {
    ($name: ident, $return_type: ty) => {
        pub(crate) fn $name(&mut self) -> Result<$return_type> {
            match self.connection.$name() {
                Err(Error::Io(_)) => (),
                Err(Error::Parse(_)) => (),
                other => return other,
            };

            debug!("IOError or ParseError, reconnecting...");
            self.connection = mpdrs::Client::connect(&self.ip)?;
            self.connection.$name()
        }
    };
}
macro_rules! ok_or_reconnect_one_arg {
    ($name: ident, $arg: ident, $arg_type: ty, $return_type: ty) => {
        pub(crate) fn $name(
            &mut self,
            $arg: $arg_type,
        ) -> Result<$return_type> {
            match self.connection.$name($arg) {
                Err(Error::Io(_)) => (),
                Err(Error::Parse(_)) => (),
                other => return other,
            };

            debug!("IOError or ParseError, reconnecting...");
            self.connection = mpdrs::Client::connect(&self.ip)?;
            self.connection.$name($arg)
        }
    };
}

impl MpdClient {
    #[instrument(ret, err)]
    pub(crate) fn connect(ip: &str) -> Result<Self> {
        let connection = mpdrs::Client::connect(ip)?;
        Ok(MpdClient {
            ip: ip.to_owned(),
            connection,
        })
    }

    #[instrument(ret, err)]
    pub(crate) fn rescan(&mut self) -> Result<()> {
        use mpdrs::Idle;

        let mut watcher = mpdrs::Client::connect(&self.ip)?;
        let thread_join_handle = thread::spawn(move || {
            watcher.wait(&[mpdrs::idle::Subsystem::Update])
        });
        self.connection.rescan()?;
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
    ok_or_reconnect_no_args! {clear, ()}

    ok_or_reconnect_one_arg! {rewind, pos, u32, ()}
    ok_or_reconnect_one_arg! {pl_remove, name, &str, ()}
    ok_or_reconnect_one_arg! {save, name, &str, ()}
    ok_or_reconnect_one_arg! {repeat, value, bool, ()}
    ok_or_reconnect_one_arg! {random, value, bool, ()}
    ok_or_reconnect_one_arg! {single, value, bool, ()}
    ok_or_reconnect_one_arg! {consume, value, bool, ()}

    pub(crate) fn load<T: Into<Range> + std::marker::Copy>(
        &mut self,
        name: &str,
        range: T,
    ) -> Result<()> {
        match self.connection.load(name, range) {
            Err(Error::Io(_)) => (),
            Err(Error::Parse(_)) => (),
            other => return other,
        };

        debug!("IOError or ParseError, reconnecting...");
        self.connection = mpdrs::Client::connect(&self.ip)?;
        self.connection.load(name, range)
    }

    pub(crate) fn seek(&mut self, place: u32, pos: u32) -> Result<()> {
        match self.connection.seek(place, pos) {
            Err(Error::Io(_)) => (),
            Err(Error::Parse(_)) => (),
            other => return other,
        };

        debug!("IOError or ParseError, reconnecting...");
        self.connection = mpdrs::Client::connect(&self.ip)?;
        self.connection.seek(place, pos)
    }
}
