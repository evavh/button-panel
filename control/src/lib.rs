use clap::Parser;
use color_eyre::Result;
use tracing::warn;

mod audiocontrol;
pub mod panel;

use self::panel::Panel;
use audiocontrol::AudioController;

#[derive(Parser, Debug, Default)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(short, long)]
    pub setup: bool,
    /// path to the USB device, for example: /dev/ttyUSB0
    pub tty: String,
    /// ip:port for the mpd server
    pub ip: String,
}

pub async fn run(mut panel: impl Panel, args: Args) -> Result<()> {
    let mut audio = AudioController::new(&args.ip);
    audio.rescan();

    loop {
        let button_press = panel.recv().await.unwrap();
        use audiocontrol::AudioMode::*;
        use protocol::{Button::*, ButtonPress::*};
        match (&audio.mode, button_press) {
            (Music | Meditation, Short(TopLeft)) => audio.previous(),
            (Book | Podcast, Short(TopLeft)) => audio.rewind(),

            (Music | Meditation, Short(TopRight)) => audio.next(),
            (Book | Podcast, Short(TopRight)) => audio.skip(),

            (_, Short(TopMiddle)) => audio.toggle_playback(),

            (_, Long(TopLeft)) => audio.prev_playlist(),
            (_, Long(TopRight)) => audio.next_playlist(),
            (_, Long(TopMiddle)) => audio.next_mode(),
            _ => warn!("Unimplemented buttonpress: {:?}", button_press),
        }
    }
}

pub fn setup_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::fmt;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::from_default_env()
        .add_directive("control=info".parse().unwrap())
        .add_directive("warn".parse().unwrap());

    let fmt_layer = fmt::layer().pretty().with_line_number(true);

    // console_subscriber::init();
    tracing_subscriber::registry()
        .with(ErrorLayer::default())
        .with(filter)
        .with(fmt_layer)
        .try_init()
        .unwrap();
}
