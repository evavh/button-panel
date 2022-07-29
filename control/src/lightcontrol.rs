use std::{fmt, thread::sleep, time::Duration};

use reqwest::{Client, Error, Response};

pub struct LightController {
    ip: String,
    port: String,
    client: reqwest::Client,
}

impl fmt::Debug for LightController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LightController")
            .field("ip", &self.ip)
            .field("port", &self.port)
            .finish()
    }
}

impl LightController {
    pub fn new(ip: &str, port: &str) -> Self {
        let client = Client::new();
        Self {
            ip: ip.to_owned(),
            port: port.to_owned(),
            client,
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
        self.send_command(command)?;
        sleep(Duration::from_millis(100));
        self.send_command(command)?;
        sleep(Duration::from_millis(100));
        self.send_command(command)
    }

    pub fn off(&self) {
        self.send_command_triplex("lights_off").unwrap();
    }

    pub fn night_on(&self) {
        self.send_command_triplex("night_light_on").unwrap();
    }

    pub fn evening_on(&self) {
        self.send_command_triplex("evening_light_on").unwrap();
    }

    pub fn early_evening_on(&self) {
        self.send_command_triplex("early_evening_light_on").unwrap();
    }

    pub fn day_on(&self) {
        self.send_command_triplex("soft_light_on").unwrap();
    }
}
