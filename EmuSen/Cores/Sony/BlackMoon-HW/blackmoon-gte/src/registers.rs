//! The register transfers and their quirks (psx-spx "GTE Registers"): what a write keeps and what a read returns
//! (BlackMoon_GTE.md §3).

use crate::Registers;

/// The data registers by number.
pub mod data {
    pub const VZ0: usize = 1;
    pub const VZ1: usize = 3;
    pub const VZ2: usize = 5;
    pub const RGBC: usize = 6;
    pub const OTZ: usize = 7;
    pub const IR0: usize = 8;
    pub const IR1: usize = 9;
    pub const SXY0: usize = 12;
    pub const SXY2: usize = 14;
    pub const SXYP: usize = 15;
    pub const SZ0: usize = 16;
    pub const SZ3: usize = 19;
    pub const RGB0: usize = 20;
    pub const RGB2: usize = 22;
    pub const MAC0: usize = 24;
    pub const IRGB: usize = 28;
    pub const ORGB: usize = 29;
    pub const LZCS: usize = 30;
    pub const LZCR: usize = 31;
}

/// The control registers by number.
pub mod control {
    pub const RT: usize = 0;
    pub const RT33: usize = 4;
    pub const TR: usize = 5;
    pub const LLM: usize = 8;
    pub const L33: usize = 12;
    pub const BK: usize = 13;
    pub const LCM: usize = 16;
    pub const LB3: usize = 20;
    pub const FC: usize = 21;
    pub const OFX: usize = 24;
    pub const OFY: usize = 25;
    pub const H: usize = 26;
    pub const DQA: usize = 27;
    pub const DQB: usize = 28;
    pub const ZSF3: usize = 29;
    pub const ZSF4: usize = 30;
    pub const FLAG: usize = 31;
}

/// FLAG's bits a write keeps, and those its bit 31 sums (psx-spx: 30-23 and 18-13).
pub const FLAG_WRITABLE: u32 = 0x7FFF_F000;
pub const FLAG_SUMMED: u32 = 0x7F87_E000;

pub fn flag_word(bits: u32) -> u32 {
    let bits = bits & FLAG_WRITABLE;
    bits | if bits & FLAG_SUMMED != 0 { 1 << 31 } else { 0 }
}

fn sign16(v: u32) -> u32 {
    v as u16 as i16 as i32 as u32
}

/// IR1-IR3 collapsed to five bits each, as IRGB and ORGB read.
fn orgb(r: &Registers) -> u32 {
    (0..3).fold(0, |a, i| a | ((r.data[data::IR1 + i] as i32 >> 7).clamp(0, 0x1F) as u32) << (5 * i))
}

/// The leading zeros of LZCS, or its leading ones when negative.
fn lzcr(v: u32) -> u32 {
    if (v as i32) < 0 { v.leading_ones() } else { v.leading_zeros() }
}

pub fn read_data(r: &Registers, n: usize) -> u32 {
    match n {
        data::SXYP => r.data[data::SXY2],
        data::IRGB | data::ORGB => orgb(r),
        data::LZCR => lzcr(r.data[data::LZCS]),
        _ => r.data[n],
    }
}

pub fn write_data(r: &mut Registers, n: usize, v: u32) {
    match n {
        data::VZ0 | data::VZ1 | data::VZ2 | data::IR0..=11 => r.data[n] = sign16(v),
        data::OTZ | data::SZ0..=data::SZ3 => r.data[n] = v & 0xFFFF,
        data::SXYP => {
            r.data.copy_within(data::SXY0 + 1..=data::SXY2, data::SXY0);
            r.data[data::SXY2] = v;
        }
        data::IRGB => {
            for i in 0..3 {
                r.data[data::IR1 + i] = (v >> (5 * i) & 0x1F) * 0x80;
            }
        }
        data::ORGB | data::LZCR => {}
        _ => r.data[n] = v,
    }
}

pub fn read_control(r: &Registers, n: usize) -> u32 {
    r.control[n]
}

pub fn write_control(r: &mut Registers, n: usize, v: u32) {
    r.control[n] = match n {
        control::RT33 | control::L33 | control::LB3 | control::H | control::DQA | control::ZSF3 | control::ZSF4 => sign16(v),
        control::FLAG => flag_word(v),
        _ => v,
    };
}
