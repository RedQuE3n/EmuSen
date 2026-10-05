//! The 68000's registers as a block of its core's state, through `emusen-native`'s `State` (Beryl_M68k.md §5.4).

use emusen_native::{State, StateReader, StateWriter, Truncated};

use crate::M68000;

impl State for M68000 {
    type Error = Truncated;

    fn write_state(&self, w: &mut StateWriter) {
        let r = &self.regs;
        w.u32s("D", &r.d);
        w.u32s("A", &r.a);
        w.u32("OtherSp", r.other_sp);
        w.u16("Sr", r.sr);
        w.u32("Pc", r.pc);
        w.u16s("Prefetch", &r.prefetch);
        w.bool("Stopped", self.stopped);
        w.bool("Halted", self.halted);
        w.u8("LastLevel", self.last_level);
        w.bool("TracePending", self.trace_pending.is_some());
        w.u32("TraceResume", self.trace_pending.unwrap_or(0));
    }

    fn read_state(&mut self, r: &mut StateReader) -> Result<(), Truncated> {
        let mut s = M68000 { processor: self.processor, space: self.space, ..M68000::default() };
        r.u32s(&mut s.regs.d)?;
        r.u32s(&mut s.regs.a)?;
        s.regs.other_sp = r.u32()?;
        s.regs.sr = r.u16()?;
        s.regs.pc = r.u32()?;
        r.u16s(&mut s.regs.prefetch)?;
        s.stopped = r.bool()?;
        s.halted = r.bool()?;
        s.last_level = r.u8()?;
        let pending = r.bool()?;
        let resume = r.u32()?;
        s.trace_pending = pending.then_some(resume);
        *self = s;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_layout_is_pinned_and_a_block_round_trips() {
        let mut w = StateWriter::layout();
        M68000::new().write_state(&mut w);
        assert_eq!(
            w.into_layout(),
            "0 32 u32[8] D\n32 32 u32[8] A\n64 4 u32 OtherSp\n68 2 u16 Sr\n70 4 u32 Pc\n74 4 u16[2] Prefetch\n78 1 bool Stopped\n79 1 bool Halted\n80 1 u8 LastLevel\n81 1 bool TracePending\n82 4 u32 TraceResume\n"
        );
        let mut cpu = M68000::new();
        cpu.regs.d[3] = 0xDEAD_BEEF;
        cpu.regs.sr = 0xA71F;
        cpu.regs.prefetch = [0x4E71, 0x1234];
        cpu.trace_pending = Some(0x00C0_FFEE);
        cpu.last_level = 6;
        let mut buf = vec![0u8; 86];
        let mut w = StateWriter::new(&mut buf);
        cpu.write_state(&mut w);
        assert_eq!(w.len(), 86);
        let mut back = M68000::new();
        back.read_state(&mut StateReader::new(&buf)).unwrap();
        assert_eq!((back.regs, back.trace_pending, back.last_level), (cpu.regs, cpu.trace_pending, cpu.last_level));
        let before = back.regs;
        assert!(back.read_state(&mut StateReader::new(&buf[..85])).is_err());
        assert_eq!(back.regs, before, "a short block changes nothing");
    }
}
