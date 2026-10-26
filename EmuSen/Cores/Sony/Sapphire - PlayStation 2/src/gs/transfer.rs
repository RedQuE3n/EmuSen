//! The transfers between buffers (GS User's Manual chapter 4): host to local through HWREG, local to host through the
//! reversed host interface, and local to local, each started by TRXDIR with BITBLTBUF, TRXPOS and TRXREG as they stand
//! then. Transfer data is one stream of pixels packed from bit 0 upwards in 64-bit units (§4.3), so a row may end inside
//! a unit and the next row continues in it; coordinates wrap at 2048 (§4.1.3). Sapphire_Native.md §24.3.

use super::local::Memory;
use super::psm::Psm;

/// A buffer as BITBLTBUF names one: format, base pointer in blocks, width in units of 64 pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Buffer {
    pub psm: Psm,
    pub bp: u32,
    pub bw: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    HostToLocal,
    LocalToHost,
}

/// BITBLTBUF's source and destination (§7.1); `None` for a format outside the table, whose operation is undefined.
pub fn buffers(bitbltbuf: u64) -> (Option<Buffer>, Option<Buffer>) {
    let at = |s: u32| Psm::from_code((bitbltbuf >> (s + 24)) as u32 & 0x3F).map(|psm| Buffer { psm, bp: (bitbltbuf >> s) as u32 & 0x3FFF, bw: (bitbltbuf >> (s + 16)) as u32 & 0x3F });
    (at(0), at(32))
}

/// TRXPOS's fields: source X and Y, destination X and Y, and the local-to-local order.
pub fn positions(trxpos: u64) -> (u32, u32, u32, u32, u32) {
    let f = |s: u32| (trxpos >> s) as u32 & 0x7FF;
    (f(0), f(16), f(32), f(48), (trxpos >> 59) as u32 & 3)
}

/// TRXREG's width and height.
pub fn size(trxreg: u64) -> (u32, u32) {
    (trxreg as u32 & 0xFFF, (trxreg >> 32) as u32 & 0xFFF)
}

/// A transfer in progress, or none.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Transfer {
    pub active: Option<Direction>,
    buffer: Option<Buffer>,
    x0: u32,
    y0: u32,
    w: u32,
    h: u32,
    x: u32,
    y: u32,
    /// Bits of transfer data not yet consumed (host to local) or not yet handed over (local to host), from bit 0.
    acc: u128,
    held: u32,
}

impl Transfer {
    /// TRXDIR written with `xdir` and the registers as they stand: a transfer ends and another begins (§4.1.4,
    /// §4.2.1); a local-to-local one runs to its end at once.
    pub fn start(&mut self, xdir: u32, bitbltbuf: u64, trxpos: u64, trxreg: u64, local: &mut impl Memory) {
        *self = Transfer::default();
        let (src, dst) = buffers(bitbltbuf);
        let (sx, sy, dx, dy, order) = positions(trxpos);
        let (w, h) = size(trxreg);
        let (buffer, x0, y0, direction) = match xdir & 3 {
            0 => (dst, dx, dy, Direction::HostToLocal),
            1 => (src, sx, sy, Direction::LocalToHost),
            2 => {
                if let (Some(s), Some(d)) = (src, dst) {
                    copy(local, s, (sx, sy), d, (dx, dy), (w, h), order);
                }
                return;
            }
            _ => return,
        };
        if buffer.is_some() && w > 0 && h > 0 {
            *self = Transfer { active: Some(direction), buffer, x0, y0, w, h, ..Transfer::default() };
        }
    }

    fn step(&mut self) {
        self.x += 1;
        if self.x == self.w {
            self.x = 0;
            self.y += 1;
            if self.y == self.h {
                self.active = None;
            }
        }
    }

    fn here(&self) -> (u32, u32) {
        ((self.x0 + self.x) & 0x7FF, (self.y0 + self.y) & 0x7FF)
    }

    /// 64 bits of host-to-local data (§4.2.1); outside such a transfer, nothing (HWREG's page). The bits left in the
    /// last unit after the last pixel are dropped.
    pub fn hwreg(&mut self, data: u64, local: &mut impl Memory) {
        if self.active != Some(Direction::HostToLocal) {
            return;
        }
        let b = self.buffer.expect("an active transfer has its buffer");
        let bits = b.psm.bits();
        self.acc |= u128::from(data) << self.held;
        self.held += 64;
        while self.held >= bits && self.active.is_some() {
            let (x, y) = self.here();
            local.write(b.psm, b.bp, b.bw, x, y, (self.acc & ((1 << bits) - 1)) as u32);
            self.acc >>= bits;
            self.held -= bits;
            self.step();
        }
        if self.active.is_none() {
            self.acc = 0;
            self.held = 0;
        }
    }

