//! UNPACK (EE User's Manual §6.3.4 to §6.3.7, the UNPACK page of §6.4): the formats a VIFcode's low four CMD bits
//! name, each vector read from the packet's data as a little-endian bit stream, sign- or zero-extended by USN; the
//! write sequence the CYCLE register's CL and WL give, skipping or filling; and each field written as the MASK
//! register's specifier for its write cycle and the MODE register's addition say. Sapphire_Native.md §28.3 records the
//! readings the manual leaves open, Sapphire_Disputes.md D-180 on the ones the console texts decide.

/// The data's shape: elements a vector (1 to 4) and bits an element, or V4-5's packed colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Format {
    pub vn: u32,
    pub vl: u32,
}

impl Format {
    /// The format of CMD's low four bits (§6.3.4's table); vl = 3 is V4-5 with vn = 3 and undefined otherwise.
    pub fn of(cmd: u32) -> Option<Format> {
        let (vn, vl) = (cmd >> 2 & 3, cmd & 3);
        (vl != 3 || vn == 3).then_some(Format { vn, vl })
    }

    /// Bits one vector takes from the packet: (32 >> vl) × (vn + 1), V4-5's 16 included (§6.4's length formula).
    pub fn bits(self) -> u32 {
        (32 >> self.vl) * (self.vn + 1)
    }

    pub fn v45(self) -> bool {
        self.vl == 3
    }
}

/// A vector's four fields from its `bits()` low bits of `raw` (D-180 for the fields V2 and V3 leave indeterminate).
pub fn decode(f: Format, raw: u128, usn: bool) -> [u32; 4] {
    if f.v45() {
        let c = raw as u32;
        return [(c & 0x1F) << 3, (c >> 5 & 0x1F) << 3, (c >> 10 & 0x1F) << 3, (c >> 15 & 1) << 7];
    }
    let size = 32 >> f.vl;
    let element = |i: u32| {
        let e = (raw >> (i * size)) as u32;
        match size {
            32 => e,
            16 if usn => e & 0xFFFF,
            16 => e as u16 as i16 as i32 as u32,
            _ if usn => e & 0xFF,
            _ => e as u8 as i8 as i32 as u32,
        }
    };
    match f.vn {
        0 => [element(0); 4],
        1 => [element(0), element(1), element(0), element(1)],
        2 => [element(0), element(1), element(2), 0],
        _ => [element(0), element(1), element(2), element(3)],
    }
}

/// CYCLE's write sequence (§6.3.5): CL >= WL skips, CL < WL fills; WL 0 is a block that never ends, every write
/// reading a vector into the next qword (D-181).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cycle {
    pub cl: u32,
    pub wl: u32,
}

impl Cycle {
    pub fn of(reg: u32) -> Cycle {
        Cycle { cl: reg & 0xFF, wl: reg >> 8 & 0xFF }
    }

    /// WL as the write cycles count it: 0 counts as 256, so a CYCLE with WL 0 fills (Sapphire_Disputes.md D-181,
    /// amended at S10 by `dma/vif/unpack`'s console text).
    fn wl(self) -> u32 {
        if self.wl == 0 { 256 } else { self.wl }
    }

    pub fn filling(self) -> bool {
        self.cl < self.wl()
    }

    /// The `k`th write's place in its block.
    fn place(self, k: u32) -> u32 {
        k % self.wl()
    }

    /// The `k`th write's qword offset from the UNPACK's address.
    pub fn offset(self, k: u32) -> u32 {
        if self.filling() { k } else { self.cl * (k / self.wl()) + k % self.wl() }
    }

    /// Whether the `k`th write takes a vector from the packet: always when skipping, the first CL of each WL when
    /// filling.
    pub fn reads(self, k: u32) -> bool {
        !self.filling() || self.place(k) < self.cl
    }

    /// The `k`th write's cycle, 0 to 3, which picks its MASK row and its Col register (the fourth row for cycle 4 on).
    pub fn row(self, k: u32) -> usize {
        self.place(k).min(3) as usize
    }
}

/// The filling registers an UNPACK reads and, in difference mode and mode 3, writes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Fill {
    pub row: [u32; 4],
    pub col: [u32; 4],
    pub mask: u32,
    pub mode: u32,
}

