//! Streams written symbol by symbol from `standard`'s copy of Annex B, for what no encoder writes: escape codes for
//! levels a code exists for, the largest levels, the reserved and the forbidden one, runs to the 64th coefficient,
//! DC differentials of eleven bits, the first coefficient of a non-intra block by escape, MPEG-1's two escape lengths,
//! its macroblock stuffing and its D-pictures; and the error cases: codes no table holds, an increment other than 1
//! inside IDEC, a run past the 64th coefficient, zeros after a macroblock that make no start code.

#![allow(dead_code)]

use super::standard::annex_b::*;

/// A bit writer, the first bit of each byte its highest.
#[derive(Default)]
pub struct W {
    pub bytes: Vec<u8>,
    pub bits: usize,
}

impl W {
    pub fn put(&mut self, v: u32, n: usize) {
        for k in (0..n).rev() {
            if self.bits % 8 == 0 {
                self.bytes.push(0);
            }
            if v >> k & 1 == 1 {
                *self.bytes.last_mut().unwrap() |= 0x80 >> (self.bits % 8);
            }
            self.bits += 1;
        }
    }

    /// A code as the standard prints it.
    pub fn code(&mut self, c: &str) {
        for ch in c.chars().filter(|c| *c != ' ') {
            self.put(u32::from(ch == '1'), 1);
        }
    }

    /// Zeros to the next byte, then a start code.
    pub fn start(&mut self, value: u8) {
        while self.bits % 8 != 0 {
            self.put(0, 1);
        }
        self.put(1, 24);
        self.put(u32::from(value), 8);
    }
}

/// A coefficient: its run and signed level, written by its variable-length code where one exists unless `escape`.
#[derive(Clone, Copy, Debug)]
pub struct C {
    pub run: u32,
    pub level: i32,
    pub escape: bool,
}

pub const fn c(run: u32, level: i32) -> C {
    C { run, level, escape: false }
}

pub const fn e(run: u32, level: i32) -> C {
    C { run, level, escape: true }
}

/// dct_dc_size and the differential's bits for a differential `d` (§7.2.1 read backwards).
pub fn dc(w: &mut W, chroma: bool, d: i32) {
    let size = if d == 0 { 0 } else { 32 - d.unsigned_abs().leading_zeros() };
    let table = if chroma { B13 } else { B12 };
    w.code(table.iter().find(|e| e.1 == size as i32).unwrap().0);
    if size > 0 {
        let bits = if d >= 0 { d as u32 } else { (d + (1 << size) - 1) as u32 };
        w.put(bits, size as usize);
    }
}

/// A block's coefficients after any DC: from Table B.15 when `one`, else B.14 (its first-coefficient form when
/// `first`); MPEG-1's escape when `mpeg1`; then End of Block unless `no_eob`.
pub fn coefficients(w: &mut W, cs: &[C], one: bool, first: bool, mpeg1: bool) {
    for (k, x) in cs.iter().enumerate() {
        let found = if one {
            B15.iter().find(|t| t.1 == x.run as i32 && t.2 == x.level.abs()).map(|t| t.0)
        } else {
            B14.iter().find(|t| t.1 == x.run as i32 && t.2 == x.level.abs() && t.3 != if k == 0 && first { 4 } else { 3 }).map(|t| t.0)
        };
        match found {
            Some(code) if !x.escape => {
                w.code(code);
                w.put(u32::from(x.level < 0), 1);
            }
            _ => {
                w.code("0000 01");
                w.put(x.run, 6);
                if mpeg1 {
                    match x.level {
                        l if (-127..=127).contains(&l) => w.put((l & 0xFF) as u32, 8),
                        l if l > 0 => {
                            w.put(0, 8);
                            w.put(l as u32, 8);
                        }
                        l => {
                            w.put(0x80, 8);
                            w.put((l + 256) as u32, 8);
                        }
                    }
                } else {
                    w.put((x.level & 0xFFF) as u32, 12);
                }
            }
        }
    }
    w.code(if one { "0110" } else { "10" });
}

