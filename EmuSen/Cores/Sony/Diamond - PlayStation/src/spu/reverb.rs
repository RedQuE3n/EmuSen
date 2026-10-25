//! The reverb's registers (psx-spx "SPU Reverb Registers"): the 32 halfwords from 1F801DC0h, their addresses in 8-byte
//! units, the work area from ESA to the end of sound RAM. The algorithm is `blackmoon-spu`'s (Diamond_SPU.md §5,
//! BlackMoon_SPU.md §3).

use blackmoon_spu::reverb::{Area, Params};

use super::voice::Watch;

/// The reverb registers' halfword indices from 1F801DC0h, psx-spx's names.
mod reg {
    pub const DAPF1: usize = 0;
    pub const VIIR: usize = 2;
    pub const VCOMB1: usize = 3;
    pub const VWALL: usize = 7;
    pub const VAPF1: usize = 8;
    pub const MSAME: usize = 10;
    pub const MCOMB1: usize = 12;
    pub const DSAME: usize = 16;
    pub const MDIFF: usize = 18;
    pub const MCOMB3: usize = 20;
    pub const DDIFF: usize = 24;
    pub const MAPF1: usize = 26;
    pub const VIN: usize = 30;
}

/// The shared reverb with its buffer address, filters and phase.
#[derive(Clone, Debug, Default)]
pub struct Reverb(pub blackmoon_spu::reverb::Reverb);

impl std::ops::Deref for Reverb {
    type Target = blackmoon_spu::reverb::Reverb;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Reverb {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// The registers as the shared reverb takes them: coefficients sign-extended, addresses from 8-byte units to bytes.
fn params(regs: &[u16; 32], evol: [u16; 2]) -> Params {
    let s = |r: usize| regs[r] as i16 as i32;
    let m = |r: usize| regs[r] as i32 * 8;
    let pair = |r: usize| [m(r), m(r + 1)];
    Params {
        input: [s(reg::VIN), s(reg::VIN + 1)],
        iir: s(reg::VIIR),
        wall: s(reg::VWALL),
        comb: [s(reg::VCOMB1), s(reg::VCOMB1 + 1), s(reg::VCOMB1 + 2), s(reg::VCOMB1 + 3)],
        apf: [s(reg::VAPF1), s(reg::VAPF1 + 1)],
        output: [evol[0] as i16 as i32, evol[1] as i16 as i32],
        same: pair(reg::MSAME),
        diff: pair(reg::MDIFF),
        d_same: pair(reg::DSAME),
        d_diff: pair(reg::DDIFF),
        comb_at: [pair(reg::MCOMB1), pair(reg::MCOMB1 + 2), pair(reg::MCOMB3), pair(reg::MCOMB3 + 2)],
        apf_at: [pair(reg::MAPF1), pair(reg::MAPF1 + 2)],
        d_apf: [m(reg::DAPF1), m(reg::DAPF1 + 1)],
    }
}

impl Reverb {
    /// ESA written: the buffer address with it.
    pub fn set_base(&mut self, esa: u16) {
        self.0.address = esa as u32 * 8;
    }

    /// One 44,100 Hz cycle. `regs` are the 32 halfwords from 1F801DC0h, `evol` EVOLL and EVOLR, `esa` the work area's
    /// start, `enabled` ATTR's master enable.
    #[allow(clippy::too_many_arguments)]
    pub fn tick(&mut self, ram: &mut [u8], watch: &mut Watch, regs: &[u16; 32], evol: [u16; 2], esa: u16, enabled: bool, input: [i32; 2]) -> [i32; 2] {
        let area = Area { start: esa as u32 * 8, end: ram.len() as u32 };
        self.0.tick(ram, watch, &params(regs, evol), area, enabled, input)
    }
}
