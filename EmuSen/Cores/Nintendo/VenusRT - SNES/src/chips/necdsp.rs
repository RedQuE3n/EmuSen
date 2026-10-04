//! The NEC µPD77C25 of the DSP-n cartridges and the µPD96050 of the ST010 and ST011, from fullsnes ("SNES Cart
//! DSP-n/ST010/ST011", its registers, ALU, LD and JP chapters): the program and data ROMs the player supplies, the
//! data RAM, the two accumulators and their flags, the multiplier, and the DR/SR handshake the S-CPU polls. It runs
//! on a clock of its own, caught up at each S-CPU access (D-32). See VenusRT_Native.md §24.

/// The chip's instruction rate against the master clock (D-32): 7.60 MHz for the DSP-n, 10 MHz for the ST01x.
pub const DSPN_RATIO: (u64, u64) = (760_000, 2_147_727);
pub const ST_RATIO: (u64, u64) = (1_000_000, 2_147_727);

mod sr {
    pub const RQM: u16 = 0x8000;
    pub const DRS: u16 = 0x1000;
    pub const DRC: u16 = 0x0400;
}

/// One accumulator's flags, fullsnes's S1, S0, C, Z, OV1 and OV0.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Flags {
    pub s1: bool,
    pub s0: bool,
    pub c: bool,
    pub z: bool,
    pub ov1: bool,
    pub ov0: bool,
}

impl Flags {
    fn bits(self) -> u8 {
        (self.s1 as u8) << 5 | (self.s0 as u8) << 4 | (self.c as u8) << 3 | (self.z as u8) << 2 | (self.ov1 as u8) << 1 | self.ov0 as u8
    }

    fn from_bits(b: u8) -> Flags {
        Flags { s1: b & 0x20 != 0, s0: b & 0x10 != 0, c: b & 8 != 0, z: b & 4 != 0, ov1: b & 2 != 0, ov0: b & 1 != 0 }
    }
}

/// Which register the S-CPU reached.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Port {
    Dr,
    Sr,
    Ram(usize),
}

/// One DR transfer, by the chip (with PC after it) or the S-CPU, or an S-CPU store into an ST01x's RAM, for the
/// oracle's log.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transfer {
    ChipRead { pc: u16 },
    ChipWrite { value: u16, pc: u16 },
    HostRead(u8),
    HostWrite(u8),
    HostRam(u16, u8),
}

#[derive(Clone, Debug)]
pub struct NecDsp {
    /// The µPD96050 (ST010, ST011) rather than the µPD77C25.
    pub st: bool,
    pub program: Box<[u32]>,
    pub data_rom: Box<[u16]>,
    pub ram: Box<[u16]>,
    pub pc: u16,
    pub rp: u16,
    pub dp: u16,
    pub stack: [u16; 8],
    pub sp: u8,
    pub k: u16,
    pub l: u16,
    pub a: u16,
    pub b: u16,
    pub fa: Flags,
    pub fb: Flags,
    pub tr: u16,
    pub trb: u16,
    pub sr: u16,
    pub dr: u16,
    pub so: u16,
    pub si: u16,
    /// Instructions run since power-on, the chip's clock.
    pub cycles: u64,
    pub ratio: (u64, u64),
    /// The debugger's seam, fitted for an observed frame only; not in the state.
    pub probe: Option<Box<crate::probe::Probe>>,
    /// The transfers with the chip cycle of each, while the oracle fits a log (VenusRT_DspHle.md §4); not in the state.
    pub transfers: Option<Vec<(u64, Transfer)>>,
}

