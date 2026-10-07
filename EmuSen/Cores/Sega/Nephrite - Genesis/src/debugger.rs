//! The debugger's view of the Genesis: its two processors with their registers, the two buses as spaces read without
//! side effects, and each processor's code disassembled from its bus (Nephrite_Native.md §41).

use emusen_native::core::{Instruction, Processor};
use emusen_native::debug::{Hooks, Observer, stop};

use crate::genesis::Hw;
use crate::machine::Machine;

/// The 68000's bus and the Z80's, as the plan numbers them so that ids never move (Nephrite_Plan.md §4.3).
pub const M68KBUS_ID: u32 = 0;
pub const Z80BUS_ID: u32 = 1;
pub const M68KBUS_SIZE: usize = 1 << 24;
pub const Z80BUS_SIZE: usize = 1 << 16;

/// The processors' numbers: the 68000 is processor 0, whose breakpoints the shared hooks keep.
pub const M68K: u32 = 0;
pub const Z80: u32 = 1;

pub const M68K_REGISTERS: [(&str, u32); 20] = [
    ("D0", 32), ("D1", 32), ("D2", 32), ("D3", 32), ("D4", 32), ("D5", 32), ("D6", 32), ("D7", 32),
    ("A0", 32), ("A1", 32), ("A2", 32), ("A3", 32), ("A4", 32), ("A5", 32), ("A6", 32), ("A7", 32),
    ("PC", 32), ("SR", 16), ("USP", 32), ("SSP", 32),
];

pub const Z80_REGISTERS: [(&str, u32); 18] = [
    ("AF", 16), ("BC", 16), ("DE", 16), ("HL", 16), ("AF'", 16), ("BC'", 16), ("DE'", 16), ("HL'", 16),
    ("IX", 16), ("IY", 16), ("SP", 16), ("PC", 16), ("I", 8), ("R", 8), ("WZ", 16), ("IM", 2), ("IFF1", 1), ("IFF2", 1),
];

/// The 68000's observer in the observed frame: its stop is asked before its step, so here it records the step, its
/// stores and its calls into the hooks.
pub struct MainWatch<'a>(pub &'a mut Hooks);

impl Observer for MainWatch<'_> {
    fn before(&mut self, _processor: usize, pc: u32) -> u32 {
        self.0.record(pc & 0xFF_FFFF);
        stop::FRAME
    }
    fn wrote(&mut self, space: u32, address: u32, value: u8, pc: u32) {
        if self.0.writes {
            self.0.note_write(space, address & 0xFF_FFFF, value, pc & 0xFF_FFFF);
        }
    }
    fn called(&mut self, source: u32, target: u32, kind: u32) {
        self.0.note_call(source & 0xFF_FFFF, target & 0xFF_FFFF, kind);
    }
    fn returned(&mut self) {
        self.0.note_return();
    }
}

/// The Z80's observer: its own breakpoints asked before each instruction, its coverage and its stores into the hooks.
/// Its calls are not the call stack's, which is the 68000's.
pub struct Z80Watch<'a> {
    pub hooks: &'a mut Hooks,
    pub breakpoints: &'a [(i32, i32)],
    /// The first instruction runs unasked, as the host's resume past a Z80 stop asks.
    pub skip: &'a mut bool,
}

impl Observer for Z80Watch<'_> {
    fn before(&mut self, processor: usize, pc: u32) -> u32 {
        if !std::mem::take(self.skip) && self.breakpoints.iter().any(|&(a, b)| (pc as i32) >= a && (pc as i32) <= b) {
            return stop::BREAKPOINT;
        }
        self.hooks.record_on(processor, pc);
        stop::FRAME
    }
    fn wrote(&mut self, space: u32, address: u32, value: u8, pc: u32) {
        if self.hooks.writes {
            self.hooks.note_write(space, address, value, pc);
        }
    }
}