    /// The next 64 bits of local-to-host data, the last unit filled with zeros above the last pixel [A]; none once
    /// the transfer has been read to its end or outside one.
    pub fn read(&mut self, local: &mut impl Memory) -> Option<u64> {
        if self.active != Some(Direction::LocalToHost) && self.held == 0 {
            return None;
        }
        if let Some(b) = self.buffer {
            let bits = b.psm.bits();
            while self.held < 64 && self.active.is_some() {
                let (x, y) = self.here();
                self.acc |= u128::from(local.read(b.psm, b.bp, b.bw, x, y)) << self.held;
                self.held += bits;
                self.step();
            }
        }
        let out = self.acc as u64;
        self.acc >>= 64;
        self.held = self.held.saturating_sub(64);
        Some(out)
    }

    /// A host-to-local transfer's buffer and the pixels it can still write, as ranges of x and y (the whole of an axis
    /// on which its rectangle wraps at 2048).
    pub fn destination(&self) -> Option<(Buffer, (u32, u32), (u32, u32))> {
        let b = self.buffer.filter(|_| self.active == Some(Direction::HostToLocal))?;
        let range = |o: u32, n: u32| if o + n > 0x800 { (0, 0x7FF) } else { (o, o + n - 1) };
        Some((b, range(self.x0, self.w), range(self.y0, self.h)))
    }

    /// Ends whatever transfer is in progress (TRXDIR 3, or a reset).
    pub fn end(&mut self) {
        *self = Transfer::default();
    }
}

/// A memory that keeps nothing: the GS's thread follows a transfer's place with it while a worker writes its pixels.
pub struct Dry;

impl Memory for Dry {
    fn read(&mut self, _: Psm, _: u32, _: u32, _: u32, _: u32) -> u32 {
        0
    }

    fn write(&mut self, _: Psm, _: u32, _: u32, _: u32, _: u32, _: u32) {}
}

/// A local-to-local transfer, pixel by pixel in TRXPOS's order (§4.1.2's figure 4-1: bit 0 bottom to top, bit 1 right
/// to left), each pixel read in the source's format and written in the destination's. Overlapping rectangles are
/// copied in that order (Sapphire_Disputes.md D-107, decided at S19, Sapphire_Native.md §37.4).
pub fn copy(local: &mut impl Memory, src: Buffer, (sx, sy): (u32, u32), dst: Buffer, (dx, dy): (u32, u32), (w, h): (u32, u32), order: u32) {
    for j in 0..h {
        let ry = if order & 1 != 0 { h - 1 - j } else { j };
        for i in 0..w {
            let rx = if order & 2 != 0 { w - 1 - i } else { i };
            let v = local.read(src.psm, src.bp, src.bw, (sx + rx) & 0x7FF, (sy + ry) & 0x7FF);
            local.write(dst.psm, dst.bp, dst.bw, (dx + rx) & 0x7FF, (dy + ry) & 0x7FF, v);
        }
    }
}

/// A transfer's block of `SPH2` (Sapphire_Native.md §38.2): its direction, buffer, rectangle, place and the bits held.
impl emusen_native::State for Transfer {
    type Error = emusen_native::Truncated;

    fn write_state(&self, w: &mut emusen_native::StateWriter) {
        w.u8("Active", match self.active {
            None => 0,
            Some(Direction::HostToLocal) => 1,
            Some(Direction::LocalToHost) => 2,
        });
        let b = self.buffer;
        w.u8("Psm", b.map_or(0xFF, |b| b.psm.code() as u8));
        w.u32s("Buffer", &[b.map_or(0, |b| b.bp), b.map_or(0, |b| b.bw)]);
        w.u32s("Rectangle", &[self.x0, self.y0, self.w, self.h, self.x, self.y]);
        w.u128("Bits", self.acc);
        w.u32("Held", self.held);
    }

    fn read_state(&mut self, r: &mut emusen_native::StateReader) -> Result<(), emusen_native::Truncated> {
        self.active = match r.u8()? {
            1 => Some(Direction::HostToLocal),
            2 => Some(Direction::LocalToHost),
            _ => None,
        };
        let psm = Psm::from_code(u32::from(r.u8()?));
        let mut b = [0u32; 2];
        r.u32s(&mut b)?;
        self.buffer = psm.map(|psm| Buffer { psm, bp: b[0], bw: b[1] });
        let mut v = [0u32; 6];
        r.u32s(&mut v)?;
        [self.x0, self.y0, self.w, self.h, self.x, self.y] = v;
        self.acc = r.u128()?;
        self.held = r.u32()?;
        Ok(())
    }
}
