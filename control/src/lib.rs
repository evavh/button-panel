#![allow(clippy::enum_glob_use)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]

use clap::Parser;
use color_eyre::Result;
use tokio::net::TcpListener;
use tracing::{instrument, warn};

pub mod audiocontrol;
pub mod lightcontrol;
pub mod panel;
pub mod tcp;

use crate::audiocontrol::AudioMode;

use self::panel::Panel;
use audiocontrol::AudioController;
use lightcontrol::LightController;
use protocol::ButtonPress;

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

#[instrument]
fn handle_buttonpress(
    audio: &mut AudioController,
    light: &LightController,
    button_press: ButtonPress,
) {
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

        (_, Short(BottomLeft)) => light.off(),
        (_, Long(BottomLeft)) => light.evening_on(),
        (_, Short(BottomMiddle)) => light.time_based_light(),
        (_, Long(BottomMiddle)) => light.early_evening_on(),
        (_, Short(BottomRight)) => light.override_light(),
        (_, Long(BottomRight)) => light.day_on(),
    }
}

fn handle_tcp_message(audio: &mut AudioController, message: &str) {
    match message {
        "alarm" => {
            audio.play_mode_playlist(&AudioMode::Music, "music_all_shuf")
        }
        _ => (),
    };
}

pub async fn run(mut panel: impl Panel, args: Args) -> Result<()> {
    let mut audio = AudioController::new(&args.ip, "6600");
    let light = LightController::new(&args.ip, "8081");
    audio.rescan();

    let tcp_listener = TcpListener::bind("127.0.0.1:3141").await?;

    loop {
        tokio::select! {
            button_press = panel.recv() => {
                handle_buttonpress(&mut audio, &light, button_press.unwrap())
            }
            message = tcp::wait_for_message(&tcp_listener) => {
                handle_tcp_message(&mut audio, &message)
            }
        }
    }
}

pub fn setup_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::fmt;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::from_default_env();

    let fmt_layer = fmt::layer().pretty().with_line_number(true);

    // console_subscriber::init();
    tracing_subscriber::registry()
        .with(ErrorLayer::default())
        .with(filter)
        .with(fmt_layer)
        .try_init()
        .unwrap();
}
