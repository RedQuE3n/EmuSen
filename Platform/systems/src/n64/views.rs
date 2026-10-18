//! The N64's debugger views, decoded from the spaces and registers any engine of the console publishes: each space's code
//! listed by the processor that runs it, the physical map a processor's address lands in, the VI's registers by their
//! libultra names, and the summary's lines. Moved from MarsRT's shim (`MarsRtDebugTarget`, `MarsDebugSpaces`), which they
//! match (EmuSen_Backend.md §18.7).

use super::disasm::{decode, decode_rsp};
use crate::debug::{Memory, Reference, find, register};
use emusen_dotnet::text::equals_ignore_case;

/// The space every N64 engine names its video interface's registers, fourteen big-endian words (MarsRT's machine info).
pub const VI: &str = "VI";

/// The VI's registers by their libultra names, in the order the space holds them.
pub const VI_REGISTERS: [&str; 14] =
    ["CONTROL", "ORIGIN", "WIDTH", "V_INTR", "V_CURRENT", "BURST", "V_SYNC", "H_SYNC", "LEAP", "H_START", "V_START", "V_BURST", "X_SCALE", "Y_SCALE"];

/// One listed instruction: its address, its four bytes, mnemonic, operands and the address it names.
pub struct Listed {
    pub address: u32,
    pub bytes: [u8; 4],
    pub mnemonic: String,
    pub operands: String,
    pub reference: Option<(Reference, u32)>,
}

/// One word of a space, decoded by the processor that runs code there, at the address that processor sees it at: the signal
/// processor for IMEM and DMEM, the VR4300 for RDRAM (through KSEG0), the ROM (through KSEG1's cartridge window), PIF RAM
/// and its own view.
pub fn decode_in(space: &str, word: u32, at: u32) -> (String, String, Option<(Reference, u32)>) {
    let is = |name: &str| equals_ignore_case(space, name);
    if is("IMEM") || is("DMEM") {
        decode_rsp(word, at & 0xFFC)
    } else if is("RDRAM") {
        decode(word, 0x8000_0000 | at)
    } else if is("ROM") {
        decode(word, 0xB000_0000u32.wrapping_add(at))
    } else if is("PIFRAM") {
        decode(word, 0xBFC0_07C0u32.wrapping_add(at))
    } else {
        decode(word, at)
    }
}

/// `count` words of `space` from `address` rounded down to a word, each read big-endian and decoded where it lies.
pub fn disassemble(m: &mut dyn Memory, space: &str, address: i32, count: i32) -> Vec<Listed> {
    let start = address & !3;
    let addresses: Vec<i32> = (0..count.max(0)).flat_map(|i| (0..4).map(move |b| start.wrapping_add(4 * i + b))).collect();
    let bytes = m.batch(space, &addresses);
    bytes
        .chunks_exact(4)
        .enumerate()
        .map(|(i, b)| {
            let at = start.wrapping_add(4 * i as i32) as u32;
            let (mnemonic, operands, reference) = decode_in(space, u32::from_be_bytes([b[0], b[1], b[2], b[3]]), at);
            Listed { address: at, bytes: [b[0], b[1], b[2], b[3]], mnemonic, operands, reference }
        })
        .collect()
}

/// Which memory a physical address names, given the RDRAM installed, or the register block it falls in, which is not
/// addressable: the name, the offset and whether a space of that name holds it (`Mars_Memory.md` §2).
pub fn physical(physical: u32, rdram_length: i64) -> (&'static str, i32, bool) {
    const RDRAM_REGISTERS: u32 = 0x03F0_0000;
    const SP_DMEM: u32 = 0x0400_0000;
    const SP_REGISTERS: u32 = 0x0404_0000;
    const SP_MEM_SIZE: u32 = 0x1000;
    const CART_DOMAIN2_ADDRESS1: u32 = 0x0500_0000;
    const CART_DOMAIN2_ADDRESS2: u32 = 0x0800_0000;
    const CART_DOMAIN1_ADDRESS2: u32 = 0x1000_0000;
    const PIF_ROM: u32 = 0x1FC0_0000;
    const PIF_RAM: u32 = 0x1FC0_07C0;
    const PIF_RAM_SIZE: u32 = 64;
    if (physical as i64) < rdram_length {
        return ("RDRAM", physical as i32, true);
    }
    if physical < RDRAM_REGISTERS {
        return ("RDRAM (not installed)", physical as i32, false);
    }
    if physical < SP_DMEM {
        return ("RDRAM registers", (physical - RDRAM_REGISTERS) as i32, false);
    }
    if physical < SP_REGISTERS {
        let local = (physical - SP_DMEM) % (2 * SP_MEM_SIZE);
        return if local < SP_MEM_SIZE { ("DMEM", local as i32, true) } else { ("IMEM", (local - SP_MEM_SIZE) as i32, true) };
    }
    if physical < CART_DOMAIN2_ADDRESS1 {
        return ("interface registers", physical as i32, false);
    }
    if physical < CART_DOMAIN2_ADDRESS2 {
        return ("64DD", physical as i32, false);
    }
    if physical < CART_DOMAIN1_ADDRESS2 {
        return ("save chip", (physical - CART_DOMAIN2_ADDRESS2) as i32, false);
    }
    if (PIF_RAM..PIF_RAM + PIF_RAM_SIZE).contains(&physical) {
        return ("PIFRAM", (physical - PIF_RAM) as i32, true);
    }
    if physical < PIF_ROM {
        return ("ROM", (physical - CART_DOMAIN1_ADDRESS2) as i32, true);
    }
    ("PIF ROM", (physical - PIF_ROM) as i32, false)
}

