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
    pub last_bit: u8,
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

    /// A read held from the cycle before sees the bit the first read saw, and does not clock.
    #[inline(always)]
    pub fn read(&mut self, clock: bool) -> u8 {
        if self.strobe {
            return self.state & 0x01;
        }
        if !clock {
            return self.last_bit;
        }
        self.last_bit = self.shift & 0x01;
        self.shift = (self.shift >> 1) | 0x80;
        self.last_bit
    }

    pub fn reset(&mut self) {
        self.shift = 0;
        self.strobe = false;
    }
}