impl Hw {
    /// A byte of the 68000's space as a debugger reads it: the cartridge, the Z80's RAM and the 68000's RAM as the bus
    /// gives them; the ports, whose reads change what they read, and unmapped addresses as $FF.
    pub fn peek_m68k(&self, a: u32) -> u8 {
        let a = a & 0xFF_FFFF;
        match a {
            0x00_0000..=0x3F_FFFF if self.cart.answers(a) => self.cart.read8(a),
            0xA0_0000..=0xA0_3FFF => self.zram[a as usize & 0x1FFF],
            0xE0_0000..=0xFF_FFFF => self.wram[a as usize & 0xFFFF],
            _ => 0xFF,
        }
    }

    /// A byte of the Z80's space as a debugger reads it: its RAM, and through the bank window the 68000's space; the
    /// YM2612, the bank register, the PSG and the VDP as $FF.
    pub fn peek_z80(&self, a: u16) -> u8 {
        match a {
            0x0000..=0x3FFF => self.zram[a as usize & 0x1FFF],
            0x8000..=0xFFFF => {
                let at = (self.z80_bank as u32) << 15 | (a as u32 & 0x7FFF);
                if (0xA0_0000..=0xA0_FFFF).contains(&at) { 0xFF } else { self.peek_m68k(at) }
            }
            _ => 0xFF,
        }
    }
}

impl Machine {
    pub fn processors(&self) -> Vec<Processor> {
        let named = |r: &[(&str, u32)]| r.iter().map(|&(n, b)| (n.to_owned(), b)).collect::<Vec<_>>();
        vec![
            Processor { id: M68K, name: "M68K".into(), pc_bits: 24, registers: named(&M68K_REGISTERS), code_space: Some(M68KBUS_ID) },
            Processor { id: Z80, name: "Z80".into(), pc_bits: 16, registers: named(&Z80_REGISTERS), code_space: Some(Z80BUS_ID) },
        ]
    }

    /// The bytes `address` on of a bus space; another space is the caller's.
    pub fn read_bus(&self, space: u32, address: u32, out: &mut [u8]) {
        let hw = &self.genesis.hw;
        for (i, b) in out.iter_mut().enumerate() {
            let at = address.wrapping_add(i as u32);
            *b = match space {
                M68KBUS_ID => hw.peek_m68k(at),
                _ => hw.peek_z80(at as u16),
            };
        }
    }

    /// The address of the instruction `processor` stands in front of.
    pub fn debug_pc(&self, processor: u32) -> Option<u64> {
        match processor {
            M68K => Some((self.genesis.cpu.regs.pc & 0xFF_FFFF) as u64),
            Z80 => Some(self.genesis.z80.regs.pc as u64),
            _ => None,
        }
    }

    /// A processor's registers in the order its list gives them.
    pub fn debug_registers(&self, processor: u32) -> Option<Vec<i64>> {
        match processor {
            M68K => {
                let r = &self.genesis.cpu.regs;
                let mut v: Vec<i64> = r.d.iter().chain(r.a.iter()).map(|&x| x as i64).collect();
                v.extend([r.pc as i64, r.sr as i64, r.usp() as i64, r.ssp() as i64]);
                Some(v)
            }
            Z80 => {
                let r = &self.genesis.z80.regs;
                Some(vec![
                    r.af, r.bc, r.de, r.hl, r.af_, r.bc_, r.de_, r.hl_, r.ix, r.iy, r.sp, r.pc,
                    r.i as u16, r.r as u16, r.wz, r.im as u16, r.iff1 as u16, r.iff2 as u16,
                ].into_iter().map(i64::from).collect())
            }
            _ => None,
        }
    }

