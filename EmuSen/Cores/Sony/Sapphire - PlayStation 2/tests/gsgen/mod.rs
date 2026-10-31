//! Generated GS streams shared by the workers' tests (`threads.rs`) and the device path's (`gs_device.rs`): every
//! primitive, environment, texture (from the frame's own pages too), CLUT load and transfer over a few pages, so that
//! frames, Z buffers, textures, CLUTs and transfers meet; and the runner that gives every value read back and the
//! memory's digest at each check.

#![allow(dead_code)]

use sapphire::gs::{Gs, HostInterface, privileged, reg};

pub struct Rng(pub u64);

impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 16
    }

    pub fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }

    pub fn chance(&mut self, k: u64) -> bool {
        self.below(k) == 0
    }
}

const FRAME_PSM: [u64; 8] = [0x00, 0x01, 0x02, 0x0A, 0x30, 0x31, 0x32, 0x3A];
const Z_PSM: [u64; 4] = [0x0, 0x1, 0x2, 0xA];
const TEX_PSM: [u64; 9] = [0x00, 0x01, 0x02, 0x0A, 0x13, 0x14, 0x1B, 0x24, 0x2C];
const Q: [u32; 6] = [0x3F80_0000, 0x3F00_0000, 0x4000_0000, 0x3E80_0000, 0x3FC0_0000, 0x3F40_0000];

/// One step of a stream: register writes, an image's units, or a read back of `n` units.
pub enum Step {
    Regs(Vec<(u8, u64)>),
    Image(Vec<u64>),
    Read(usize),
    Check,
}

fn float(r: &mut Rng) -> u32 {
    if r.chance(3) { Q[r.below(6) as usize] } else { 0x3E00_0000 + r.below(0x0180_0000) as u32 }
}

