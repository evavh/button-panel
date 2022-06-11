use clap::Parser;
use color_eyre::eyre::Context;
use color_eyre::Result;

use crate::mpd::Mpd;
mod mpd;
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

    let mut mpd = Mpd::connect("192.168.1.101:6600");
    mpd.rescan();
    let mut panel = panel::UsartPanel::try_connect()
        .wrap_err("Could not connect to Panel")?;

    loop {
        let button_press = panel.recv().await.unwrap();
        use mpd::AudioMode::*;
        use protocol::{Button::*, ButtonPress::*};
        match (&mpd.mode, button_press) {
            (Music | Meditation, Short(TopLeft)) => mpd.previous(),
            (Book | Podcast, Short(TopLeft)) => mpd.rewind(),

            (Music | Meditation, Short(TopRight)) => mpd.next(),
            (Book | Podcast, Short(TopRight)) => mpd.skip(),

            (_, Short(TopMiddle)) => mpd.toggle_playback(),

            (_, Long(TopLeft)) => mpd.prev_playlist(),
            (_, Long(TopRight)) => mpd.next_playlist(),
            (_, Long(TopMiddle)) => mpd.next_mode(),
            _ => todo!("some other buttonpress"),
        }
    }
}
