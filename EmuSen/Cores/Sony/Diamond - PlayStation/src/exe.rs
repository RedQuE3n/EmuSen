//! PS-EXE loading (psx-spx "CDROM File Formats", the PS-X EXE header; "A(43h) - Exec"): the program's bytes to RAM,
//! its memfill region zeroed, and the registers Exec sets: PC, GP, SP and FP at the stack's base plus offset when the
//! base is not zero, and the two parameters R4 = 1 and R5 = 0 (Diamond_Native.md §11.9).

use blackmoon_r3000::{BranchDelay, LoadDelay, R3000};

use crate::media::EXE_HEADER;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub pc: u32,
    pub gp: u32,
    pub load: u32,
    pub size: u32,
    pub fill: u32,
    pub fill_size: u32,
    pub stack: u32,
    pub stack_offset: u32,
}

fn word(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

impl Header {
    /// The header of an image `media` has already accepted as a PS-EXE.
    pub fn read(image: &[u8]) -> Header {
        Header {
            pc: word(image, 0x10),
            gp: word(image, 0x14),
            load: word(image, 0x18),
            size: word(image, 0x1C),
            fill: word(image, 0x28),
            fill_size: word(image, 0x2C),
            stack: word(image, 0x30),
            stack_offset: word(image, 0x34),
        }
    }
}

/// Loads the program into `ram` (2 MiB, mirrored) and starts the processor at its entry with Exec's registers.
pub fn load(image: &[u8], ram: &mut [u8], cpu: &mut R3000) {
    let h = Header::read(image);
    let mask = ram.len() - 1;
    let body = &image[EXE_HEADER..EXE_HEADER + h.size as usize];
    for (i, &b) in body.iter().enumerate() {
        ram[(h.load as usize + i) & mask] = b;
    }
    for i in 0..(h.fill_size & !3) as usize {
        ram[(h.fill as usize + i) & mask] = 0;
    }
    let r = &mut cpu.regs;
    r.pc = h.pc;
    r.r[28] = h.gp;
    if h.stack != 0 {
        r.r[29] = h.stack.wrapping_add(h.stack_offset);
        r.r[30] = r.r[29];
    }
    r.r[4] = 1;
    r.r[5] = 0;
    r.branch = BranchDelay::default();
    r.load = LoadDelay::default();
}

/// A PS-EXE of `words` loaded and entered at 80010000h with the stack at 801FFFF0h and no market marker: the header
/// psx-spx gives, the size rounded up to 800h (Diamond_Native.md §11.9).
pub fn build(words: &[u32]) -> Vec<u8> {
    let size = (words.len() * 4).div_ceil(EXE_HEADER).max(1) * EXE_HEADER;
    let mut b = vec![0u8; EXE_HEADER + size];
    b[..8].copy_from_slice(b"PS-X EXE");
    for (at, v) in [(0x10, 0x8001_0000u32), (0x18, 0x8001_0000), (0x1C, size as u32), (0x30, 0x801F_FFF0)] {
        b[at..at + 4].copy_from_slice(&v.to_le_bytes());
    }
    for (i, w) in words.iter().enumerate() {
        b[EXE_HEADER + 4 * i..EXE_HEADER + 4 * i + 4].copy_from_slice(&w.to_le_bytes());
    }
    b
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::media::sample::exe;

    #[test]
    fn a_program_lands_with_execs_registers() {
        let mut image = exe(None, 8);
        image[0x800..0x804].copy_from_slice(&0x1234_5678u32.to_le_bytes());
        image[0x14..0x18].copy_from_slice(&0x8002_0000u32.to_le_bytes());
        image[0x28..0x2C].copy_from_slice(&0x8001_0800u32.to_le_bytes());
        image[0x2C..0x30].copy_from_slice(&8u32.to_le_bytes());
        let mut ram = vec![0xAAu8; 0x20_0000];
        let mut cpu = R3000::new();
        load(&image, &mut ram, &mut cpu);
        assert_eq!(&ram[0x10000..0x10004], &[0x78, 0x56, 0x34, 0x12]);
        assert_eq!(&ram[0x10800..0x10809], &[0, 0, 0, 0, 0, 0, 0, 0, 0xAA], "the memfill region, and no further");
        assert_eq!((cpu.regs.pc, cpu.regs.r[28], cpu.regs.r[29], cpu.regs.r[30], cpu.regs.r[4], cpu.regs.r[5]), (0x8001_0000, 0x8002_0000, 0x801F_FFF0, 0x801F_FFF0, 1, 0));
    }
}
