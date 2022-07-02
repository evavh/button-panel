#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt::*;
use defmt_rtt as _;
use embassy::time::Instant;
use futures::join;
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
use embassy::time::{Duration, Timer};
use embassy_stm32::dma::NoDma;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Pin, Pull};
use embassy_stm32::Peripherals;
use embassy_stm32::{usart, Unborrow};

use protocol::{Button, ButtonPress};

#[embassy::main]
async fn main(_spawner: Spawner, p: Peripherals) -> ! {
    info!("Press a button...");

    let mut config = usart::Config::default();
    config.baudrate = 9600;

    let mut usart =
        usart::Uart::new(p.USART1, p.PA10, p.PA9, NoDma, NoDma, config);

    let mut buf = [0, '\n' as u8];
    loop {
        let buttonpress = ButtonPress::Short(Button::TopMiddle);
        buf[0] = buttonpress.serialize();
        unwrap!(usart.blocking_write(&buf));
        Timer::after(Duration::from_millis(300)).await;
    }

    /* let a = wait_for_button(p.PB12, p.EXTI12, Button::TopLeft);
    let b = wait_for_button(p.PB13, p.EXTI13, Button::TopMiddle);
    let c = wait_for_button(p.PB1, p.EXTI1, Button::TopRight);
    let d = wait_for_button(p.PC15, p.EXTI15, Button::BottomLeft);
    let e = wait_for_button(p.PB0, p.EXTI0, Button::BottomMiddle);
    let f = wait_for_button(p.PC14, p.EXTI14, Button::BottomRight);
    join!(a, b, c, d, e, f); */
}

async fn wait_for_button<'d, T: Pin>(
    pin: impl Unborrow<Target = T> + 'd,
    ch: impl Unborrow<Target = T::ExtiChannel> + 'd,
    name: protocol::Button,
) {
    let button = Input::new(pin, Pull::Down);
    let mut button = ExtiInput::new(button, ch);

    loop {
        button.wait_for_high().await;
        let press_time = Instant::now();
        button.wait_for_low().await;

        let button_press = match press_time.elapsed().as_millis() {
            0..=50 => continue,
            51..=800 => ButtonPress::Short(name),
            801..=2000 => ButtonPress::Long(name),
            _ => continue,
        };
        info!("Press: {}", button_press)
    }
}
