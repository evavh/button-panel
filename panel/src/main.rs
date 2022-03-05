#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::{info, unwrap};
use defmt_rtt as _; // global logger
use embassy::interrupt::InterruptExt;
use embassy::time::{Duration, Timer, with_timeout};
use embassy_stm32::interrupt::OTG_FS;
use embassy_stm32::peripherals::USB_OTG_FS;
use futures::pin_mut;
use panic_probe as _; // print out panic messages

use embassy::executor::Spawner;
use embassy::io::{AsyncBufReadExt, AsyncWriteExt};
use embassy_stm32::usb_otg::{State, Usb, UsbBus, UsbOtg, UsbSerial, ReadInterface, Index0, ClassSet1};
use embassy_stm32::{interrupt, time::Hertz, Config, Peripherals};
use usb_device::device::{UsbDeviceBuilder, UsbVidPid};

use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};

// global logger
use panic_probe as _;

use core::sync::atomic::{AtomicUsize, Ordering};
defmt::timestamp! {"{=u64}", {
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        // NOTE(no-CAS) `timestamps` runs with interrupts disabled
        let n = COUNT.load(Ordering::Relaxed);
        COUNT.store(n + 1, Ordering::Relaxed);
        n as u64
    }
}

static mut EP_MEMORY: [u32; 2048] = [0; 2048];

// USB requires at least 48 MHz clock
fn config() -> Config {
    let mut config = Config::default();
    config.rcc.sys_ck = Some(Hertz(48_000_000));
    config
}

pub use defmt::*;

#[embassy::main(config = "config()")]
async fn main(_spawner: Spawner, p: Peripherals) -> ! {
    let mut rx_buffer = [0u8; 64];
    let mut tx_buffer = [0u8; 66];
    let peri = UsbOtg::new_fs(p.USB_OTG_FS, p.PA12, p.PA11);
    let usb_bus = UsbBus::new(peri, unsafe { &mut EP_MEMORY });

    let serial = UsbSerial::new(&usb_bus, &mut rx_buffer, &mut tx_buffer);

    // usb vendor id and product id for which linux kernel 
    // does not do strange things
    let device = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x0424, 0x274e))
        .manufacturer("dvdva")
        .product("desk button panel")
        .serial_number("TEST")
        .device_class(0x02) // Communication via ACM/CDC
        .build();

    let irq = interrupt::take!(OTG_FS);
    irq.set_priority(interrupt::Priority::P3);

    let mut state = State::new();
    let usb = unsafe { Usb::new(&mut state, device, serial, irq) };
    pin_mut!(usb);

    let (mut reader, mut writer) = usb.as_ref().take_serial_0();

    info!("usb initialized!");

    loop {
        info!("sending text");
        unwrap!(writer.write_all(b"hi hoi hoi hoi\r\n").await);
        unwrap!(writer.flush().await);
        Timer::after(Duration::from_millis(500)).await;

        let fut = read_byte(&mut reader);
        match with_timeout(Duration::from_millis(100), fut).await{
            Ok(b) => info!("read byte: {}", b),
            Err(_) => (),
        }
    }
}

// type Reader<'a, 'b, 'c> = ReadInterface<'a, 'b, 'c, Index0, UsbBus<UsbOtg<'a, USB_OTG_FS>>, ClassSet1<UsbBus<UsbOtg<'a, USB_OTG_FS>>, UsbSerial<'a, 'b, UsbBus<UsbOtg<'c, USB_OTG_FS>>>>, OTG_FS>;

async fn read_byte(reader: &mut (impl AsyncBufReadExt + core::marker::Unpin)) -> u8 {
    let char = unwrap!(reader.read_byte().await);
    char
}
