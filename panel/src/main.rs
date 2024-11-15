#![no_std]
#![no_main]

use defmt::*;
use defmt_rtt as _;
use embassy_stm32::peripherals::USART1;
use embassy_stm32::usart::Uart;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Instant;
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

use embassy_executor::Spawner;
use embassy_stm32::dma::NoDma;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Pin, Pull};
use embassy_stm32::usart;
use embassy_stm32::{Peripheral, Peripherals};

use button_protocol::{Button, ButtonPress};

type UsartMutex<'a> = Mutex<NoopRawMutex, Uart<'a, USART1>>;

// #[embassy_executor::main]
// async fn main(_spawner: Spawner, p: Peripherals) {
//     info!("Press a button...");
//
//     let mut config = usart::Config::default();
//     config.baudrate = 9600;
//
//     let usart =
//         Uart::new(p.USART1, p.PA10, p.PA9, _, NoDma, NoDma, config).unwrap();
//     let usart: UsartMutex = Mutex::new(usart);
//
//     let a = wait_for_button(&usart, p.PB12, p.EXTI12, Button::TopLeft);
//     let b = wait_for_button(&usart, p.PB13, p.EXTI13, Button::TopMiddle);
//     let c = wait_for_button(&usart, p.PB1, p.EXTI1, Button::TopRight);
//     let d = wait_for_button(&usart, p.PC15, p.EXTI15, Button::BottomLeft);
//     let e = wait_for_button(&usart, p.PB0, p.EXTI0, Button::BottomMiddle);
//     let f = wait_for_button(&usart, p.PC14, p.EXTI14, Button::BottomRight);
//     join!(a, b, c, d, e, f);
// }
//
// async fn wait_for_button<'d, T: Pin>(
//     usart: &UsartMutex<'_>,
//     pin: impl Peripheral<P = T> + 'd,
//     ch: impl Peripheral<P = T::ExtiChannel> + 'd,
//     name: button_protocol::Button,
// ) {
//     let button = Input::new(pin, Pull::Down);
//     let mut button = ExtiInput::new(button, ch);
//
//     loop {
//         button.wait_for_high().await;
//         let press_time = Instant::now();
//         button.wait_for_low().await;
//
//         let press_millis = press_time.elapsed().as_millis();
//         let button_press = match press_millis {
//             0..=50 => continue,
//             51..=400 => ButtonPress::Short(name),
//             401..=2000 => ButtonPress::Long(name),
//             _ => continue,
//         };
//         debug!("Button {} pressed for {}ms", name, press_millis);
//
//         let mut buf = [0, b'\n'];
//         buf[0] = button_press.serialize();
//         let mut usart = usart.lock().await;
//         unwrap!(usart.blocking_write(&buf));
//
//         info!("Press: {}", button_press)
//     }
// }
