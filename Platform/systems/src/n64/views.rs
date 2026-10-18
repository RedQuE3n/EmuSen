//! The N64's debugger views, decoded from the spaces, registers and units any engine of the console publishes: each
//! space's code listed by the processor that runs it, the physical map a processor's address lands in, and the summary's
//! lines. Moved from MarsRT's shim (`MarsRtDebugTarget`, `MarsDebugSpaces`), which they match (EmuSen_Backend.md §18.7,
//! §18.10).

use super::disasm::{decode, decode_rsp};
use crate::debug::{Memory, Reference, find, register};
use emusen_dotnet::text::equals_ignore_case;

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

/// The summary's lines the console gives: the VR4300's PC, SP and RA, and the VI's origin, width and sync from the
/// engine's `video` unit named VI, where it lists one; none without the processor's three registers.
pub fn summary(registers: &[(String, u64)], vi: Option<&[(String, u64)]>) -> Option<String> {
    let (pc, sp, ra) = (register(registers, "PC")?, register(registers, "sp")?, register(registers, "ra")?);
    let mut text = format!("CPU  PC={pc:016X} SP={sp:016X} RA={ra:016X}");
    if let Some(vi) = vi
        && let (Some(origin), Some(width), Some(sync)) = (register(vi, "ORIGIN"), register(vi, "WIDTH"), register(vi, "V_SYNC"))
    {
        text.push_str(&format!("\nVI   ORIGIN={:08X} WIDTH={} V_SYNC={}", origin as u32, width as u32, sync as u32));
    }
    Some(text)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn the_summary_reads_the_vi_from_its_unit() {
        let registers = vec![("PC".to_string(), 0xFFFF_FFFF_A400_0040u64), ("sp".into(), 0xA400_1FF0), ("ra".into(), 0x10)];
        let vi = vec![("ORIGIN".to_string(), 0x0010_0000u64), ("WIDTH".into(), 640), ("V_SYNC".into(), 525)];
        assert_eq!(
            summary(&registers, Some(&vi)).unwrap(),
            "CPU  PC=FFFFFFFFA4000040 SP=00000000A4001FF0 RA=0000000000000010\nVI   ORIGIN=00100000 WIDTH=640 V_SYNC=525"
        );
        assert_eq!(summary(&registers, None).unwrap(), "CPU  PC=FFFFFFFFA4000040 SP=00000000A4001FF0 RA=0000000000000010");
        assert_eq!(summary(&registers[..2], Some(&vi)), None);
    }
}
