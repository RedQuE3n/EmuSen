//! The EE User's Manual's chapter 7, written apart from `src/gif/`: what a GS packet sends the GS, primitive by
//! primitive, by §7.2's GIFtag, §7.3's PACKED formats read at the bit positions their figures print, §7.4's REGLIST and
//! §7.5's IMAGE; the arbitration's order is each program's own, stated where it builds the record.

#![allow(dead_code)]

/// The bits `hi` to `lo` of a qword, as the manual's figures number them.
pub fn field(q: u128, hi: u32, lo: u32) -> u64 {
    ((q >> lo) & ((1u128 << (hi - lo + 1)) - 1)) as u64
}

/// The GIF's state that outlives a packet: Q (§7.3.2's RGBAQ and ST).
pub struct Gif {
    pub q: u64,
}

impl Default for Gif {
    fn default() -> Gif {
        Gif { q: 0x3F80_0000 }
    }
}

impl Gif {
    /// The GS writes of the packet starting at `data[0]`, in order, HWREG (54h) for image data, and the qwords it took:
    /// up to its EOP primitive, or every whole primitive `data` holds.
    pub fn packet(&mut self, data: &[u128]) -> (Vec<(u8, u64)>, usize) {
        let mut out = Vec::new();
        let mut i = 0;
        while i < data.len() {
            let tag = data[i];
            i += 1;
            let nloop = field(tag, 14, 0) as usize;
            let eop = field(tag, 15, 15) == 1;
            let nreg = match field(tag, 63, 60) as usize {
                0 => 16,
                n => n,
            };
            let desc = |k: usize| field(tag, 64 + 4 * k as u32 + 3, 64 + 4 * k as u32) as u8;
            self.q = 0x3F80_0000;
            if nloop > 0 {
                match field(tag, 59, 58) {
                    0 => {
                        if field(tag, 46, 46) == 1 {
                            out.push((0x00, field(tag, 57, 47)));
                        }
                        for _ in 0..nloop {
                            for k in 0..nreg {
                                self.packed(desc(k), data[i], &mut out);
                                i += 1;
                            }
                        }
                    }
                    1 => {
                        let n = nloop * nreg;
                        for j in 0..n {
                            let w = data[i + j / 2];
                            let half = if j % 2 == 0 { field(w, 63, 0) } else { field(w, 127, 64) };
                            let d = desc(j % nreg);
                            if d != 0x0E && d != 0x0F {
                                out.push((d, half));
                            }
                        }
                        i += n.div_ceil(2);
                    }
                    _ => {
                        for _ in 0..nloop {
                            out.push((0x54, field(data[i], 63, 0)));
                            out.push((0x54, field(data[i], 127, 64)));
                            i += 1;
                        }
                    }
                }
            }
            if eop {
                break;
            }
        }
        (out, i)
    }

    /// One PACKED qword under descriptor `d`, §7.3.2's figure for it.
    fn packed(&mut self, d: u8, w: u128, out: &mut Vec<(u8, u64)>) {
        match d {
            0x00 => out.push((0x00, field(w, 10, 0))),
            0x01 => out.push((0x01, field(w, 7, 0) | field(w, 39, 32) << 8 | field(w, 71, 64) << 16 | field(w, 103, 96) << 24 | self.q << 32)),
            0x02 => {
                out.push((0x02, field(w, 63, 0)));
                self.q = field(w, 95, 64);
            }
            0x03 => out.push((0x03, field(w, 13, 0) | field(w, 45, 32) << 16)),
            0x04 => {
                let to = if field(w, 111, 111) == 1 { 0x0C } else { 0x04 };
                out.push((to, field(w, 15, 0) | field(w, 47, 32) << 16 | field(w, 91, 68) << 32 | field(w, 107, 100) << 56));
            }
            0x05 => {
                let to = if field(w, 111, 111) == 1 { 0x0D } else { 0x05 };
                out.push((to, field(w, 15, 0) | field(w, 47, 32) << 16 | field(w, 95, 64) << 32));
            }
            0x0A => out.push((0x0A, field(w, 107, 100) << 56)),
            0x0E => out.push((field(w, 71, 64) as u8, field(w, 63, 0))),
            0x0F => {}
            _ => out.push((d, field(w, 63, 0))),
        }
    }
}

/// A GIFtag (§7.2.2): NLOOP, EOP, the PRIM field when PRE is to be 1, FLG, and the register descriptors (NREG their
/// count, 16 written as 0).
pub fn tag(nloop: u32, eop: bool, prim: Option<u32>, flg: u32, regs: &[u8]) -> u128 {
    assert!(regs.len() <= 16 && nloop < 0x8000);
    let mut lo = u64::from(nloop) | u64::from(eop) << 15 | u64::from(flg & 3) << 58 | ((regs.len() as u64) & 0xF) << 60;
    if let Some(p) = prim {
        lo |= 1 << 46 | u64::from(p & 0x7FF) << 47;
    }
    let hi = regs.iter().enumerate().fold(0u64, |a, (k, &r)| a | u64::from(r & 0xF) << (4 * k));
    u128::from(lo) | u128::from(hi) << 64
}

/// An A+D qword (§7.3.2): the data and the register's address.
pub fn ad(reg: u8, v: u64) -> u128 {
    u128::from(v) | u128::from(reg) << 64
}

/// A qword from four words, the first in bits 31..0.
pub fn quad(w: [u32; 4]) -> u128 {
    w.iter().rev().fold(0u128, |a, &x| a << 32 | u128::from(x))
}

/// A packet of A+D register writes, one primitive with EOP.
pub fn regs_packet(regs: &[(u8, u64)]) -> Vec<u128> {
    let mut p = vec![tag(regs.len() as u32, true, None, 0, &[0x0E])];
    p.extend(regs.iter().map(|&(r, v)| ad(r, v)));
    p
}
