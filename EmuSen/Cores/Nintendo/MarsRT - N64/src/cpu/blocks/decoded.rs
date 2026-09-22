//! Step 1 of the recompiler: a block's instructions run as the interpreter's steps, decoded once. It is also the path compiled code falls back to. See Mars_Native.md §5.8.

use crate::cpu::Cpu;
use crate::cpu::blocks::Block;
use crate::cpu::blocks::ops::{Handler, RAISED};
use crate::cpu::blocks::verify::Shadow;
use crate::cpu::interp::{NOT_STORED, THROUGH_BUS};
use crate::memory::bus::MemoryBus;

/// One instruction: its word and its handler.
#[derive(Clone, Copy)]
pub struct Op {
    pub word: u32,
    pub run: Handler,
}

/// Whether a store could have changed the block's own words, or went where a device may answer.
#[inline(always)]
pub fn rewrote(block: &Block, landed: u32) -> bool {
    landed == THROUGH_BUS || (landed != NOT_STORED && landed.wrapping_add(8) > block.start && landed < block.end())
}

/// Runs the block's instructions as `Cpu::step` runs them, less the fetch and the interrupt check, from a step boundary the
/// dispatcher has opened at its first; returns at the first boundary where the interpreter would look at something this loop
/// does not: the interrupt check's inputs, a write that could have changed the code, the frame's end, or the straight line left.
pub fn run(cpu: &mut Cpu, bus: &mut MemoryBus, block: &Block, entry: u64, cap_at: i64, fields: i64, mut shadow: Option<&mut Shadow>) {
    let mut written = *bus.written;
    for (k, op) in block.ops.iter().enumerate() {
        if k != 0 {
            let at = entry.wrapping_add(4 * k as u64);
            if cpu.pc != at
                || cpu.run.recheck
                || bus.mi.asserted() != cpu.run.asserted_seen
                || *bus.written != written
                || bus.cycles >= cap_at
                || bus.vi.fields != fields
            {
                return;
            }
            cpu.current_pc = at;
            cpu.in_delay_slot = cpu.branch_pending;
        }
        cpu.branch_pending = false;
        cpu.pc = cpu.next_pc;
        cpu.next_pc = cpu.pc.wrapping_add(4);
        // SAFETY: the handler borrows the two halves for the call alone.
        let answer = unsafe { (op.run)(cpu, bus, op.word) };
        if answer == RAISED {
            cpu.enter_exception();
            bus.tick(1);
            if let Some(s) = shadow.as_deref_mut() {
                s.follow(cpu, bus, "a raise in a decoded block");
            }
            return;
        }
        let landed = answer as u32;
        let rewrote = rewrote(block, landed);
        if landed != NOT_STORED && !rewrote {
            written = *bus.written;
        }
        bus.tick(1 + cpu.extra_cycles as i64);
        cpu.extra_cycles = 0;
        cpu.instructions += 1;
        if bus.cycles >= cpu.run.timer_due {
            cpu.timer_reached(bus);
        }
        cpu.last_count = bus.count();
        if let Some(s) = shadow.as_deref_mut() {
            s.follow(cpu, bus, "a decoded instruction");
        }
        if rewrote {
            return;
        }
    }
}
