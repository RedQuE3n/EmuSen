//! The VDP's ports, FIFO and DMA on the slot schedule, its HV counter and its interrupts (stage 4, step 1). A line
//! begins where the V counter is incremented; its access slots, the H counter and the blanking flags are placed in it
//! from Nemesis's logic-analyser tables (SpritesMind topics 851 and 1291, and his VRAM timing charts); the FIFO and the
//! read buffer follow the behaviour his VDPFIFOTesting demonstrates; DMA follows MacDonald's "Sega Genesis VDP
//! documentation" §11. Nephrite_Native.md §13 is the record, with what is argued.

use std::sync::OnceLock;

/// Master clocks a line.
pub const LINE: u64 = 3420;

/// One width's line: where each H counter value and each access slot falls, in master clocks from the line's start.
pub struct Timing {
    /// The 9-bit H counter at each master clock of the line.
    h: Vec<u16>,
    /// The external access slots of an active line, and of a blank one, as master clocks into the line, ascending.
    pub active: Vec<u16>,
    pub blank: Vec<u16>,
    /// Where the vertical interrupt and the blanking flag's change fall.
    pub vint: u64,
    pub vblank: u64,
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
    Timing {
        h: hm,
        active,
        blank,
        vint: at(0x001),
        // MacDonald's m5hvc.txt: the blanking flag changes at H $A8 (40-cell) and $87 (32-cell), of the 8-bit counter.
        vblank: if h40 { at(0x150) } else { at(0x10E) },
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
        }
    }

    pub fn set_regs_state(&mut self, s: VdpRegs) {
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
        }
    }

    fn increment(&self) -> u16 {
        self.regs[15] as u16
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
    pub fn lines(&self) -> u32 {
        if self.pal { 313 } else { 262 }
    }
    pub fn vint_line(&self) -> u32 {
        if self.pal && self.regs[1] & 8 != 0 { 240 } else { 224 }
    }

    /// Whether `line`'s slots follow the active pattern: the display on and the line drawn, or the last line of the
    /// frame, whose sprites the first line shows (Eke, SpritesMind topic 1291: the blanking flag clears there).
    pub fn active_line(&self, line: u32) -> bool {
        self.display() && (line < self.vint_line() || line == self.lines() - 1)
    }

    /// The external slots of `line`, as master clocks into it.
    pub fn slots(&self, line: u32) -> &'static [u16] {
        let t = self.timing();
        if self.active_line(line) { &t.active } else { &t.blank }
    }

    pub fn fifo_empty(&self) -> bool {
        self.fifo_count == 0
    }

    pub fn fifo_full(&self) -> bool {
        self.fifo_count >= 4
    }

    /// Whether a slot has anything to do: the FIFO to drain, a DMA to run, or a read to fetch.
    pub fn busy(&self) -> bool {
        self.fifo_count > 0 || !matches!(self.dma, Dma::None | Dma::FillWait) || (self.reads() && !self.read_ready)
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
        self.fifo_next = (self.fifo_next + 1) & 3;
        self.fifo_count += 1;
        self.address = self.address.wrapping_add(self.increment());
    }

    /// The entry that will be filled next: its data gives a CRAM or VSRAM read its unused bits.
    fn next_entry(&self) -> u16 {
        self.fifo[self.fifo_next as usize].data
    }

    /// One external access slot: the FIFO's oldest entry (a VRAM word takes two), else a step of a fill or a copy,
    /// else the read the read buffer waits for.
    pub fn slot(&mut self) {
        if self.fifo_count > 0 {
            let i = ((self.fifo_next + 4 - self.fifo_count) & 3) as usize;
            let e = self.fifo[i];
            if e.code == 1 && !e.half {
                self.fifo[i].half = true;
                return;
            }
            self.write_entry(e);
            self.fifo_count -= 1;
            // A word written during a fill becomes its data (VDPFIFOTesting's data port writes during a fill).
            match self.dma {
                Dma::Fill { started: false, data, left } if self.fifo_count == 0 => self.dma = Dma::Fill { data, left, started: true },
                Dma::Fill { started: true, left, .. } => self.dma = Dma::Fill { data: e.data, left, started: true },
                _ => {}
            }
            return;
        }
        match self.dma {
            Dma::Fill { data, left, started: true } => {
                let a = self.address as usize;
                let next = self.next_entry();
                match self.code & 0xF {
                    1 => self.vram[a ^ 1] = (data >> 8) as u8,
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
                self.vram[self.address as usize ^ 1] = b;
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

    fn write_entry(&mut self, e: Entry) {
        let a = e.address as usize;
        match e.code {
            1 => {
                let (hi, lo) = ((e.data >> 8) as u8, e.data as u8);
                let (x, y) = if a & 1 == 0 { (hi, lo) } else { (lo, hi) };
                self.vram[a & 0xFFFE] = x;
                self.vram[(a & 0xFFFE) | 1] = y;
            }
            3 => self.write_cram(a, e.data),
            5 => self.write_vsram(a, e.data),
            _ => {}
        }
    }

    fn write_cram(&mut self, a: usize, v: u16) {
        let v = v & CRAM_BITS;
        self.cram[a & 0x7E] = (v >> 8) as u8;
        self.cram[(a & 0x7E) | 1] = v as u8;
    }

    fn write_vsram(&mut self, a: usize, v: u16) {
        if a & 0x7E < 80 {
            let v = v & VSRAM_BITS;
            self.vsram[a & 0x7E] = (v >> 8) as u8;
            self.vsram[(a & 0x7E) | 1] = v as u8;
        }
    }

    fn word(m: &[u8], a: usize) -> u16 {
        (m[a] as u16) << 8 | m[a + 1] as u16
    }

    /// The read the buffer waits for, made at a slot: VRAM's word, the 8-bit VRAM target's byte, or CRAM's and
    /// VSRAM's bits with the rest from the FIFO's next entry. VSRAM past its 40 words reads the first (argued: the
    /// test shows the scroll value the VDP last fetched, which stands in for it until the renderer fetches).
    fn fetch_read(&mut self) {
        let a = self.address as usize;
        let next = self.next_entry();
        self.read_buf = match self.code & 0xF {
            0 => Self::word(&self.vram, a & 0xFFFE),
            0xC => (next & 0xFF00) | self.vram[a ^ 1] as u16,
            8 => (next & !CRAM_BITS) | (Self::word(&self.cram, a & 0x7E) & CRAM_BITS),
            4 => {
                let v = Self::word(&self.vsram, if a & 0x7E < 80 { a & 0x7E } else { 0 });
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
                self.regs[r] = v as u8;
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
        (self.fifo_empty() as u16) << 9
            | (self.fifo_full() as u16) << 8
            | (self.vint_pending as u16) << 7
            | (vblank as u16) << 3
            | (t.hblank(offset) as u16) << 2
            | (dma as u16) << 1
            | self.pal as u16
    }

    /// The V counter's 9-bit value on `line`: NTSC's jump from $EA to $1E5 (MacDonald §5), PAL's from $102 to $1CA in
    /// V28 and from $10A to $1D2 in V30 (argued from the line count).
    pub fn v_counter(&self, line: u32) -> u16 {
        let (last, to) = match (self.pal, self.regs[1] & 8 != 0) {
            (false, _) => (0xEA, 0x1E5),
            (true, false) => (0x102, 0x1CA),
            (true, true) => (0x10A, 0x1D2),
        };
        if line <= last { line as u16 } else { (to + (line - last - 1)) as u16 }
    }

    /// The HV counter: the V counter's low byte and the H counter's top eight bits.
    pub fn hv(&self, line: u32, offset: u64) -> u16 {
        if let Some(l) = self.hv_latch.filter(|_| self.regs[0] & 2 != 0) {
            return l;
        }
        (self.v_counter(line) & 0xFF) << 8 | (self.timing().h(offset) >> 1)
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
}
