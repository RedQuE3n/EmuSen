//! The debugger's view of the Genesis: its two processors with their registers, the two buses as spaces read without
//! side effects, and each processor's code disassembled from its bus (Nephrite_Native.md §41).

use emusen_native::core::{Instruction, Processor};

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
