//! The VDP's ports, FIFO and DMA on the slot schedule, its HV counter and its interrupts (stage 4, step 1). A line
//! begins where the V counter is incremented; its access slots, the H counter and the blanking flags are placed in it
//! from Nemesis's logic-analyser tables (SpritesMind topics 851 and 1291, and his VRAM timing charts); the FIFO and the
//! read buffer follow the behaviour his VDPFIFOTesting demonstrates; DMA follows MacDonald's "Sega Genesis VDP
//! documentation" §11. Nephrite_Native.md §13 is the record, with what is argued.

use std::sync::OnceLock;

/// The VRAM byte a 128 KiB-mode access to `a` reaches on a board with 64 KiB: address bit 1 inverted at bit 0, bits
/// 2-9 in place, bit 10 at bit 1 and bits 11-15 one place down (measured on the board, Nephrite_Disputes.md D-10).
pub fn vram128_address(a: usize) -> usize {
    (a >> 1 & 1 ^ 1) | (a & 0x3FC) | (a >> 9 & 2) | (a & 0xF800) >> 1
}

/// Pixels before a 2-cell column is shown that the VDP reads its vertical scroll (bracketed on the board between 20 and
/// 34), and its name table and patterns (between 20 and 32, weakly); Nephrite_Disputes.md D-9.
pub const VSCROLL_LEAD: i64 = 27;
pub const PATTERN_LEAD: u64 = 24;

/// Master clocks before a word that arrives with nothing queued can be written: the write path's start, measured on
/// the board in both widths, the display on and off (Nephrite_Disputes.md D-11).
pub const WRITE_START: u64 = 176;

/// Master clocks a line.
pub const LINE: u64 = 3420;

/// One width's line: where each H counter value and each access slot falls, in master clocks from the line's start.
pub struct Timing {
    /// The 9-bit H counter at each master clock of the line.
    h: Vec<u16>,
    /// The external access slots of an active line, and of a blank one, as master clocks into the line, ascending.
    pub active: Vec<u16>,
    pub blank: Vec<u16>,
    /// The first blank line's: the active pattern until H $14E or $10E, the blank one after (measured, D-6).
    pub first_blank: Vec<u16>,
    /// Where the vertical interrupt and the blanking flag's change fall.
    pub vint: u64,
    pub vblank: u64,
    /// Where the line's pixel 0 begins (H $18), in master clocks from the line's start.
    pub pixel0: u64,
    hblank_set: u16,
    hblank_clear: u16,
}

/// The 1-based access slots that are the 68000's on an active line, and the refresh slots, per Nemesis's charts.
const H40_EXTERNAL: [u16; 18] = [15, 23, 31, 47, 55, 63, 79, 87, 95, 111, 119, 127, 143, 151, 159, 174, 175, 199];
const H40_REFRESH: [u16; 5] = [39, 71, 103, 135, 167];
const H32_EXTERNAL: [u16; 16] = [15, 23, 31, 47, 55, 63, 79, 87, 95, 111, 119, 127, 142, 143, 157, 171];
const H32_REFRESH: [u16; 4] = [39, 71, 103, 135];

fn build(h40: bool) -> Timing {
    // The H counter's progression from the V counter's increment, and the master clocks of each serial clock tick.
    let (start, end_first, jump_to, pixels): (u16, u16, u16, usize) = if h40 { (0x14A, 0x16C, 0x1C9, 420) } else { (0x10A, 0x127, 0x1D2, 342) };
    let mut values = Vec::with_capacity(pixels);
    let mut h = start;
    for _ in 0..pixels {
        values.push(h);
        h = if h == end_first { jump_to } else { (h + 1) & 0x1FF };
    }
    let mut sc: Vec<u16> = Vec::with_capacity(2 * pixels);
    let hsync = values.iter().position(|&v| v == 0x1CD).unwrap_or(0) * 2;
    for i in 0..2 * pixels {
        let d = if !h40 {
            5
        } else {
            // In hsync the serial clock runs at MCLK/5 but for three pixels at MCLK/4, then 0x1ED at MCLK/5.
            match i.checked_sub(hsync) {
                Some(k) if k < 64 => {
                    if (15..17).contains(&k) || (32..34).contains(&k) || (49..51).contains(&k) { 4 } else { 5 }
                }
                Some(k) if k < 66 => 5,
                _ => 4,
            }
        };
        sc.push(d);
    }
    let mut sc_mc = Vec::with_capacity(sc.len() + 1);
    let mut t = 0u16;
    for &d in &sc {
        sc_mc.push(t);
        t += d;
    }
    sc_mc.push(t);
    debug_assert_eq!(t as u64, LINE);
    let mut hm = vec![0u16; LINE as usize];
    for (p, &v) in values.iter().enumerate() {
        for m in sc_mc[2 * p]..sc_mc[2 * p + 2] {
            hm[m as usize] = v;
        }
    }
    // Slot 1 begins at H $1E9 in both widths; a slot is four serial clock ticks.
    let total = sc.len();
    let base = 2 * values.iter().position(|&v| v == 0x1E9).unwrap();
    let (slots, ext, refresh): (u16, &[u16], &[u16]) = if h40 { (210, &H40_EXTERNAL, &H40_REFRESH) } else { (171, &H32_EXTERNAL, &H32_REFRESH) };
    let (mut active, mut blank) = (Vec::new(), Vec::new());
    for s in 1..=slots {
        let m = sc_mc[(base + 4 * (s as usize - 1)) % total];
        if ext.contains(&s) {
            active.push(m);
        }
        if !refresh.contains(&s) {
            blank.push(m);
        }
    }
    active.sort_unstable();
    blank.sort_unstable();
    let at = |v: u16| sc_mc[2 * values.iter().position(|&x| x == v).unwrap()] as u64;
    let edge = if h40 { at(0x14E) } else { at(0x10E) } as u16;
    let first_blank = active.iter().filter(|&&m| m < edge).chain(blank.iter().filter(|&&m| m >= edge)).copied().collect();
    Timing {
        pixel0: at(0x018),
        h: hm,
        active,
        blank,
        first_blank,
        vint: at(0x001),
        // Two pixels after the V counter's step, as the board shows (D-12), not m5hvc.txt's $A8 and $87.
        vblank: if h40 { at(0x14C) } else { at(0x10C) },
        hblank_set: if h40 { 0x166 } else { 0x126 },
        hblank_clear: if h40 { 0x00B } else { 0x00A },
    }
}

