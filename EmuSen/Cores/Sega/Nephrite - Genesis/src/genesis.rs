//! The Genesis: the 68000 and the Z80 on one master clock with the VDP's lines, the 68000's memory map, the Z80's
//! bus and its window onto the 68000's, BUSREQ and RESET, TMSS, and the interrupts. Charles MacDonald's "Sega Genesis
//! hardware notes" §1-§2 is the map; Nephrite_Plan.md §5.1 and §5.3 the scheduling; Nephrite_Native.md §9 the record.

use beryl_m68k::{Access, Bus as MainBusTrait, M68000, Size, Step};
use beryl_z80::{Bus as Z80BusTrait, Z80};
use emusen_native::{StateReader, StateWriter, Truncated};

use crate::cart::Cart;
use crate::io::Io;
use crate::sound::Sound;
use crate::vdp::{LINE, Vdp};

/// Master clocks a 68000 clock, and a Z80 T-state.
pub const M68K: u64 = 7;
/// The bus's refresh, every 128 of the 68000's clocks, placed against the picture as the board has it: the first access
/// at or after one waits two clocks if it is to the cartridge's area, and passes it without a wait if it is anywhere
/// else (Nephrite_Disputes.md D-2).
pub const REFRESH: u64 = 128 * M68K;
pub const REFRESH_FIRST: u64 = 762;
pub const REFRESH_WAIT: u64 = 2 * M68K;
/// The main RAM's refresh (D-2): first requested at `RAM_REFRESH_FIRST`, then `RAM_REFRESH_AFTER` after each one
/// is done. A RAM access within `RAM_REFRESH_WINDOW` of a request waits three clocks while it is done; with none, it
/// is done at the window's end without a wait.
pub const RAM_REFRESH_FIRST: u64 = 1414;
pub const RAM_REFRESH_AFTER: u64 = 806;
pub const RAM_REFRESH_WINDOW: u64 = 133;
pub const RAM_REFRESH_WAIT: u64 = 3 * M68K;
/// An access to the Z80's area, `$A00000`-`$A0FFFF`, takes the 68000 a clock more than four, and its strobe on the
/// Z80's bus ends four clocks before the 68000's cycle does; the reset line moves as early in a write to `$A11200`
/// (Nephrite_Disputes.md D-14, D-19).
pub const Z80_AREA_WAIT: u64 = M68K;
pub const Z80_AREA_LEAD: u64 = 4 * M68K;
pub const Z80_T: u64 = 15;
/// The Z80's INT, raised with the vertical interrupt, held for one line (argued: a pulse the Z80 misses with
/// interrupts disabled, as MacDonald says).
pub const Z80_INT: u64 = LINE;
/// A Z80 access through its window onto the 68000's bus: master clocks the Z80 waits (2.75 T) and the 68000 loses
/// (9.5 clocks, 66.5 rounded down), measured means (Nephrite_Disputes.md D-1).
pub const WINDOW_Z80_WAIT: u64 = 41;
pub const WINDOW_68K_STALL: u64 = 66;

/// Master clocks between a transfer's reads of the 68000's bus, and from its command to its first read (the board's
/// pins and its transfer sweeps, with VDPFIFOTesting's wait states; Nephrite_Disputes.md D-11).
pub const DMA_FETCH: u64 = 20;
pub const DMA_START: u64 = 88;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Event {
    Line,
    Slot,
    Fetch,
    Vint,
}

/// The model: its version register's low nibble, and whether it has TMSS (from version 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Model {
    pub overseas: bool,
    pub pal: bool,
    pub version: u8,
}

impl Model {
    pub fn tmss(self) -> bool {
        self.version != 0
    }
}

/// Everything on the buses but the two processors.
pub struct Hw {
    pub clock: u64,
    pub cart: Cart,
    pub io: Io,
    pub vdp: Vdp,
    pub wram: Vec<u8>,
    pub zram: Vec<u8>,
    /// The Z80's window: bits 23-15 of the 68000 address its $8000-$FFFF reach.
    pub z80_bank: u16,
    /// BUSREQ asked by the 68000, and the Z80 held in reset.
    pub z80_busreq: bool,
    pub z80_reset: bool,
    /// The master clock the Z80 has reached, and the end of its INT pulse.
    pub z80_clock: u64,
    pub z80_int: (u64, u64),
    /// The line being drawn and where it began.
    pub line: u32,
    pub line_begun: u64,
    vint_at: Option<u64>,
    pub frame_done: bool,
    pub model: Model,
    /// TMSS: the VDP unlocked by "SEGA" at `$A14000`, and the bytes written there.
    pub tmss_unlocked: bool,
    tmss: [u8; 4],
    /// The machine has locked up: a TMSS console's VDP touched before unlocking, as the hardware hangs.
    pub locked_up: bool,
    /// Master clocks the 68000 owes the Z80's window, paid at its next access.
    stall: u64,
    /// The master clocks of the next refresh of the bus and of the main RAM, each paid by the 68000's next access
    /// it applies to (Nephrite_Disputes.md D-2).
    pub refresh_at: u64,
    pub ram_refresh_at: u64,
    /// The last word the 68000 read in program space: the next instruction, which an unmapped read returns.
    pub prefetch: u16,
    /// Whether this frame's picture is drawn at all (the sprite pass runs either way); the VDP holds the picture.
    pub draw: bool,
    /// The YM2612 and the PSG, and their mix.
    pub sound: Sound,
}

pub struct Genesis {
    pub cpu: M68000,
    pub z80: Z80,
    pub hw: Hw,
}

