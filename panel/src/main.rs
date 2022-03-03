#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use cortex_m_rt::entry;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};

use defmt_rtt as _; // global logger
use panic_probe as _;

mod usb;

use core::sync::atomic::{AtomicUsize, Ordering};
defmt::timestamp! {"{=u64}", {
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        // NOTE(no-CAS) `timestamps` runs with interrupts disabled
        let n = COUNT.load(Ordering::Relaxed);
        COUNT.store(n + 1, Ordering::Relaxed);
        n as u64
    }
}

pub use defmt::*;

#[entry]
fn main() -> ! {
    info!("Hello World!");

    let p = embassy_stm32::init(Default::default());

    let button = Input::new(p.PC13, Pull::Down);
    let mut led1 = Output::new(p.PB0, Level::High, Speed::Low);
    let _led2 = Output::new(p.PB7, Level::High, Speed::Low);
    let mut led3 = Output::new(p.PB14, Level::High, Speed::Low);

    loop {
        if button.is_high() {
            info!("high");
            led1.set_high();
            led3.set_low();
        } else {
            info!("low");
            led1.set_low();
            led3.set_high();
        }
    }
}
