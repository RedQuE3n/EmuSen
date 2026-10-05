//! The Z80's registers as a block of its core's state, through `emusen-native`'s `State` (Beryl_Z80.md §6).

use emusen_native::{State, StateReader, StateWriter, Truncated};

use crate::Z80;

impl State for Z80 {
    type Error = Truncated;

    fn write_state(&self, w: &mut StateWriter) {
        let r = &self.regs;
        w.u16s("Main", &[r.af, r.bc, r.de, r.hl]);
        w.u16s("Alternate", &[r.af_, r.bc_, r.de_, r.hl_]);
        w.u16s("Index", &[r.ix, r.iy]);
        w.u16("Sp", r.sp);
        w.u16("Pc", r.pc);
        w.u8("I", r.i);
        w.u8("R", r.r);
        w.u16("Wz", r.wz);
        w.u8("Q", r.q);
        w.bool("P", r.p);
        w.bool("Iff1", r.iff1);
        w.bool("Iff2", r.iff2);
        w.u8("Im", r.im);
        w.bool("EiPending", r.ei_pending);
        w.bool("Halted", self.halted);
    }

    fn read_state(&mut self, r: &mut StateReader) -> Result<(), Truncated> {
        let mut s = Z80 { processor: self.processor, space: self.space, ..Z80::default() };
        let g = &mut s.regs;
        let mut main = [0u16; 4];
        r.u16s(&mut main)?;
        [g.af, g.bc, g.de, g.hl] = main;
        r.u16s(&mut main)?;
        [g.af_, g.bc_, g.de_, g.hl_] = main;
        let mut index = [0u16; 2];
        r.u16s(&mut index)?;
        [g.ix, g.iy] = index;
        g.sp = r.u16()?;
        g.pc = r.u16()?;
        g.i = r.u8()?;
        g.r = r.u8()?;
        g.wz = r.u16()?;
        g.q = r.u8()?;
        g.p = r.bool()?;
        g.iff1 = r.bool()?;
        g.iff2 = r.bool()?;
        g.im = r.u8()?;
        g.ei_pending = r.bool()?;
        s.halted = r.bool()?;
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
        Z80::new().write_state(&mut w);
        assert_eq!(
            w.into_layout(),
            "0 8 u16[4] Main\n8 8 u16[4] Alternate\n16 4 u16[2] Index\n20 2 u16 Sp\n22 2 u16 Pc\n24 1 u8 I\n25 1 u8 R\n26 2 u16 Wz\n28 1 u8 Q\n29 1 bool P\n30 1 bool Iff1\n31 1 bool Iff2\n32 1 u8 Im\n33 1 bool EiPending\n34 1 bool Halted\n"
        );
        let mut cpu = Z80::new();
        cpu.regs.af_ = 0xBEEF;
        cpu.regs.iy = 0x1234;
        cpu.regs.wz = 0x5A5A;
        cpu.regs.im = 2;
        cpu.regs.ei_pending = true;
        cpu.halted = true;
        let mut buf = vec![0u8; 35];
        let mut w = StateWriter::new(&mut buf);
        cpu.write_state(&mut w);
        assert_eq!(w.len(), 35);
        let mut back = Z80::new();
        back.read_state(&mut StateReader::new(&buf)).unwrap();
        assert_eq!((back.regs, back.halted), (cpu.regs, cpu.halted));
        let before = back.regs;
        assert!(back.read_state(&mut StateReader::new(&buf[..34])).is_err());
        assert_eq!(back.regs, before, "a short block changes nothing");
    }
}
