//! The GS User's Manual's record of local memory and the transfers, written apart from `src/gs/`: the page, block and
//! column figures of chapter 8 transcribed as they are printed and read cell by cell, the transfer data of §4.3 as a
//! stream of bits, and the events of chapter 6 and §7.2. The expected records of the S16 programs come from here.

#![allow(dead_code)]

/// A format by its BITBLTBUF code (§7.1).
pub const CT32: u32 = 0x00;
pub const CT24: u32 = 0x01;
pub const CT16: u32 = 0x02;
pub const CT16S: u32 = 0x0A;
pub const T8: u32 = 0x13;
pub const T4: u32 = 0x14;
pub const T8H: u32 = 0x1B;
pub const T4HL: u32 = 0x24;
pub const T4HH: u32 = 0x2C;
pub const Z32: u32 = 0x30;
pub const Z24: u32 = 0x31;
pub const Z16: u32 = 0x32;
pub const Z16S: u32 = 0x3A;

pub const FORMATS: [u32; 13] = [CT32, CT24, CT16, CT16S, T8, T4, T8H, T4HL, T4HH, Z32, Z24, Z16, Z16S];

pub fn name(psm: u32) -> &'static str {
    match psm {
        CT32 => "PSMCT32",
        CT24 => "PSMCT24",
        CT16 => "PSMCT16",
        CT16S => "PSMCT16S",
        T8 => "PSMT8",
        T4 => "PSMT4",
        T8H => "PSMT8H",
        T4HL => "PSMT4HL",
        T4HH => "PSMT4HH",
        Z32 => "PSMZ32",
        Z24 => "PSMZ24",
        Z16 => "PSMZ16",
        _ => "PSMZ16S",
    }
}

/// §8.3's block figures, as printed (the block's number at each place in the page, row by row).
const BLOCKS_CT32: &str = "0 1 4 5 16 17 20 21 / 2 3 6 7 18 19 22 23 / 8 9 12 13 24 25 28 29 / 10 11 14 15 26 27 30 31";
const BLOCKS_Z32: &str = "24 25 28 29 8 9 12 13 / 26 27 30 31 10 11 14 15 / 16 17 20 21 0 1 4 5 / 18 19 22 23 2 3 6 7";
const BLOCKS_CT16: &str = "0 2 8 10 / 1 3 9 11 / 4 6 12 14 / 5 7 13 15 / 16 18 24 26 / 17 19 25 27 / 20 22 28 30 / 21 23 29 31";
const BLOCKS_Z16: &str = "24 26 16 18 / 25 27 17 19 / 28 30 20 22 / 29 31 21 23 / 8 10 0 2 / 9 11 1 3 / 12 14 4 6 / 13 15 5 7";
const BLOCKS_CT16S: &str = "0 2 16 18 / 1 3 17 19 / 8 10 24 26 / 9 11 25 27 / 4 6 20 22 / 5 7 21 23 / 12 14 28 30 / 13 15 29 31";
const BLOCKS_Z16S: &str = "24 26 8 10 / 25 27 9 11 / 16 18 0 2 / 17 19 1 3 / 28 30 12 14 / 29 31 13 15 / 20 22 4 6 / 21 23 5 7";
const BLOCKS_T8: &str = "0 1 4 5 16 17 20 21 / 2 3 6 7 18 19 22 23 / 8 9 12 13 24 25 28 29 / 10 11 14 15 26 27 30 31";
const BLOCKS_T4: &str = "0 2 8 10 / 1 3 9 11 / 4 6 12 14 / 5 7 13 15 / 16 18 24 26 / 17 19 25 27 / 20 22 28 30 / 21 23 29 31";

/// §8.3's column figures: the word each pixel of a column takes, row by row.
const COLUMN_32: &str = "0 1 4 5 8 9 12 13 / 2 3 6 7 10 11 14 15";
const COLUMN_16: &str = "0 1 4 5 8 9 12 13 0 1 4 5 8 9 12 13 / 2 3 6 7 10 11 14 15 2 3 6 7 10 11 14 15";
const COLUMN_8_EVEN: &str = "0 1 4 5 8 9 12 13 0 1 4 5 8 9 12 13 / 2 3 6 7 10 11 14 15 2 3 6 7 10 11 14 15 / \
                             8 9 12 13 0 1 4 5 8 9 12 13 0 1 4 5 / 10 11 14 15 2 3 6 7 10 11 14 15 2 3 6 7";
