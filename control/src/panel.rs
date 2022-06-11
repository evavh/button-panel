use core::time;
use std::io;
use std::path::Path;
use std::thread;

use bytes::BytesMut;
use color_eyre::eyre::WrapErr;
use color_eyre::{eyre::eyre, Help, Result};
use futures::stream::StreamExt;
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tokio_util::codec::{Decoder, Encoder, Framed};

use protocol::{Button, ButtonPress};

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
            return Ok(Some(line[0]))
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

pub(crate) struct UsartPanel {
    reader: Framed<SerialStream, LineCodec>,
}

impl UsartPanel {
    pub(crate) fn try_connect() -> Result<Self> {
        let tty_path = "/dev/ttyUSB0";
        let mut port = tokio_serial::new(tty_path, 9600).open_native_async()?;

        #[cfg(unix)]
        port.set_exclusive(false)
            .expect("Unable to set serial port exclusive to false");

        let reader = LineCodec.framed(port);

        Ok(Self { reader })
    }

    pub(crate) async fn recv(&mut self) -> Result<ButtonPress, &'static str> {
        let line = self
            .reader
            .next()
            .await
            .expect("Serial disconnected")
            .unwrap();

        ButtonPress::deserialize(dbg!(line))
    }
}

pub(crate) struct MockPanel {
    actions: Vec<ButtonPress>,
}

impl MockPanel {
    pub(crate) fn try_connect() -> Result<Self> {
        let mut actions = vec![
            ButtonPress::Short(Button::TopMiddle), //play (Music)
            ButtonPress::Short(Button::TopRight),  //next (Music)
            ButtonPress::Long(Button::TopRight),   //next playlist (Music)
            ButtonPress::Long(Button::TopMiddle),  //Music to Book
            ButtonPress::Short(Button::TopRight),  //next (Book)
            ButtonPress::Long(Button::TopMiddle),  //Book to Podcast
            ButtonPress::Short(Button::TopRight),  //next (Podcast)
            ButtonPress::Long(Button::TopRight),   //next playlist (Podcast)
            ButtonPress::Long(Button::TopMiddle),  //Podcast to Meditation
            ButtonPress::Short(Button::TopRight),  //next (Meditation)
            ButtonPress::Long(Button::TopMiddle),  //Meditation to Music
        ];
        actions.reverse();
        Ok(MockPanel { actions })
    }

    pub(crate) async fn recv(&mut self) -> Option<ButtonPress> {
        thread::sleep(time::Duration::from_secs(2));
        self.actions.pop()
    }
}

pub(crate) fn setup_udev_access() -> Result<()> {
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
