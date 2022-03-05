use std::path::{Path, PathBuf};

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

// fn get_tty() -> Result<()> {
// use std::time::Duration;
// use rusb::{Device, DeviceDescriptor, UsbContext};
//     const MANUFACTURER: &str = "dvdva";
//     const PRODUCT: &str = "desk button panel";

//     for device in rusb::devices().unwrap().iter() {
//         let desc = device.device_descriptor().unwrap();
//         if desc.vendor_id() != 0x0424 && desc.product_id() != 0x274e {
//             continue;
//         }

//         let handle = match device.open() {
//             Err(rusb::Error::Access) => {
//                 return Err(eyre!("Could not open usb device, is udev rule present?")
//                     .suggestion("run control with --setup to install udev rule")
//                     .suggestion("run control as superuser"))
//             }
//             Err(e) => return Err(e).wrap_err("Could not open usb device, is udev rule present?"),
//             Ok(h) => h,
//         };

//         let timeout = Duration::from_secs(1);
//         let lang = handle.read_languages(timeout)?.pop().unwrap();

//         let manufacturer = handle.read_manufacturer_string(lang, &desc, timeout)?;
//         let product = handle.read_product_string(lang, &desc, timeout)?;

//         if manufacturer == MANUFACTURER && product == PRODUCT {

//         }
//     }
//     Err(eyre!("Device not {PRODUCT} found"))
// }

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
        eyre!("No device found for '{MANUFACTURER}, {PRODUCT}'").suggestion("Connect the button panel")
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

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    if args.setup {
        setup_udev_access().wrap_err("Could not set up udev rules")?;
        return Ok(());
    }

    let path = get_tty_path()?;
    dbg!(path);
    Ok(())
}
