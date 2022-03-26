use mpdrs::client::Client;
use mpdrs::status::State::Play;

enum AudioMode {
    Music,
    Book,
    Podcast,
    Meditation,
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

    pub(crate) fn rewind() {
        todo!();
    }
}