impl Genesis {
    pub fn new(cart: Cart, model: Model) -> Genesis {
        let hw = Hw {
            clock: 0,
            cart,
            io: Io::new(model.overseas, model.pal, model.version),
            vdp: Vdp::new(model.pal),
            wram: vec![0; 0x1_0000],
            zram: vec![0; 0x2000],
            z80_bank: 0,
            z80_busreq: false,
            z80_reset: true,
            z80_clock: 0,
            z80_int: (0, 0),
            line: 0,
            line_begun: 0,
            vint_at: None,
            frame_done: false,
            model,
            tmss_unlocked: false,
            tmss: [0; 4],
            locked_up: false,
            stall: 0,
            refresh_at: REFRESH_FIRST,
            ram_refresh_at: RAM_REFRESH_FIRST,
            prefetch: 0,
            draw: true,
            sound: if model.pal { Sound::new(53_203_425, 1, true) } else { Sound::new(4_725_000_000, 88, true) },
        };
        let mut g = Genesis { cpu: M68000::new(), z80: Z80::new(), hw };
        // The YM2612 is held with the Z80 from power-on, its sample cycle where the board's is (Nephrite_Disputes.md D-19).
        let ym = &mut g.hw.sound.ym;
        let po = crate::ym2612::POWER_ON;
        (ym.held, ym.next, ym.timers_next, ym.held_at) = (true, po, po, po);
        g.cpu.reset(&mut MainBus(&mut g.hw));
        g
    }

    /// One frame: the 68000 instruction by instruction, the VDP's line events and the Z80 caught up after each.
    pub fn run_frame(&mut self) {
        self.hw.vdp.draw = self.hw.draw;
        loop {
            if self.hw.vdp.bus_dma() {
                // A transfer's hold that a frame's end interrupted.
                self.hw.dma_hold();
            } else if self.hw.locked_up || self.cpu.halted {
                self.hw.clock += 4 * M68K;
            } else {
                let _: Step = self.cpu.step(&mut MainBus(&mut self.hw));
            }
            self.hw.catch_up(&mut self.z80);
            if std::mem::take(&mut self.hw.frame_done) {
                break;
            }
        }
    }
}

impl Hw {
    /// The VDP's slots, the line starts and the vertical interrupt up to `to`, in the order they fall.
    fn run_vdp(&mut self, to: u64) {
        loop {
            let next_line = self.line_begun + LINE;
            let mut t = next_line;
            let mut kind = Event::Line;
            if self.vdp.busy()
                && let Some(s) = self.next_slot(self.vdp.time)
                && s < t
            {
                (t, kind) = (s, Event::Slot);
            }
            if self.vdp.dma_fetch().is_some() {
                let f = self.vdp.fetch_at.max(self.vdp.time);
                if f < t {
                    (t, kind) = (f, Event::Fetch);
                }
            }
            if let Some(v) = self.vint_at.filter(|&v| v < t || (v == t && kind == Event::Line)) {
                (t, kind) = (v, Event::Vint);
            }
            if t > to {
                self.vdp.time = self.vdp.time.max(to);
                return;
            }
            self.vdp.time = t;
            if kind == Event::Fetch {
                let a = self.vdp.dma_fetch().expect("a transfer with room");
                let w = self.dma_read(a);
                self.vdp.dma_fetched(w);
                self.vdp.fetch_at = t + DMA_FETCH;
            } else if kind == Event::Slot {
                self.vdp.slot();
            } else if kind == Event::Vint {
                // The field changes with the frame interrupt, at H $001 (the board's odd flag, D-12).
                self.vdp.field_start();
                self.vint_at = None;
                self.vdp.vint_pending = true;
                self.z80_int = (t, t + Z80_INT);
            } else {
                self.vdp.line_end(self.line as usize);
                self.line_begun = next_line;
                self.line += 1;
                // A frame shortened under its current line (V30 cleared on an NTSC board) ends at once.
                if self.line >= self.vdp.lines() {
                    self.line = 0;
                    self.frame_done = true;
                }
                self.vdp.line_start(self.line);
                (self.vdp.cur_line, self.vdp.cur_line_start) = (self.line, self.line_begun);
                let line = self.line as usize;
                self.vdp.latch_line(line);
                if self.line == self.vdp.vint_line() {
                    self.vint_at = Some(self.line_begun + self.vdp.timing().vint);
                }
            }
        }
    }

    /// The first external slot of the present line after `after`.
    fn next_slot(&self, after: u64) -> Option<u64> {
        let off = after.saturating_sub(self.line_begun);
        let slots = self.vdp.slots(self.line);
        let i = slots.partition_point(|&m| (m as u64) <= off);
        slots.get(i).map(|&m| self.line_begun + m as u64)
    }

    /// The 68000 held until `done`: the clock moved slot by slot.
    fn vdp_wait(&mut self, done: fn(&Vdp) -> bool) {
        self.run_vdp(self.clock);
        while !done(&self.vdp) {
            let next = self.next_slot(self.vdp.time).unwrap_or(self.line_begun + LINE);
            self.clock = self.clock.max(next);
            self.run_vdp(self.clock);
        }
    }

    /// A 68000 access to the main RAM about to begin: the RAM's refresh, if one has been requested by now.
    fn ram_refresh(&mut self) {
        let at = self.clock;
        while self.ram_refresh_at <= at {
            let requested = self.ram_refresh_at;
            if at - requested < RAM_REFRESH_WINDOW {
                self.clock += RAM_REFRESH_WAIT;
                self.ram_refresh_at = at + RAM_REFRESH_AFTER;
            } else {
                self.ram_refresh_at = requested + RAM_REFRESH_WINDOW + RAM_REFRESH_AFTER;
            }
        }
    }

    /// One step of the 68000's hold during a transfer: the clock to the transfer's next bus read or slot.
    fn dma_hold(&mut self) {
        self.run_vdp(self.clock);
        let slot = self.next_slot(self.vdp.time).unwrap_or(self.line_begun + LINE);
        let next = if self.vdp.dma_fetch().is_some() { slot.min(self.vdp.fetch_at.max(self.vdp.time + 1)) } else { slot };
        self.clock = self.clock.max(next);
    }

    /// A word a transfer reads from the 68000's bus; the VDP's own addresses give nothing.
    fn dma_read(&mut self, a: u32) -> u16 {
        if (0xC0_0000..=0xDF_FFFF).contains(&(a & 0xFF_FFFF)) { 0 } else { self.read16(a) }
    }

