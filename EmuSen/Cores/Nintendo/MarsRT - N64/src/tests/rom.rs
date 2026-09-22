//! The cartridge image and its security chip: `MarsRomImageTests` and `MarsCicTests`.

use super::support::{SyntheticRom, build_homebrew_with, build_rom, byte_swapped, from_hex, little_endian, new_bus, rom_image, to_hex};
use crate::memory::bus::MemoryBus;
use crate::memory::bus_access::map::{SI_BASE, SP_DMEM_BASE};
use crate::cpu::Cpu;
use crate::machine::{boot, hand_off};
use crate::rom::{self, BOOT_CODE_LENGTH, Cic, MINIMUM_LENGTH, RomError, RomImage};
use crate::memory::save::save_type;
use crate::memory::si::TRANSFER_CYCLES;

// ---- MarsRomImageTests ----

#[test]
fn a_big_endian_image_parses_its_header() {
    let image = rom_image(&SyntheticRom { entry_point: 0x8000_1234, title: "WISEMAN", ..SyntheticRom::default() }.build());
    assert_eq!(image.entry_point, 0x8000_1234);
    assert_eq!(image.destination, b'E');
}

#[test]
fn every_container_normalises_to_the_same_bytes() {
    let big_endian = SyntheticRom { entry_point: 0x8000_1234, title: "WISEMAN", ..SyntheticRom::default() }.build();
    for file in [byte_swapped(&big_endian), little_endian(&big_endian)] {
        let image = RomImage::from_image(&file).unwrap();
        assert!(image.rom == big_endian);
        assert_eq!(image.entry_point, 0x8000_1234);
    }
}

#[test]
fn an_image_with_no_recognised_magic_is_refused() {
    let mut bytes = vec![0u8; MINIMUM_LENGTH];
    bytes[0] = 0x12;
    assert_eq!(RomImage::from_image(&bytes).err(), Some(RomError::NotAnImage(0x1200_0000)));
}

#[test]
fn an_image_shorter_than_a_magic_word_is_refused() {
    assert!(matches!(RomImage::from_image(&[0x80, 0x37]), Err(RomError::TooShort(_))));
}

#[test]
fn an_image_shorter_than_the_boot_code_is_refused() {
    let truncated = &build_rom()[..0x800];
    assert_eq!(RomImage::from_image(truncated).err(), Some(RomError::TooShort(0x800)));
}

#[test]
fn a_byte_swapped_image_of_odd_length_is_refused_rather_than_half_converted() {
    let swapped = byte_swapped(&build_rom());
    let odd = &swapped[..swapped.len() - 1];
    assert_eq!(RomImage::from_image(odd).err(), Some(RomError::Truncated(odd.len())));
}

#[test]
fn a_homebrew_header_states_its_own_save_type() {
    let kinds = [
        save_type::NONE,
        save_type::EEPROM_4K,
        save_type::EEPROM_16K,
        save_type::SRAM_256K,
        save_type::SRAM_BANKED_768K,
        save_type::FLASH_RAM,
        save_type::SRAM_1M,
    ];
    for kind in kinds {
        let image = rom_image(&build_homebrew_with(kind, false, false));
        assert!(image.has_ed64_header, "kind {kind}");
        assert_eq!(image.save_type, kind);
    }
}

#[test]
fn a_homebrew_header_carries_the_clock_and_region_flags_beside_the_save_type() {
    let image = rom_image(&build_homebrew_with(save_type::FLASH_RAM, true, true));
    assert_eq!(image.save_type, save_type::FLASH_RAM);
}

#[test]
fn a_commercial_header_states_no_save_type_at_all() {
    let image = rom_image(&SyntheticRom { unique_code: "SM", ..SyntheticRom::default() }.build());
    assert!(!image.has_ed64_header);
    assert_eq!(image.save_type, save_type::UNKNOWN);
}

