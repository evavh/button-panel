use std::time::Duration;

use color_eyre::eyre::Context;
use color_eyre::{eyre::eyre, Help, Result};
use rusb::{Device, DeviceDescriptor, UsbContext};

fn setup_udev_access() {}

fn get_tty() -> Result<()> {
    for device in rusb::devices().unwrap().iter() {
        let desc = device.device_descriptor().unwrap();
        if desc.vendor_id() != 0x0424 && desc.product_id() != 0x274e {
            continue;
        }

        // let timeout = Duration::from_secs(1);

        let handle = match device.open() {
            Err(rusb::Error::Access) => {
                return Err(eyre!("Could not open usb device, is udev rule present?")
                    .suggestion("run control with --setup to install udev rule")
                    .suggestion("run control as superuser"))
            }
            Err(e) => return Err(e).wrap_err("Could not open usb device, is udev rule present?"),
            Ok(h) => h,
        };

        // let lang = handle.read_languages(timeout)?;
        let manufacturer = handle.read_manufacturer_string_ascii(&desc)?;
        dbg!(manufacturer);
    }
    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;

    get_tty().unwrap();
    Ok(())
}