const COLUMN_8_ODD: &str = "8 9 12 13 0 1 4 5 8 9 12 13 0 1 4 5 / 10 11 14 15 2 3 6 7 10 11 14 15 2 3 6 7 / \
                            0 1 4 5 8 9 12 13 0 1 4 5 8 9 12 13 / 2 3 6 7 10 11 14 15 2 3 6 7 10 11 14 15";

pub fn grid(text: &str) -> Vec<Vec<u32>> {
    text.split('/').map(|row| row.split_whitespace().map(|n| n.parse().unwrap()).collect()).collect()
}

/// The 4-bit columns' rows are the 8-bit columns' halves repeated (§8.3.4's figure): the same words for pixels 0 to 15
/// and 16 to 31 of a row.
fn column_4(odd: bool) -> Vec<Vec<u32>> {
    let eight = grid(if odd { COLUMN_8_ODD } else { COLUMN_8_EVEN });
    eight.iter().map(|row| row.iter().chain(row.iter()).copied().collect()).collect()
}

/// A format's geometry as §8.1 and §8.3 give it: page and block sizes in pixels, column height, block figure.
pub struct Geometry {
    pub page: (u32, u32),
    pub block: (u32, u32),
    pub column_rows: u32,
    pub blocks: &'static str,
}

pub fn geometry(psm: u32) -> Geometry {
    let (page, block, column_rows, blocks) = match psm {
        CT32 | CT24 | T8H | T4HL | T4HH => ((64, 32), (8, 8), 2, BLOCKS_CT32),
        Z32 | Z24 => ((64, 32), (8, 8), 2, BLOCKS_Z32),
        CT16 => ((64, 64), (16, 8), 2, BLOCKS_CT16),
        CT16S => ((64, 64), (16, 8), 2, BLOCKS_CT16S),
        Z16 => ((64, 64), (16, 8), 2, BLOCKS_Z16),
        Z16S => ((64, 64), (16, 8), 2, BLOCKS_Z16S),
        T8 => ((128, 64), (16, 16), 4, BLOCKS_T8),
        T4 => ((128, 128), (32, 16), 4, BLOCKS_T4),
        _ => panic!("no format {psm:x}"),
    };
    Geometry { page, block, column_rows, blocks }
}

/// The bits of a pixel (§2.4.5, §2.6.3, §4.3).
pub fn bits(psm: u32) -> u32 {
    match psm {
        CT32 | Z32 => 32,
        CT24 | Z24 => 24,
        CT16 | CT16S | Z16 | Z16S => 16,
        T8 | T8H => 8,
        _ => 4,
    }
}

/// The word (of the 1 Mi) and the lowest bit of pixel (x, y) of a buffer at base pointer `bp` and width `bw`.
pub fn locate(psm: u32, bp: u32, bw: u32, x: u32, y: u32) -> (usize, u32) {
    let g = geometry(psm);
    let pages_across = bw * 64 / g.page.0;
    let page = (y / g.page.1) * pages_across + x / g.page.0;
    let (px, py) = (x % g.page.0, y % g.page.1);
    let block = grid(g.blocks)[(py / g.block.1) as usize][(px / g.block.0) as usize];
    let (bx, by) = (px % g.block.0, py % g.block.1);
    let column = by / g.column_rows;
    let row = (by % g.column_rows) as usize;
    let (word, lane) = match psm {
        CT32 | CT24 | Z32 | Z24 => (grid(COLUMN_32)[row][bx as usize], 0),
        T8H => (grid(COLUMN_32)[row][bx as usize], 24),
        T4HL => (grid(COLUMN_32)[row][bx as usize], 24),
        T4HH => (grid(COLUMN_32)[row][bx as usize], 28),
        CT16 | CT16S | Z16 | Z16S => (grid(COLUMN_16)[row][bx as usize], if bx < 8 { 0 } else { 16 }),
        T8 => {
            let g = grid(if column % 2 == 1 { COLUMN_8_ODD } else { COLUMN_8_EVEN });
            // Rows 0 and 1 carry bits 0-7 (left) and 16-23 (right); rows 2 and 3 bits 8-15 and 24-31.
            let lane = if row < 2 { 0 } else { 8 } + if bx < 8 { 0 } else { 16 };
            (g[row][bx as usize], lane)
        }
        _ => {
            // Rows 0 and 1: bits 0-3, 8-11, 16-19, 24-27 by eights of pixels; rows 2 and 3: bits 4-7, 12-15, ...
            let lane = (bx / 8) * 8 + if row < 2 { 0 } else { 4 };
            (column_4(column % 2 == 1)[row][bx as usize], lane)
        }
    };
    let block_address = (bp + page * 32 + block) % (1 << 14);
    ((block_address * 64 + column * 16 + word) as usize, lane)
}