impl NecDsp {
    /// The chip from a firmware image of 8,192 bytes (µPD77C25: 2,048 opcodes, 1,024 data words) or 53,248
    /// (µPD96050: 16,384 and 2,048), in fullsnes's "newer" little-endian layout, or its big-endian "old" one, told
    /// apart by the "JRQM $" every image opens with.
    pub fn from_firmware(image: &[u8]) -> Option<NecDsp> {
        let (st, opcodes, words) = match image.len() {
            8192 => (false, 2048, 1024),
            53248 => (true, 16384, 2048),
            _ => return None,
        };
        let big = (0..4).any(|i| image[i * 3] == 0x97 && image[i * 3 + 1] == 0xC0);
        let program: Box<[u32]> = (0..opcodes)
            .map(|i| {
                let b = &image[i * 3..i * 3 + 3];
                if big { (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32 } else { b[0] as u32 | (b[1] as u32) << 8 | (b[2] as u32) << 16 }
            })
            .collect();
        let data = &image[opcodes * 3..];
        let data_rom: Box<[u16]> =
            (0..words).map(|i| if big { u16::from_be_bytes([data[i * 2], data[i * 2 + 1]]) } else { u16::from_le_bytes([data[i * 2], data[i * 2 + 1]]) }).collect();
        let mut d = NecDsp {
            st,
            program,
            data_rom,
            ram: vec![0; if st { 2048 } else { 256 }].into(),
            pc: 0,
            rp: 0,
            dp: 0,
            stack: [0; 8],
            sp: 0,
            k: 0,
            l: 0,
            a: 0,
            b: 0,
            fa: Flags::default(),
            fb: Flags::default(),
            tr: 0,
            trb: 0,
            sr: 0,
            dr: 0,
            so: 0,
            si: 0,
            cycles: 0,
            ratio: if st { ST_RATIO } else { DSPN_RATIO },
            probe: None,
            transfers: None,
        };
        d.reset();
        Some(d)
    }

    /// fullsnes, "Reset": PC, the flags and SR cleared, RP all ones; the RAM and other registers kept.
    pub fn reset(&mut self) {
        self.pc = 0;
        self.fa = Flags::default();
        self.fb = Flags::default();
        self.sr = 0;
        self.rp = self.rp_mask();
    }

    fn pc_mask(&self) -> u16 {
        if self.st { 0x3FFF } else { 0x07FF }
    }

    fn rp_mask(&self) -> u16 {
        if self.st { 0x07FF } else { 0x03FF }
    }

    fn dp_mask(&self) -> u16 {
        if self.st { 0x07FF } else { 0x00FF }
    }

    fn depth(&self) -> u8 {
        if self.st { 8 } else { 4 }
    }

    /// Runs the chip until its clock reaches the master clock `clock` stands for.
    pub fn run_to(&mut self, clock: u64) {
        let target = (clock as u128 * self.ratio.0 as u128 / self.ratio.1 as u128) as u64;
        while self.cycles < target {
            if let Some(p) = self.probe.as_mut() {
                // The debugger's address is the opcode's byte offset in the program, three bytes each.
                if p.before(self.pc as u32 * 3) {
                    return;
                }
            }
            self.step();
            self.cycles += 1;
        }
    }

    /// The S-CPU's read; `side_effects` false is a look that moves no handshake.
    pub fn host_read(&mut self, port: Port, side_effects: bool) -> u8 {
        match port {
            Port::Sr => (self.sr >> 8) as u8,
            Port::Ram(i) => {
                let w = self.ram[(i >> 1) & (self.ram.len() - 1)];
                if i & 1 == 0 { w as u8 } else { (w >> 8) as u8 }
            }
            Port::Dr => {
                if side_effects {
                    self.note(Transfer::HostRead((self.dr >> if self.sr & (sr::DRC | sr::DRS) == sr::DRS { 8 } else { 0 }) as u8));
                }
                if self.sr & sr::DRC != 0 {
                    if side_effects {
                        self.sr &= !sr::RQM;
                    }
                    self.dr as u8
                } else if self.sr & sr::DRS == 0 {
                    if side_effects {
                        self.sr |= sr::DRS;
                    }
                    self.dr as u8
                } else {
                    if side_effects {
                        self.sr &= !(sr::DRS | sr::RQM);
                    }
                    (self.dr >> 8) as u8
                }
            }
        }
    }

    pub fn host_write(&mut self, port: Port, value: u8) {
        match port {
            Port::Sr => {}
            Port::Ram(i) => {
                self.note(Transfer::HostRam(i as u16, value));
                let n = self.ram.len() - 1;
                let w = &mut self.ram[(i >> 1) & n];
                *w = if i & 1 == 0 { (*w & 0xFF00) | value as u16 } else { (*w & 0x00FF) | (value as u16) << 8 };
            }
            Port::Dr => {
                self.note(Transfer::HostWrite(value));
                if self.sr & sr::DRC != 0 {
                    self.dr = (self.dr & 0xFF00) | value as u16;
                    self.sr &= !sr::RQM;
                } else if self.sr & sr::DRS == 0 {
                    self.dr = (self.dr & 0xFF00) | value as u16;
                    self.sr |= sr::DRS;
                } else {
                    self.dr = (self.dr & 0x00FF) | (value as u16) << 8;
                    self.sr &= !(sr::DRS | sr::RQM);
                }
            }
        }
    }

    #[inline]
    fn note(&mut self, t: Transfer) {
        if let Some(log) = self.transfers.as_mut() {
            log.push((self.cycles, t));
        }
    }

    /// One instruction: ALU, LD or JP by bits 23-22.
    pub fn step(&mut self) {
        let op = self.program[self.pc as usize & (self.program.len() - 1)];
        self.pc = (self.pc + 1) & self.pc_mask();
        match op >> 22 {
            0 | 1 => self.alu(op),
            2 => self.jump(op),
            _ => {
                let id = (op >> 6) as u16;
                self.write_dst(op & 0xF, id);
            }
        }
    }

    fn product(&self) -> (u16, u16) {
        let p = (self.k as i16 as i32 * self.l as i16 as i32) << 1;
        ((p >> 16) as u16, p as u16)
    }

    fn read_src(&mut self, src: u32) -> u16 {
        match src {
            0 => self.trb,
            1 => self.a,
            2 => self.b,
            3 => self.tr,
            4 => self.dp,
            5 => self.rp,
            6 => self.data_rom[self.rp as usize & (self.data_rom.len() - 1)],
            7 => if self.fa.s1 { 0x7FFF } else { 0x8000 },
            8 => {
                self.note(Transfer::ChipRead { pc: self.pc });
                self.sr |= sr::RQM;
                self.dr
            }
            9 => self.dr,
            10 => self.sr,
            11 | 12 => self.si,
            13 => self.k,
            14 => self.l,
            _ => self.ram[self.dp as usize & (self.ram.len() - 1)],
        }
    }

    fn write_dst(&mut self, dst: u32, v: u16) {
        match dst {
            0 => {}
            1 => self.a = v,
            2 => self.b = v,
            3 => self.tr = v,
            4 => self.dp = v & self.dp_mask(),
            5 => self.rp = v & self.rp_mask(),
            6 => {
                self.note(Transfer::ChipWrite { value: v, pc: self.pc });
                self.dr = v;
                self.sr |= sr::RQM;
            }
            // RQM and DRS are the handshake's, read-only to the program.
            7 => self.sr = (self.sr & (sr::RQM | sr::DRS)) | (v & !(sr::RQM | sr::DRS)),
            8 | 9 => self.so = v,
            10 => self.k = v,
            11 => {
                self.k = v;
                self.l = self.data_rom[self.rp as usize & (self.data_rom.len() - 1)];
            }
            12 => {
                self.l = v;
                self.k = self.ram[(self.dp | 0x40) as usize & (self.ram.len() - 1)];
            }
            13 => self.l = v,
            14 => self.trb = v,
            _ => {
                let n = self.ram.len() - 1;
                self.ram[self.dp as usize & n] = v;
            }
        }
    }

    fn alu(&mut self, op: u32) {
        let idb = self.read_src((op >> 4) & 0xF);
        let code = (op >> 16) & 0xF;
        if code != 0 {
            let (m, n) = self.product();
            let p = match (op >> 20) & 3 {
                0 => self.ram[self.dp as usize & (self.ram.len() - 1)],
                1 => idb,
                2 => m,
                _ => n,
            };
            let b_side = op & 0x8000 != 0;
            let (q, other_c) = if b_side { (self.b, self.fa.c) } else { (self.a, self.fb.c) };
            let mut f = if b_side { self.fb } else { self.fa };
            let (r, carry, overflow) = match code {
                1 => (q | p, false, None),
                2 => (q & p, false, None),
                3 => (q ^ p, false, None),
                4 | 6 | 8 => {
                    let pv = if code == 8 { 1 } else { p };
                    let rhs = pv as u32 + (code == 6 && other_c) as u32;
                    let r = (q as u32).wrapping_sub(rhs) as u16;
                    (r, (q as u32) < rhs, Some((q ^ pv) & (q ^ r) & 0x8000 != 0))
                }
                5 | 7 | 9 => {
                    let pv = if code == 9 { 1 } else { p };
                    let sum = q as u32 + pv as u32 + (code == 7 && other_c) as u32;
                    let r = sum as u16;
                    (r, sum > 0xFFFF, Some(!(q ^ pv) & (q ^ r) & 0x8000 != 0))
                }
                10 => (!q, false, None),
                11 => (((q as i16) >> 1) as u16, q & 1 != 0, None),
                12 => ((q << 1) | other_c as u16, q & 0x8000 != 0, None),
                13 => ((q << 2) | 3, false, None),
                14 => ((q << 4) | 15, false, None),
                _ => (q.rotate_left(8), false, None),
            };
            f.s0 = r & 0x8000 != 0;
            f.z = r == 0;
            f.c = carry;
            match overflow {
                // fullsnes: an overflow sets S1 to the sign and toggles OV1; without one both are kept.
                Some(ov) => {
                    f.ov0 = ov;
                    if ov {
                        f.s1 = f.s0;
                        f.ov1 = !f.ov1;
                    }
                }
                None => {
                    f.ov0 = false;
                    f.ov1 = false;
                    f.s1 = f.s0;
                }
            }
            if b_side {
                self.b = r;
                self.fb = f;
            } else {
                self.a = r;
                self.fa = f;
            }
        }
        self.write_dst(op & 0xF, idb);
        let low = match (op >> 13) & 3 {
            1 => (self.dp + 1) & 0xF,
            2 => self.dp.wrapping_sub(1) & 0xF,
            3 => 0,
            _ => self.dp & 0xF,
        };
        self.dp = ((self.dp & !0xF) | low) ^ ((((op >> 9) & 0xF) as u16) << 4);
        self.dp &= self.dp_mask();
        if op & 0x100 != 0 {
            self.rp = self.rp.wrapping_sub(1) & self.rp_mask();
        }
        if op & 0x40_0000 != 0 {
            self.pc = self.pop();
        }
    }

    fn push(&mut self, v: u16) {
        let depth = self.depth();
        self.stack[self.sp as usize] = v;
        self.sp = (self.sp + 1) % depth;
    }

    fn pop(&mut self) -> u16 {
        let depth = self.depth();
        self.sp = (self.sp + depth - 1) % depth;
        self.stack[self.sp as usize]
    }

    fn jump(&mut self, op: u32) {
        let brch = (op >> 13) & 0x1FF;
        let mut na = ((op >> 2) & 0x7FF) as u16;
        if self.st {
            na |= ((op & 3) as u16) << 11;
        }
        let (fa, fb) = (self.fa, self.fb);
        let take = match brch {
            0x000 if self.st => {
                self.pc = self.so & self.pc_mask();
                return;
            }
            0x100 | 0x101 => {
                self.pc = (na | if brch == 0x101 && self.st { 0x2000 } else { 0 }) & self.pc_mask();
                return;
            }
            0x140 | 0x141 => {
                let back = self.pc;
                self.push(back);
                self.pc = (na | if brch == 0x141 && self.st { 0x2000 } else { 0 }) & self.pc_mask();
                return;
            }
            0x080 => !fa.c,
            0x082 => fa.c,
            0x084 => !fb.c,
            0x086 => fb.c,
            0x088 => !fa.z,
            0x08A => fa.z,
            0x08C => !fb.z,
            0x08E => fb.z,
            0x090 => !fa.ov0,
            0x092 => fa.ov0,
            0x094 => !fb.ov0,
            0x096 => fb.ov0,
            0x098 => !fa.ov1,
            0x09A => fa.ov1,
            0x09C => !fb.ov1,
            0x09E => fb.ov1,
            0x0A0 => !fa.s0,
            0x0A2 => fa.s0,
            0x0A4 => !fb.s0,
            0x0A6 => fb.s0,
            0x0A8 => !fa.s1,
            0x0AA => fa.s1,
            0x0AC => !fb.s1,
            0x0AE => fb.s1,
            0x0B0 => self.dp & 0xF == 0,
            0x0B1 => self.dp & 0xF != 0,
            0x0B2 => self.dp & 0xF == 0xF,
            0x0B3 => self.dp & 0xF != 0xF,
            // The serial port is not wired on the SNES; its acknowledges read as set.
            0x0B4 => false,
            0x0B6 => true,
            0x0B8 => false,
            0x0BA => true,
            0x0BC => self.sr & sr::RQM == 0,
            0x0BE => self.sr & sr::RQM != 0,
            _ => false,
        };
        if take {
            // Conditional jumps keep PC bit 13 on the µPD96050 (fullsnes).
            self.pc = ((self.pc & 0x2000) | (na & 0x1FFF)) & self.pc_mask();
        }
    }

    /// The registers and RAM for the machine's state, in a fixed size per chip.
    pub fn pack(&self) -> Vec<u8> {
        let mut o = Vec::new();
        for v in [self.pc, self.rp, self.dp, self.k, self.l, self.a, self.b, self.tr, self.trb, self.sr, self.dr, self.so, self.si] {
            o.extend(v.to_le_bytes());
        }
        for v in self.stack {
            o.extend(v.to_le_bytes());
        }
        o.extend([self.sp, self.fa.bits(), self.fb.bits(), 0]);
        o.extend(self.cycles.to_le_bytes());
        for v in self.ram.iter() {
            o.extend(v.to_le_bytes());
        }
        o
    }

    pub fn unpack(&mut self, d: &[u8]) {
        let w = |i: usize| u16::from_le_bytes([d[i * 2], d[i * 2 + 1]]);
        let regs: Vec<u16> = (0..13).map(w).collect();
        (self.pc, self.rp, self.dp, self.k, self.l, self.a, self.b) = (regs[0] & self.pc_mask(), regs[1] & self.rp_mask(), regs[2] & self.dp_mask(), regs[3], regs[4], regs[5], regs[6]);
        (self.tr, self.trb, self.sr, self.dr, self.so, self.si) = (regs[7], regs[8], regs[9], regs[10], regs[11], regs[12]);
        for i in 0..8 {
            self.stack[i] = w(13 + i);
        }
        let at = 42;
        self.sp = d[at] % self.depth();
        self.fa = Flags::from_bits(d[at + 1]);
        self.fb = Flags::from_bits(d[at + 2]);
        self.cycles = u64::from_le_bytes(d[at + 4..at + 12].try_into().expect("eight bytes"));
        let base = at + 12;
        for (i, v) in self.ram.iter_mut().enumerate() {
            *v = u16::from_le_bytes([d[base + i * 2], d[base + i * 2 + 1]]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A firmware image of the given opcodes, padded to the µPD77C25's size.
    fn image(ops: &[u32], data: &[u16]) -> Vec<u8> {
        let mut v = vec![0u8; 8192];
        for (i, op) in ops.iter().enumerate() {
            v[i * 3..i * 3 + 3].copy_from_slice(&op.to_le_bytes()[..3]);
        }
        for (i, w) in data.iter().enumerate() {
            v[6144 + i * 2..6144 + i * 2 + 2].copy_from_slice(&w.to_le_bytes());
        }
        v
    }

    fn ld(id: u16, dst: u32) -> u32 {
        0xC0_0000 | (id as u32) << 6 | dst
    }

    fn alu(p: u32, code: u32, b: bool, src: u32, dst: u32) -> u32 {
        p << 20 | code << 16 | (b as u32) << 15 | src << 4 | dst
    }

    // fullsnes's handshake: the program's write to DR raises RQM; the S-CPU's two 8-bit reads in 16-bit mode, LSB
    // first, lower it; JRQM waits on it.
    #[test]
    fn the_dr_handshake_moves_a_word_lsb_first() {
        let jrqm_self = 0x80_0000 | 0x0BE << 13 | 2 << 2;
        let mut d = NecDsp::from_firmware(&image(&[ld(0x1234, 6), ld(0x0000, 0), jrqm_self, ld(0x5678, 1)], &[])).unwrap();
        for _ in 0..8 {
            d.step();
        }
        assert_eq!(d.sr & sr::RQM, sr::RQM);
        assert_eq!(d.pc, 2);
        assert_eq!(d.host_read(Port::Sr, true), 0x80);
        assert_eq!((d.host_read(Port::Dr, true), d.host_read(Port::Dr, true)), (0x34, 0x12));
        assert_eq!(d.sr & sr::RQM, 0);
        d.step();
        d.step();
        assert_eq!(d.a, 0x5678);
    }

    // fullsnes's ALU table: SUB sets carry on a borrow and OV0 on a signed overflow, which also sets S1 to the sign
    // and toggles OV1; the multiplier gives K*L*2 in M (high) and N (low).
    #[test]
    fn the_alu_sets_its_flags_and_the_multiplier_doubles() {
        let mut d = NecDsp::from_firmware(&image(
            &[ld(0x8000, 1), ld(0x0001, 3), alu(1, 4, false, 3, 0), ld(0x4000, 10), ld(0x0003, 13), alu(3, 1, true, 0, 0), alu(2, 1, false, 0, 0)],
            &[],
        ))
        .unwrap();
        for _ in 0..3 {
            d.step();
        }
        assert_eq!(d.a, 0x7FFF);
        assert!(d.fa.ov0 && d.fa.ov1 && !d.fa.c && !d.fa.s0 && !d.fa.s1);
        for _ in 0..3 {
            d.step();
        }
        assert_eq!(d.b, 0x8000);
        d.a = 0;
        d.step();
        assert_eq!(d.a, 0x0001);
    }
}