    /// The line events up to the present, then the Z80 to the same time.
    fn catch_up(&mut self, z80: &mut Z80) {
        self.run_vdp(self.clock);
        if self.z80_reset {
            reset_z80(z80);
        }
        if self.z80_reset || self.z80_busreq {
            self.z80_clock = self.z80_clock.max(self.clock);
            return;
        }
        while self.z80_clock < self.clock && !self.z80_busreq {
            z80.step(&mut Z80Bus(self));
        }
    }

    /// A device on `port` driving TH, as a light gun does: the external interrupt when the port enables it.
    pub fn device_th(&mut self, port: usize, th: bool) {
        if self.io.device_th(port, th) {
            let (line, dot) = (self.line, self.dot());
            self.vdp.external(line, dot);
        }
    }

    fn dot(&self) -> u64 {
        self.clock.saturating_sub(self.line_begun).min(LINE - 1)
    }

    /// Whether the 68000 holds the Z80's bus: requested, and the Z80 out of reset.
    fn has_z80_bus(&self) -> bool {
        self.z80_busreq && !self.z80_reset
    }

    // ---- the 68000's map

    /// An unmapped byte: the next instruction's high byte at an even address, zero at an odd one; an unmapped word is
    /// the same two bytes (MacDonald's notes §1, note 1).
    fn open8(&self, a: u32) -> u8 {
        if a & 1 == 0 { (self.prefetch >> 8) as u8 } else { 0 }
    }

    fn read8(&mut self, a: u32) -> u8 {
        let a = a & 0xFF_FFFF;
        match a {
            0x00_0000..=0x3F_FFFF if self.cart.answers(a) => self.cart.read8(a),
            0xA0_0000..=0xA0_FFFF => {
                if self.has_z80_bus() {
                    self.z80_space_read(a as u16 & 0x7FFF, self.clock - Z80_AREA_LEAD)
                } else {
                    self.open8(a)
                }
            }
            0xA1_0000..=0xA1_001F => self.io.read(a, self.clock),
            0xA1_1100 => (self.open8(a) & 0xFE) | (!self.has_z80_bus()) as u8,
            0xC0_0000..=0xDF_FFFF => self.vdp_read8(a),
            0xE0_0000..=0xFF_FFFF => self.wram[a as usize & 0xFFFF],
            _ => self.open8(a),
        }
    }

    fn read16(&mut self, a: u32) -> u16 {
        let a = a & 0xFF_FFFE;
        match a {
            0x00_0000..=0x3F_FFFF if self.cart.answers(a) => self.cart.read16(a),
            0xA0_0000..=0xA0_FFFF if self.has_z80_bus() => {
                let b = self.read8(a) as u16;
                b << 8 | b
            }
            0xA1_0000..=0xA1_001F => {
                let b = self.io.read(a, self.clock) as u16;
                b << 8 | b
            }
            0xA1_1100 => (self.prefetch & 0xFE00) | ((!self.has_z80_bus()) as u16) << 8,
            0xC0_0000..=0xDF_FFFF => self.vdp_read16(a),
            0xE0_0000..=0xFF_FFFF => (self.wram[a as usize & 0xFFFF] as u16) << 8 | self.wram[(a as usize & 0xFFFF) | 1] as u16,
            _ => self.prefetch & 0xFF00,
        }
    }

    fn write8(&mut self, a: u32, v: u8) {
        let a = a & 0xFF_FFFF;
        match a {
            0x00_0000..=0x3F_FFFF => self.cart.write8(a, v),
            0xA0_0000..=0xA0_FFFF => {
                if self.has_z80_bus() {
                    self.z80_space_write(a as u16 & 0x7FFF, v, self.clock - Z80_AREA_LEAD);
                }
            }
            0xA1_0000..=0xA1_001F => self.io.write(a, v, self.clock),
            0xA1_1100 => self.z80_busreq = v & 1 != 0,
            0xA1_1200 => self.set_z80_reset(v & 1 == 0),
            0xA1_30F0..=0xA1_30FF => self.cart.register(a, v),
            0xA1_4000..=0xA1_4003 => self.tmss_write(a, v),
            0xC0_0000..=0xDF_FFFF => self.vdp_write(a, (v as u16) << 8 | v as u16, self.clock),
            0xE0_0000..=0xFF_FFFF => self.wram[a as usize & 0xFFFF] = v,
            _ => {}
        }
    }

    fn write16(&mut self, a: u32, v: u16) {
        let a = a & 0xFF_FFFE;
        match a {
            0xA0_0000..=0xA0_FFFF => self.write8(a, (v >> 8) as u8),
            0xA1_1100 | 0xA1_1200 => self.write8(a, (v >> 8) as u8),
            0xC0_0000..=0xDF_FFFF => self.vdp_write(a, v, self.clock),
            0xE0_0000..=0xFF_FFFF => {
                self.wram[a as usize & 0xFFFF] = (v >> 8) as u8;
                self.wram[(a as usize & 0xFFFF) | 1] = v as u8;
            }
            0x00_0000..=0x3F_FFFF => self.cart.write16(a, v),
            _ => {
                self.write8(a, (v >> 8) as u8);
                self.write8(a | 1, v as u8);
            }
        }
    }

    fn set_z80_reset(&mut self, held: bool) {
        if self.z80_reset && !held {
            self.z80_clock = self.clock;
        }
        if self.z80_reset != held {
            self.sound.ym_reset(held, self.clock - Z80_AREA_LEAD);
        }
        self.z80_reset = held;
    }

    fn tmss_write(&mut self, a: u32, v: u8) {
        if !self.model.tmss() {
            return;
        }
        self.tmss[(a & 3) as usize] = v;
        self.tmss_unlocked = &self.tmss == b"SEGA";
    }

    /// A TMSS console's VDP before "SEGA" is written: the hardware locks up, and so does the machine here.
    fn vdp_locked(&mut self) -> bool {
        if self.model.tmss() && !self.tmss_unlocked {
            self.locked_up = true;
        }
        self.locked_up
    }