/// An MPEG-2 sequence of 64 × 16 (four macroblocks a picture): its header and extension.
fn sequence(w: &mut W, mpeg2: bool) {
    w.start(0xB3);
    w.put(64, 12);
    w.put(16, 12);
    w.put(1, 4);
    w.put(3, 4);
    w.put(0x3FFFF, 18);
    w.put(1, 1);
    w.put(16, 10);
    w.put(0, 1);
    w.put(0, 1);
    w.put(0, 1);
    if mpeg2 {
        w.start(0xB5);
        w.put(1, 4);
        w.put(0x48, 8);
        w.put(1, 1);
        w.put(1, 2);
        w.put(0, 2);
        w.put(0, 2);
        w.put(0, 12);
        w.put(1, 1);
        w.put(0, 8);
        w.put(0, 1);
        w.put(0, 2);
        w.put(0, 5);
    }
}

/// A picture header (P-pictures with f_code 1), and for MPEG-2 its coding extension.
fn picture(w: &mut W, kind: u32, mpeg2: Option<(u32, bool, bool)>) {
    w.start(0x00);
    w.put(0, 10);
    w.put(kind, 3);
    w.put(0xFFFF, 16);
    if kind == 2 || kind == 3 {
        w.put(0, 1);
        w.put(if mpeg2.is_some() { 7 } else { 1 }, 3);
    }
    w.put(0, 1);
    if let Some((dc_precision, ivf, nonlinear)) = mpeg2 {
        w.start(0xB5);
        w.put(8, 4);
        for _ in 0..4 {
            w.put(if kind == 2 { 1 } else { 15 }, 4);
        }
        w.put(dc_precision, 2);
        w.put(3, 2);
        w.put(0, 1);
        w.put(1, 1);
        w.put(0, 1);
        w.put(u32::from(nonlinear), 1);
        w.put(u32::from(ivf), 1);
        w.put(0, 1);
        w.put(0, 1);
        w.put(1, 1);
        w.put(1, 1);
        w.put(0, 1);
    }
}

/// An intra macroblock: increment 1 (with `stuffing` codes of MPEG-1 before it), Intra or Intra + Quant, six blocks of
/// a DC differential and coefficients each.
pub fn intra_mb(w: &mut W, quant: Option<u32>, stuffing: usize, blocks: &[(i32, &[C]); 6], one: bool, mpeg1: bool) {
    for _ in 0..stuffing {
        w.code("0000 0001 111");
    }
    w.code("1");
    match quant {
        Some(q) => {
            w.code("01");
            w.put(q, 5);
        }
        None => w.code("1"),
    }
    for (i, (d, cs)) in blocks.iter().enumerate() {
        dc(w, i >= 4, *d);
        coefficients(w, cs, one, false, mpeg1);
    }
}