#[test]
fn the_destination_code_carries_the_video_standard_by_convention() {
    for (destination, pal) in [(b'E', false), (b'J', false), (b'P', true), (b'D', true), (b'U', true)] {
        let image = rom_image(&SyntheticRom { destination, ..SyntheticRom::default() }.build());
        assert_eq!(image.is_pal, pal, "{}", destination as char);
    }
}

// ---- MarsCicTests ----

const DRAM: u32 = 0x0010_0000;
const CHALLENGE_REQUEST: u8 = 0x02;
const CHALLENGE_AT: u32 = 0x30;
const TOTAL_6105: u64 = 0x0000_011A_49F6_0E96;
const TOTAL_6102: u64 = 0x0000_00D0_57C8_5244;

/// A boot code whose words sum to the total a reference lists, built from nothing but that total.
fn image_summing_to(total: u64, destination: u8) -> Vec<u8> {
    let mut ipl3 = vec![0u8; BOOT_CODE_LENGTH];
    let full = (total / 0xFFFF_FFFF) as usize;
    let rest = (total % 0xFFFF_FFFF) as u32;
    ipl3[..full * 4].fill(0xFF);
    ipl3[full * 4..full * 4 + 4].copy_from_slice(&rest.to_be_bytes());
    SyntheticRom { destination, patches: vec![(0, ipl3)], ..SyntheticRom::default() }.build()
}

#[test]
fn each_listed_boot_code_names_its_chip() {
    let rows = [
        (0x0000_00D0_027F_DF31u64, Cic::Nus6101),
        (0x0000_00CF_FB63_1223, Cic::Nus6101),
        (0x0000_00D0_57C8_5244, Cic::Nus6102),
        (0x0000_007C_5624_2373, Cic::Nus6102),
        (0x0000_00D6_497E_414B, Cic::Nus6103),
        (0x0000_011A_49F6_0E96, Cic::Nus6105),
        (0x0000_00D6_D5BE_5580, Cic::Nus6106),
    ];
    for (total, chip) in rows {
        assert_eq!(rom_image(&image_summing_to(total, b'E')).cic, chip, "{total:016X}");
    }
}

#[test]
fn a_boot_code_nobody_lists_is_unknown_and_gets_the_6102_seed() {
    let image = rom_image(&build_rom());
    assert_eq!(image.cic, Cic::Unknown);
    assert_eq!(rom::seed(image.cic), 0x3F);
}

#[test]
fn each_chip_hands_ipl3_its_own_seed() {
    for (chip, seed) in [(Cic::Nus6101, 0x3F), (Cic::Nus6102, 0x3F), (Cic::Nus6103, 0x78), (Cic::Nus6105, 0x91), (Cic::Nus6106, 0x85)] {
        assert_eq!(rom::seed(chip), seed, "{chip:?}");
    }
}

#[test]
fn the_handoff_leaves_a_6105_what_ipl2_would_have() {
    let mut bus = new_bus();
    let mut cpu = Cpu::power_on(&bus);
    hand_off(&mut bus, &mut cpu, rom_image(&image_summing_to(TOTAL_6105, b'P')));
    assert_eq!(cpu.gpr[22], 0x91);
    assert_eq!(cpu.gpr[11], 0xFFFF_FFFF_A400_0040);
    assert_eq!(cpu.gpr[31], 0xFFFF_FFFF_A400_1550);
    assert_eq!(cpu.gpr[20], 0);
    for (i, &word) in boot::IPL2_HEAD.iter().enumerate() {
        assert_eq!(bus.read32(SP_DMEM_BASE + 0x1000 + i as u32 * 4), word, "word {i}");
    }
}

#[test]
fn the_eighth_word_of_ipl2_is_the_first_to_end_the_decryption_loop() {
    assert_eq!(boot::IPL2_HEAD.iter().position(|word| word & 0xFFF == 0), Some(7));
}