pub fn timing(h40: bool) -> &'static Timing {
    static T: OnceLock<[Timing; 2]> = OnceLock::new();
    &T.get_or_init(|| [build(false), build(true)])[h40 as usize]
}

impl Timing {
    pub fn h(&self, offset: u64) -> u16 {
        self.h[(offset.min(LINE - 1)) as usize]
    }

    pub fn hblank(&self, offset: u64) -> bool {
        let h = self.h(offset);
        !(self.hblank_clear..self.hblank_set).contains(&h)
    }
}

/// A word the 68000 or a DMA transfer wrote, waiting for a slot: its target, address and data, and the bytes of a
/// VRAM write done.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Entry {
    pub code: u8,
    pub address: u16,
    pub data: u16,
    pub half: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Dma {
    #[default]
    None,
    /// From the 68000's bus: the next source address and the words left.
    Bus { source: u32, left: u32 },
    /// A fill waiting for its data word, then filling once that word is written (advancing the source registers as a
    /// copy does): VRAM with the word's high byte, CRAM
    /// and VSRAM with the FIFO's next entry, as VDPFIFOTesting's fills show.
    FillWait,
    Fill { data: u16, left: u32, started: bool },
    /// A copy within VRAM: the source, the bytes left, and a byte read and not yet written. A copy's bytes, like a
    /// fill's, are addressed with bit 0 inverted, as VDPFIFOTesting's copies show.
    Copy { source: u16, left: u32, byte: Option<u8> },
}

pub struct Vdp {
    pub regs: [u8; 24],
    pending: bool,
    code: u8,
    address: u16,
    pub vram: Vec<u8>,
    pub cram: Vec<u8>,
    pub vsram: Vec<u8>,
    /// The four-word write FIFO: a ring whose entries keep their data after they are written out.
    fifo: [Entry; 4],
    fifo_count: u8,
    fifo_next: u8,
    /// The time from which a slot may write each FIFO entry to memory, and the words that have left the FIFO and not
    /// yet reached memory, with theirs (Nephrite_Disputes.md D-11).
    pub fifo_at: [u64; 4],
    pub landing: std::collections::VecDeque<(u64, Entry)>,
    /// The read buffer and whether it holds the word the next data port read returns.
    read_buf: u16,
    read_ready: bool,
    pub dma: Dma,
    pub vint_pending: bool,
    pub hint_pending: bool,
    hint_counter: u8,
    pub pal: bool,
    /// The external interrupt, and the HV counter latched by it while register 0's bit 1 is set.
    pub ext_pending: bool,
    hv_latch: Option<u16>,
    /// The master clock the slots have been run to, and the earliest a transfer's next bus read may come.
    pub time: u64,
    pub fetch_at: u64,
    /// The first four bytes of each sprite's entry, as the VDP caches them from writes into the table (Nemesis,
    /// SpritesMind topic 1291: the cache follows writes, not the register).
    pub sat_cache: Vec<u8>,
    /// The sprite flags of the status register, whether the last line drawn ended in a dot overflow, and the last
    /// vertical scroll value the renderer read.
    pub sprite_overflow: bool,
    pub sprite_collision: bool,
    pub dot_overflow_line: bool,
    pub vscroll_latch: u16,
    /// The interlace mode latched at vertical blanking (0, 1, or 3 for double resolution), and the odd field.
    pub interlace: u8,
    pub odd: bool,
    /// The next line's sprite pixels as parsed on this one (colour, priority in bit 7), and the line the slots are
    /// in and where it began.
    pub sprite_buffer: Vec<u8>,
    pub cur_line: u32,
    pub cur_line_start: u64,
    /// VSRAM and the line's horizontal scroll as the line began, which its fetches read (Nephrite_Native.md §14.1).
    pub line_vsram: Vec<u8>,
    pub line_hscroll: (u16, u16),
    /// Registers 17 and 18 as the line began: a write later in it moves the next line's window (measured, D-5).
    pub line_window: [u8; 2],
    /// The line whose pixels are being drawn, drawn up to `span_x` in its one or two rows, with the sprite pixels it
    /// took as it opened: each change to what the picture reads draws the line up to the beam first (Nephrite_Native.md
    /// §16).
    pub open: Option<usize>,
    pub span_x: usize,
    pub span_rgba: Vec<u8>,
    pub span_sprites: Vec<u8>,
    /// The open line's scroll and window values as it began, since the next line's are taken before its last pixels.
    pub span_latch: (Vec<u8>, (u16, u16), [u8; 2]),
    /// The 64 colours at the three intensities as CRAM and register 0 give them, made again after either changes.
    pub palette: [[u8; 4]; 192],
    pub palette_dirty: bool,
    /// The picture being drawn, and whether this frame's is drawn at all (the sprite pass runs either way).
    pub frame: crate::render::Frame,
    pub draw: bool,
}