/// The physical map read with the size of the engine's RDRAM space.
pub fn physical_in(m: &dyn Memory, address: u32) -> (&'static str, i32, bool) {
    physical(address, find(m, "RDRAM").map_or(0, |s| s.size as i64))
}

/// The VI's registers from the engine's VI space, as many as it holds up to fourteen; none where it publishes no such space.
pub fn video_registers(m: &mut dyn Memory) -> Vec<(&'static str, u32)> {
    let Some((name, size)) = find(m, VI).map(|s| (s.name.clone(), s.size)) else { return Vec::new() };
    let n = (size.max(0) as usize / 4).min(VI_REGISTERS.len());
    let bytes = m.run(&name, 4 * n);
    bytes.chunks_exact(4).zip(VI_REGISTERS).map(|(b, name)| (name, u32::from_be_bytes([b[0], b[1], b[2], b[3]]))).collect()
}

/// The summary's lines the console gives: the VR4300's PC, SP and RA, and the VI's origin, width and sync where the VI space
/// is published; none without the processor's three registers.
pub fn summary(registers: &[(String, u64)], m: &mut dyn Memory) -> Option<String> {
    let (pc, sp, ra) = (register(registers, "PC")?, register(registers, "sp")?, register(registers, "ra")?);
    let mut text = format!("CPU  PC={pc:016X} SP={sp:016X} RA={ra:016X}");
    let vi = video_registers(m);
    if vi.len() >= 7 {
        text.push_str(&format!("\nVI   ORIGIN={:08X} WIDTH={} V_SYNC={}", vi[1].1, vi[2].1, vi[6].1));
    }
    Some(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::debug::{Space, Tape};

    fn spaces() -> Vec<Space> {
        vec![
            Space { name: "RDRAM".into(), size: 8 << 20, side_effects: false },
            Space { name: "IMEM".into(), size: 4096, side_effects: false },
            Space { name: "VI".into(), size: 56, side_effects: false },
        ]
    }

    #[test]
    fn each_space_is_listed_by_its_own_processor_at_its_own_address() {
        assert_eq!(decode_in("rdram", 0x1000_FFFF, 0x40).1, "0x80000040");
        assert_eq!(decode_in("IMEM", 0x1000_FFFF, 0x1040).1, "0x040");
        assert_eq!(decode_in("ROM", 0x0800_0000, 0x10).1, "0xB0000000");
        assert_eq!(decode_in("CPU", 0x1000_FFFF, 0xA400_0040).1, "0xA4000040");
    }

    #[test]
    fn the_physical_map_names_memories_and_register_blocks() {
        assert_eq!(physical(0x100, 8 << 20), ("RDRAM", 0x100, true));
        assert_eq!(physical(0x0050_0000, 4 << 20), ("RDRAM (not installed)", 0x0050_0000, false));
        assert_eq!(physical(0x0400_1010, 4 << 20), ("IMEM", 0x10, true));
        assert_eq!(physical(0x1FC0_07C4, 4 << 20), ("PIFRAM", 4, true));
        assert_eq!(physical(0x1000_0040, 4 << 20), ("ROM", 0x40, true));
        assert_eq!(physical(0x1FC0_0000, 4 << 20), ("PIF ROM", 0, false));
    }

    #[test]
    fn the_vi_s_registers_are_read_from_its_space_in_one_round() {
        let mut first = Tape::new(spaces(), &[]);
        assert!(video_registers(&mut first).iter().all(|&(_, v)| v == 0));
        assert_eq!(first.wants.len(), 1);
        let mut bytes = vec![0u8; 56];
        bytes[4..8].copy_from_slice(&0x0010_0000u32.to_be_bytes());
        bytes[8..12].copy_from_slice(&640u32.to_be_bytes());
        bytes[24..28].copy_from_slice(&525u32.to_be_bytes());
        let mut second = Tape::new(spaces(), &bytes);
        let registers = vec![("PC".to_string(), 0xFFFF_FFFF_A400_0040u64), ("sp".into(), 0xA400_1FF0), ("ra".into(), 0x10)];
        assert_eq!(
            summary(&registers, &mut second).unwrap(),
            "CPU  PC=FFFFFFFFA4000040 SP=00000000A4001FF0 RA=0000000000000010\nVI   ORIGIN=00100000 WIDTH=640 V_SYNC=525"
        );
        assert!(second.wants.is_empty());
    }
}
