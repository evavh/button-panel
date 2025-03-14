#![allow(clippy::missing_panics_doc)]

use core::time;
use std::io;
use std::path::Path;
use std::thread;

use async_trait::async_trait;
use bytes::BytesMut;
use color_eyre::{eyre::eyre, Help, Result};
use futures::stream::StreamExt;
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio_util::codec::{Decoder, Encoder, Framed};

use button_protocol::{Button, ButtonPress};

struct LineCodec;

impl Decoder for LineCodec {
    type Item = u8;
    type Error = io::Error;

    fn decode(
        &mut self,
        src: &mut BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        let newline = src.as_ref().iter().position(|b| *b == b'\n');
        if let Some(n) = newline {
            let line = src.split_to(n + 1);
            return Ok(Some(line[0]));
        }
        Ok(None)
    }
}

impl Encoder<String> for LineCodec {
    type Error = io::Error;

    fn encode(
        &mut self,
        _item: String,
        _dst: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[async_trait]
pub trait Panel {
    async fn recv(&mut self) -> Result<ButtonPress, &'static str>;
}

pub struct Usart {
    reader: Framed<SerialStream, LineCodec>,
}

impl Usart {
    pub fn try_connect(tty_path: &str) -> Result<Self> {
        let mut port = tokio_serial::new(tty_path, 9600).open_native_async()?;

        #[cfg(unix)]
        port.set_exclusive(false)
            .expect("Unable to set serial port exclusive to false");

        let reader = LineCodec.framed(port);

        Ok(Self { reader })
    }
}

#[async_trait]
impl Panel for Usart {
    async fn recv(&mut self) -> Result<ButtonPress, &'static str> {
        let line = self
            .reader
            .next()
            .await
            .expect("Serial disconnected")
            .unwrap();

        ButtonPress::deserialize(line)
    }
}

pub struct Mock {
    actions: Vec<ButtonPress>,
    sleep_s: u64,
}

impl Mock {
    pub fn full_test() -> Result<Self> {
        let mut actions = vec![
            ButtonPress::Short(Button::BottomMiddle), //evening light
            ButtonPress::Short(Button::TopMiddle),    //play (Music)
            ButtonPress::Long(Button::TopRight),      //next playlist (Music)
            ButtonPress::Long(Button::TopRight),      //next playlist (Music)
            ButtonPress::Long(Button::TopLeft),       //prev playlist (Music)
            ButtonPress::Long(Button::TopLeft),       //prev playlist (Music)
        ];
        actions.reverse();
        Ok(Mock { actions, sleep_s: 2 })
    }

    pub fn bottom_left_only() -> Result<Self> {
        let actions = vec![
            ButtonPress::Short(Button::BottomLeft),
        ];
        Ok(Mock { actions, sleep_s: 0 })
    }
}

#[async_trait]
impl Panel for Mock {
    async fn recv(&mut self) -> Result<ButtonPress, &'static str> {
        thread::sleep(time::Duration::from_secs(self.sleep_s));
        self.actions.pop().ok_or("No more actions in MockPanel")
    }
}

pub fn setup_udev_access() -> Result<()> {
    let path = Path::new("/etc/udev/rules.d/70-dvdva.rules");
    let rule = r###"ATTRS{idVendor}=="0483", ATTRS{idProduct}=="3748", TAG+="uaccess""###;
    if path.exists() {
        return Err(eyre!("udev file already exists"));
    }
    if sudo::check() != sudo::RunningAs::Root {
        return Err(eyre!("need to run as root to create udev rules")
            .suggestion("restart using sudo"));
    };
    std::fs::write(path, rule)?;
    Ok(())
}
