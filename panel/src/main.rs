#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![macro_use]

use defmt::*;
use defmt_rtt as _;
use futures::{join, select_biased};
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

use embassy::executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Pin, Pull};
use embassy_stm32::Peripherals;

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) -> ! {
    info!("Hello World!");

    // let top_middle = Input::new(p.PB13, Pull::Down);
    // let top_middle = ExtiInput::new(top_middle, p.EXTI13);
    // let bottom_middle = Input::new(p.PB0, Pull::Down);
    // let bottom_middle = ExtiInput::new(bottom_middle, p.EXTI0);

    info!("Press a button...");

    let a = wait_for_button(p.PB13, p.EXTI13, "top middle");
    let b = wait_for_button(p.PB0, p.EXTI0, "bottom middle");
    join!(a, b);
}

async fn wait_for_button<'d, T: Pin>(
    pin: impl Unborrow<Target = T> + 'd,
    ch: impl Unborrow<Target = T::ExtiChannel> + 'd,
    name: &str,
) {
    let button = Input::new(pin, Pull::Down);
    let button = ExtiInput::new(button, ch);

    loop {
        button.wait_for_rising_edge().await;
        info!("Pressed {}", name);
        button.wait_for_falling_edge().await;
        info!("Released {}", name);
    }
}
