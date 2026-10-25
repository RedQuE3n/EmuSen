//! The GIFtag and the three data formats (EE User's Manual §7.2 to §7.5): the one owner of how a GS packet's qwords
//! become register writes and image data on the GS's host interface, and of what each costs in bus cycles
//! (Sapphire_Native.md §33.3, §33.4). A `Packet` is one path's place in its GS packet; the GIF keeps one a path, so
//! PATH3's IMAGE data interrupted in the intermittent mode resumes where it stopped.

use crate::gs::reg;

/// Q's value after a GIFtag is read, 1.0 (§7.3.2's RGBAQ: "The initial value is 1.0, and the Q value is initialized
/// whenever the GIFtag is read").
pub const Q_ONE: u32 = 0x3F80_0000;

/// The format a GIFtag gives its data (§7.2.2's FLG; 11 "Same operation with the IMAGE mode").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Flg {
    #[default]
    Packed,
    Reglist,
    Image,
}

/// A GIFtag's fields (§7.2.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Tag {
    pub nloop: u16,
    pub eop: bool,
    pub pre: bool,
    pub prim: u16,
    pub flg: Flg,
    /// The register descriptors, 1 to 16 (NREG 0 is 16).
    pub nreg: u8,
    pub regs: u64,
}

impl Tag {
    pub fn of(q: u128) -> Tag {
        let lo = q as u64;
        Tag {
            nloop: (lo & 0x7FFF) as u16,
            eop: lo >> 15 & 1 == 1,
            pre: lo >> 46 & 1 == 1,
            prim: (lo >> 47 & 0x7FF) as u16,
            flg: match lo >> 58 & 3 {
                0 => Flg::Packed,
                1 => Flg::Reglist,
                _ => Flg::Image,
            },
            nreg: match (lo >> 60) as u8 {
                0 => 16,
                n => n,
            },
            regs: (q >> 64) as u64,
        }
    }

    /// The `i`th register descriptor, from bits 67..64 up (§7.2.3).
    pub fn desc(&self, i: u8) -> u8 {
        (self.regs >> (4 * u32::from(i)) & 0xF) as u8
    }
}

/// Where the GIF's output goes: the GS's host interface (as `gif::Gif` adapts it), or a counter.
pub trait Out {
    fn write(&mut self, reg: u8, data: u64);
}

/// An output that drops everything, for counting a packet's cycles.
pub struct Nowhere;

impl Out for Nowhere {
    fn write(&mut self, _: u8, _: u64) {}
}

/// What one qword did.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Step {
    /// Bus cycles the qword occupies the GIF (§33.4).
    pub cycles: u64,
    /// The qword was a GIFtag.
    pub tag: bool,
    /// The GS packet ended with it (the primitive of an EOP tag ended).
    pub end: bool,
}

/// One path's place in a GS packet (§7.2.1): the primitive in progress and its counters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Packet {
    pub tag: Tag,
    /// A primitive's data is expected: a tag with NLOOP above 0 was read and loops remain.
    pub open: bool,
    /// GIF_CNT's LOOPCNT: the loops left, counted down as each loop's last descriptor is taken.
    pub loops: u16,
    /// The descriptor the next datum takes, 0 to NREG - 1.
    pub reg: u8,
    /// The primitive's data qwords taken so far (§7.1.3's slices of IMAGE data).
    pub taken: u32,
}

impl Packet {
    /// Takes one qword: a GIFtag when no primitive is open, else the primitive's data, written to `out`; `q` is the
    /// GIF's Q register (§7.3.2).
    pub fn take(&mut self, w: u128, q: &mut u32, out: &mut dyn Out) -> Step {
        if !self.open {
            let t = Tag::of(w);
            *self = Packet { tag: t, open: t.nloop != 0, loops: t.nloop, reg: 0, taken: 0 };
            *q = Q_ONE;
            if t.nloop != 0 && t.flg == Flg::Packed && t.pre {
                out.write(reg::PRIM, u64::from(t.prim));
            }
            return Step { cycles: 1, tag: true, end: t.nloop == 0 && t.eop };
        }
        self.taken += 1;
        let cycles = match self.tag.flg {
            Flg::Packed => {
                packed(self.tag.desc(self.reg), w, q, out);
                self.advance();
                1
            }
            Flg::Reglist => {
                let mut n = 0;
                for half in [w as u64, (w >> 64) as u64] {
                    if self.loops == 0 {
                        break;
                    }
                    let d = self.tag.desc(self.reg);
                    if d < 0xE {
                        out.write(d, half);
                    }
                    self.advance();
                    n += 1;
                }
                n
            }
            Flg::Image => {
                out.write(reg::HWREG, w as u64);
                out.write(reg::HWREG, (w >> 64) as u64);
                self.loops -= 1;
                2
            }
        };
        if self.loops == 0 {
            self.open = false;
        }
        Step { cycles, tag: false, end: !self.open && self.tag.eop }
    }