/// The VDP's registers and latches, for the state; the memories are spaces of their own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VdpRegs {
    pub regs: [u8; 24],
    pub pending: bool,
    pub code: u8,
    pub address: u16,
    pub vint_pending: bool,
    pub hint_pending: bool,
    pub hint_counter: u8,
    pub ext_pending: bool,
    pub hv_latch: Option<u16>,
    pub fifo: [Entry; 4],
    pub fifo_count: u8,
    pub fifo_next: u8,
    pub read_buf: u16,
    pub read_ready: bool,
    pub dma: Dma,
    pub time: u64,
    pub fetch_at: u64,
    pub sprite_flags: [bool; 3],
    pub vscroll_latch: u16,
    pub interlace: u8,
    pub odd: bool,
}

const CRAM_BITS: u16 = 0x0EEE;
const VSRAM_BITS: u16 = 0x07FF;

impl Vdp {
    pub fn regs_state(&self) -> VdpRegs {
        VdpRegs {
            regs: self.regs,
            pending: self.pending,
            code: self.code,
            address: self.address,
            vint_pending: self.vint_pending,
            hint_pending: self.hint_pending,
            hint_counter: self.hint_counter,
            ext_pending: self.ext_pending,
            hv_latch: self.hv_latch,
            fifo: self.fifo,
            fifo_count: self.fifo_count,
            fifo_next: self.fifo_next,
            read_buf: self.read_buf,
            read_ready: self.read_ready,
            dma: self.dma,
            time: self.time,
            fetch_at: self.fetch_at,
            sprite_flags: [self.sprite_overflow, self.sprite_collision, self.dot_overflow_line],
            vscroll_latch: self.vscroll_latch,
            interlace: self.interlace,
            odd: self.odd,
        }
    }

    pub fn set_regs_state(&mut self, s: VdpRegs) {
        self.palette_dirty = true;
        self.regs = s.regs;
        self.pending = s.pending;
        self.code = s.code;
        self.address = s.address;
        self.vint_pending = s.vint_pending;
        self.hint_pending = s.hint_pending;
        self.hint_counter = s.hint_counter;
        self.ext_pending = s.ext_pending;
        self.hv_latch = s.hv_latch;
        self.fifo = s.fifo;
        self.fifo_count = s.fifo_count;
        self.fifo_next = s.fifo_next;
        self.read_buf = s.read_buf;
        self.read_ready = s.read_ready;
        self.dma = s.dma;
        self.time = s.time;
        self.fetch_at = s.fetch_at;
        [self.sprite_overflow, self.sprite_collision, self.dot_overflow_line] = s.sprite_flags;
        self.vscroll_latch = s.vscroll_latch;
        (self.interlace, self.odd) = (s.interlace, s.odd);
    }

    pub fn new(pal: bool) -> Vdp {
        Vdp {
            regs: [0; 24],
            pending: false,
            code: 0,
            address: 0,
            vram: vec![0; 0x1_0000],
            cram: vec![0; 128],
            vsram: vec![0; 80],
            fifo: [Entry::default(); 4],
            fifo_at: [0; 4],
            landing: std::collections::VecDeque::new(),
            fifo_count: 0,
            fifo_next: 0,
            read_buf: 0,
            read_ready: false,
            dma: Dma::None,
            vint_pending: false,
            hint_pending: false,
            hint_counter: 0,
            pal,
            ext_pending: false,
            hv_latch: None,
            time: 0,
            fetch_at: 0,
            sat_cache: vec![0; 320],
            sprite_overflow: false,
            sprite_collision: false,
            dot_overflow_line: false,
            vscroll_latch: 0,
            interlace: 0,
            odd: false,
            sprite_buffer: vec![0; 320],
            cur_line: 0,
            cur_line_start: 0,
            line_vsram: vec![0; 80],
            line_hscroll: (0, 0),
            line_window: [0, 0],
            open: None,
            span_x: 0,
            span_rgba: vec![0; 2 * crate::render::MAX_W * 4],
            span_sprites: vec![0; 2 * crate::render::MAX_W],
            span_latch: (vec![0; 80], (0, 0), [0, 0]),
            palette: [[0; 4]; 192],
            palette_dirty: true,
            frame: crate::render::Frame::new(),
            draw: true,
        }
    }

    /// Register 15, or 1 in mode 4, which keeps the Master System's step whatever register 15 holds (D-7).
    fn increment(&self) -> u16 {
        if self.regs[1] & 4 == 0 { 1 } else { self.regs[15] as u16 }
    }

    pub fn h40(&self) -> bool {
        self.regs[12] & 0x01 != 0
    }

