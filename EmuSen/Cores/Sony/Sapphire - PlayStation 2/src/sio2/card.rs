//! The PlayStation 2 memory card as a state machine on ps2tek's "SIO2 PS2 Memcards" (Sapphire_Native.md §30.5): the
//! probe, the specs, the terminator, page reads and writes with their spare bytes through a 528-byte page, block erase,
//! the end of a read or write, and the flash's rules, a write clearing bits and an erase setting a block to FFh (Ridge,
//! "NAND Flash Memory Basics"). The card's busy times are its flash's (`timing`): a command that reaches a busy card
//! waits. The authentication commands get no answer (Sapphire_Plan.md §11 Q7(b)).
//!
//! **The reply's place** (Sapphire_Disputes.md D-222): the card answers byte k of a command with ps2tek's reply list's
//! byte k - 2, after HiZ (FFh) for the peripheral byte and the command byte; so a command sent with two bytes beyond
//! ps2tek's command list receives the whole reply, its terminator last.

/// A page with its spare bytes, the pages an erase block, and Sony's card's pages (ps2tek "Get Specs").
pub const RAW_PAGE: usize = 528;
pub const PAGES_PER_BLOCK: u32 = 16;
pub const PAGES: u32 = 16_384;
pub const CARD_BYTES: usize = RAW_PAGE * PAGES as usize;
/// The reply byte and the terminator at reset.
pub const ACK: u8 = 0x2B;
pub const TERMINATOR: u8 = 0x55;
/// HiZ: what a byte nobody drives reads.
pub const HIZ: u8 = 0xFF;

/// The flash's times, in EE cycles [A: Toshiba's TC58128AFT data sheet, of the family of the TC58V64AFT Ridge names in
/// Sony's cards: a page's programming 200 to 300 µs typical, a block's erase 2 ms typical, a page's load into the
/// register 25 µs at most; the middle of the first range is taken]. Sapphire_Native.md §30.5.
pub mod timing {
    use crate::clock::EE_HZ;
    pub const PROGRAM: u64 = EE_HZ / 4000;
    pub const ERASE: u64 = EE_HZ / 500;
    pub const LOAD: u64 = EE_HZ / 40_000;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryCard {
    /// The raw card, every page followed by its spare bytes: the medium `ps2.card`'s file.
    pub image: Vec<u8>,
    pub terminator: u8,
    /// The page the last Start Read, Write or Erase named, and where the next read or write falls in it.
    page: u32,
    offset: usize,
    /// A write's page, gathered until Read/Write End programs it.
    buffer: Box<[u8; RAW_PAGE]>,
    writing: bool,
    /// The card is busy until this EE cycle.
    pub busy_until: u64,
    /// The image has changed since this was last cleared.
    pub dirty: bool,
}

impl MemoryCard {
    /// A card holding `image`, refused unless it is Sony's card's size.
    pub fn new(image: Vec<u8>) -> Option<MemoryCard> {
        (image.len() == CARD_BYTES).then(|| MemoryCard {
            image,
            terminator: TERMINATOR,
            page: 0,
            offset: 0,
            buffer: Box::new([0xFF; RAW_PAGE]),
            writing: false,
            busy_until: 0,
            dirty: false,
        })
    }

    /// An erased card, every byte FFh, as the flash leaves an erased block.
    pub fn erased() -> MemoryCard {
        MemoryCard::new(vec![0xFF; CARD_BYTES]).expect("the card's size")
    }

    fn raw(&self, page: u32) -> Option<&[u8]> {
        let at = page as usize * RAW_PAGE;
        self.image.get(at..at + RAW_PAGE)
    }

    /// Whether the card answers command `c` at all.
    pub fn answers(c: u8) -> bool {
        !matches!(c, 0xF0 | 0xF3 | 0xF7)
    }

