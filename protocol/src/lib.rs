#![no_std]
use defmt::*;

#[derive(Format, Clone, Copy)]
pub enum Button {
    TopLeft,
    TopMiddle,
    TopRight,
    BottomLeft,
    BottomMiddle,
    BottomRight,
}

#[derive(Format, Clone, Copy)]
pub enum ButtonPress {
    Short(Button),
    Long(Button),
}