    /// MacDonald's rule for the VDP's mirrors: bits 21, 22 and 23 set as `110`, and bits 15-17 and 5-7 clear.
    fn vdp_valid(a: u32) -> bool {
        a & 0xE7_00E0 == 0xC0_0000
    }

    fn vdp_read16(&mut self, a: u32) -> u16 {
        if !Self::vdp_valid(a) || self.vdp_locked() {
            return 0;
        }
        self.run_vdp(self.clock);
        if a & 0x1E < 4 {
            self.vdp_wait(Vdp::read_ready);
        }
        let (line, dot) = (self.line, self.dot());
        match a & 0x1E {
            0x00 | 0x02 => self.vdp.read_data(),
            0x04 | 0x06 => (self.vdp.status(line, dot) & 0x03FF) | (self.prefetch & 0xFC00),
            0x08..=0x0E => self.vdp.hv(line, dot),
            _ => self.prefetch,
        }
    }

    fn vdp_read8(&mut self, a: u32) -> u8 {
        let w = self.vdp_read16(a & !1);
        if a & 1 == 0 { (w >> 8) as u8 } else { w as u8 }
    }

    /// A write to the VDP's ports at master clock `t`, the clock of the processor making it; the PSG at `$10`-`$17`
    /// takes the low byte.
    fn vdp_write(&mut self, a: u32, v: u16, t: u64) {
        if !Self::vdp_valid(a) || self.vdp_locked() {
            return;
        }
        if (0x10..=0x17).contains(&(a & 0x1F)) {
            self.sound.psg_write(v as u8, t);
            return;
        }
        self.run_vdp(self.clock);
        match a & 0x1E {
            0x00 | 0x02 => {
                self.vdp_wait(|v| !v.fifo_full());
                self.vdp.data(v);
            }
            0x04 | 0x06 => {
                let latch = self.vdp.regs[0] & 2;
                let was = self.vdp.bus_dma();
                self.vdp.control(v);
                self.vdp.fetch_at = self.clock;
                if !was && self.vdp.bus_dma() {
                    self.vdp.fetch_at += DMA_START;
                }
                if latch == 0 && self.vdp.regs[0] & 2 != 0 {
                    let (line, dot) = (self.line, self.dot());
                    self.vdp.latch_hv(line, dot);
                }
                // The 68000 is held from the write that started a transfer, but the hold yields at the frame's end
                // and goes on between instructions, so that a frame ends where its last line does.
                while self.vdp.bus_dma() && !self.frame_done {
                    self.dma_hold();
                    self.run_vdp(self.clock);
                }
            }
            _ => {}
        }
    }

    // ---- the Z80's space, as both processors reach it

    /// $0000-$7FFF of the Z80's space at master clock `t`, the clock of the processor reaching it: its RAM, the
    /// YM2612's four ports, the bank register, the VDP.
    fn z80_space_read(&mut self, a: u16, t: u64) -> u8 {
        match a {
            0x0000..=0x3FFF => self.zram[a as usize & 0x1FFF],
            0x4000..=0x5FFF => self.sound.ym_read(a & 3, t),
            0x7F00..=0x7F1F => {
                let w = self.vdp_read16(0xC0_0000 | (a as u32 & 0x1E));
                if a & 1 == 0 { (w >> 8) as u8 } else { w as u8 }
            }
            _ => 0xFF,
        }
    }

    fn z80_space_write(&mut self, a: u16, v: u8, t: u64) {
        match a {
            0x0000..=0x3FFF => self.zram[a as usize & 0x1FFF] = v,
            0x4000..=0x5FFF => self.sound.ym_write(a & 3, v, t),
            0x6000..=0x60FF => self.z80_bank = (self.z80_bank >> 1) | ((v as u16 & 1) << 8),
            0x7F00..=0x7F1F => self.vdp_write(0xC0_0000 | (a as u32 & 0x1F), (v as u16) << 8 | v as u16, t),
            _ => {}
        }
    }

    fn window_address(&self, a: u16) -> u32 {
        (self.z80_bank as u32) << 15 | (a as u32 & 0x7FFF)
    }
}

/// RESET held: the Z80's state as its reset leaves it (PC, I, R, the flip-flops and the mode cleared; AF and SP set).
fn reset_z80(z80: &mut Z80) {
    let g = &mut z80.regs;
    (g.pc, g.i, g.r, g.iff1, g.iff2, g.im, g.ei_pending) = (0, 0, 0, false, false, 0, false);
    (g.af, g.sp) = (0xFFFF, 0xFFFF);
    z80.halted = false;
}

/// The 68000's bus: four clocks an access, after any stall the Z80's window left it.
pub struct MainBus<'a>(pub &'a mut Hw);

impl MainBus<'_> {
    /// A bus cycle to `a`: any refresh's wait that has come due, the Z80 area's extra clock, then four clocks.
    fn cycle(&mut self, a: u32) {
        let hw = &mut *self.0;
        let a = a & 0xFF_FFFF;
        if hw.clock >= hw.refresh_at {
            if a < 0x80_0000 {
                hw.clock += REFRESH_WAIT;
            }
            hw.refresh_at += (hw.clock - hw.refresh_at) / REFRESH * REFRESH + REFRESH;
        }
        if a >= 0xE0_0000 {
            hw.ram_refresh();
        }
        let area = if (0xA0_0000..=0xA0_FFFF).contains(&a) { Z80_AREA_WAIT } else { 0 };
        hw.clock += 4 * M68K + area + std::mem::take(&mut hw.stall);
    }
}

impl MainBusTrait for MainBus<'_> {
    fn read(&mut self, a: Access) -> u16 {
        self.cycle(a.address);
        let v = match a.size {
            Size::Word => self.0.read16(a.address),
            Size::Byte => self.0.read8(a.address) as u16,
        };
        if a.function & 3 == 2 && a.size == Size::Word {
            self.0.prefetch = v;
        }
        v
    }

    fn write(&mut self, a: Access, v: u16) {
        self.cycle(a.address);
        match a.size {
            Size::Word => self.0.write16(a.address, v),
            Size::Byte => self.0.write8(a.address, v as u8),
        }
    }

    fn idle(&mut self, clocks: u32) {
        self.0.clock += clocks as u64 * M68K;
    }

    fn interrupt_level(&mut self) -> u8 {
        self.0.vdp.level()
    }

    fn acknowledge(&mut self, level: u8) -> Option<u8> {
        self.0.vdp.acknowledge(level);
        None
    }
}