/// One write's fields (§6.3.6, §6.3.7): each field's specifier from MASK (all 0 without the m bit); 0 writes the
/// decompressed data, added to its Row register in offset mode, added and kept in Row in difference mode, and kept in
/// Row unadded in mode 3 (D-182); 1 writes Row, 2 the write cycle's Col, 3 nothing. The fields to write come back as a
/// bit each, x lowest.
pub fn write(fill: &mut Fill, data: [u32; 4], masked: bool, row: usize) -> ([u32; 4], u32) {
    let mut out = [0; 4];
    let mut enable = 0;
    for f in 0..4 {
        let m = if masked { fill.mask >> (2 * (4 * row + f)) & 3 } else { 0 };
        let v = match m {
            0 => {
                let d = data[f];
                match fill.mode {
                    0 => d,
                    1 => d.wrapping_add(fill.row[f]),
                    2 => {
                        fill.row[f] = d.wrapping_add(fill.row[f]);
                        fill.row[f]
                    }
                    _ => {
                        fill.row[f] = d;
                        d
                    }
                }
            }
            1 => fill.row[f],
            2 => fill.col[row],
            _ => continue,
        };
        out[f] = v;
        enable |= 1 << f;
    }
    (out, enable)
}

/// An UNPACK in progress: its format, where it writes, and the data bits taken from the packet but not yet used.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unpack {
    pub format: Format,
    pub usn: bool,
    pub masked: bool,
    /// The first write's qword address (ADDR, with TOPS added under FLG on VIF1).
    pub addr: u32,
    pub num: u32,
    pub done: u32,
    pub cycle: Cycle,
    bits: u128,
    held: u32,
}

/// What the next write needs.
pub enum Next {
    /// A word more of the packet's data.
    Word,
    /// The write: its qword offset in VU memory (before wrapping), its fields and which of them to write.
    Write(u32, [u32; 4], u32),
}

impl Unpack {
    pub fn new(code: u32, cycle: u32, tops: Option<u32>) -> Option<Unpack> {
        let cmd = code >> 24;
        let format = Format::of(cmd)?;
        let num = match code >> 16 & 0xFF {
            0 => 256,
            n => n,
        };
        let imm = code & 0xFFFF;
        let addr = (imm & 0x3FF) + if imm & 0x8000 != 0 { tops.unwrap_or(0) } else { 0 };
        Some(Unpack { format, usn: imm & 0x4000 != 0, masked: cmd & 0x10 != 0, addr, num, done: 0, cycle: Cycle::of(cycle), bits: 0, held: 0 })
    }

    pub fn finished(&self) -> bool {
        self.done == self.num
    }

    /// Takes a word of the packet's data into the bit stream.
    pub fn take(&mut self, w: u32) {
        self.bits |= u128::from(w) << self.held;
        self.held += 32;
    }

    /// The next write, or the word it waits for; `fill` is the VIF's Row, Col, MASK and MODE. A filling write takes no
    /// vector but decompresses the one at the stream's pointer all the same (D-183): `peek(i)` is the `i`th word of
    /// the stream past the bits held, 0 where none has arrived.
    pub fn next(&mut self, fill: &mut Fill, peek: &dyn Fn(usize) -> u32) -> Next {
        let k = self.done;
        let need = self.format.bits();
        let reads = self.cycle.reads(k);
        if reads && self.held < need {
            return Next::Word;
        }
        let mut raw = self.bits;
        let mut have = self.held;
        let mut i = 0;
        while have < need {
            raw |= u128::from(peek(i)) << have;
            have += 32;
            i += 1;
        }
        let raw = if need == 128 { raw } else { raw & ((1u128 << need) - 1) };
        if reads {
            self.bits = if need == 128 { 0 } else { self.bits >> need };
            self.held -= need;
        }
        let (fields, enable) = write(fill, decode(self.format, raw, self.usn), self.masked, self.cycle.row(k));
        self.done += 1;
        if self.finished() {
            self.bits = 0;
            self.held = 0;
        }
        Next::Write(self.addr + self.cycle.offset(k), fields, enable)
    }
}

impl Unpack {
    /// An UNPACK in progress as `SPH2` holds it (Sapphire_Native.md §38.2): USN and m, eight words, the bits held.
    pub(super) fn saved(&self) -> ([bool; 2], [u32; 8], u128) {
        let f = &self.format;
        ([self.usn, self.masked], [f.vn, f.vl, self.addr, self.num, self.done, self.cycle.cl, self.cycle.wl, self.held], self.bits)
    }

    pub(super) fn restored(flags: [bool; 2], w: [u32; 8], bits: u128) -> Unpack {
        Unpack { format: Format { vn: w[0] & 3, vl: w[1] & 3 }, usn: flags[0], masked: flags[1], addr: w[2], num: w[3], done: w[4], cycle: Cycle { cl: w[5], wl: w[6] }, bits, held: w[7] }
    }
}
