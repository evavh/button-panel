use std::{fmt, thread::sleep, time::Duration};

use chrono::{offset::Local, NaiveTime};
use reqwest::{Client, Error, Response};
use tracing::{info, instrument, warn};

struct TimeFrame {
    start: NaiveTime,
    end: NaiveTime,
}

impl TimeFrame {
    pub(crate) fn new(
        start_hour: u32,
        start_min: u32,
        end_hour: u32,
        end_min: u32,
    ) -> Self {
        let start = NaiveTime::from_hms(start_hour, start_min, 0);
        let end = NaiveTime::from_hms(end_hour, end_min, 0);
        TimeFrame { start, end }
    }

    pub(crate) fn contains(&self, time: NaiveTime) -> bool {
        // timeframe goes overnight, ie 22:00 - 8:00
        if self.end < self.start {
            self.start < time || time < self.end
        } else {
            self.start < time && time < self.end
        }
    }
}

struct Setting {
    time_frame: TimeFrame,
    command: &'static str,
    override_command: &'static str,
}

impl Setting {
    pub(crate) fn new(
        time_frame: TimeFrame,
        command: &'static str,
        override_command: &'static str,
    ) -> Self {
        Self {
            time_frame,
            command,
            override_command,
        }
    }
}

pub struct LightController {
    ip: String,
    port: String,
    client: reqwest::Client,
    time_settings: Vec<Setting>,
}

impl fmt::Debug for LightController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LightController")
            .field("ip", &self.ip)
            .field("port", &self.port)
            .finish()
    }
}

mod command {
    pub const OFF: &str = "lights_off";
    pub const NIGHT: &str = "night_light_on";
    pub const EVENING: &str = "evening_light_on";
    pub const EARLY_EVENING: &str = "early_evening_light_on";
    pub const DAY: &str = "soft_light_on";
}

impl LightController {
    pub fn new(ip: &str, port: &str) -> Self {
        let client = Client::new();

        let time_settings = vec![
            Setting::new(
                TimeFrame::new(8, 30, 17, 00),
                command::DAY,
                command::EARLY_EVENING,
            ),
            Setting::new(
                TimeFrame::new(17, 0, 21, 30),
                command::EARLY_EVENING,
                command::DAY,
            ),
            Setting::new(
                TimeFrame::new(21, 30, 22, 0),
                command::EVENING,
                command::EARLY_EVENING,
            ),
            Setting::new(
                TimeFrame::new(22, 0, 8, 30),
                command::NIGHT,
                command::EVENING,
            ),
        ];

        Self {
            ip: ip.to_owned(),
            port: port.to_owned(),
            client,
            time_settings,
        }
    }

    fn send_command(&self, command: &str) -> Result<Response, Error> {
        let res = self
            .client
            .post(format!(
                "http://{}:{}/command/{}",
                self.ip, self.port, command
            ))
            .body("")
            .send();
        futures::executor::block_on(res)
    }

    fn send_command_triplex(&self, command: &str) -> Result<Response, Error> {
        info!("Sending light command {command} three times");
        self.send_command(command)?;
        sleep(Duration::from_millis(100));
        self.send_command(command)?;
        sleep(Duration::from_millis(100));
        self.send_command(command)
    }

    pub fn off(&self) {
        self.send_command_triplex(command::OFF).unwrap();
    }

    pub fn night_on(&self) {
        self.send_command_triplex(command::NIGHT).unwrap();
    }

    pub fn evening_on(&self) {
        self.send_command_triplex(command::EVENING).unwrap();
    }

    pub fn early_evening_on(&self) {
        self.send_command_triplex(command::EARLY_EVENING).unwrap();
    }

    pub fn day_on(&self) {
        self.send_command_triplex(command::DAY).unwrap();
    }

    #[instrument(skip(command_getter))]
    fn apply_time_setting(
        &self,
        command_getter: impl FnOnce(&Setting) -> &'static str,
        warn_type: &'static str,
    ) {
        let now = Local::now().time();

        if let Some(command) = self
            .time_settings
            .iter()
            .find(|setting| setting.time_frame.contains(now))
            .map(command_getter)
        {
            self.send_command_triplex(command).unwrap();
        } else {
            warn!("Current time {now} not found in {warn_type} settings for lights");
        }
    }

    pub fn time_based_light(&self) {
        self.apply_time_setting(|setting| &setting.command, "time");
    }

    pub fn override_light(&self) {
        self.apply_time_setting(
            |setting| &setting.override_command,
            "override",
        );
    }
}