    /// Answers a whole command (`cmd[0]` the peripheral byte 81h) at EE cycle `t` into `reply`, as long as `cmd`; returns
    /// whether the card answered. `t` is when the command ends, from which a write's or an erase's busy time runs.
    pub fn exchange(&mut self, cmd: &[u8], reply: &mut [u8], t: u64) -> bool {
        reply.fill(HIZ);
        let c = cmd.get(1).copied().unwrap_or(0);
        if !Self::answers(c) {
            return false;
        }
        let arg = |i: usize| cmd.get(i).copied().unwrap_or(0);
        let term = self.terminator;
        let mut list = [HIZ; 134];
        let n = match c {
            0x11 | 0x12 => {
                list[..2].copy_from_slice(&[ACK, term]);
                2
            }
            0x21..=0x23 => {
                self.page = u32::from_le_bytes([arg(2), arg(3), arg(4), arg(5)]);
                self.offset = 0;
                if c == 0x22 {
                    self.buffer.fill(0xFF);
                    self.writing = true;
                }
                if c == 0x23 {
                    self.busy_until = t + timing::LOAD;
                }
                list[5..7].copy_from_slice(&[ACK, term]);
                7
            }
            0x26 => {
                let specs = [0x00, 0x02, PAGES_PER_BLOCK as u8, 0x00, (PAGES & 0xFF) as u8, (PAGES >> 8) as u8, 0x00, 0x00];
                list[0] = ACK;
                list[1..9].copy_from_slice(&specs);
                list[9] = specs.iter().fold(0, |x, b| x ^ b);
                list[10] = term;
                11
            }
            0x27 => {
                self.terminator = arg(2);
                list[1..3].copy_from_slice(&[ACK, term]);
                3
            }
            0x28 => {
                list[..3].copy_from_slice(&[ACK, term, TERMINATOR]);
                3
            }
            0x42 => {
                let size = (arg(2) as usize).min(128);
                for i in 0..size {
                    if self.offset < RAW_PAGE {
                        self.buffer[self.offset] = arg(3 + i);
                    }
                    self.offset += 1;
                }
                list[..2].copy_from_slice(&[ACK, term]);
                list[131] = term;
                132
            }
            0x43 => {
                let size = (arg(2) as usize).min(128);
                list[..2].copy_from_slice(&[ACK, term]);
                let mut x = 0u8;
                for i in 0..size {
                    let b = self.raw(self.page).and_then(|r| r.get(self.offset)).copied().unwrap_or(HIZ);
                    list[2 + i] = b;
                    x ^= b;
                    self.offset += 1;
                }
                list[130] = x;
                list[131] = term;
                132
            }
            0x81 => {
                if self.writing {
                    self.writing = false;
                    let at = self.page as usize * RAW_PAGE;
                    if let Some(raw) = self.image.get_mut(at..at + RAW_PAGE) {
                        for (d, s) in raw.iter_mut().zip(self.buffer.iter()) {
                            *d &= s;
                        }
                        self.dirty = true;
                    }
                    self.busy_until = t + timing::PROGRAM;
                }
                list[..2].copy_from_slice(&[ACK, term]);
                2
            }
            0x82 => {
                let first = (self.page / PAGES_PER_BLOCK * PAGES_PER_BLOCK) as usize * RAW_PAGE;
                if let Some(block) = self.image.get_mut(first..first + RAW_PAGE * PAGES_PER_BLOCK as usize) {
                    block.fill(0xFF);
                    self.dirty = true;
                }
                self.busy_until = t + timing::ERASE;
                list[..2].copy_from_slice(&[ACK, term]);
                2
            }
            0xBF => {
                list[1..3].copy_from_slice(&[ACK, term]);
                3
            }
            _ => 0,
        };
        for (k, r) in reply.iter_mut().enumerate().skip(2) {
            if k - 2 < n {
                *r = list[k - 2];
            }
        }
        true
    }
}

impl MemoryCard {
    /// The card's bus state in `SPH2` (Sapphire_Native.md §38.2): the terminator, the page and place named, the page
    /// gathered, a write in progress and the flash's busy time. The image is the player's card, kept apart from every
    /// state (§38.3).
    pub fn write_bus(&self, w: &mut emusen_native::StateWriter) {
        w.u8("Terminator", self.terminator);
        w.u32s("Page", &[self.page, self.offset as u32]);
        w.bytes("Buffer", &self.buffer[..]);
        w.bool("Writing", self.writing);
        w.u64("BusyUntil", self.busy_until);
    }

    pub fn read_bus(&mut self, r: &mut emusen_native::StateReader) -> Result<(), emusen_native::Truncated> {
        self.terminator = r.u8()?;
        let mut p = [0u32; 2];
        r.u32s(&mut p)?;
        (self.page, self.offset) = (p[0], (p[1] as usize).min(RAW_PAGE));
        r.bytes(&mut self.buffer[..])?;
        self.writing = r.bool()?;
        self.busy_until = r.u64()?;
        Ok(())
    }

    /// The bus state of a slot with no card, so that every state of a machine has one length.
    pub fn write_absent(w: &mut emusen_native::StateWriter) {
        MemoryCard::erased_bus().write_bus(w);
    }

    fn erased_bus() -> MemoryCard {
        MemoryCard { image: Vec::new(), terminator: 0, page: 0, offset: 0, buffer: Box::new([0; RAW_PAGE]), writing: false, busy_until: 0, dirty: false }
    }

    pub fn skip_bus(r: &mut emusen_native::StateReader) -> Result<(), emusen_native::Truncated> {
        MemoryCard::erased_bus().read_bus(r)
    }
}
