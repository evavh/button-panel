use clap::Parser;
use color_eyre::eyre::Context;
use color_eyre::Result;

use crate::audiocontrol::AudioController;
mod audiocontrol;
mod panel;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    setup: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    if args.setup {
        panel::setup_udev_access().wrap_err("Could not set up udev rules")?;
        return Ok(());
    }

    let mut audio = AudioController::connect("192.168.1.101:6600");
    audio.rescan();
    let mut panel = panel::MockPanel::try_connect()
        .wrap_err("Could not connect to Panel")?;

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
            _ => todo!("some other buttonpress"),
        }
    }
}