    fn advance(&mut self) {
        self.reg += 1;
        if self.reg == self.tag.nreg {
            self.reg = 0;
            self.loops -= 1;
        }
    }

    /// Bus cycles the open primitive's data still needs at full rate.
    pub fn remaining(&self) -> u64 {
        if !self.open {
            return 0;
        }
        let left = u64::from(self.loops) * u64::from(self.tag.nreg) - u64::from(self.reg);
        match self.tag.flg {
            Flg::Packed | Flg::Reglist => left,
            Flg::Image => 2 * u64::from(self.loops),
        }
    }

    /// GIF_CNT (§7.6): LOOPCNT, REGCNT (the descriptor in process, 1 the lowest and 0 the sixteenth) and VUADDR.
    pub fn cnt(&self, vuaddr: u16) -> u32 {
        u32::from(self.loops) | u32::from((self.reg + 1) & 0xF) << 16 | u32::from(vuaddr & 0x3FF) << 20
    }
}

/// One PACKED qword under descriptor `d` (§7.3.2's packing formats).
fn packed(d: u8, w: u128, q: &mut u32, out: &mut dyn Out) {
    let f = |i: u32| (w >> (32 * i)) as u32 as u64;
    match d {
        0x0 => out.write(reg::PRIM, f(0) & 0x7FF),
        0x1 => out.write(reg::RGBAQ, f(0) & 0xFF | (f(1) & 0xFF) << 8 | (f(2) & 0xFF) << 16 | (f(3) & 0xFF) << 24 | u64::from(*q) << 32),
        0x2 => {
            out.write(reg::ST, w as u64);
            *q = f(2) as u32;
        }
        0x3 => out.write(reg::UV, f(0) & 0x3FFF | (f(1) & 0x3FFF) << 16),
        0x4 => {
            let to = if f(3) >> 15 & 1 == 1 { reg::XYZF3 } else { reg::XYZF2 };
            out.write(to, f(0) & 0xFFFF | (f(1) & 0xFFFF) << 16 | (f(2) >> 4 & 0xFF_FFFF) << 32 | (f(3) >> 4 & 0xFF) << 56);
        }
        0x5 => {
            let to = if f(3) >> 15 & 1 == 1 { reg::XYZ3 } else { reg::XYZ2 };
            out.write(to, f(0) & 0xFFFF | (f(1) & 0xFFFF) << 16 | f(2) << 32);
        }
        0xA => out.write(reg::FOG, (f(3) >> 4 & 0xFF) << 56),
        0xE => out.write(f(2) as u8, w as u64),
        0xF => {}
        _ => out.write(d, w as u64),
    }
}

/// The bus cycles of the GS packet whose first qword `read` gives at `from`, each qword read at its index (the
/// caller wraps it), or none when no EOP ends it within `limit` qwords.
pub fn cycles(mut read: impl FnMut(u32) -> u128, from: u32, limit: u32) -> Option<u64> {
    let (mut p, mut q, mut total) = (Packet::default(), Q_ONE, 0);
    for i in 0..limit {
        let s = p.take(read(from.wrapping_add(i)), &mut q, &mut Nowhere);
        total += s.cycles;
        if s.end {
            return Some(total);
        }
    }
    None
}

/// A path's packet in `SPH2` (Sapphire_Native.md §38.2): its GIFtag's fields and how far it has gone.
impl emusen_native::State for Packet {
    type Error = emusen_native::Truncated;

    fn write_state(&self, w: &mut emusen_native::StateWriter) {
        let t = &self.tag;
        let flg = match t.flg {
            Flg::Packed => 0,
            Flg::Reglist => 1,
            Flg::Image => 2,
        };
        w.u16s("Counts", &[t.nloop, t.prim, self.loops]);
        w.u8("Bits", u8::from(t.eop) | u8::from(t.pre) << 1 | u8::from(self.open) << 2 | flg << 3);
        w.bytes("Regs", &[t.nreg, self.reg]);
        w.u64("RegList", t.regs);
        w.u32("Taken", self.taken);
    }

    fn read_state(&mut self, r: &mut emusen_native::StateReader) -> Result<(), emusen_native::Truncated> {
        let mut c = [0u16; 3];
        r.u16s(&mut c)?;
        let bits = r.u8()?;
        let mut regs = [0u8; 2];
        r.bytes(&mut regs)?;
        let flg = match bits >> 3 & 3 {
            1 => Flg::Reglist,
            2 => Flg::Image,
            _ => Flg::Packed,
        };
        self.tag = Tag { nloop: c[0], eop: bits & 1 != 0, pre: bits & 2 != 0, prim: c[1], flg, nreg: regs[0], regs: r.u64()? };
        (self.loops, self.open, self.reg) = (c[2], bits & 4 != 0, regs[1]);
        self.taken = r.u32()?;
        Ok(())
    }
}