/// The Z80's bus: its T-states on the master clock, and the window that reaches the 68000's.
pub struct Z80Bus<'a>(pub &'a mut Hw);

impl Z80Bus<'_> {
    fn t(&mut self, n: u32) {
        self.0.z80_clock += n as u64 * Z80_T;
    }

    fn read_any(&mut self, a: u16) -> u8 {
        if a < 0x8000 {
            let t = self.0.z80_clock;
            return self.0.z80_space_read(a, t);
        }
        self.0.z80_clock += WINDOW_Z80_WAIT;
        self.0.stall += WINDOW_68K_STALL;
        let addr = self.0.window_address(a);
        if (0xA0_0000..=0xA0_FFFF).contains(&addr) { 0xFF } else { self.0.read8(addr) }
    }

    fn write_any(&mut self, a: u16, v: u8) {
        if a < 0x8000 {
            let t = self.0.z80_clock;
            return self.0.z80_space_write(a, v, t);
        }
        self.0.z80_clock += WINDOW_Z80_WAIT;
        self.0.stall += WINDOW_68K_STALL;
        let addr = self.0.window_address(a);
        if !(0xA0_0000..=0xA0_FFFF).contains(&addr) {
            self.0.write8(addr, v);
        }
    }
}

impl Z80BusTrait for Z80Bus<'_> {
    fn fetch(&mut self, address: u16, _refresh: u16) -> u8 {
        self.t(4);
        self.read_any(address)
    }
    fn read(&mut self, address: u16) -> u8 {
        self.t(3);
        self.read_any(address)
    }
    fn write(&mut self, address: u16, value: u8) {
        self.t(3);
        self.write_any(address, value)
    }
    fn input(&mut self, _port: u16) -> u8 {
        self.t(4);
        0xFF
    }
    fn output(&mut self, _port: u16, _value: u8) {
        self.t(4);
    }
    fn idle(&mut self, t: u32) {
        self.t(t);
    }
    fn int_line(&mut self) -> bool {
        let (from, to) = self.0.z80_int;
        (from..to).contains(&self.0.z80_clock)
    }
    fn nmi_edge(&mut self) -> bool {
        false
    }
    fn acknowledge(&mut self) -> u8 {
        self.t(6);
        0xFF
    }
}

/// A DMA in four words: its kind, then its source, length and data or byte.
fn dma_words(d: crate::vdp::Dma) -> [u64; 4] {
    use crate::vdp::Dma;
    match d {
        Dma::None => [0, 0, 0, 0],
        Dma::Bus { source, left } => [1, source as u64, left as u64, 0],
        Dma::FillWait => [2, 0, 0, 0],
        Dma::Fill { data, left, started } => [3, started as u64, left as u64, data as u64],
        Dma::Copy { source, left, byte } => [4, source as u64, left as u64, byte.map_or(u64::MAX, u64::from)],
    }
}

fn dma_from(d: [u64; 4]) -> crate::vdp::Dma {
    use crate::vdp::Dma;
    match d[0] {
        1 => Dma::Bus { source: d[1] as u32 & 0xFF_FFFE, left: d[2] as u32 },
        2 => Dma::FillWait,
        3 => Dma::Fill { data: d[3] as u16, left: d[2] as u32, started: d[1] != 0 },
        4 => Dma::Copy { source: d[1] as u16, left: d[2] as u32, byte: (d[3] != u64::MAX).then_some(d[3] as u8) },
        _ => Dma::None,
    }
}

/// The words that have left the FIFO and not yet reached memory, at most 16: their count, then each one's time and
/// itself (code, the half-written flag, address and data).
const LANDING_WORDS: usize = 1 + 2 * 16;

fn landing_words(q: &std::collections::VecDeque<(u64, crate::vdp::Entry)>) -> [u64; LANDING_WORDS] {
    let mut w = [0u64; LANDING_WORDS];
    w[0] = q.len().min(16) as u64;
    for (i, (ready, e)) in q.iter().take(16).enumerate() {
        w[1 + 2 * i] = *ready;
        w[2 + 2 * i] = e.code as u64 | (e.half as u64) << 8 | (e.address as u64) << 16 | (e.data as u64) << 32;
    }
    w
}

fn landing_from(w: &[u64; LANDING_WORDS]) -> std::collections::VecDeque<(u64, crate::vdp::Entry)> {
    (0..(w[0] as usize).min(16))
        .map(|i| {
            let p = w[2 + 2 * i];
            (w[1 + 2 * i], crate::vdp::Entry { code: p as u8, half: p >> 8 & 1 != 0, address: (p >> 16) as u16, data: (p >> 32) as u16 })
        })
        .collect()
}

/// The line being drawn when a state was taken: its number plus one (0 for none) and how far it was drawn, its sprite
/// pixels and its latched scroll and window values. Its pixels are not kept, being the picture's and not the machine's
/// (a state must not depend on whether frames were drawn); a loaded state draws them again from itself.
pub struct OpenLine {
    open: [u16; 2],
    sprites: Vec<u8>,
    latch: (Vec<u8>, (u16, u16), [u8; 2]),
}

