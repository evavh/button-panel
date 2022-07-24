use std::fmt;

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
        Self { ip: ip.to_owned(), port: port.to_owned(), client }
    }

    fn send_command(&self, _command: String) -> Result<Response, Error> {
        let res = self
            .client
            .post("{self.ip}:{self.port}/command/{_command}")
            .body("")
            .send();
        futures::executor::block_on(res)
    }

    pub fn off(&self) {
        self.send_command("lights_off".to_string()).unwrap();
    }

    pub fn night_on(&self) {
        self.send_command("night_light_on".to_string()).unwrap();
    }

    pub fn evening_on(&self) {
        self.send_command("evening_light_on".to_string()).unwrap();
    }

    pub fn early_evening_on(&self) {
        self.send_command("early_evening_light_on".to_string()).unwrap();
    }

    pub fn day_on(&self) {
        self.send_command("soft_light_on".to_string()).unwrap();
    }
}
