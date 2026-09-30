//! The debugger's hooks: emusen-native's shared tables and logs, and MercuryRT's own way of observing a step, `ObservedBus`.
//! Nothing here calls the host; a frame that meets a table ends early with a reason. See Mercury_Native.md §8.5.

pub use emusen_native::debug::{Call, Hooks, Range, Write, flag, kind, stop};

use crate::cpu::CpuBus;
use crate::memory::bus::MemoryBus;

/// One bit for each of the CPU's 64K addresses: coverage's width for processor 0.
pub const COVERAGE_WIDTH: u32 = 16;

/// The hooks a machine starts with: one processor, 64K addresses.
pub fn new_hooks() -> Hooks {
    Hooks::new(&[COVERAGE_WIDTH])
}

/// The bus as the observed step sees it: every access passes to the machine's bus, and stores, calls and returns are noted on the way.
/// The plain step is monomorphised on `MemoryBus` itself, so none of this is in it.
pub struct ObservedBus<'a> {
    pub bus: &'a mut MemoryBus,
    pub hooks: &'a mut Hooks,
}

impl CpuBus for ObservedBus<'_> {
    #[inline(always)]
    fn read(&mut self, address: u16) -> u8 {
        self.bus.read(address)
    }

    #[inline(always)]
    fn write(&mut self, address: u16, data: u8) {
        let reported = if self.hooks.writes { self.bus.reported_space(address) } else { None };
        self.bus.write(address, data);
        if let Some((space, offset)) = reported {
            self.hooks.note_write(space, offset, data, 0);
        }
    }

    #[inline(always)]
    fn tick(&mut self, cycles: i32) {
        self.bus.tick(cycles)
    }

    fn stop(&mut self) {
        self.bus.stop()
    }

    fn note_call(&mut self, source: u16, target: u16, kind: u32) {
        self.hooks.note_call(source as u32, target as u32, kind);
    }

    fn note_return(&mut self) {
        self.hooks.note_return();
    }
}
