//! The VIF's rules as the EE User's Manual's chapter 6 states them, transcribed apart from `src/vif` to give the S14
//! programs' expected records: UNPACK's formats (§6.3.4), its write sequence (§6.3.5), the mask (§6.3.6) and the
//! addition modes (§6.3.7), with the readings Sapphire_Disputes.md D-180 to D-183 record where the manual leaves them
//! open; and VIF1's double buffer (§6.3.8, VIF1_TOPS's page).

#![allow(dead_code)]

/// The registers an UNPACK reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Regs {
    pub row: [u32; 4],
    pub col: [u32; 4],
    pub mask: u32,
    pub mode: u32,
    pub cl: u32,
    pub wl: u32,
}

/// Elements of `bits` bits from a packet's data words, the first at bit 0 of the first word.
fn element(data: &[u32], index: u32, bits: u32) -> u32 {
    let bit = index * bits;
    let w = data.get((bit / 32) as usize).copied().unwrap_or(0);
    let v = w >> (bit % 32);
    if bits == 32 { v } else { v & ((1 << bits) - 1) }
}

fn extend(v: u32, bits: u32, usn: bool) -> u32 {
    if bits == 32 || usn {
        v
    } else {
        let shift = 32 - bits;
        (((v << shift) as i32) >> shift) as u32
    }
}

/// The `j`th vector of an UNPACK's data in CMD's format (`vn`, `vl` from its low bits): §6.3.4's figures, V2's Z and W
/// as its X and Y and V3's W as 0 (D-180), V4-5's colour fields shifted up.
pub fn vector(cmd: u32, data: &[u32], j: u32, usn: bool) -> [u32; 4] {
    let (vn, vl) = (cmd >> 2 & 3, cmd & 3);
    if vl == 3 {
        let c = element(data, j, 16);
        return [(c & 31) << 3, (c >> 5 & 31) << 3, (c >> 10 & 31) << 3, (c >> 15) << 7];
    }
    let bits = 32 >> vl;
    let n = vn + 1;
    let e = |i: u32| extend(element(data, j * n + i, bits), bits, usn);
    match vn {
        0 => [e(0); 4],
        1 => [e(0), e(1), e(0), e(1)],
        2 => [e(0), e(1), e(2), 0],
        _ => [e(0), e(1), e(2), e(3)],
    }
}

/// The data words an UNPACK takes (§6.4's length formula, less the VIFcode).
pub fn data_words(cmd: u32, num: u32, cl: u32, wl: u32) -> u32 {
    let (vn, vl) = (cmd >> 2 & 3, cmd & 3);
    let n = if wl <= cl { num } else { cl * (num / wl) + (num % wl).min(cl) };
    (((32 >> vl) * (vn + 1)) * n).div_ceil(32)
}

/// An UNPACK's writes in order: each one's qword address before wrapping and its four fields (none: not written).
/// `num` is NUM with 0 as 256; `addr` ADDR with TOPS already added; `data` the stream from the UNPACK's first data
/// word to the end of what has arrived, which a filling write decompresses from without taking (D-183).
pub fn unpack(r: &mut Regs, cmd: u32, num: u32, addr: u32, usn: bool, data: &[u32]) -> Vec<(u32, [Option<u32>; 4])> {
    let masked = cmd & 0x10 != 0;
    let (cl, wl) = (r.cl, r.wl);
    let mut out = Vec::new();
    let mut read = 0;
    for k in 0..num {
        // WL 0: one block that never ends, every write reading (D-181).
        let cycle = if wl == 0 { k } else { k % wl };
        let (at, input) = if wl == 0 {
            (addr + k, true)
        } else if cl >= wl {
            (addr + cl * (k / wl) + cycle, true)
        } else {
            (addr + k, cycle < cl)
        };
        let d = vector(cmd, data, read, usn);
        if input {
            read += 1;
        }
        let row = cycle.min(3) as usize;
        let mut fields = [None; 4];
        for (f, field) in fields.iter_mut().enumerate() {
            let m = if masked { r.mask >> (2 * (4 * row + f)) & 3 } else { 0 };
            *field = match m {
                0 => Some(match r.mode {
                    0 => d[f],
                    1 => d[f].wrapping_add(r.row[f]),
                    2 => {
                        r.row[f] = r.row[f].wrapping_add(d[f]);
                        r.row[f]
                    }
                    _ => {
                        r.row[f] = d[f];
                        d[f]
                    }
                }),
                1 => Some(r.row[f]),
                2 => Some(r.col[row]),
                _ => None,
            };
        }
        out.push((at, fields));
    }
    out
}

/// A VU data memory as the programs see it, qword by qword, with UNPACK's writes applied wrapping at its size.
pub struct Memory(pub Vec<[u32; 4]>);

impl Memory {
    pub fn new(qwords: usize, fill: impl Fn(usize) -> [u32; 4]) -> Memory {
        Memory((0..qwords).map(fill).collect())
    }

    pub fn apply(&mut self, writes: &[(u32, [Option<u32>; 4])]) {
        let n = self.0.len() as u32;
        for (at, fields) in writes {
            let q = &mut self.0[(at % n) as usize];
            for f in 0..4 {
                if let Some(v) = fields[f] {
                    q[f] = v;
                }
            }
        }
    }
}

/// VIF1's double buffer (§6.3.8, VIF1_OFST's, VIF1_TOPS's and VIF1_STAT's pages): BASE, OFST, TOPS, TOP and DBF.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Double {
    pub base: u32,
    pub ofst: u32,
    pub tops: u32,
    pub top: u32,
    pub dbf: bool,
}

impl Double {
    /// OFFSET: OFST set, DBF cleared, TOPS = BASE (§6.4's page; D-184 for VIF1_OFST's page, which names TOP).
    pub fn offset(&mut self, v: u32) {
        self.ofst = v & 0x3FF;
        self.dbf = false;
        self.tops = self.base;
    }

    /// The MSCAL family: TOP = TOPS, DBF reversed, TOPS the buffer DBF now names (STAT's field: 0 BASE, 1
    /// BASE + OFFSET).
    pub fn start(&mut self) {
        self.top = self.tops;
        self.dbf = !self.dbf;
        self.tops = (if self.dbf { self.base + self.ofst } else { self.base }) & 0x3FF;
    }
}
