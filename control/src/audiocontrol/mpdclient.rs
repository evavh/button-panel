use std::net::TcpStream;
use std::thread;

use mpdrs::error::{Error, Result};
use mpdrs::song::Range;
use mpdrs::{Playlist, Song, Status};

pub(crate) struct MpdClient {
    ip: String,
    connection: mpdrs::Client,
}

impl MpdClient {
    fn ok_or_reconnect(
        &mut self,
        function: &dyn Fn(&mut mpdrs::Client) -> Result<()>,
    ) -> Result<()> {
        match function(&mut self.connection) {
            Ok(_) => return Ok(()),
            Err(Error::Io(_)) => (),
            Err(Error::Parse(_)) => (),
            Err(error) => return Err(error),
        };

        println!("IOError or ParseError, reconnecting...");
        self.connection = mpdrs::Client::connect(&self.ip)?;
        println!("Reconnect succesful, retoggling pause");
        function(&mut self.connection)
    }

    pub(crate) fn connect(ip: &str) -> Result<Self> {
        println!("Connecting to mpd");

        let connection = mpdrs::Client::connect(ip)?;
        Ok(MpdClient {
            ip: ip.to_owned(),
            connection,
        })
    }

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

    pub(crate) fn status(&mut self) -> Result<Status> {
        self.connection.status()
    }

    pub(crate) fn playlists(&mut self) -> Result<Vec<Playlist>> {
        todo!()
    }

    pub(crate) fn toggle_pause(&mut self) -> Result<()> {
        self.ok_or_reconnect(&mpdrs::Client::toggle_pause)
    }

    pub(crate) fn play(&mut self) -> Result<()> {
        self.ok_or_reconnect(&mpdrs::Client::play)
    }

    pub(crate) fn rewind(&mut self, pos: u32) -> Result<()> {
        todo!()
    }

    pub(crate) fn prev(&mut self) -> Result<()> {
        self.ok_or_reconnect(&mpdrs::Client::prev)
    }

    pub(crate) fn next(&mut self) -> Result<()> {
        self.ok_or_reconnect(&mpdrs::Client::next)
    }

    pub(crate) fn pl_remove(&mut self, name: &str) -> Result<()> {
        todo!()
    }

    pub(crate) fn save(&mut self, name: &str) -> Result<()> {
        todo!()
    }

    pub(crate) fn clear(&mut self) -> Result<()> {
        self.ok_or_reconnect(&mpdrs::Client::next)
    }

    pub(crate) fn load<T: Into<Range>>(
        &mut self,
        name: &str,
        range: T,
    ) -> Result<()> {
        todo!()
    }

    pub(crate) fn queue(&mut self) -> Result<Vec<Song>> {
        todo!()
    }

    pub(crate) fn seek(&mut self, place: u32, pos: u32) -> Result<()> {
        todo!()
    }

    pub(crate) fn repeat(&mut self, value: bool) -> Result<()> {
        todo!()
    }

    pub(crate) fn random(&mut self, value: bool) -> Result<()> {
        todo!()
    }

    pub(crate) fn single(&mut self, value: bool) -> Result<()> {
        todo!()
    }

    pub(crate) fn consume(&mut self, value: bool) -> Result<()> {
        todo!()
    }
}
