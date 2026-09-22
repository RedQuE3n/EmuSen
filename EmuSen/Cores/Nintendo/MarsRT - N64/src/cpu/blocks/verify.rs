//! The verifier: an interpreter run beside the blocks, stepped after every instruction they run and compared. C#'s `VerifyBlockStep`, made stronger; see Mars_Native.md §5.8.

use crate::cpu::Cpu;
use crate::memory::bus::MemoryBus;

/// The interpreter's machine, a step behind or level with the one the blocks run.
pub struct Shadow {
    pub cpu: Cpu,
    pub bus: MemoryBus,
    pub steps: u64,
}

impl Shadow {
    pub fn of(cpu: &Cpu, bus: &MemoryBus) -> Box<Shadow> {
        Box::new(Shadow { cpu: cpu.clone(), bus: bus.clone(), steps: 0 })
    }

    /// One interpreter step, then the two compared; `what` names the path the blocks took.
    pub fn follow(&mut self, cpu: &Cpu, bus: &MemoryBus, what: &str) {
        self.cpu.step(&mut self.bus);
        self.steps += 1;
        self.check(cpu, bus, what);
    }

    /// The idle loop, which both run by the same code, from the same state.
    pub fn idle(&mut self, cap_at: i64, rsp_whole: bool) -> bool {
        self.cpu.try_idle(&mut self.bus, cap_at, rsp_whole)
    }

    /// Everything a step can change but RDRAM, which `check_all` compares.
    pub fn check(&self, cpu: &Cpu, bus: &MemoryBus, what: &str) {
        let (a, b) = (&self.cpu, cpu);
        let run = |c: &Cpu| (c.run.mode, c.run.recheck, c.run.asserted_seen, c.run.timer_due, c.run.exceptions);
        let same = a == b
            && run(a) == run(b)
            && self.bus.cycles == bus.cycles
            && self.bus.count_bias == bus.count_bias
            && *self.bus.next_event == *bus.next_event
            && self.bus.mi == bus.mi
            && self.bus.sp == bus.sp
            && self.bus.si == bus.si
            && self.bus.pi == bus.pi
            && self.bus.vi == bus.vi
            && self.bus.ai == bus.ai;
        if !same {
            panic!(
                "the recompiler parted from the interpreter after {} steps, after {what}: interpreter pc {:X} current {:X} next {:X} pending {} slot {} instructions {} cycles {} last_count {}; blocks pc {:X} current {:X} next {:X} pending {} slot {} instructions {} cycles {} last_count {}; registers {}; run {:?} vs {:?}",
                self.steps,
                a.pc,
                a.current_pc,
                a.next_pc,
                a.branch_pending,
                a.in_delay_slot,
                a.instructions,
                self.bus.cycles,
                a.last_count,
                b.pc,
                b.current_pc,
                b.next_pc,
                b.branch_pending,
                b.in_delay_slot,
                b.instructions,
                bus.cycles,
                b.last_count,
                (0..32).filter(|&r| a.gpr[r] != b.gpr[r]).map(|r| format!("r{r} {:X}/{:X}", a.gpr[r], b.gpr[r])).collect::<Vec<_>>().join(" "),
                run(a),
                run(b),
            );
        }
    }

    /// The whole machine, RDRAM and all; made at the end of a run.
    pub fn check_all(&self, cpu: &Cpu, bus: &MemoryBus) {
        self.check(cpu, bus, "the end of the run");
        assert!(self.bus == *bus, "the recompiler's machine parted from the interpreter's after {} steps, outside the processor", self.steps);
    }
}
