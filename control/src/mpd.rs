use mpdrs::client::Client;
use mpdrs::status::State::Play;

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
}

pub(crate) struct Mpd {
    client: Client,
}

impl Mpd {
    pub(crate) fn connect(ip: &str) -> Self {
        let client = Client::connect(ip).unwrap();
        let mut mpd = Mpd { client };
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
}
