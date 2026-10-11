//! The R3000A's registers as a block of its core's state, through `emusen-native`'s `State` (BlackMoon_R3000.md §4).

use emusen_native::{State, StateReader, StateWriter, Truncated};

use crate::{BranchDelay, LoadDelay, R3000, Registers};

impl State for R3000 {
    type Error = Truncated;

    fn write_state(&self, w: &mut StateWriter) {
        let r = &self.regs;
        w.u32s("R", &r.r);
        w.u32("Hi", r.hi);
        w.u32("Lo", r.lo);
        w.u32("Pc", r.pc);
        w.u32s("Cop0", &[r.sr, r.cause, r.epc, r.bad_vaddr, r.tar, r.dcic, r.bpc, r.bda, r.bpcm, r.bdam]);
        w.group("Branch", |w| {
            w.bool("Slot", r.branch.slot);
            w.bool("Taken", r.branch.taken);
            w.u32("Target", r.branch.target);
        });
        w.group("Load", |w| {
            w.bool("Pending", r.load.register.is_some());
            w.u8("Register", r.load.register.unwrap_or(0));
            w.u32("Value", r.load.value);
        });
    }

    fn read_state(&mut self, r: &mut StateReader) -> Result<(), Truncated> {
        let mut g = Registers::default();
        r.u32s(&mut g.r)?;
        g.hi = r.u32()?;
        g.lo = r.u32()?;
        g.pc = r.u32()?;
        let mut cop0 = [0u32; 10];
        r.u32s(&mut cop0)?;
        [g.sr, g.cause, g.epc, g.bad_vaddr, g.tar, g.dcic, g.bpc, g.bda, g.bpcm, g.bdam] = cop0;
        g.branch = BranchDelay { slot: r.bool()?, taken: r.bool()?, target: r.u32()? };
        let pending = r.bool()?;
        let register = r.u8()?;
        g.load = LoadDelay { register: pending.then_some(register), value: r.u32()? };
        self.regs = g;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_layout_is_pinned_and_a_block_round_trips() {
        let mut w = StateWriter::layout();
        R3000::new().write_state(&mut w);
        assert_eq!(
            w.into_layout(),
            "0 128 u32[32] R\n128 4 u32 Hi\n132 4 u32 Lo\n136 4 u32 Pc\n140 40 u32[10] Cop0\n180 1 bool Branch.Slot\n181 1 bool Branch.Taken\n182 4 u32 Branch.Target\n186 1 bool Load.Pending\n187 1 u8 Load.Register\n188 4 u32 Load.Value\n"
        );
        let mut cpu = R3000::new();
        cpu.regs.r[31] = 0x8001_0000;
        cpu.regs.epc = 0xBFC0_0180;
        cpu.regs.bdam = 0xFFFF_FFFF;
        cpu.regs.branch = BranchDelay { slot: true, taken: true, target: 0x8003_0000 };
        cpu.regs.load = LoadDelay { register: Some(0), value: 0x1234_5678 };
        let mut buf = vec![0u8; 192];
        let mut w = StateWriter::new(&mut buf);
        cpu.write_state(&mut w);
        assert_eq!(w.len(), 192);
        let mut back = R3000::new();
        back.read_state(&mut StateReader::new(&buf)).unwrap();
        assert_eq!(back.regs, cpu.regs);
        let before = back.regs;
        assert!(back.read_state(&mut StateReader::new(&buf[..191])).is_err());
        assert_eq!(back.regs, before, "a short block changes nothing");
    }
}