pub fn mask(bits: u32) -> u32 {
    if bits == 32 { u32::MAX } else { (1 << bits) - 1 }
}

/// Local memory as the manual describes it, with the transfer registers and the events.
#[derive(Clone)]
pub struct Gs {
    pub mem: Vec<u32>,
    pub bitbltbuf: u64,
    pub trxpos: u64,
    pub trxreg: u64,
    /// The pixels of a host-to-local transfer still to come, and the bits of data not yet used.
    incoming: Vec<(u32, u32, u32, u32, u32)>,
    bits_in: Vec<bool>,
    /// The data of a local-to-host transfer, as 64-bit units.
    outgoing: Vec<u64>,
    pub csr: u32,
    pub imr: u32,
    pub siglblid: u64,
    pub busdir: u64,
    pub pending_signal: Option<u64>,
    pub held: Vec<(u8, u64)>,
    pub privileged: std::collections::HashMap<u32, u64>,
    /// I_STAT's GS bit, as the INTC would latch it.
    pub intc: bool,
    line: bool,
}

impl Default for Gs {
    fn default() -> Gs {
        Gs {
            mem: vec![0; 1 << 20],
            bitbltbuf: 0,
            trxpos: 0,
            trxreg: 0,
            incoming: Vec::new(),
            bits_in: Vec::new(),
            outgoing: Vec::new(),
            csr: 0,
            imr: 0x7F00,
            siglblid: 0,
            busdir: 0,
            pending_signal: None,
            held: Vec::new(),
            privileged: Default::default(),
            intc: false,
            line: false,
        }
    }
}

fn field(v: u64, lo: u32, width: u32) -> u32 {
    ((v >> lo) & ((1 << width) - 1)) as u32
}

impl Gs {
    pub fn get(&self, psm: u32, bp: u32, bw: u32, x: u32, y: u32) -> u32 {
        let (w, lane) = locate(psm, bp, bw, x, y);
        (self.mem[w] >> lane) & mask(bits(psm))
    }

    pub fn put(&mut self, psm: u32, bp: u32, bw: u32, x: u32, y: u32, v: u32) {
        let (w, lane) = locate(psm, bp, bw, x, y);
        let m = mask(bits(psm)) << lane;
        self.mem[w] = (self.mem[w] & !m) | ((v << lane) & m);
    }

    /// The rectangle's pixels in §4.1.2's order 0, left to right and top to bottom, wrapping at 2048 (§4.1.3).
    fn rectangle(x0: u32, y0: u32, w: u32, h: u32) -> Vec<(u32, u32)> {
        (0..h).flat_map(|j| (0..w).map(move |i| ((x0 + i) % 2048, (y0 + j) % 2048))).collect()
    }

    fn line_now(&self) -> bool {
        (self.csr & 0x1F) & !(self.imr >> 8) & 0x1F != 0
    }

    fn sample(&mut self) {
        let l = self.line_now();
        if l && !self.line {
            self.intc = true;
        }
        self.line = l;
    }

    fn signal(&mut self, v: u64) {
        let (id, m) = (v as u32, (v >> 32) as u32);
        let sig = (self.siglblid as u32 & !m) | (id & m);
        self.siglblid = (self.siglblid & 0xFFFF_FFFF_0000_0000) | u64::from(sig);
        self.csr |= 1;
    }

    /// An event raised by the PCRTC (HSINT, VSINT), by its CSR bit.
    pub fn event(&mut self, bit: u32) {
        self.csr |= bit;
        self.sample();
    }

    /// A general register written through the host interface (§7.1, chapter 6).
    pub fn write(&mut self, reg: u8, v: u64) {
        if self.busdir & 1 == 1 {
            return;
        }
        if self.pending_signal.is_some() {
            self.held.push((reg, v));
            return;
        }
        match reg {
            0x50 => self.bitbltbuf = v,
            0x51 => self.trxpos = v,
            0x52 => self.trxreg = v,
            0x53 => self.trxdir(field(v, 0, 2)),
            0x54 => self.hwreg(v),
            0x60 => {
                if self.csr & 1 == 1 {
                    self.pending_signal = Some(v);
                } else {
                    self.signal(v);
                }
            }
            0x61 => self.csr |= 2,
            0x62 => {
                let (id, m) = (v as u32, (v >> 32) as u32);
                let lbl = ((self.siglblid >> 32) as u32 & !m) | (id & m);
                self.siglblid = (self.siglblid & 0xFFFF_FFFF) | u64::from(lbl) << 32;
            }
            _ => {}
        }
        self.sample();
    }

