#![allow(clippy::enum_glob_use)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]

use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use clap::Parser;
use data_server::api::data_source::reconnecting::Client;
use tokio::{net::TcpListener, sync::Mutex};

pub mod audiocontrol;
pub mod panel;
pub mod tcp;

use crate::audiocontrol::AudioMode;

use self::panel::Panel;
use audiocontrol::AudioController;
use button_protocol::ButtonPress;

const ALARM_DELAY_MINS: u64 = 7;
const ALARM_SOUND_PATH: &str = "relaxing-guitar-loop-v5.m4a";

const DATA_SERVER_IP: &str = "192.168.1.43";
const DATA_SERVER_PORT: u16 = 0;

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

async fn handle_buttonpress(
    audio: &mut AudioController,
    data_server: &Client,
    button_press: ButtonPress,
) {
    use audiocontrol::AudioMode::*;
    use button_protocol::{Button::*, ButtonPress::*};

    match (&audio.mode, button_press) {
        (Music | Meditation, Short(TopLeft)) => audio.previous(),
        (Book | Podcast, Short(TopLeft)) => audio.rewind(),

        (Music | Meditation, Short(TopRight)) => audio.next(),
        (Book | Podcast, Short(TopRight)) => audio.skip(),

        (_, Short(TopMiddle)) => audio.toggle_playback(),

        (_, Long(TopLeft)) => audio.prev_playlist(),
        (_, Long(TopRight)) => audio.next_playlist(),
        (_, Long(TopMiddle)) => audio.next_mode(),

        (_, _b) => todo!(),
        // (_, b) => data_server.send_reading(b.into()).await,
    }
}

async fn handle_tcp_message(
    audio_mutex: &Mutex<AudioController>,
    message: &str,
) {
    match message {
        "alarm" => {
            let mut audio = audio_mutex.lock().await;
            audio.play_mode_playlist(&AudioMode::Music, "music_all_shuf");

            drop(audio);
            tokio::time::sleep(Duration::from_secs(60 * ALARM_DELAY_MINS))
                .await;

            let mut audio = audio_mutex.lock().await;
            audio.insert_next(ALARM_SOUND_PATH);
        }
        _ => (),
    };
}

pub async fn run(panel: impl Panel + Send + 'static, args: Args) -> ! {
    let audio = Arc::new(Mutex::new(AudioController::new(&args.ip, "6600")));
    audio.lock().await.rescan();

    let addr = SocketAddr::new(
        IpAddr::from_str(DATA_SERVER_IP).expect("Valid const"),
        DATA_SERVER_PORT,
    );
    let data_server = Client::new(addr, Vec::new());

    let tcp_listener = TcpListener::bind("127.0.0.1:3141").await.unwrap();

    let buttons = buttonpress_task(panel, audio.clone(), data_server);
    let tcp = tcp_task(tcp_listener, audio);
    tokio::task::spawn(buttons);
    tokio::task::spawn(tcp);

    let () = std::future::pending().await;
    unreachable!();
}

async fn tcp_task(
    tcp_listener: TcpListener,
    audio: Arc<Mutex<AudioController>>,
) -> ! {
    loop {
        let message = tcp::wait_for_message(&tcp_listener).await;
        handle_tcp_message(&audio, &message).await;
    }
}

async fn buttonpress_task(
    mut panel: impl Panel,
    audio: Arc<Mutex<AudioController>>,
    data_server: Client,
) -> ! {
    loop {
        let button_press = panel.recv().await;
        let mut audio = audio.lock().await;
        handle_buttonpress(&mut audio, &data_server, button_press.unwrap())
            .await;
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
