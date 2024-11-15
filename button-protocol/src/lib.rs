#![no_std]
use defmt::Format;
use ha_protocol::{small_bedroom, Reading};

#[derive(Format, Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Button {
    TopLeft = 1,
    TopMiddle = 2,
    TopRight = 3,
    BottomLeft = 4,
    BottomMiddle = 5,
    BottomRight = 6,
}

impl Button {
    pub const fn serialize(self) -> u8 {
        self as u8
    }

    pub fn deserialize(byte: u8) -> Result<Self, &'static str> {
        use Button::*;
        Ok(match byte {
            1 => TopLeft,
            2 => TopMiddle,
            3 => TopRight,
            4 => BottomLeft,
            5 => BottomMiddle,
            6 => BottomRight,
            _ => return Err("Could not deserialize bytes into Button"),
        })
    }
}

#[derive(Format, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ButtonPress {
    Short(Button),
    Long(Button),
}

impl From<ButtonPress> for Reading {
    fn from(value: ButtonPress) -> Self {
        let button = todo!();
        Reading::SmallBedroom(small_bedroom::Reading::ButtonPanel(button))
    }
}

impl ButtonPress {
    pub fn serialize(self) -> u8 {
        use ButtonPress::*;
        match self {
            Short(button) => button.serialize(),
            Long(button) => button.serialize() + 6,
        }
    }

    pub fn deserialize(byte: u8) -> Result<Self, &'static str> {
        use ButtonPress::*;
        Ok(match byte {
            1..=6 => Short(Button::deserialize(byte)?),
            7..=12 => Long(Button::deserialize(byte - 6)?),
            _ => return Err("Could not deserialize byte into ButtonPress"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Button::*;
    use super::ButtonPress;
    use super::ButtonPress::*;

    #[test]
    fn test_all_buttonpresses() {
        let buttons = [
            TopLeft,
            TopMiddle,
            TopRight,
            BottomLeft,
            BottomMiddle,
            BottomRight,
        ];

        for press in [Short, Long] {
            for button in buttons {
                let buttonpress = press(button);
                let serialized = buttonpress.serialize();
                let deserialized =
                    ButtonPress::deserialize(serialized).unwrap();
                assert_eq!(buttonpress, deserialized);
            }
        }
    }

    #[test]
    fn test_nonsense_values() {
        for byte in 0..u8::MAX {
            let res = match byte {
                1..=12 => continue,
                _ => ButtonPress::deserialize(byte),
            };

            assert!(res.is_err());
        }
    }
}
