use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Instant;

use clap::Parser;
use color_eyre::eyre::Context;
use color_eyre::{eyre::eyre, Help, Result};

fn setup_udev_access() -> Result<()> {
    let path = Path::new("/etc/udev/rules.d/70-dvdva.rules");
    let rule = r###"ATTRS{idVendor}=="0483", ATTRS{idProduct}=="3748", TAG+="uaccess""###;
    if path.exists() {
        return Err(eyre!("udev file already exists"));
    }
    if sudo::check() != sudo::RunningAs::Root {
        return Err(
            eyre!("need to run as root to create udev rules").suggestion("restart using sudo")
        );
    };
    std::fs::write(path, rule)?;
    Ok(())
}

fn get_tty_path() -> Result<PathBuf> {
    const MANUFACTURER: &str = "dvdva";
    const PRODUCT: &str = "desk button panel";
    let prefix = format!(
        "usb-{}_{}_",
        MANUFACTURER.replace(" ", "_"),
        PRODUCT.replace(" ", "_")
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
                return Err(eyre!("Multiple matching ttys: {existing:?} and {path:?}"))
            }
            _ => continue,
        }
    }
    let device = device.ok_or_else(|| {
        eyre!("No device found for '{MANUFACTURER}, {PRODUCT}'")
            .suggestion("Connect the button panel")
    })?;

    std::fs::canonicalize(device).wrap_err("Could not resolve tty path")
}

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    setup: bool,
}

struct Panel {
    file: File,
}

impl Panel {
    fn try_connect() -> Result<Self> {
        let path = get_tty_path()?;
        let file = File::open(path).wrap_err("Error opening connection")?;
        Ok(Panel { file })
    }

    fn recv(&mut self) -> Result<String> {
        let mut buf = [0u8; 29];
        self.file
            .read_exact(&mut buf)
            .wrap_err("Recieved invalid message")
            .with_note(|| "Is the panel still connected?")?;
        let bytes = &buf[..buf.len()-1];
        let res = String::from_utf8(bytes.to_vec())?;
        Ok(res)
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    if args.setup {
        setup_udev_access().wrap_err("Could not set up udev rules")?;
        return Ok(());
    }

    let mut panel = Panel::try_connect().wrap_err("Could not connect to Panel")?;

    let start = Instant::now();
    for _ in 0..100_000 {
        let msg = panel.recv()?;
    }
    dbg!(start.elapsed() / 100_000);

    Ok(())
}