/// Everything of the Genesis a state holds beyond its memories, read whole before any of it is applied.
pub struct Saved {
    cpu: M68000,
    z80: Z80,
    vdp: crate::vdp::VdpRegs,
    sat_cache: Vec<u8>,
    line_scroll: (Vec<u8>, (u16, u16)),
    line_window: [u8; 2],
    open_line: OpenLine,
    write_path: ([u64; 4], std::collections::VecDeque<(u64, crate::vdp::Entry)>),
    sound: Sound,
    sprite_buffer: Vec<u8>,
    io: crate::io::IoRegs,
    sram_reg: u8,
    banks: Option<[u8; 8]>,
    clock: u64,
    z80_clock: u64,
    z80_int: (u64, u64),
    line: u32,
    line_begun: u64,
    vint_at: Option<u64>,
    z80_bank: u16,
    prefetch: u16,
    z80_busreq: bool,
    z80_reset: bool,
    tmss_unlocked: bool,
    tmss: [u8; 4],
    locked_up: bool,
    stall: u64,
    refresh_at: u64,
    ram_refresh_at: u64,
}

impl Genesis {
    pub fn write_state(&self, w: &mut StateWriter) {
        use emusen_native::State;
        w.group("M68000", |w| self.cpu.write_state(w));
        w.group("Z80", |w| self.z80.write_state(w));
        let h = &self.hw;
        let v = h.vdp.regs_state();
        w.bytes("VdpRegisters", &v.regs);
        w.bools("VdpLatches", &[v.pending, v.read_ready, v.vint_pending, v.hint_pending, v.ext_pending]);
        w.u32("HvLatch", v.hv_latch.map_or(u32::MAX, u32::from));
        w.u8("VdpCode", v.code);
        w.u16("VdpAddress", v.address);
        w.u8("VdpLineCounter", v.hint_counter);
        let fifo: Vec<u16> = v.fifo.iter().flat_map(|e| [e.code as u16 | (e.half as u16) << 8, e.address, e.data]).collect();
        w.u16s("VdpFifo", &fifo);
        w.bytes("VdpFifoPointers", &[v.fifo_count, v.fifo_next]);
        w.u64s("VdpFifoReady", &h.vdp.fifo_at);
        w.u64s("VdpLanding", &landing_words(&h.vdp.landing));
        w.u16("VdpReadBuffer", v.read_buf);
        w.u64s("VdpDma", &dma_words(v.dma));
        w.u64s("VdpTimes", &[v.time, v.fetch_at]);
        w.bytes("SpriteCache", &h.vdp.sat_cache);
        w.bools("SpriteFlags", &v.sprite_flags);
        w.u16("VscrollLatch", v.vscroll_latch);
        w.bytes("Field", &[v.interlace, v.odd as u8]);
        w.bytes("SpriteLineBuffer", &h.vdp.sprite_buffer);
        w.bytes("LineVsram", &h.vdp.line_vsram);
        w.u16s("LineHscroll", &[h.vdp.line_hscroll.0, h.vdp.line_hscroll.1]);
        w.bytes("LineWindow", &h.vdp.line_window);
        w.u16s("OpenLine", &[h.vdp.open.map_or(0, |l| l as u16 + 1), h.vdp.span_x as u16]);
        w.bytes("OpenLineSprites", &h.vdp.span_sprites);
        w.bytes("OpenLineVsram", &h.vdp.span_latch.0);
        w.u16s("OpenLineScroll", &[h.vdp.span_latch.1.0, h.vdp.span_latch.1.1]);
        w.bytes("OpenLineWindow", &h.vdp.span_latch.2);
        let io = h.io.regs_state();
        w.bytes("IoData", &io.data);
        w.bytes("IoCtrl", &io.ctrl);
        w.bytes("IoTx", &io.tx);
        w.bytes("IoSctrl", &io.sctrl);
        w.bools("PadTh", &io.th);
        w.bytes("PadFalls", &io.falls);
        w.u64s("PadLastFall", &io.last_fall);
        w.u8("SramRegister", h.cart.sram_reg);
        w.bool("Mapper", h.cart.banks.is_some());
        w.bytes("MapperPages", &h.cart.banks.unwrap_or_default());
        w.u64s("Clocks", &[h.clock, h.z80_clock, h.z80_int.0, h.z80_int.1, h.line_begun, h.vint_at.unwrap_or(u64::MAX), h.stall, h.refresh_at, h.ram_refresh_at]);
        w.u32("Line", h.line);
        w.u16("Z80Bank", h.z80_bank);
        w.u16("OpenBus", h.prefetch);
        w.bools("Lines", &[h.z80_busreq, h.z80_reset, h.tmss_unlocked, h.locked_up]);
        w.bytes("Tmss", &h.tmss);
        w.group("Sound", |w| h.sound.write_state(w));
    }

