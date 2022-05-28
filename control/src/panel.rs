use core::time;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::thread;

use color_eyre::eyre::WrapErr;
use color_eyre::{eyre::eyre, Help, Result};

use protocol::{Button, ButtonPress};

struct _Panel {
    file: File,
}

impl _Panel {
    fn _get_tty_path() -> Result<PathBuf> {
        const _MANUFACTURER: &str = "dvdva";
        const _PRODUCT: &str = "desk button panel";
        let prefix = format!(
            "usb-{}_{}_",
            _MANUFACTURER.replace(' ', "_"),
            _PRODUCT.replace(' ', "_")
        );

        let mut device = None;
        for res in std::fs::read_dir("/dev/serial/by-id")
            .wrap_err("no usb serial devices present")
            .suggestion("Connect the button panel")?
        {
            let path = res?.path();
            let name = path.file_name().unwrap().to_str().unwrap();
            match (&mut device, name.starts_with(&prefix)) {
                (None, true) => device = Some(path),
                (Some(existing), true) => {
                    return Err(eyre!(
                        "Multiple matching ttys: {existing:?} and {path:?}"
                    ))
                }
                _ => continue,
            }
        }
        let device = device.ok_or_else(|| {
            eyre!("No device found for '{_MANUFACTURER}, {_PRODUCT}'")
                .suggestion("Connect the button panel")
        })?;

        std::fs::canonicalize(device).wrap_err("Could not resolve tty path")
    }

    fn _try_connect() -> Result<Self> {
        let path = _Panel::_get_tty_path()?;
        let file = File::open(path).wrap_err("Error opening connection")?;
        Ok(_Panel { file })
    }

    fn _recv(&mut self) -> Result<String> {
        let mut buf = [0u8; 29];
        self.file
            .read_exact(&mut buf)
            .wrap_err("Recieved invalid message")
            .with_note(|| "Is the panel still connected?")?;
        let _bytes = &buf[..buf.len() - 1];
        todo!("Deserialize to ButtonPress enum")
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

    pub(crate) fn recv(&mut self) -> Option<ButtonPress> {
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
