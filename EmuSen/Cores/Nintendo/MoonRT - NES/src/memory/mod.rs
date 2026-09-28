//! The bus, the cartridge, the boards and the pads. See Moon_Memory.md.

pub mod bus;
pub mod cartridge;
pub mod mappers;

use cartridge::Cartridge;
use mappers::Mapper;

/// The cartridge and its board, which C# keeps as `Cart` and `Cart.Mapper`.
#[derive(Clone, Debug, PartialEq)]
pub struct Board {
    pub cart: Cartridge,
    pub mapper: Mapper,
}

/// C#'s `Controller`: a parallel-load shift register clocked by reads; none of it in a state.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Controller {
    pub state: u8,
    pub shift: u8,
    pub strobe: bool,
}

impl Controller {
    pub fn set_strobe(&mut self, high: bool) {
        if self.strobe && !high {
            self.shift = self.state;
        }
        self.strobe = high;
        if high {
            self.shift = self.state;
        }
    }

    #[inline(always)]
    pub fn read(&mut self) -> u8 {
        if self.strobe {
            return self.state & 0x01;
        }
        let value = self.shift & 0x01;
        self.shift = (self.shift >> 1) | 0x80;
        value
    }

    pub fn reset(&mut self) {
        self.shift = 0;
        self.strobe = false;
    }
}