#[test]
fn a_6102_cartridge_still_gets_the_6102_seed() {
    let mut bus = new_bus();
    let mut cpu = Cpu::power_on(&bus);
    hand_off(&mut bus, &mut cpu, rom_image(&image_summing_to(TOTAL_6102, b'E')));
    assert_eq!(cpu.gpr[22], 0x3F);
    assert_eq!(cpu.gpr[20], 1);
}

#[test]
fn the_6105_gives_the_recorded_answer_to_a_recorded_challenge() {
    let pairs = [
        ("000040001000040001000000000000", "BF9FD371C6EC62A8CBF9F9F9F9F9F9"),
        ("010040001000040001004000100000", "B4E62A8CBF9F937176ECA8C6371717"),
        ("810060001800060001004000100000", "3C6EA8C63F9F9D371C6E04E6371717"),
        ("AA006A001A00060001008000A00000", "D5553999EE6EC4E6E171F9F9171717"),
        ("2C004B001200040001000000C00000", "51115C6E117175555A8C6EC6A8C6EC"),
        ("33004C00130004000100C000300000", "A71751116D371BF9FE6E8C6EBF9F9F"),
    ];
    for (challenge, response) in pairs {
        let asked = from_hex(challenge);
        let mut answer = asked.clone();
        rom::respond(&asked, &mut answer);
        assert_eq!(to_hex(&answer), response, "{challenge}");
    }
}

/// A block asking the challenge, in RDRAM where a game would build it, carried in through the serial interface.
fn with_challenge(chip: Cic, challenge: &str, command: u8) -> MemoryBus {
    let mut bus = new_bus();
    let total = if chip == Cic::Nus6105 { TOTAL_6105 } else { TOTAL_6102 };
    *bus.cart = Some(rom_image(&image_summing_to(total, b'E')));
    let bytes = from_hex(challenge);
    let at = (DRAM + CHALLENGE_AT) as usize;
    bus.rdram[DRAM as usize + 0x2E] = 0xAA;
    bus.rdram[DRAM as usize + 0x2F] = 0xBB;
    bus.rdram[at..at + bytes.len()].copy_from_slice(&bytes);
    bus.rdram[DRAM as usize + 0x3F] = command;
    bus.write32(SI_BASE, DRAM);
    bus.write32(SI_BASE + 0x10, 0);
    bus
}

/// The answer reaches memory when the read's transfer is done.
fn read_back(bus: &mut MemoryBus) {
    bus.tick(TRANSFER_CYCLES);
    bus.write32(SI_BASE + 0x04, 0);
    bus.tick(TRANSFER_CYCLES);
}

fn landed(bus: &MemoryBus) -> String {
    let at = DRAM as usize + 0x2E;
    to_hex(&bus.rdram[at..at + 18])
}

#[test]
fn a_6105_cartridge_answers_as_the_block_goes_out() {
    let mut bus = with_challenge(Cic::Nus6105, "000040001000040001000000000000", CHALLENGE_REQUEST);
    assert_eq!(bus.pif_ram[0x3F], CHALLENGE_REQUEST);
    assert_eq!(bus.pif_ram[CHALLENGE_AT as usize], 0x00);
    read_back(&mut bus);
    assert_eq!(landed(&bus), "0000BF9FD371C6EC62A8CBF9F9F9F9F9F900");
}

#[test]
fn any_other_chip_drops_the_request_and_leaves_the_challenge() {
    let mut bus = with_challenge(Cic::Nus6102, "000040001000040001000000000000", CHALLENGE_REQUEST);
    read_back(&mut bus);
    assert_eq!(landed(&bus), "AABB00004000100004000100000000000000");
}

#[test]
fn answering_clears_the_request_bit_and_only_that_bit() {
    let mut bus = with_challenge(Cic::Nus6105, "000040001000040001000000000000", 0x0A);
    read_back(&mut bus);
    assert_eq!(bus.rdram[(DRAM + CHALLENGE_AT) as usize], 0xBF);
    assert_eq!(bus.rdram[DRAM as usize + 0x3F], 0x08);
}