    pub fn read_state(&self, r: &mut StateReader) -> Result<Saved, Truncated> {
        use emusen_native::State;
        let mut cpu = self.cpu.clone();
        cpu.read_state(r)?;
        let mut z80 = self.z80.clone();
        z80.read_state(r)?;
        let mut v = crate::vdp::VdpRegs::default();
        r.bytes(&mut v.regs)?;
        let mut latches = [false; 5];
        r.bools(&mut latches)?;
        [v.pending, v.read_ready, v.vint_pending, v.hint_pending, v.ext_pending] = latches;
        let latch = r.u32()?;
        v.hv_latch = (latch != u32::MAX).then_some(latch as u16);
        v.code = r.u8()?;
        v.address = r.u16()?;
        v.hint_counter = r.u8()?;
        let mut fifo = [0u16; 12];
        r.u16s(&mut fifo)?;
        for (e, f) in v.fifo.iter_mut().zip(fifo.chunks(3)) {
            *e = crate::vdp::Entry { code: f[0] as u8, half: f[0] >> 8 != 0, address: f[1], data: f[2] };
        }
        let mut p = [0u8; 2];
        r.bytes(&mut p)?;
        [v.fifo_count, v.fifo_next] = [p[0].min(4), p[1] & 3];
        let mut fifo_at = [0u64; 4];
        r.u64s(&mut fifo_at)?;
        let mut landing = [0u64; LANDING_WORDS];
        r.u64s(&mut landing)?;
        v.read_buf = r.u16()?;
        let mut d = [0u64; 4];
        r.u64s(&mut d)?;
        v.dma = dma_from(d);
        let mut times = [0u64; 2];
        r.u64s(&mut times)?;
        [v.time, v.fetch_at] = times;
        let mut cache = vec![0u8; 320];
        r.bytes(&mut cache)?;
        r.bools(&mut v.sprite_flags)?;
        v.vscroll_latch = r.u16()?;
        let mut field = [0u8; 2];
        r.bytes(&mut field)?;
        (v.interlace, v.odd) = (field[0] & 3, field[1] != 0);
        let mut sprite_buffer = vec![0u8; 320];
        r.bytes(&mut sprite_buffer)?;
        let mut line_vsram = vec![0u8; 80];
        r.bytes(&mut line_vsram)?;
        let mut hs = [0u16; 2];
        r.u16s(&mut hs)?;
        let mut line_window = [0u8; 2];
        r.bytes(&mut line_window)?;
        let mut open = [0u16; 2];
        r.u16s(&mut open)?;
        let mut sprites = vec![0u8; 2 * crate::render::MAX_W];
        r.bytes(&mut sprites)?;
        let mut span_vsram = vec![0u8; 80];
        r.bytes(&mut span_vsram)?;
        let mut span_scroll = [0u16; 2];
        r.u16s(&mut span_scroll)?;
        let mut span_window = [0u8; 2];
        r.bytes(&mut span_window)?;
        let mut io = crate::io::IoRegs::default();
        r.bytes(&mut io.data)?;
        r.bytes(&mut io.ctrl)?;
        r.bytes(&mut io.tx)?;
        r.bytes(&mut io.sctrl)?;
        r.bools(&mut io.th)?;
        r.bytes(&mut io.falls)?;
        r.u64s(&mut io.last_fall)?;
        let sram_reg = r.u8()?;
        let mapper = r.bool()?;
        let mut pages = [0u8; 8];
        r.bytes(&mut pages)?;
        let mut c = [0u64; 9];
        r.u64s(&mut c)?;
        let line = r.u32()?;
        let z80_bank = r.u16()?;
        let prefetch = r.u16()?;
        let mut lines = [false; 4];
        r.bools(&mut lines)?;
        let mut tmss = [0u8; 4];
        r.bytes(&mut tmss)?;
        let mut sound = self.hw.sound.clone();
        sound.read_state(r)?;
        Ok(Saved {
            sound,
            cpu,
            z80,
            vdp: v,
            sat_cache: cache,
            line_scroll: (line_vsram, (hs[0], hs[1])),
            line_window,
            open_line: OpenLine { open, sprites, latch: (span_vsram, (span_scroll[0], span_scroll[1]), span_window) },
            write_path: (fifo_at, landing_from(&landing)),
            sprite_buffer,
            io,
            sram_reg,
            banks: mapper.then_some(pages),
            clock: c[0],
            z80_clock: c[1],
            z80_int: (c[2], c[3]),
            line_begun: c[4],
            vint_at: (c[5] != u64::MAX).then_some(c[5]),
            stall: c[6],
            refresh_at: c[7],
            ram_refresh_at: c[8],
            line,
            z80_bank,
            prefetch,
            z80_busreq: lines[0],
            z80_reset: lines[1],
            tmss_unlocked: lines[2],
            locked_up: lines[3],
            tmss,
        })
    }

