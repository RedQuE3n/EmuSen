//! The idle loop, a branch to itself over a no-operation, run in one piece: C#'s `RunIdle`, entered from the interpreter. See Mars_Native.md §5.2.

use crate::memory::bus::MemoryBus;
use crate::memory::dp_threads::site;
use crate::cpu::cop0::ENTRY_HI;
use crate::cpu::Cpu;
use crate::cpu::interp::{KERNEL_DIRECT_BASE, KERNEL_DIRECT_SIZE};
use crate::cpu::segments::{self, Mode, Segment};
use crate::cpu::tlb::TlbResult;

/// The cycles of one turn, and of its branch, as C#'s `BlockShape` decodes them: one each.
const IDLE_CYCLES: i64 = 2;
const IDLE_BRANCH_CYCLES: i64 = 1;

impl Cpu {
    /// Where C#'s `StepBlock` would run `RunIdle`: at the loop's head, in kernel mode, with no branch pending.
    #[inline(never)]
    pub fn try_idle(&mut self, bus: &mut MemoryBus, cap_at: i64, rsp_whole: bool) -> bool {
        let pc = self.pc;
        if self.branch_pending || self.run.mode != Mode::Kernel || pc & 3 != 0 {
            return false;
        }
        let direct = pc.wrapping_sub(KERNEL_DIRECT_BASE) < KERNEL_DIRECT_SIZE;
        let physical = if direct {
            (pc as u32) & 0x1FFF_FFFF
        } else {
            if self.wide_addressing() || segments::decode(pc, self.run.mode, false) != Segment::Mapped {
                return false;
            }
            match self.tlb.try_translate(pc, self.cop0[ENTRY_HI], false) {
                TlbResult::Mapped(physical) => physical,
                _ => return false,
            }
        };
        if physical as usize + 8 > bus.rdram.len() || (!direct && (physical & 0xFFF) + 8 > 0x1000) {
            return false;
        }
        bus.dp.wait_read_range(physical, 8, site::BLOCK);
        let branch = bus.rdram.be32(physical);
        let to_itself = branch == 0x1000_FFFF || ((branch >> 26) == 2 && ((branch & 0x03FF_FFFF) << 2) == (physical & 0x0FFF_FFFF));
        if !to_itself || bus.rdram.be32(physical + 4) != 0 || (!direct && branch != 0x1000_FFFF) {
            return false;
        }

        // The check a step makes before its instruction; a raise is entered as the step's own would be.
        self.current_pc = pc;
        self.in_delay_slot = false;
        let asserted = bus.mi.asserted();
        if (self.run.recheck || asserted != self.run.asserted_seen) && self.check_interrupts(asserted).is_err() {
            self.enter_exception();
            bus.tick(1);
            return true;
        }
        self.run_idle(bus, pc, cap_at, rsp_whole);
        true
    }

    /// `RunIdle`: whole turns passed at once while the signal processor is halted, stepped beside it while it runs.
    fn run_idle(&mut self, bus: &mut MemoryBus, entry: u64, cap_at: i64, rsp_whole: bool) {
        let stop = (*bus.next_event).min(self.run.timer_due).min(cap_at);
        let written = *bus.written;
        let branch = IDLE_BRANCH_CYCLES;
        let slot = IDLE_CYCLES - branch;
        let mut after_slot;
        let mut at_slot = false;
        let idle_from = self.instructions;
        let whole = branch == 1 && slot == 1 && rsp_whole && !bus.sp.single_step;

        loop {
            if bus.sp.processor.halted {
                let turns = if at_slot { 0 } else { (stop - bus.cycles - 1) / IDLE_CYCLES - 1 };
                if turns > 0 {
                    bus.cycles += turns * IDLE_CYCLES;
                    self.instructions += turns * 2;
                    self.run.idle_turns_passed += turns;
                }
            } else if whole {
                let ran = bus.rsp_run_to_event(stop - bus.cycles - 1);
                if ran > 0 {
                    bus.cycles += ran;
                    self.instructions += ran;
                    self.run.rsp_steps += ran;
                    after_slot = if ran & 1 != 0 { at_slot } else { !at_slot };
                    at_slot = !after_slot;
                    if *bus.written != written || bus.mi.asserted() != self.run.asserted_seen {
                        break;
                    }
                }
            }

            let cycles = if at_slot { slot } else { branch };
            after_slot = at_slot;
            bus.cycles += cycles;
            self.instructions += 1;
            if !bus.sp.processor.halted && self.rsp_ran(bus, cycles, written) {
                break;
            }
            if bus.cycles >= stop {
                break;
            }
            at_slot = !at_slot;
        }

        self.pc = if after_slot { entry } else { entry.wrapping_add(4) };
        self.next_pc = if after_slot { entry.wrapping_add(4) } else { entry };
        self.branch_pending = !after_slot;
        self.in_delay_slot = after_slot;
        self.current_pc = if after_slot { entry.wrapping_add(4) } else { entry };
        self.run.idle_instructions += self.instructions - idle_from;
        self.after_instruction(bus);
    }

    /// `RspRan`: the signal processor's share of one instruction, stepped as a tick steps it, and whether it ends the run.
    #[inline(always)]
    fn rsp_ran(&mut self, bus: &mut MemoryBus, cycles: i64, written: i64) -> bool {
        self.run.rsp_steps += cycles;
        bus.sp_step(cycles);
        *bus.written != written || bus.mi.asserted() != self.run.asserted_seen
    }

    /// `AfterInstruction`: the events due, the timer, the count.
    fn after_instruction(&mut self, bus: &mut MemoryBus) {
        if bus.cycles >= *bus.next_event {
            bus.run_events();
        }
        if bus.cycles >= self.run.timer_due {
            self.timer_reached(bus);
        }
        self.last_count = bus.count();
    }
}