    pub fn timing(&self) -> &'static Timing {
        timing(self.h40())
    }

    pub fn display(&self) -> bool {
        self.regs[1] & 0x40 != 0
    }

    /// Lines in a frame, the active lines, and the line of the vertical interrupt (240 in PAL's V30).
    /// Interlace alternates fields of 262 and 263 lines (NTSC) or 313 and 312 (PAL), the odd field the longer, and
    /// NTSC's V30 lets the V counter run its 512 lines (measured on Nuked-MD's board, Nephrite_Disputes.md D-4).
    pub fn lines(&self) -> u32 {
        if !self.pal && self.regs[1] & 0x0C == 0x0C {
            return 512;
        }
        match (self.pal, self.interlace != 0) {
            (false, false) => 262,
            (false, true) => 262 + self.odd as u32,
            (true, false) => 313,
            (true, true) => 312 + self.odd as u32,
        }
    }

    /// With the frame interrupt the interlace setting is latched and, interlaced, the field changes.
    pub fn field_start(&mut self) {
        self.interlace = match (self.regs[12] >> 1) & 3 {
            1 => 1,
            3 => 3,
            _ => 0,
        };
        self.odd = self.interlace != 0 && !self.odd;
    }
    /// Mode 4 shows 192 lines and raises its frame interrupt at line $C0 (MacDonald's `vdpint.txt`), the TMS9918's
    /// modes (register 0's bit 2 clear too) 224 as the board shows at power-on; V30 shows 240 on either board (D-4).
    pub fn vint_line(&self) -> u32 {
        if self.regs[1] & 4 == 0 && self.regs[0] & 4 != 0 {
            192
        } else if self.regs[1] & 8 != 0 {
            240
        } else {
            224
        }
    }

    /// Whether `line`'s slots follow the active pattern: the display on and the line drawn, or the last line of the
    /// frame, whose sprites the first line shows (Eke, SpritesMind topic 1291: the blanking flag clears there).
    pub fn active_line(&self, line: u32) -> bool {
        self.display() && (line < self.vint_line() || line == self.lines() - 1)
    }

    /// The external slots of `line`, as master clocks into it.
    pub fn slots(&self, line: u32) -> &'static [u16] {
        let t = self.timing();
        if self.active_line(line) {
            &t.active
        } else if self.display() && line == self.vint_line() {
            &t.first_blank
        } else {
            &t.blank
        }
    }

    pub fn fifo_empty(&self) -> bool {
        self.fifo_count == 0
    }

    pub fn fifo_full(&self) -> bool {
        self.fifo_count >= 4
    }

    /// Whether a slot has anything to do: the FIFO to drain, a DMA to run, or a read to fetch.
    pub fn busy(&self) -> bool {
        self.fifo_count > 0 || !self.landing.is_empty() || !matches!(self.dma, Dma::None | Dma::FillWait) || (self.reads() && !self.read_ready)
    }

    fn reads(&self) -> bool {
        matches!(self.code & 0xF, 0x0 | 0x4 | 0x8 | 0xC)
    }

    /// Whether the 68000 is held for a transfer from its bus.
    pub fn bus_dma(&self) -> bool {
        matches!(self.dma, Dma::Bus { .. })
    }

    /// The word a bus transfer wants next, when the FIFO has room for it.
    pub fn dma_fetch(&self) -> Option<u32> {
        match self.dma {
            Dma::Bus { source, .. } if !self.fifo_full() => Some(source),
            _ => None,
        }
    }

    /// A word fetched for a bus transfer: into the FIFO, the source advanced within its 128 KiB, the registers kept.
    pub fn dma_fetched(&mut self, w: u16) {
        let Dma::Bus { source, left } = self.dma else { return };
        self.push(w);
        let next = (source & 0xFE_0000) | (source.wrapping_add(2) & 0x1_FFFF);
        self.regs[21] = (next >> 1) as u8;
        self.regs[22] = (next >> 9) as u8;
        let len = left - 1;
        self.regs[19] = len as u8;
        self.regs[20] = (len >> 8) as u8;
        self.dma = if len == 0 { Dma::None } else { Dma::Bus { source: next, left: len } };
    }

    fn push(&mut self, data: u16) {
        let i = self.fifo_next as usize;
        self.fifo[i] = Entry { code: self.code & 0xF, address: self.address, data, half: false };
        // A word arriving with nothing queued waits the write path's start, unless a transfer brings it with the display off (D-11).
        let cold = self.fifo_count == 0 && self.landing.is_empty() && !(self.bus_dma() && !self.display());
        self.fifo_at[i] = self.time + if cold { WRITE_START } else { 0 };
        self.fifo_next = (self.fifo_next + 1) & 3;
        self.fifo_count += 1;
        self.address = self.address.wrapping_add(self.increment());
    }

    /// The entry that will be filled next: its data gives a CRAM or VSRAM read its unused bits.
    fn next_entry(&self) -> u16 {
        self.fifo[self.fifo_next as usize].data
    }

    /// The oldest word that has left the FIFO written to memory, if its time has come: one a slot.
    fn land(&mut self) {
        if let Some(&(ready, e)) = self.landing.front()
            && ready <= self.time
        {
            self.landing.pop_front();
            self.write_entry(e);
        }
    }

    /// One external access slot: the FIFO's oldest entry (a VRAM word takes two), else a step of a fill or a copy,
    /// else the read the read buffer waits for.
    pub fn slot(&mut self) {
        if self.fifo_count > 0 {
            let i = ((self.fifo_next + 4 - self.fifo_count) & 3) as usize;
            let e = self.fifo[i];
            if e.code == 1 && !e.half {
                self.fifo[i].half = true;
                self.land();
                return;
            }
            self.landing.push_back((self.fifo_at[i], e));
            self.fifo_count -= 1;
            // A word written during a fill becomes its data (VDPFIFOTesting's data port writes during a fill).
            match self.dma {
                Dma::Fill { started: false, data, left } if self.fifo_count == 0 => self.dma = Dma::Fill { data, left, started: true },
                Dma::Fill { started: true, left, .. } => self.dma = Dma::Fill { data: e.data, left, started: true },
                _ => {}
            }
            self.land();
            return;
        }
        if !self.landing.is_empty() {
            self.land();
            return;
        }
        match self.dma {
            Dma::Fill { data, left, started: true } => {
                let a = self.address as usize;
                let next = self.next_entry();
                match self.code & 0xF {
                    1 => self.vram_write(a ^ 1, (data >> 8) as u8),
                    3 => self.write_cram(a, next),
                    5 => self.write_vsram(a, next),
                    _ => {}
                }
                self.address = self.address.wrapping_add(self.increment());
                let src = ((self.regs[22] as u16) << 8 | self.regs[21] as u16).wrapping_add(1);
                self.regs[21] = src as u8;
                self.regs[22] = (src >> 8) as u8;
                self.dma_count(left);
            }
            Dma::Copy { source, left, byte: None } => {
                self.dma = Dma::Copy { source, left, byte: Some(self.vram[source as usize ^ 1]) };
            }
            Dma::Copy { source, left, byte: Some(b) } => {
                self.vram_write(self.address as usize ^ 1, b);
                self.address = self.address.wrapping_add(self.increment());
                let s = source.wrapping_add(1);
                self.regs[21] = s as u8;
                self.regs[22] = (s >> 8) as u8;
                self.dma = Dma::Copy { source: s, left, byte: None };
                self.dma_count(left);
            }
            _ => {
                if self.reads() && !self.read_ready {
                    self.fetch_read();
                }
            }
        }
    }

    /// A fill or copy has done one byte: the length registers count down and the operation ends at zero.
    fn dma_count(&mut self, left: u32) {
        let len = left - 1;
        self.regs[19] = len as u8;
        self.regs[20] = (len >> 8) as u8;
        if len == 0 {
            self.dma = Dma::None;
        } else {
            self.dma = match self.dma {
                Dma::Fill { data, .. } => Dma::Fill { data, left: len, started: true },
                Dma::Copy { source, byte, .. } => Dma::Copy { source, left: len, byte },
                d => d,
            };
        }
    }

    /// A byte into VRAM, the sprite cache taking it when it falls in the first half of a sprite's entry.
    pub fn vram_write(&mut self, a: usize, v: u8) {
        let a = a & 0xFFFF;
        // The pixels the VDP has fetched already are drawn from the old data (D-9).
        self.before_change_after(None, PATTERN_LEAD * if self.h40() { 8 } else { 10 });
        self.vram[a] = v;
        let base = ((self.regs[5] & if self.h40() { 0x7E } else { 0x7F }) as usize) << 9;
        let off = a.wrapping_sub(base);
        if off < 640 && off & 4 == 0 {
            self.sat_cache[(off >> 3) * 4 + (off & 3)] = v;
        }
    }

    pub fn vsram_word(&self, i: usize) -> u16 {
        Self::word(&self.vsram, 2 * i)
    }

    pub fn cram_word(cram: &[u8], i: usize) -> u16 {
        Self::word(cram, 2 * i)
    }

    /// Rebuilds the sprite cache from VRAM, for a state that predates it or a space written from outside.
    pub fn refresh_sat_cache(&mut self) {
        let base = ((self.regs[5] & if self.h40() { 0x7E } else { 0x7F }) as usize) << 9;
        for i in 0..80 {
            for k in 0..4 {
                self.sat_cache[i * 4 + k] = self.vram[(base + i * 8 + k) & 0xFFFF];
            }
        }
    }

    fn write_entry(&mut self, e: Entry) {
        let a = e.address as usize;
        match e.code {
            1 if self.regs[1] & 0x84 == 0x84 => {
                // Register 1's bit 7, the 128 KiB mode, on a board with 64: one byte, the word's low one, at the
                // address the expansion's interleave gives (measured, D-10).
                self.vram_write(vram128_address(a), e.data as u8);
            }
            1 => {
                let (hi, lo) = ((e.data >> 8) as u8, e.data as u8);
                let (x, y) = if a & 1 == 0 { (hi, lo) } else { (lo, hi) };
                // Mode 4 writes the word where its 16 KiB puts the address (D-7).
                let w = if self.regs[1] & 4 == 0 { crate::render::vram4_address(a & 0x3FFE) & 0xFFFE } else { a & 0xFFFE };
                self.vram_write(w, x);
                self.vram_write(w | 1, y);
            }
            3 => self.write_cram(a, e.data),
            5 => self.write_vsram(a, e.data),
            _ => {}
        }
    }

    fn write_cram(&mut self, a: usize, v: u16) {
        // Mode 4 addresses an entry a byte, and keeps bits 0-2, 3-5 and 9-11 of the word as red, green and blue (D-7).
        let (a, v) = if self.regs[1] & 4 == 0 { ((a & 0x1F) << 1, v & 0xE00 | (v >> 3 & 7) << 5 | (v & 7) << 1) } else { (a, v) };
        let v = v & CRAM_BITS;
        self.before_change(Some(v));
        self.palette_dirty = true;
        self.cram[a & 0x7E] = (v >> 8) as u8;
        self.cram[(a & 0x7E) | 1] = v as u8;
    }

    fn write_vsram(&mut self, a: usize, v: u16) {
        if a & 0x7E < 80 {
            self.before_change(None);
            self.vsram_fetch_order(a & 0x7E, v & VSRAM_BITS);
            let v = v & VSRAM_BITS;
            self.vsram[a & 0x7E] = (v >> 8) as u8;
            self.vsram[(a & 0x7E) | 1] = v as u8;
        }
    }

    /// A VSRAM write reaches the present line's columns whose scroll the VDP reads after it: a column's is read
    /// `VSCROLL_LEAD` pixels before it is shown (measured, D-9); those read already keep the old value.
    fn vsram_fetch_order(&mut self, a: usize, v: u16) {
        if self.regs[1] & 4 == 0 {
            return;
        }
        let t = self.timing();
        let off = self.time.saturating_sub(self.cur_line_start) as i64;
        let ppx = if self.h40() { 8 } else { 10 };
        let x = (off - t.pixel0 as i64).div_euclid(ppx);
        let lead: i64 = VSCROLL_LEAD;
        let column = (a / 4) as i64;
        if 16 * column - lead >= x {
            self.line_vsram[a] = (v >> 8) as u8;
            self.line_vsram[a | 1] = v as u8;
            if self.open == Some(self.cur_line as usize) {
                self.span_latch.0[a] = (v >> 8) as u8;
                self.span_latch.0[a | 1] = v as u8;
            }
        }
    }

    fn word(m: &[u8], a: usize) -> u16 {
        (m[a] as u16) << 8 | m[a + 1] as u16
    }

    /// The read the buffer waits for, made at a slot: VRAM's word, the 8-bit VRAM target's byte, or CRAM's and
    /// VSRAM's bits with the rest from the FIFO's next entry. VSRAM past its 40 words reads the scroll value the renderer
    /// last fetched, as VDPFIFOTesting's fills to VSRAM show.
    fn fetch_read(&mut self) {
        let a = self.address as usize;
        let next = self.next_entry();
        self.read_buf = match self.code & 0xF {
            0 => Self::word(&self.vram, a & 0xFFFE),
            0xC => (next & 0xFF00) | self.vram[a ^ 1] as u16,
            8 => (next & !CRAM_BITS) | (Self::word(&self.cram, a & 0x7E) & CRAM_BITS),
            4 => {
                let v = if a & 0x7E < 80 { Self::word(&self.vsram, a & 0x7E) } else { self.vscroll_latch };
                (next & !VSRAM_BITS) | (v & VSRAM_BITS)
            }
            _ => self.read_buf,
        };
        self.read_ready = true;
    }

    /// The control port. A register write, or a command's first or second word; a second word with CD5 set and DMA
    /// enabled starts a DMA, which disabling DMA does not stop, as VDPFIFOTesting's toggled fill shows.
    pub fn control(&mut self, v: u16) {
        if self.pending {
            self.pending = false;
            self.address = (self.address & 0x3FFF) | (v & 3) << 14;
            self.code = (self.code & 3) | ((v >> 2) & 0x3C) as u8;
            self.read_ready = false;
            if self.code & 0x20 != 0 && self.regs[1] & 0x10 != 0 {
                self.start_dma();
            }
            return;
        }
        if v & 0xC000 == 0x8000 {
            // In mode 4 only registers 0-10 take a write; any register write clears the code register's two bits a
            // command's first word sets (VDPFIFOTesting's register tests).
            let r = ((v >> 8) & 0x1F) as usize;
            if r < 24 && (r <= 10 || self.regs[1] & 4 != 0) {
                // A register write shows two and a half pixels after the 68000 makes it (measured, D-9).
                let delay = if self.h40() { 20 } else { 25 };
                if r == 1 && (self.regs[1] ^ v as u8) & 0x40 != 0 {
                    self.display_change(v as u8 & 0x40 != 0, delay);
                } else {
                    self.before_change_after(None, delay);
                }
                self.regs[r] = v as u8;
                self.palette_dirty |= r == 0;
            }
            self.code &= !3;
            return;
        }
        self.code = (self.code & 0x3C) | (v >> 14) as u8;
        self.address = (self.address & 0xC000) | (v & 0x3FFF);
        self.pending = true;
    }

    fn dma_length(&self) -> u32 {
        let n = (self.regs[20] as u32) << 8 | self.regs[19] as u32;
        if n == 0 { 0x1_0000 } else { n }
    }

    fn start_dma(&mut self) {
        let left = self.dma_length();
        self.dma = match self.regs[23] >> 6 {
            0 | 1 => {
                let source = ((self.regs[23] & 0x7F) as u32) << 17 | (self.regs[22] as u32) << 9 | (self.regs[21] as u32) << 1;
                Dma::Bus { source, left }
            }
            2 => Dma::FillWait,
            _ => Dma::Copy { source: (self.regs[22] as u16) << 8 | self.regs[21] as u16, left, byte: None },
        };
    }

    /// A word through the data port into the FIFO; a fill waiting for its data starts once this word is written.
    pub fn data(&mut self, v: u16) {
        self.pending = false;
        if self.dma == Dma::FillWait {
            self.dma = Dma::Fill { data: v, left: self.dma_length(), started: false };
        }
        self.push(v);
    }

    /// Whether a data port read can be answered now.
    pub fn read_ready(&self) -> bool {
        self.read_ready || !self.reads()
    }

    /// A word read through the data port: the read buffer, the address advanced and the next read awaited.
    pub fn read_data(&mut self) -> u16 {
        self.pending = false;
        let v = self.read_buf;
        if self.reads() {
            self.read_ready = false;
            self.address = self.address.wrapping_add(self.increment());
        }
        v
    }

    /// The status register at `line` and `offset` master clocks into it; reading it ends a command half written.
    pub fn status(&mut self, line: u32, offset: u64) -> u16 {
        self.pending = false;
        let t = self.timing();
        let vl = self.vint_line();
        let vblank = !self.display() || ((line > vl || (line == vl && offset >= t.vblank)) && !(line == self.lines() - 1 && offset >= t.vblank));
        let dma = self.dma != Dma::None;
        let sprites = (self.sprite_overflow as u16) << 6 | (self.sprite_collision as u16) << 5;
        self.sprite_overflow = false;
        self.sprite_collision = false;
        let pending = self.vint_pending;
        if self.regs[1] & 4 == 0 {
            // In mode 4 a status read, not the acknowledge, clears the interrupts (MacDonald's `vdpint.txt`).
            self.vint_pending = false;
            self.hint_pending = false;
        }
        sprites
            | (self.fifo_empty() as u16) << 9
            | (self.fifo_full() as u16) << 8
            | (pending as u16) << 7
            | ((self.interlace != 0 && self.odd) as u16) << 4
            | (vblank as u16) << 3
            | (t.hblank(offset) as u16) << 2
            | (dma as u16) << 1
            | self.pal as u16
    }

    /// The V counter's 9-bit value on `line`: NTSC's jump from $EA to $1E5 (MacDonald §5), PAL's from $102 to $1CA in
    /// V28 and from $10A to $1D2 in V30 (argued from the line count).
    pub fn v_counter(&self, line: u32) -> u16 {
        let (mut last, mut to) = match (self.pal, self.regs[1] & 8 != 0) {
            (false, false) => (0xEA, 0x1E5),
            (false, true) => (0x1FF, 0x1FF),
            (true, false) => (0x102, 0x1CA),
            (true, true) => (0x10A, 0x1D2),
        };
        if self.interlace != 0 && last != 0x1FF {
            // Interlaced, the longer field jumps one line lower, and PAL's counter jumps a line earlier (the board).
            if self.pal {
                last -= 1;
            }
            to -= self.odd as u32;
        }
        if line <= last { line as u16 } else { (to + (line - last - 1)) as u16 }
    }

    /// The V counter as the HV port gives it: its low byte, or with bit 0 replaced by bit 8 in interlace mode 1 and
    /// the counter doubled with bit 8 below it in mode 2 (the board).
    pub fn v_read(&self, line: u32) -> u16 {
        let v = self.v_counter(line);
        match self.interlace {
            1 => (v & 0xFE) | (v >> 8 & 1),
            3 => ((v << 1) & 0xFE) | (v >> 7 & 1),
            _ => v & 0xFF,
        }
    }

    /// The HV counter: the V counter's low byte and the H counter's top eight bits.
    pub fn hv(&self, line: u32, offset: u64) -> u16 {
        if let Some(l) = self.hv_latch.filter(|_| self.regs[0] & 2 != 0) {
            return l;
        }
        self.v_read(line) << 8 | (self.timing().h(offset) >> 1)
    }

    /// Register 0's bit 1 set: the HV counter held at its present value until the bit is cleared (VDPFIFOTesting's
    /// latch test), and from then on taken by TH.
    pub fn latch_hv(&mut self, line: u32, offset: u64) {
        self.hv_latch = None;
        self.hv_latch = Some(self.hv(line, offset));
    }

    /// TH changed on a port whose interrupt is enabled: the external interrupt, and the HV counter latched.
    pub fn external(&mut self, line: u32, offset: u64) {
        self.ext_pending = true;
        if self.regs[0] & 2 != 0 {
            self.hv_latch = None;
            self.hv_latch = Some(self.hv(line, offset));
        }
    }

    /// The line counter at the start of `line`: decremented through the active lines and line 224, an interrupt where
    /// it expires, reloaded from register 10 then and on every other line.
    pub fn line_start(&mut self, line: u32) {
        if line <= self.vint_line() {
            if self.hint_counter == 0 {
                self.hint_counter = self.regs[10];
                self.hint_pending = true;
            } else {
                self.hint_counter -= 1;
            }
        } else {
            self.hint_counter = self.regs[10];
        }
    }

    /// The level the 68000 is asked for: 6 for a pending vertical interrupt it enabled, else 4 for a line interrupt.
    pub fn level(&self) -> u8 {
        if self.vint_pending && self.regs[1] & 0x20 != 0 {
            6
        } else if self.hint_pending && self.regs[0] & 0x10 != 0 {
            4
        } else if self.ext_pending && self.regs[11] & 8 != 0 {
            2
        } else {
            0
        }
    }

    pub fn acknowledge(&mut self, level: u8) {
        match level {
            6 => self.vint_pending = false,
            4 => self.hint_pending = false,
            2 => self.ext_pending = false,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drain(v: &mut Vdp) {
        for _ in 0..64 {
            v.time += 16;
            v.slot();
        }
    }

    #[test]
    fn registers_and_a_vram_write_through_the_ports() {
        let mut v = Vdp::new(false);
        v.control(0x8F02);
        assert_eq!(v.regs[15], 0, "mode 4 takes registers 0-10 only");
        v.control(0x8104);
        v.control(0x8F02);
        assert_eq!(v.regs[15], 2);
        v.control(0x4000);
        v.control(0x0000);
        v.data(0x1234);
        v.data(0x5678);
        drain(&mut v);
        assert_eq!(&v.vram[..4], &[0x12, 0x34, 0x56, 0x78]);
        v.control(0xC000);
        v.control(0x0000);
        v.data(0x0EEE);
        drain(&mut v);
        assert_eq!(&v.cram[..2], &[0x0E, 0xEE]);
    }

    #[test]
    fn a_fill_writes_its_byte_after_the_word_that_starts_it() {
        let mut v = Vdp::new(false);
        v.control(0x8114);
        v.control(0x8F01);
        v.control(0x930A);
        v.control(0x9400);
        v.control(0x9780);
        v.control(0x4000);
        v.control(0x0080);
        v.data(0x1234);
        drain(&mut v);
        assert_eq!(&v.vram[..12], &[0x12, 0x34, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x12, 0x00, 0x12]);
        assert_eq!(v.dma, Dma::None);
    }

    #[test]
    fn a_cram_read_takes_its_unused_bits_from_the_fifo_next_entry() {
        let mut v = Vdp::new(false);
        v.control(0x8104);
        v.control(0x8F02);
        v.control(0xC020);
        v.control(0x0000);
        for w in [0x1122, 0x3344, 0x5566, 0x7788, 0x99AA, 0xBBCC] {
            v.data(w);
            v.slot();
        }
        drain(&mut v);
        v.control(0x0020);
        v.control(0x0020);
        drain(&mut v);
        assert_eq!(v.read_data(), 0x5122);
    }

    #[test]
    fn the_line_counter_interrupts_every_n_plus_one_lines() {
        let mut v = Vdp::new(false);
        v.regs[10] = 3;
        for line in 0..262 {
            v.line_start(line);
        }
        v.hint_pending = false;
        let mut again = Vec::new();
        for line in 0..262 {
            v.line_start(line);
            if std::mem::take(&mut v.hint_pending) {
                again.push(line);
            }
        }
        assert_eq!(&again[..3], &[3, 7, 11]);
        assert!(again.iter().all(|&l| l <= 224));
    }

    #[test]
    fn the_lines_hold_nemesis_slot_counts_and_counter_ranges() {
        for (h40, active, blank, first, last) in [(false, 16, 167, 0x10A, 0x109), (true, 18, 205, 0x14A, 0x149)] {
            let t = timing(h40);
            assert_eq!((t.active.len(), t.blank.len()), (active, blank));
            assert_eq!((t.h(0), t.h(LINE - 1)), (first, last));
        }
        let t = timing(true);
        assert!(t.hblank(0) == false && t.hblank(t.vint));
    }

    /// The blanking flag rises two pixels after the V counter's step into line 224 and falls two after the step into
    /// the last line, in both widths (the board's status samples, Nephrite_Disputes.md D-12).
    #[test]
    fn the_blanking_flag_changes_two_pixels_after_the_v_counter() {
        for (h40, r12, edge) in [(true, 0x81, 0x14C), (false, 0x00, 0x10C)] {
            let mut v = Vdp::new(false);
            (v.regs[1], v.regs[12]) = (0x44, r12);
            let t = timing(h40);
            let at = (0..LINE).find(|&m| t.h(m) == edge).unwrap();
            for (line, before, after) in [(224, false, true), (261, true, false)] {
                assert_eq!((v.status(line, at - 1) & 8 != 0, v.status(line, at) & 8 != 0), (before, after), "line {line}, H40 {h40}");
            }
        }
    }

    /// The V counter's jumps and line counts as the board showed them (Nephrite_Disputes.md D-4).
    #[test]
    fn the_v_counter_jumps_where_the_board_does() {
        let jumps = |pal: bool, r1: u8, interlace: u8, odd: bool| {
            let mut v = Vdp::new(pal);
            v.regs[1] = r1;
            (v.interlace, v.odd) = (interlace, odd);
            let seq: Vec<u16> = (0..v.lines()).map(|l| v.v_counter(l)).collect();
            let jump = seq.windows(2).find(|p| p[1] != (p[0] + 1) & 0x1FF).map(|p| (p[0], p[1]));
            (v.lines(), jump)
        };
        assert_eq!(jumps(false, 0x44, 0, false), (262, Some((0xEA, 0x1E5))));
        assert_eq!(jumps(false, 0x4C, 0, false), (512, None));
        assert_eq!(jumps(true, 0x44, 0, false), (313, Some((0x102, 0x1CA))));
        assert_eq!(jumps(true, 0x4C, 0, false), (313, Some((0x10A, 0x1D2))));
        assert_eq!(jumps(false, 0x44, 3, true), (263, Some((0xEA, 0x1E4))));
        assert_eq!(jumps(false, 0x44, 3, false), (262, Some((0xEA, 0x1E5))));
        assert_eq!(jumps(true, 0x44, 1, true), (313, Some((0x101, 0x1C9))));
        assert_eq!(jumps(true, 0x44, 1, false), (312, Some((0x101, 0x1CA))));
        let mut v = Vdp::new(false);
        v.interlace = 1;
        assert_eq!((v.v_read(0x23), v.v_read(0xEB)), (0x22, 0xE5));
        v.interlace = 3;
        assert_eq!((v.v_read(0x43), v.v_read(0xEB)), (0x86, 0xCB));
    }
}