/// A stream of `n` steps over a few pages, so that frames, Z buffers, textures, CLUTs and transfers meet.
pub fn stream(seed: u64, n: usize) -> Vec<Step> {
    let mut r = Rng(seed);
    let mut out = Vec::new();
    for _ in 0..n {
        match r.below(20) {
            0..=5 => {
                let c = r.below(2) as u8;
                let mut regs = vec![
                    (reg::FRAME_1 + c, r.below(12) | (1 + r.below(4)) << 16 | FRAME_PSM[r.below(8) as usize] << 24 | if r.chance(4) { r.below(1 << 32) << 32 } else { 0 }),
                    (reg::ZBUF_1 + c, r.below(12) | Z_PSM[r.below(4) as usize] << 24 | u64::from(r.chance(4)) << 32),
                    (reg::TEST_1 + c, r.below(1 << 19)),
                    (reg::SCISSOR_1 + c, r.below(64) | (64 + r.below(260)) << 16 | r.below(64) << 32 | (64 + r.below(260)) << 48),
                    (reg::ALPHA_1 + c, r.below(256) | r.below(256) << 32),
                    (reg::FBA_1 + c, u64::from(r.chance(4))),
                    (reg::PABE, u64::from(r.chance(4))),
                    (reg::DTHE, u64::from(r.chance(3))),
                    (reg::DIMX, r.next() << 16 | r.below(1 << 16)),
                    (reg::COLCLAMP, r.below(2)),
                    (reg::SCANMSK, if r.chance(6) { 2 + r.below(2) } else { 0 }),
                    (reg::XYOFFSET_1 + c, if r.chance(3) { r.below(64) << 4 | r.below(64) << 36 } else { 0 }),
                    (reg::PRMODECONT, u64::from(!r.chance(8))),
                    (reg::PRMODE, r.below(256) << 3),
                    (reg::FOGCOL, r.below(1 << 24)),
                    (reg::TEXA, r.below(256) | r.below(2) << 15 | r.below(256) << 32),
                    (reg::TEXCLUT, 1 + r.below(4) | r.below(4) << 6 | r.below(16) << 12),
                    (reg::TEX1_1 + c, r.below(2) | r.below(7) << 2 | r.below(2) << 5 | r.below(6) << 6 | u64::from(r.chance(4)) << 9 | r.below(4) << 19 | r.below(1 << 12) << 32),
                    (reg::CLAMP_1 + c, r.below(16) | r.below(1024) << 4 | r.below(1024) << 14 | r.below(1024) << 24 | r.below(1024) << 34),
                    (reg::MIPTBP1_1 + c, r.next() & ((1 << 60) - 1)),
                    (reg::MIPTBP2_1 + c, r.next() & ((1 << 60) - 1)),
                ];
                let tex0 = r.below(12 * 32) | (1 + r.below(4)) << 14 | TEX_PSM[r.below(9) as usize] << 20 | (2 + r.below(7)) << 26 | (2 + r.below(7)) << 30 | r.below(2) << 34 | r.below(4) << 35 | r.below(12 * 32) << 37 | [0u64, 2, 0xA][r.below(3) as usize] << 51 | r.below(2) << 55 | r.below(32) << 56 | r.below(6) << 61;
                regs.push((reg::TEX0_1 + c, tex0));
                if r.chance(4) {
                    regs.push((reg::TEX2_1 + c, tex0 ^ (r.below(32) << 56)));
                }
                out.push(Step::Regs(regs));
            }
            6..=14 => {
                let kind = r.below(8);
                let attributes = r.below(256) << 3;
                let mut regs = vec![(reg::PRIM, kind | attributes)];
                for _ in 0..1 + r.below(6) {
                    let q = float(&mut r);
                    regs.push((reg::RGBAQ, r.below(1 << 32) | u64::from(q) << 32));
                    regs.push((reg::ST, u64::from(float(&mut r)) | u64::from(float(&mut r)) << 32));
                    regs.push((reg::UV, r.below(1 << 14) | r.below(1 << 14) << 16));
                    let (x, y) = (r.below(340 * 16), r.below(340 * 16));
                    let which = [reg::XYZ2, reg::XYZF2, reg::XYZ3, reg::XYZF3][if r.chance(6) { 2 + r.below(2) } else { r.below(2) } as usize];
                    regs.push((which, x | y << 16 | r.below(1 << 32) << 32));
                }
                out.push(Step::Regs(regs));
            }
            15 | 16 => {
                let psm = TEX_PSM[r.below(9) as usize];
                let (w, h) = (1 + r.below(48), 1 + r.below(48));
                let bits = match psm {
                    0x00 => 32,
                    0x01 => 24,
                    0x02 | 0x0A => 16,
                    0x13 | 0x1B => 8,
                    _ => 4,
                };
                out.push(Step::Regs(vec![
                    (reg::BITBLTBUF, (r.below(12 * 32)) << 32 | (1 + r.below(4)) << 48 | psm << 56),
                    (reg::TRXPOS, r.below(200) << 32 | r.below(200) << 48),
                    (reg::TRXREG, w | h << 32),
                    (reg::TRXDIR, 0),
                ]));
                // A quarter of the transfers are cut short, so that the next TRXDIR ends them.
                let units = (w * h * bits).div_ceil(64);
                let units = if r.chance(4) { r.below(units) } else { units };
                out.push(Step::Image((0..units).map(|_| r.next() << 16 ^ r.next()).collect()));
            }
            17 => {
                let psm = FRAME_PSM[r.below(4) as usize];
                let (w, h) = (1 + r.below(32), 1 + r.below(32));
                out.push(Step::Regs(vec![
                    (reg::BITBLTBUF, r.below(12 * 32) | (1 + r.below(4)) << 16 | psm << 24 | (r.below(12 * 32)) << 32 | (1 + r.below(4)) << 48 | psm << 56),
                    (reg::TRXPOS, r.below(200) | r.below(200) << 16 | r.below(200) << 32 | r.below(200) << 48 | r.below(4) << 59),
                    (reg::TRXREG, w | h << 32),
                    (reg::TRXDIR, 2),
                ]));
            }
            18 => {
                let psm = FRAME_PSM[r.below(8) as usize];
                let (w, h) = (1 + r.below(40), 1 + r.below(40));
                out.push(Step::Regs(vec![
                    (reg::BITBLTBUF, r.below(12 * 32) | (1 + r.below(4)) << 16 | psm << 24),
                    (reg::TRXPOS, r.below(260) | r.below(260) << 16),
                    (reg::TRXREG, w | h << 32),
                    (reg::TRXDIR, 1),
                ]));
                let bits = if psm & 2 != 0 { 16 } else if psm & 1 != 0 { 24 } else { 32 };
                out.push(Step::Read((w * h * bits).div_ceil(64) as usize));
            }
            _ => out.push(Step::Check),
        }
    }
    out.push(Step::Check);
    out
}

pub fn busdir(gs: &mut Gs, v: u64) {
    gs.write_privileged(privileged::address(privileged::BUSDIR), 8, v);
}

/// The stream run on `gs`; every value read back and the memory's digest at each check, in order.
pub fn run(gs: &mut Gs, steps: &[Step]) -> Vec<u64> {
    let mut out = Vec::new();
    for s in steps {
        match s {
            Step::Regs(list) => list.iter().for_each(|&(r, v)| gs.write(r, v)),
            Step::Image(units) => units.iter().for_each(|&u| gs.image(u)),
            Step::Read(n) => {
                busdir(gs, 1);
                out.extend((0..*n).map(|_| gs.read().unwrap_or(0xDEAD)));
                busdir(gs, 0);
            }
            Step::Check => out.push(gs.local().words().iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &w| (h ^ u64::from(w)).wrapping_mul(0x100_0000_01B3))),
        }
    }
    out
}