    fn trxdir(&mut self, xdir: u32) {
        self.incoming.clear();
        self.bits_in.clear();
        self.outgoing.clear();
        let b = self.bitbltbuf;
        let (sbp, sbw, spsm) = (field(b, 0, 14), field(b, 16, 6), field(b, 24, 6));
        let (dbp, dbw, dpsm) = (field(b, 32, 14), field(b, 48, 6), field(b, 56, 6));
        let p = self.trxpos;
        let (ssax, ssay, dsax, dsay, dir) = (field(p, 0, 11), field(p, 16, 11), field(p, 32, 11), field(p, 48, 11), field(p, 59, 2));
        let (w, h) = (field(self.trxreg, 0, 12), field(self.trxreg, 32, 12));
        match xdir {
            0 => self.incoming = Self::rectangle(dsax, dsay, w, h).into_iter().map(|(x, y)| (dpsm, dbp, dbw, x, y)).rev().collect(),
            1 => {
                let mut stream: Vec<bool> = Vec::new();
                for (x, y) in Self::rectangle(ssax, ssay, w, h) {
                    let v = self.get(spsm, sbp, sbw, x, y);
                    stream.extend((0..bits(spsm)).map(|i| v >> i & 1 == 1));
                }
                while stream.len() % 64 != 0 {
                    stream.push(false);
                }
                self.outgoing = stream.chunks(64).map(|c| c.iter().enumerate().fold(0u64, |a, (i, &b)| a | u64::from(b) << i)).rev().collect();
            }
            2 => {
                // Figure 4-1: order 1 runs bottom to top, 2 right to left, 3 both.
                let ys: Vec<u32> = if dir & 1 == 1 { (0..h).rev().collect() } else { (0..h).collect() };
                let xs: Vec<u32> = if dir & 2 == 2 { (0..w).rev().collect() } else { (0..w).collect() };
                for &j in &ys {
                    for &i in &xs {
                        let v = self.get(spsm, sbp, sbw, (ssax + i) % 2048, (ssay + j) % 2048);
                        self.put(dpsm, dbp, dbw, (dsax + i) % 2048, (dsay + j) % 2048, v);
                    }
                }
            }
            _ => {}
        }
    }

    fn hwreg(&mut self, v: u64) {
        if self.incoming.is_empty() {
            return;
        }
        self.bits_in.extend((0..64).map(|i| v >> i & 1 == 1));
        loop {
            let Some(&(psm, bp, bw, x, y)) = self.incoming.last() else { break };
            let n = bits(psm) as usize;
            if self.bits_in.len() < n {
                break;
            }
            let value = self.bits_in[..n].iter().enumerate().fold(0u32, |a, (i, &b)| a | u32::from(b) << i);
            self.bits_in.drain(..n);
            self.put(psm, bp, bw, x, y, value);
            self.incoming.pop();
        }
        if self.incoming.is_empty() {
            self.bits_in.clear();
        }
    }

    /// The host reading 64 bits while BUSDIR is 1.
    pub fn read(&mut self) -> u64 {
        if self.busdir & 1 == 0 {
            return 0;
        }
        self.outgoing.pop().unwrap_or(0)
    }

    /// A privileged register read by LD (§7.2): CSR and SIGLBLID as the manual gives them, CSR's REV and ID left 0
    /// (no document gives them); every other register CSR's value, the reading D-100 records.
    pub fn read_privileged(&self, ga: u32) -> u64 {
        match ga {
            0x48 => self.siglblid,
            _ => u64::from(self.csr & 0x1F | if self.held.is_empty() && self.pending_signal.is_none() { 0x4000 } else { 0 }),
        }
    }

    /// A privileged register written by the EE with SD (§7.2).
    pub fn write_privileged(&mut self, ga: u32, v: u64) {
        self.privileged.insert(ga, v);
        match ga {
            0x40 => {
                let a = v as u32;
                if a & 0x200 != 0 {
                    self.csr = 0;
                    self.pending_signal = None;
                    self.held.clear();
                    self.incoming.clear();
                    self.outgoing.clear();
                    self.imr = 0x7F00;
                    self.privileged.insert(0x41, 0x7F00);
                }
                self.csr &= !(a & 0x1F);
                if a & 1 != 0
                    && let Some(s) = self.pending_signal.take()
                {
                    self.signal(s);
                    for (r, d) in std::mem::take(&mut self.held) {
                        self.write(r, d);
                    }
                }
            }
            0x41 => self.imr = v as u32,
            0x44 => self.busdir = v,
            0x48 => self.siglblid = v,
            _ => {}
        }
        self.sample();
    }
}