    /// `count` instructions from `address`, each in its processor's set: the 68000's on its bus, the Z80's on its own
    /// bus and in its RAM; another space is read as the processor asked would see it.
    pub fn debug_disassemble(&self, processor: u32, space: u32, address: u32, count: u32) -> Option<Vec<Instruction>> {
        let z80 = match space {
            Z80BUS_ID => true,
            M68KBUS_ID => false,
            crate::machine::Z80RAM_ID => true,
            _ => processor == Z80,
        };
        let byte = |a: u32| -> u8 {
            match space {
                M68KBUS_ID | Z80BUS_ID => {
                    let mut b = [0];
                    self.read_bus(space, a, &mut b);
                    b[0]
                }
                _ => self.bytes(space).and_then(|m| m.get(a as usize).copied()).unwrap_or(0xFF),
            }
        };
        if processor > Z80 || (space > 1 && self.bytes(space).is_none()) {
            return None;
        }
        let mut out = Vec::with_capacity(count as usize);
        let mut at = address;
        for _ in 0..count {
            let i = if z80 {
                beryl_z80::disasm::disassemble(&|a: u16| byte(a as u32), at as u16)
            } else {
                beryl_m68k::disasm::disassemble(&|a: u32| (byte(a) as u16) << 8 | byte(a.wrapping_add(1)) as u16, at & 0xFF_FFFE)
            };
            at = i.address.wrapping_add(i.bytes.len().max(1) as u32);
            out.push(i);
        }
        Some(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::Media;

    fn w(words: &[u16]) -> Vec<u8> {
        words.iter().flat_map(|x| x.to_be_bytes()).collect()
    }

    /// A cartridge whose code sets D0, D1 and A0, calls a routine at $300 and loops.
    fn machine() -> Machine {
        let mut r = vec![0xFFu8; 0x1_0000];
        r[0..8].copy_from_slice(&w(&[0x00FF, 0xFE00, 0x0000, 0x0200]));
        r[0x100..0x110].copy_from_slice(b"SEGA MEGA DRIVE ");
        let code = w(&[0x7005, 0x223C, 0x1234, 0x5678, 0x41F9, 0x00FF, 0x0000, 0x4EB9, 0x0000, 0x0300, 0x60FE]);
        r[0x200..0x200 + code.len()].copy_from_slice(&code);
        r[0x300..0x302].copy_from_slice(&w(&[0x4E75]));
        let mut m = Machine::new(&r, Media::read(&r));
        m.advance();
        m
    }

    /// A 68000 that calls a routine at $300 in a loop, the routine counting in D2; a Z80 out of reset that counts in A
    /// and jumps back, from its RAM.
    fn looping() -> Machine {
        let mut r = vec![0xFFu8; 0x1_0000];
        r[0..8].copy_from_slice(&w(&[0x00FF, 0xFE00, 0x0000, 0x0200]));
        r[0x100..0x110].copy_from_slice(b"SEGA MEGA DRIVE ");
        r[0x200..0x20C].copy_from_slice(&w(&[0x7005, 0x4EB9, 0x0000, 0x0300, 0x60F8, 0x4E71]));
        r[0x300..0x304].copy_from_slice(&w(&[0x5282, 0x4E75]));
        let mut m = Machine::new(&r, Media::read(&r));
        m.genesis.hw.zram[..4].copy_from_slice(&[0x3C, 0xC3, 0x00, 0x00]);
        (m.genesis.hw.z80_reset, m.genesis.hw.z80_busreq) = (false, false);
        m
    }

    fn state(m: &Machine) -> Vec<u8> {
        use emusen_native::ffi::StateMachine;
        let mut v = vec![0; m.state_size()];
        let n = m.save_state(&mut v).unwrap();
        v.truncate(n);
        v
    }

    fn run_out(m: &mut Machine) -> u32 {
        let mut stops = 0;
        while m.run_frame_debug(emusen_native::debug::run::UNCHECKED | emusen_native::debug::run::CONTINUE) != 0 {
            stops += 1;
        }
        stops
    }

    // Every table armed with nothing to hit gives the plain frame, picture and state (the kit's C15, on both processors).
    #[test]
    fn an_armed_frame_with_nothing_to_hit_is_the_plain_frame() {
        use emusen_native::debug::flag;
        let (mut a, mut b) = (looping(), looping());
        b.hooks.configure(flag::CALLS | flag::WRITES | flag::PROFILING | 1 << flag::COVERAGE | 1 << (flag::COVERAGE + 1), i32::MIN, -1);
        b.hooks.set_breakpoints(&[-2, -2]);
        b.z80_breakpoints = vec![(-2, -2)];
        for _ in 0..3 {
            a.advance();
            assert_eq!(b.run_frame_debug(0), 0);
            assert_eq!((state(&b), &b.picture), (state(&a), &a.picture));
        }
        let mut covered = 0;
        assert!(emusen_native::debug::drain_coverage(&mut b.hooks, 1, &mut vec![0; 1 << 13], &mut covered) > 0 && covered > 0, "the Z80's steps are recorded");
        assert!(b.hooks.calls_log.iter().any(|c| c.target == 0x300), "the 68000's calls are logged");
    }

    // A 68000 breakpoint stops in front of its instruction on processor 0, and an unchecked resume runs past it.
    #[test]
    fn a_68000_breakpoint_stops_in_front_of_its_instruction() {
        let mut m = looping();
        m.hooks.set_breakpoints(&[0x300, 0x300]);
        assert_eq!(m.run_frame_debug(0), stop::BREAKPOINT);
        assert_eq!((m.debug_stopped, m.debug_pc(M68K)), (M68K, Some(0x300)));
        let d2 = m.genesis.cpu.regs.d[2];
        assert_eq!(m.run_frame_debug(emusen_native::debug::run::UNCHECKED | emusen_native::debug::run::CONTINUE), stop::BREAKPOINT);
        assert_eq!((m.debug_pc(M68K), m.genesis.cpu.regs.d[2]), (Some(0x300), d2 + 1), "one pass of the routine between the stops");
        assert_eq!(m.frames, 0, "the frame stays open");
    }

    // A Z80 breakpoint stops in front of its instruction on processor 1, the 68000 left where it was, and resumes the same way.
    #[test]
    fn a_z80_breakpoint_stops_in_front_of_its_instruction() {
        let mut m = looping();
        m.z80_breakpoints = vec![(1, 1)];
        assert_eq!(m.run_frame_debug(0), stop::BREAKPOINT);
        assert_eq!((m.debug_stopped, m.debug_pc(Z80)), (Z80, Some(1)));
        let (a, main_pc) = (m.genesis.z80.regs.af >> 8, m.debug_pc(M68K));
        assert_eq!(m.run_frame_debug(emusen_native::debug::run::UNCHECKED | emusen_native::debug::run::CONTINUE), stop::BREAKPOINT);
        assert_eq!((m.debug_stopped, m.debug_pc(Z80), m.genesis.z80.regs.af >> 8), (Z80, Some(1), (a + 1) & 0xFF));
        assert!(m.genesis.hw.z80_clock <= m.genesis.hw.clock, "the Z80 stopped behind the 68000");
        let _ = main_pc;
    }

    // Stops on either processor, each resumed, leave the frame as a plain one makes it: the Z80 finishes its turn first.
    #[test]
    fn a_frame_stopped_and_resumed_is_the_plain_frame() {
        let (mut a, mut b) = (looping(), looping());
        b.hooks.set_breakpoints(&[0x300, 0x300]);
        b.z80_breakpoints = vec![(0, 0)];
        for _ in 0..2 {
            a.advance();
            assert!(run_out(&mut b) > 100);
            assert_eq!((state(&b), &b.picture), (state(&a), &a.picture));
        }
    }

    // EACH stops in front of every 68000 instruction, and each unchecked step moves the machine by one.
    #[test]
    fn each_stops_in_front_of_every_68000_instruction() {
        use emusen_native::debug::{flag, run};
        let mut m = looping();
        m.hooks.configure(flag::EACH, i32::MIN, -1);
        assert_eq!(m.run_frame_debug(0), stop::EACH);
        let mut seen = Vec::new();
        for _ in 0..6 {
            seen.push(m.debug_pc(M68K).unwrap());
            assert_eq!(m.run_frame_debug(run::UNCHECKED | run::CONTINUE), stop::EACH);
        }
        let start = seen.iter().position(|&p| p == 0x300).unwrap();
        assert_eq!(&seen[start..start + 4], [0x300, 0x302, 0x208, 0x202], "ADDQ, RTS, BRA, JSR");
    }

    #[test]
    fn each_processors_registers_are_its_lists_in_order() {
        let m = machine();
        let ps = m.processors();
        assert_eq!(ps.iter().map(|p| (p.id, p.name.as_str(), p.code_space)).collect::<Vec<_>>(), [(0, "M68K", Some(0)), (1, "Z80", Some(1))]);
        let r = m.debug_registers(M68K).unwrap();
        assert_eq!(r.len(), ps[0].registers.len());
        let cpu = &m.genesis.cpu.regs;
        assert_eq!((r[0], r[1], r[8], r[15]), (5, 0x1234_5678, 0xFF_0000, cpu.a[7] as i64));
        assert_eq!((r[16], r[17], r[19]), (cpu.pc as i64, cpu.sr as i64, cpu.a[7] as i64), "PC, SR, and SSP in supervisor mode");
        assert_eq!(m.debug_pc(M68K), Some(0x214));
        let z = m.debug_registers(Z80).unwrap();
        assert_eq!(z.len(), ps[1].registers.len());
        assert_eq!((z[11], z[15]), (m.genesis.z80.regs.pc as i64, m.genesis.z80.regs.im as i64));
        assert_eq!((m.debug_registers(2), m.debug_pc(2)), (None, None));
    }

    #[test]
    fn the_buses_read_as_the_processors_see_them_and_their_ports_are_not_touched() {
        let mut m = machine();
        let hw = &mut m.genesis.hw;
        hw.wram[0x10] = 0xAB;
        hw.zram[5] = 0x77;
        assert_eq!((hw.peek_m68k(0xFF_0010), hw.peek_m68k(0xE0_0010), hw.peek_m68k(0x200)), (0xAB, 0xAB, 0x70));
        assert_eq!((hw.peek_m68k(0xA0_0005), hw.peek_m68k(0xA0_2005), hw.peek_m68k(0xA0_4000)), (0x77, 0x77, 0xFF));
        assert_eq!((hw.peek_z80(5), hw.peek_z80(0x2005), hw.peek_z80(0x4000), hw.peek_z80(0x8200)), (0x77, 0x77, 0xFF, 0x70));
        hw.z80_bank = 0x1FE;
        assert_eq!(hw.peek_z80(0x8010), 0xAB, "the bank window onto the 68000's RAM");
        let before = hw.vdp.regs_state();
        assert_eq!((hw.peek_m68k(0xC0_0004), hw.peek_m68k(0xA1_0003), hw.peek_m68k(0x50_0000)), (0xFF, 0xFF, 0xFF));
        assert_eq!(hw.vdp.regs_state(), before);
        let mut out = [0u8; 4];
        m.read_bus(M68KBUS_ID, 0xFF_FFFE, &mut out);
        assert_eq!(out[..2], [m.genesis.hw.wram[0xFFFE], m.genesis.hw.wram[0xFFFF]]);
        assert_eq!(out[2], m.genesis.hw.peek_m68k(0), "the bus wraps at 16 MiB");
    }

    #[test]
    fn each_processors_code_reads_in_its_own_instruction_set() {
        let mut m = machine();
        let list = m.debug_disassemble(M68K, M68KBUS_ID, 0x200, 6).unwrap();
        let text: Vec<(u32, String)> = list.iter().map(|i| (i.address, format!("{} {}", i.mnemonic, i.operands).trim().to_owned())).collect();
        assert_eq!(text, [
            (0x200, "MOVEQ #$5,D0".to_owned()),
            (0x202, "MOVE.L #$12345678,D1".to_owned()),
            (0x208, "LEA $00FF0000.L,A0".to_owned()),
            (0x20E, "JSR $00000300.L".to_owned()),
            (0x214, "BRA.S $000214".to_owned()),
            (0x216, "DC.W $FFFF".to_owned()),
        ]);
        m.genesis.hw.zram[..5].copy_from_slice(&[0x3E, 0x05, 0xC3, 0x34, 0x12]);
        for space in [Z80BUS_ID, crate::machine::Z80RAM_ID] {
            let z: Vec<String> = m.debug_disassemble(Z80, space, 0, 2).unwrap().iter().map(|i| format!("{} {}", i.mnemonic, i.operands)).collect();
            assert_eq!(z, ["LD A,$05", "JP $1234"], "space {space}");
        }
        assert_eq!(m.debug_disassemble(M68K, crate::machine::ROM_ID, 0x20E, 1).unwrap()[0].bytes, [0x4E, 0xB9, 0, 0, 3, 0]);
        assert!(m.debug_disassemble(M68K, 99, 0, 1).is_none());
    }
}
