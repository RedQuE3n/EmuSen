//! The video interface's serialized state, the C# `Vi`.

pub mod scan;

use crate::Skip;
use crate::memory::bus::MemoryBus;
use crate::memory::mi::interrupt;
use crate::state::{State, StateReader, StateResult, StateWriter, boxed};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vi {
    /// `<Fields>k__BackingField`, the auto-property's backing field.
    pub fields: i64,
    pub debt: i64,
    pub field: bool,
    pub half_line: i32,
    pub held: Box<[i32; 625]>,
    pub registers: [u32; 14],
    pub was_blank: bool,
    /// `_debtAt`: the cycle the debt was last brought up to.
    pub debt_at: Skip<i64>,
    /// `Due`: the first cycle the next half line is owed, or never while the signal is unprogrammed.
    pub due: Skip<i64>,
}

impl Default for Vi {
    fn default() -> Self {
        Vi {
            fields: 0,
            debt: 0,
            field: false,
            half_line: 0,
            held: boxed(0),
            registers: [0; 14],
            was_blank: false,
            debt_at: Skip(0),
            due: Skip(0),
        }
    }
}

impl State for Vi {
    fn write_state(&self, w: &mut StateWriter) {
        w.i64("<Fields>k__BackingField", self.fields);
        w.i64("_debt", self.debt);
        w.bool("_field", self.field);
        w.i32("_halfLine", self.half_line);
        w.i32s("_held", &self.held[..]);
        w.u32s("_registers", &self.registers[..]);
        w.bool("_wasBlank", self.was_blank);
    }

    fn read_state(&mut self, r: &mut StateReader) -> StateResult {
        self.fields = r.i64()?; // <Fields>k__BackingField
        self.debt = r.i64()?; // _debt
        self.field = r.bool()?; // _field
        self.half_line = r.i32()?; // _halfLine
        r.i32s(&mut self.held[..])?; // _held
        r.u32s(&mut self.registers[..])?; // _registers
        self.was_blank = r.bool()?; // _wasBlank
        Ok(())
    }
}

pub const CURRENT_LINE: u32 = 0x10;
pub const INTERRUPT: u32 = 0x0C;
pub const VERTICAL_SYNC: u32 = 0x18;
pub const HORIZONTAL_SYNC: u32 = 0x1C;
pub const REGISTERS: u32 = 14;
const PROCESSOR_CLOCK: i64 = 93_750_000;
const NTSC_CLOCK: i64 = 48_681_818;
const PAL_CLOCK: i64 = 49_656_530;
const NTSC_SYNC_LINES: i32 = 525;

impl Vi {
    #[inline]
    fn register(&self, offset: u32) -> u32 {
        self.registers[(offset >> 2) as usize]
    }

    pub fn read32(&self, offset: u32) -> u32 {
        if offset < REGISTERS * 4 { self.registers[(offset >> 2) as usize] } else { 0 }
    }

    pub fn serrate(&self) -> bool {
        self.register(0) & (1 << 6) != 0
    }

    pub fn is_pal(&self) -> bool {
        (self.register(VERTICAL_SYNC) & 0x3FF) as i32 > NTSC_SYNC_LINES + 25
    }

    /// `VideoClock`: the console's video clock, which the AI's DAC divides as well.
    #[inline]
    pub fn video_clock(&self) -> i64 {
        if self.is_pal() { PAL_CLOCK } else { NTSC_CLOCK }
    }

    /// `Programmed`: a line is VI_H_SYNC of the interface's clocks and the field VI_V_SYNC half lines.
    #[inline]
    fn programmed(&self) -> Option<(i32, i64)> {
        let sync = (self.register(VERTICAL_SYNC) & 0x3FF) as i32;
        let line = (self.register(HORIZONTAL_SYNC) & 0xFFF) as i64 * PROCESSOR_CLOCK;
        if sync > 0 && line > 0 { Some((sync, line)) } else { None }
    }
}

impl MemoryBus {
    /// `Write32`: the sync registers set the clock both interfaces run at, so both settle first.
    pub fn vi_write32(&mut self, offset: u32, value: u32) {
        self.settle();
        if offset & !3 == CURRENT_LINE {
            self.mi.clear(interrupt::VIDEO_INTERFACE);
        }
        if offset < REGISTERS * 4 {
            self.vi.registers[(offset >> 2) as usize] = value;
        }
        self.reschedule();
    }

    pub fn vi_settle(&mut self) {
        let now = self.cycles;
        if self.vi.programmed().is_some() {
            self.vi.debt = self.vi.debt.wrapping_add((now - *self.vi.debt_at).wrapping_mul(self.vi.video_clock() * 2));
        }
        *self.vi.debt_at = now;
    }

    /// `Catch`: every half line owed by now, in one go.
    pub fn vi_catch(&mut self) {
        if self.cycles < *self.vi.due {
            return;
        }
        self.vi_settle();
        if let Some((sync, line)) = self.vi.programmed() {
            while self.vi.debt >= line {
                self.vi.debt -= line;
                self.vi_advance(sync);
            }
        }
        self.vi_schedule();
    }

    pub fn vi_schedule(&mut self) {
        let Some((_, line)) = self.vi.programmed() else {
            *self.vi.due = i64::MAX;
            return;
        };
        let step = self.vi.video_clock() * 2;
        let remaining = line - self.vi.debt;
        *self.vi.due = if remaining <= 0 { *self.vi.debt_at } else { *self.vi.debt_at + (remaining + step - 1) / step };
    }

    /// `Rebase`: a loaded state was settled when it was saved.
    pub fn vi_rebase(&mut self) {
        *self.vi.debt_at = self.cycles;
    }

    /// `Advance`: one half line; the count wraps at the vertical sync, and only an interlaced signal changes field.
    fn vi_advance(&mut self, sync: i32) {
        let vi = &mut self.vi;
        vi.half_line += 1;
        if vi.half_line >= sync {
            vi.half_line = 0;
            if vi.serrate() {
                vi.field = !vi.field;
            }
            vi.fields += 1;
        }
        vi.registers[(CURRENT_LINE >> 2) as usize] = ((vi.half_line & !1) | if vi.field { 1 } else { 0 }) as u32;
        if vi.half_line & 1 == 0 && vi.half_line == (vi.register(INTERRUPT) & 0x3FF) as i32 {
            self.mi.raise(interrupt::VIDEO_INTERFACE);
        }
    }
}
