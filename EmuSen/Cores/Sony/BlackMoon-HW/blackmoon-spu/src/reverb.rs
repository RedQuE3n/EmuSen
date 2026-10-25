//! The reverb's algorithm (psx-spx "SPU Reverb Formula", "SPU Internal State Machine"): at half the sample rate, the
//! left side on one sample and the right on the next, its work area in sound RAM, its input taken down and its output
//! back up through the 39-tap filter. Diamond_SPU.md §5 records the order of its accesses and every rounding. The
//! registers are the core's: it hands each sample's volumes and offsets as `Params` and the work area as `Area`
//! (BlackMoon_SPU.md §3).

use crate::Touch;
use crate::mask;
use crate::tables::FIR;

/// The filter's history is a ring of 64 samples, of which the last 39 are taken.
pub const RING: usize = 64;

/// The work area: from `start` to `end` (exclusive), in bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Area {
    pub start: u32,
    pub end: u32,
}

/// One sample's coefficients, signed 16-bit as the registers hold them, and offsets from the buffer address in bytes;
/// each pair is the left side's, then the right's. psx-spx's names.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Params {
    /// vLIN and vRIN, vIIR, vWALL, vCOMB1-4, vAPF1-2, and the output volumes (EVOL).
    pub input: [i32; 2],
    pub iir: i32,
    pub wall: i32,
    pub comb: [i32; 4],
    pub apf: [i32; 2],
    pub output: [i32; 2],
    /// mSAME, mDIFF, dSAME, dDIFF, mCOMB1-4, mAPF1-2 by side, and dAPF1-2.
    pub same: [i32; 2],
    pub diff: [i32; 2],
    pub d_same: [i32; 2],
    pub d_diff: [i32; 2],
    pub comb_at: [[i32; 2]; 4],
    pub apf_at: [[i32; 2]; 2],
    pub d_apf: [i32; 2],
}

#[derive(Clone, Debug)]
pub struct Reverb {
    /// The current buffer address, in bytes.
    pub address: u32,
    /// The input at the sample rate, and the output at the sample rate with each side's half-rate samples between
    /// zeros.
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

/// The reverb's view of sound RAM: the work area, every access watched for the IRQ address.
struct Window<'a, W: Touch> {
    ram: &'a mut [u8],
    area: Area,
    address: u32,
    watch: &'a mut W,
}

impl<W: Touch> Window<'_, W> {
    /// The byte address of `offset` bytes from the buffer address, wrapped within the work area.
    fn at(&self, offset: i32) -> usize {
        let base = self.area.start as i32;
        let size = (self.area.end as i32 - base).max(2);
        let rel = (self.address as i32 - base + offset).rem_euclid(size);
        (base + rel) as usize & mask(self.ram) & !1
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
    fn filter(ring: &[i16; RING], at: usize) -> i32 {
        let mut sum = 0;
        for (k, &c) in FIR.iter().enumerate() {
            sum += c * ring[(at + RING - k) % RING] as i32;
        }
        sum
    }

    /// One sample: the input pushed, one side computed, and both sides' output. `enabled` is the core's master enable
    /// of the reverb's writes.
    pub fn tick<W: Touch>(&mut self, ram: &mut [u8], watch: &mut W, p: &Params, area: Area, enabled: bool, input: [i32; 2]) -> [i32; 2] {
        let at = (self.at as usize + 1) % RING;
        self.at = at as u8;
        self.input[0][at] = sat(input[0]);
        self.input[1][at] = sat(input[1]);
        let side = self.right as usize;
        let value = self.side(ram, watch, p, area, enabled, side, Self::filter(&self.input[side], at) >> 15);
        let other = 1 - side;
        self.output[side][at] = sat(mul(value, p.output[side]));
        self.output[other][at] = 0;
        if self.right {
            let mut next = (self.address + 2) & (mask(ram) as u32 & !1);
            if next >= area.end {
                next = area.start;
            }
            self.address = next.max(area.start);
        }
        self.right = !self.right;
        [sat(Self::filter(&self.output[0], at) >> 14) as i32, sat(Self::filter(&self.output[1], at) >> 14) as i32]
    }

    /// One side's step of psx-spx's formula, its accesses in the order its state machine section lists them.
    #[allow(clippy::too_many_arguments)]
    fn side<W: Touch>(&mut self, ram: &mut [u8], watch: &mut W, p: &Params, area: Area, enabled: bool, side: usize, input: i32) -> i32 {
        let other = 1 - side;
        let mut a = Window { ram, area, address: self.address, watch };
        let lin = mul(sat(input) as i32, p.input[side]);
        let (iir, wall) = (p.iir, p.wall);
        let d_same = a.read(p.d_same[side]);
        let same_before = a.read(p.same[side] - 2);
        let d_diff = a.read(p.d_diff[other]);
        let flip = if iir == -0x8000 { -1 } else { 1 };
        a.write(p.same[side], flip * (mul(lin + mul(d_same, wall) - same_before, iir) + same_before), enabled);
        let diff_before = a.read(p.diff[side] - 2);
        let comb1 = a.read(p.comb_at[0][side]);
        a.write(p.diff[side], flip * (mul(lin + mul(d_diff, wall) - diff_before, iir) + diff_before), enabled);
        let comb2 = a.read(p.comb_at[1][side]);
        let comb3 = a.read(p.comb_at[2][side]);
        let comb4 = a.read(p.comb_at[3][side]);
        let apf1 = a.read(p.apf_at[0][side] - p.d_apf[0]);
        let apf2 = a.read(p.apf_at[1][side] - p.d_apf[1]);
        let out = mul(p.comb[0], comb1) + mul(p.comb[1], comb2) + mul(p.comb[2], comb3) + mul(p.comb[3], comb4);
        let (v1, v2) = (p.apf[0], p.apf[1]);
        let out = out - mul(v1, apf1);
        a.write(p.apf_at[0][side], out, enabled);
        let out = mul(sat(out) as i32, v1) + apf1;
        let out = out - mul(v2, apf2);
        a.write(p.apf_at[1][side], out, enabled);
        sat(mul(sat(out) as i32, v2) + apf2) as i32
    }
}
