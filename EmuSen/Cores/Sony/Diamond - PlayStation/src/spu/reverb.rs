//! The reverb unit (psx-spx "SPU Reverb Registers", "SPU Reverb Formula", "SPU Internal State Machine"): at 22,050 Hz,
//! the left side on one 44,100 Hz cycle and the right on the next, its work area in sound RAM from ESA to the end, its
//! input taken down and its output back up through the 39-tap filter. Diamond_SPU.md §5 records the order of its
//! accesses and every rounding.

use super::tables::FIR;
use super::voice::Watch;

/// The reverb registers' halfword indices from 1F801DC0h, psx-spx's names.
mod reg {
    pub const DAPF1: usize = 0;
    pub const DAPF2: usize = 1;
    pub const VIIR: usize = 2;
    pub const VCOMB1: usize = 3;
    pub const VWALL: usize = 7;
    pub const VAPF1: usize = 8;
    pub const VAPF2: usize = 9;
    pub const MSAME: usize = 10;
    pub const MCOMB1: usize = 12;
    pub const MCOMB2: usize = 14;
    pub const DSAME: usize = 16;
    pub const MDIFF: usize = 18;
    pub const MCOMB3: usize = 20;
    pub const MCOMB4: usize = 22;
    pub const DDIFF: usize = 24;
    pub const MAPF1: usize = 26;
    pub const MAPF2: usize = 28;
    pub const VIN: usize = 30;
}

/// The filter's history is a ring of 64 samples, of which the last 39 are taken.
const RING: usize = 64;

#[derive(Clone, Debug)]
pub struct Reverb {
    /// The current buffer address, in bytes.
    pub address: u32,
    /// The input at 44,100 Hz, and the output at 44,100 Hz with each side's 22,050 Hz samples between zeros.
    pub input: [[i16; RING]; 2],
    pub output: [[i16; RING]; 2],
    pub at: u8,
    /// Whether the next cycle is the right side's.
    pub right: bool,
}

impl Default for Reverb {
    fn default() -> Reverb {
        Reverb { address: 0, input: [[0; RING]; 2], output: [[0; RING]; 2], at: 0, right: false }
    }
}

#[inline]
fn mul(a: i32, b: i32) -> i32 {
    a * b >> 15
}

#[inline]
fn sat(v: i32) -> i16 {
    v.clamp(-0x8000, 0x7FFF) as i16
}

/// The reverb's view of sound RAM: the work area from ESA, every access watched for the IRQ address.
struct Area<'a> {
    ram: &'a mut [u8],
    base: u32,
    address: u32,
    watch: &'a mut Watch,
}

impl Area<'_> {
    /// The byte address of `offset` bytes from the buffer address, wrapped within ESA..7FFFFh.
    fn at(&self, offset: i32) -> usize {
        let size = 0x8_0000 - self.base as i32;
        let rel = (self.address as i32 - self.base as i32 + offset).rem_euclid(size);
        (self.base as i32 + rel) as usize & 0x7FFFE
    }

    fn read(&mut self, offset: i32) -> i32 {
        let a = self.at(offset);
        self.watch.touch(a as u32);
        i16::from_le_bytes([self.ram[a], self.ram[a + 1]]) as i32
    }

    /// A write, or with the master enable off the read it becomes (psx-spx "Reverb Computation Order").
    fn write(&mut self, offset: i32, value: i32, enabled: bool) {
        let a = self.at(offset);
        self.watch.touch(a as u32);
        if enabled {
            self.ram[a..a + 2].copy_from_slice(&sat(value).to_le_bytes());
        }
    }
}

impl Reverb {
    /// ESA written: the buffer address with it.
    pub fn set_base(&mut self, esa: u16) {
        self.address = esa as u32 * 8;
    }

    fn filter(ring: &[i16; RING], at: usize) -> i32 {
        let mut sum = 0;
        for (k, &c) in FIR.iter().enumerate() {
            sum += c * ring[(at + RING - k) % RING] as i32;
        }
        sum
    }

    /// One 44,100 Hz cycle: the input pushed, one side computed, and both sides' output. `regs` are the 32 halfwords
    /// from 1F801DC0h, `evol` EVOLL and EVOLR, `esa` the work area's start, `enabled` ATTR's master enable.
    pub fn tick(&mut self, ram: &mut [u8], watch: &mut Watch, regs: &[u16; 32], evol: [u16; 2], esa: u16, enabled: bool, input: [i32; 2]) -> [i32; 2] {
        let at = (self.at as usize + 1) % RING;
        self.at = at as u8;
        self.input[0][at] = sat(input[0]);
        self.input[1][at] = sat(input[1]);
        let side = self.right as usize;
        let value = self.side(ram, watch, regs, esa, enabled, side, Self::filter(&self.input[side], at) >> 15);
        let other = 1 - side;
        self.output[side][at] = sat(mul(value, evol[side] as i16 as i32));
        self.output[other][at] = 0;
        if self.right {
            let base = esa as u32 * 8;
            self.address = ((self.address + 2) & 0x7FFFE).max(base);
        }
        self.right = !self.right;
        [sat(Self::filter(&self.output[0], at) >> 14) as i32, sat(Self::filter(&self.output[1], at) >> 14) as i32]
    }

    /// One side's step of psx-spx's formula, its accesses in the order its state machine section lists them.
    #[allow(clippy::too_many_arguments)]
    fn side(&mut self, ram: &mut [u8], watch: &mut Watch, regs: &[u16; 32], esa: u16, enabled: bool, side: usize, input: i32) -> i32 {
        let s = |r: usize| regs[r] as i16 as i32;
        let m = |r: usize| regs[r] as i32 * 8;
        let other = 1 - side;
        let mut a = Area { ram, base: esa as u32 * 8, address: self.address, watch };
        let lin = mul(sat(input) as i32, s(reg::VIN + side));
        let (iir, wall) = (s(reg::VIIR), s(reg::VWALL));
        let d_same = a.read(m(reg::DSAME + side));
        let same_before = a.read(m(reg::MSAME + side) - 2);
        let d_diff = a.read(m(reg::DDIFF + other));
        let flip = if iir == -0x8000 { -1 } else { 1 };
        a.write(m(reg::MSAME + side), flip * (mul(lin + mul(d_same, wall) - same_before, iir) + same_before), enabled);
        let diff_before = a.read(m(reg::MDIFF + side) - 2);
        let comb1 = a.read(m(reg::MCOMB1 + side));
        a.write(m(reg::MDIFF + side), flip * (mul(lin + mul(d_diff, wall) - diff_before, iir) + diff_before), enabled);
        let comb2 = a.read(m(reg::MCOMB2 + side));
        let comb3 = a.read(m(reg::MCOMB3 + side));
        let comb4 = a.read(m(reg::MCOMB4 + side));
        let apf1 = a.read(m(reg::MAPF1 + side) - m(reg::DAPF1));
        let apf2 = a.read(m(reg::MAPF2 + side) - m(reg::DAPF2));
        let out = mul(s(reg::VCOMB1), comb1) + mul(s(reg::VCOMB1 + 1), comb2) + mul(s(reg::VCOMB1 + 2), comb3) + mul(s(reg::VCOMB1 + 3), comb4);
        let (v1, v2) = (s(reg::VAPF1), s(reg::VAPF2));
        let out = out - mul(v1, apf1);
        a.write(m(reg::MAPF1 + side), out, enabled);
        let out = mul(sat(out) as i32, v1) + apf1;
        let out = out - mul(v2, apf2);
        a.write(m(reg::MAPF2 + side), out, enabled);
        sat(mul(sat(out) as i32, v2) + apf2) as i32
    }
}