    pub fn apply_state(&mut self, s: Saved) {
        self.cpu = s.cpu;
        self.z80 = s.z80;
        let h = &mut self.hw;
        h.vdp.set_regs_state(s.vdp);
        h.sound = s.sound;
        h.vdp.sat_cache = s.sat_cache;
        (h.vdp.line_vsram, h.vdp.line_hscroll) = s.line_scroll;
        h.vdp.line_window = s.line_window;
        (h.vdp.fifo_at, h.vdp.landing) = s.write_path;
        let o = s.open_line;
        (h.vdp.open, h.vdp.span_x) = ((o.open[0] != 0).then(|| o.open[0] as usize - 1), o.open[1] as usize);
        (h.vdp.span_sprites, h.vdp.span_latch) = (o.sprites, o.latch);
        h.vdp.redraw_open_line();
        h.vdp.sprite_buffer = s.sprite_buffer;
        h.io.set_regs_state(s.io);
        h.cart.sram_reg = s.sram_reg;
        if h.cart.banks.is_some() {
            h.cart.banks = s.banks;
        }
        (h.clock, h.z80_clock, h.z80_int, h.line, h.line_begun, h.vint_at) = (s.clock, s.z80_clock, s.z80_int, s.line, s.line_begun, s.vint_at);
        (h.vdp.cur_line, h.vdp.cur_line_start) = (h.line, h.line_begun);
        h.prefetch = s.prefetch;
        (h.z80_bank, h.z80_busreq, h.z80_reset, h.tmss_unlocked, h.tmss, h.locked_up, h.stall) =
            (s.z80_bank, s.z80_busreq, s.z80_reset, s.tmss_unlocked, s.tmss, s.locked_up, s.stall);
        (h.refresh_at, h.ram_refresh_at) = (s.refresh_at, s.ram_refresh_at);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_device_on_th_raises_level_2_and_latches_the_hv_counter() {
        let mut rom = vec![0u8; 0x400];
        rom[0..8].copy_from_slice(&[0, 0xFF, 0, 0, 0, 0, 2, 0]);
        let mut g = Genesis::new(Cart::new(rom, None, "SEGA GENESIS", ""), Model { overseas: true, pal: false, version: 0 });
        let hw = &mut g.hw;
        hw.vdp.regs[0] = 2;
        hw.vdp.regs[11] = 8;
        hw.device_th(0, false);
        assert_eq!(hw.vdp.level(), 0, "TH an input but the port's interrupt off");
        hw.io.write(0xA1_0009, 0x80, 0);
        hw.clock = 5 * LINE + 1000;
        hw.line = 5;
        hw.line_begun = 5 * LINE;
        hw.device_th(0, true);
        assert_eq!(hw.vdp.level(), 2);
        let latched = hw.vdp.hv(5, 1000);
        assert_eq!(hw.vdp.hv(100, 3000), latched, "the counter holds what TH latched");
        hw.vdp.acknowledge(2);
        assert_eq!(hw.vdp.level(), 0);
    }

    /// `mdboard.py`'s RAM loops, from the 68000's start as on the board: the start of each instruction's last read of
    /// the cartridge, which is its prefetch, made when the instruction ends.
    fn loop_fetches(name: &str, words: &[u16], count: usize) -> Vec<u32> {
        let mut code = vec![0x47F9, 0x00FF, 0x0100];
        for _ in 0..count {
            code.extend_from_slice(words);
        }
        code.push(0x60FE);
        let image = crate::pictures::program(&[0x8004, 0x8104, 0x8F02], &[], &code);
        let board = crate::board_bus::LOOPS.iter().find(|l| l.name == name).unwrap();
        let fnv = image.iter().fold(0x811C_9DC5u32, |h, &b| (h ^ b as u32).wrapping_mul(0x0100_0193));
        assert_eq!(fnv, board.image, "{name}: the program is not the one the board ran");
        let mut m = crate::machine::Machine::new(&image, crate::media::Media::read(&image));
        // The board's run had its bus refresh this far from its 68000's first bus cycle, which comes 98 into Nephrite's.
        m.genesis.hw.refresh_at = 98 + 777;
        let g = &mut m.genesis;
        let end = 0x222 + 2 * (words.len() * count) as u32;
        let mut ends = Vec::new();
        while g.cpu.regs.pc < end {
            let pc = g.cpu.regs.pc;
            let _: Step = g.cpu.step(&mut MainBus(&mut g.hw));
            if pc >= 0x222 {
                ends.push((g.hw.clock - 98 - 4 * M68K) as u32);
            }
        }
        ends
    }

    /// The refreshes against the board (D-2): in loops of RAM reads, every instruction ends when the board's does.
    #[test]
    fn ram_loops_keep_the_boards_time_through_both_refreshes() {
        for (name, words, count) in [("ram-reads", &[0x3013][..], 400), ("ram-longs", &[0x2013][..], 300), ("ram-reads-3", &[0x3013, 0x4E71, 0x4E71, 0x4E71][..], 400)] {
            let board = &crate::board_bus::LOOPS.iter().find(|l| l.name == name).unwrap().fetches;
            let ends = loop_fetches(name, words, count);
            let last = *board.last().unwrap();
            let wrong: Vec<u32> = ends.iter().copied().filter(|t| *t < last && !board.contains(t)).collect();
            assert!(wrong.is_empty(), "{name}: {} of {} instructions end off the board's reads, the first at {:?}", wrong.len(), ends.len(), wrong.first());
        }
        // Six NOPs between reads: the board's one late refresh, a wait of four clocks where the rule gives three, at 2,366.
        let board = &crate::board_bus::LOOPS.iter().find(|l| l.name == "ram-reads-6").unwrap().fetches;
        let ends = loop_fetches("ram-reads-6", &[0x3013, 0x4E71, 0x4E71, 0x4E71, 0x4E71, 0x4E71, 0x4E71], 228);
        let first_wrong = ends.iter().find(|t| !board.contains(t));
        assert_eq!(first_wrong, Some(&2415), "ram-reads-6 keeps the board's time until its four-clock wait");
    }

    /// `mdboard.py rom-loop`: `add.w (a0)+,d1; cmp.l a0,d0; bcc.s`, 24 clocks by the manual, over the cartridge or the
    /// main RAM; each round's length in clocks, counted.
    fn rom_loop(base: u32) -> std::collections::BTreeMap<u64, usize> {
        let w = |v: &[u16]| v.iter().flat_map(|x| x.to_be_bytes()).collect::<Vec<u8>>();
        let mut r = vec![0xFFu8; 0x2000];
        r[0..8].copy_from_slice(&w(&[0x00FF, 0xFE00, 0x0000, 0x0200]));
        r[0x100..0x110].copy_from_slice(b"SEGA MEGA DRIVE ");
        let end = base + 0xFFFE;
        r[0x200..0x218].copy_from_slice(&w(&[0x46FC, 0x2700, 0x207C, (base >> 16) as u16, base as u16, 0x203C, (end >> 16) as u16, end as u16, 0x7200, 0xD258, 0xB088, 0x64FA]));
        let mut m = crate::machine::Machine::new(&r, crate::media::Media::read(&r));
        let g = &mut m.genesis;
        let mut times = Vec::new();
        while times.len() < 4000 {
            let pc = g.cpu.regs.pc;
            let _: Step = g.cpu.step(&mut MainBus(&mut g.hw));
            if pc == 0x212 {
                times.push(g.hw.clock);
            }
        }
        let mut rounds = std::collections::BTreeMap::new();
        for p in times.windows(2).skip(10) {
            *rounds.entry((p[1] - p[0]) / M68K).or_insert(0) += 1;
        }
        rounds
    }

    /// D-2's first measurement, which the refreshes' rules were not fitted to: on the board 19.1% of the cartridge's
    /// rounds take 26 clocks, and of the RAM's 13.0% take 26, 16.2% 27 and 3.8% 29, the rest 24.
    #[test]
    fn the_refreshes_cost_the_boards_share_of_rounds() {
        let share = |rounds: &std::collections::BTreeMap<u64, usize>, k: u64| 100.0 * *rounds.get(&k).unwrap_or(&0) as f64 / rounds.values().sum::<usize>() as f64;
        let cartridge = rom_loop(0x1000);
        assert_eq!(cartridge.keys().copied().collect::<Vec<_>>(), [24, 26]);
        assert!((share(&cartridge, 26) - 19.1).abs() < 1.0, "{cartridge:?}");
        let ram = rom_loop(0xFF_0000);
        assert_eq!(ram.keys().copied().collect::<Vec<_>>(), [24, 26, 27, 29]);
        for (k, board) in [(26, 13.0), (27, 16.2), (29, 3.8)] {
            assert!((share(&ram, k) - board).abs() < 1.0, "{k} clocks: {ram:?}");
        }
    }
}