/// The odd MPEG-2 stream: an I-picture of escapes, the largest levels and the reserved one, an I-picture at 11-bit DC
/// precision with Table B.15 and the non-linear scale, and a P-picture of non-intra blocks whose first coefficients
/// are escapes.
pub fn odd_mpeg2() -> Vec<u8> {
    let mut w = W::default();
    sequence(&mut w, true);
    picture(&mut w, 1, Some((0, false, false)));
    w.start(0x01);
    w.put(4, 5);
    w.put(0, 1);
    let lots: &[C] = &[e(0, 1), e(1, -1), c(0, 40), e(0, 41), e(3, 200), e(0, -300), c(5, 2)];
    let large: &[C] = &[e(0, 2047), e(0, -2047), e(10, 1000), e(40, -2047)];
    let reserved: &[C] = &[e(0, -2048), e(2, 0), e(0, 5)];
    let to_end: &[C] = &[c(0, 3), e(61, 7)];
    let none: &[C] = &[];
    intra_mb(&mut w, None, 0, &[(20, lots), (-20, large), (5, reserved), (0, to_end), (-100, lots), (100, large)], false, false);
    intra_mb(&mut w, Some(31), 0, &[(100, large), (-233, lots), (0, none), (1, to_end), (2, reserved), (-3, none)], false, false);
    intra_mb(&mut w, Some(1), 0, &[(50, to_end), (0, lots), (0, large), (0, reserved), (127, lots), (-128, none)], false, false);
    intra_mb(&mut w, None, 0, &[(1, none), (0, none), (0, none), (0, none), (0, none), (0, none)], false, false);
    picture(&mut w, 1, Some((3, true, true)));
    w.start(0x01);
    w.put(17, 5);
    w.put(0, 1);
    let one_lots: &[C] = &[c(0, 12), e(0, 15), c(1, 8), e(2, -999), c(0, -1)];
    intra_mb(&mut w, None, 0, &[(1023, one_lots), (-2047, one_lots), (2047, large), (-1024, to_end), (0, one_lots), (1, none)], true, false);
    intra_mb(&mut w, Some(31), 0, &[(-1023, large), (2047, none), (-1024, reserved), (0, one_lots), (-1, one_lots), (1, none)], true, false);
    intra_mb(&mut w, None, 0, &[(0, none); 6], true, false);
    intra_mb(&mut w, None, 0, &[(0, none); 6], true, false);
    picture(&mut w, 2, Some((0, false, false)));
    w.start(0x01);
    w.put(8, 5);
    w.put(0, 1);
    let blocks: [[&[C]; 4]; 4] = [
        [&[e(0, 1)], &[c(0, 1), c(0, -1)], &[e(0, -1), c(2, 3)], &[c(0, 2)]],
        [&[e(63, 7)], &[c(1, 1)], &[e(0, 2047), e(62, -2047)], &[c(0, 1)]],
        [&[c(0, 1), e(5, 2047), e(1, -2048)], &[c(0, 1)], &[c(0, -1)], &[c(0, 1)]],
        [&[e(0, 0)], &[e(62, 3)], &[c(0, -1)], &[c(0, 1)]],
    ];
    for mb in blocks {
        w.code("1");
        w.code("01");
        w.code("111");
        for cs in mb {
            coefficients(&mut w, cs, false, true, false);
        }
    }
    w.start(0xB7);
    w.bytes
}

/// The odd MPEG-1 stream: an I-picture with macroblock stuffing and both escape lengths, and a D-picture.
pub fn odd_mpeg1() -> Vec<u8> {
    let mut w = W::default();
    sequence(&mut w, false);
    picture(&mut w, 1, None);
    w.start(0x01);
    w.put(6, 5);
    w.put(0, 1);
    let short: &[C] = &[e(0, 1), e(1, -127), e(2, 127), c(0, 3)];
    let long: &[C] = &[e(0, 128), e(0, -128), e(3, 255), e(0, -255), e(1, 200)];
    let none: &[C] = &[];
    intra_mb(&mut w, None, 2, &[(10, short), (-10, long), (0, none), (3, short), (-3, long), (1, none)], false, true);
    intra_mb(&mut w, Some(20), 1, &[(100, long), (-100, short), (0, long), (0, short), (0, none), (5, long)], false, true);
    intra_mb(&mut w, None, 0, &[(0, short); 6], false, true);
    intra_mb(&mut w, None, 3, &[(-1, long), (0, none), (0, none), (0, none), (0, none), (0, none)], false, true);
    picture(&mut w, 4, None);
    w.start(0x01);
    w.put(9, 5);
    w.put(0, 1);
    for ds in [[50, -50, 0, 7, -7, 1], [0, 0, 0, 0, 0, 0], [-100, 100, 1, -1, 2, -2], [3, 3, 3, 3, 3, 3]] {
        w.code("1");
        w.code("1");
        for (i, d) in ds.iter().enumerate() {
            dc(&mut w, i >= 4, *d);
        }
        w.code("1");
    }
    w.start(0xB7);
    w.bytes
}
