use core::borrow::BorrowMut;
use core::mem::MaybeUninit;
use core::pin::Pin;

use defmt::{info, unwrap};
use defmt_rtt as _; // global logger
use embassy::interrupt::InterruptExt;
use embassy_stm32::interrupt::OTG_FS;
use futures::pin_mut;
use panic_probe as _; // print out panic messages

use embassy::executor::Spawner;
use embassy::io::{AsyncBufReadExt, AsyncWriteExt};
use embassy_stm32::usb_otg::{ClassSet1, State, Usb, UsbBus, UsbOtg, UsbSerial};
use embassy_stm32::{interrupt, time::Hertz, Config, Peripherals};
use usb_device::class_prelude::UsbBusAllocator;
use usb_device::device::{UsbDeviceBuilder, UsbVidPid};

static mut EP_MEMORY: [u32; 2048] = [0; 2048];
type UsbClass<'a> =
    ClassSet1<UsbBus<UsbOtg<'a, USB_OTG_FS>>, UsbSerial<'a, 'a, UsbBus<UsbOtg<'a, USB_OTG_FS>>>>;
pub struct Conn<'a> {
    rx_buffer: [u8; 66],
    tx_buffer: [u8; 66],
    usb_bus: UsbBusAllocator<embassy_stm32::usb_otg::UsbBus<UsbOtg<'a, USB_OTG_FS>>>,
    serial: MaybeUninit<UsbSerial<'a, 'a, embassy_stm32::usb_otg::UsbBus<UsbOtg<'a, USB_OTG_FS>>>>,
    // usb: Usb<
    //     'a,
    //     UsbBus<UsbOtg<'a, USB_OTG_FS>>,
    //     UsbClass<'a>,
    //     OTG_FS,
    // >,
}

use embassy_stm32::peripherals::{PA11, PA12, USB_OTG_FS};
impl<'a> Conn<'a> {
    fn alloc(perhipheral: USB_OTG_FS, pin_12: PA12, pin_11: PA11) -> Self {
        let peri = UsbOtg::new_fs(perhipheral, pin_12, pin_11);
        let usb_bus = UsbBus::new(peri, unsafe { &mut EP_MEMORY });
        Self {
            rx_buffer: [0u8; 66],
            tx_buffer: [0u8; 66],
            serial: MaybeUninit::uninit(),
            usb_bus,

        }
    }
    fn init(self: Pin<&'a mut Self>) {
        let mut_s = self.get_mut();

        let serial = UsbSerial::new(&mut_s.usb_bus, &mut mut_s.rx_buffer, &mut mut_s.tx_buffer);
        mut_s.serial.write(serial);

        // let device = UsbDeviceBuilder::new(&mut_s.usb_bus, UsbVidPid(0x16c0, 0x27dd))
        //     .manufacturer("Fake company")
        //     .product("Serial port")
        //     .serial_number("TEST")
        //     .device_class(0x02)
            // .build();

        // let irq = interrupt::take!(OTG_FS);
        // irq.set_priority(interrupt::Priority::P3);

        // let mut state = State::new();
        // let usb = unsafe { Usb::new(&mut state, device, serial, irq) };

        // Self {
        //     rx_buffer,
        //     tx_buffer,
        //     usb,
        // }
    }
}
