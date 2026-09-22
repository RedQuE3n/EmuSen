//! The cartridge's save chip and the Controller Pak: `MarsSaveTests`.

use std::sync::Arc;

use super::support::{build_homebrew, build_rom, new_bus, rom_image};
use crate::bus::MemoryBus;
use crate::bus_access::map::{CART_DOMAIN2_ADDRESS2, PI_BASE};
use crate::controller::ControllerPak;
use crate::joybus;
use crate::save::{Eeprom, FlashRam, SRAM_BANK, SaveChip, SaveDevice, Sram, save_type};

const DRAM: u32 = 0x0010_0000;
const SAVE_BASE: u32 = CART_DOMAIN2_ADDRESS2;
const FLASH_COMMAND: u32 = SAVE_BASE + 0x1_0000;
const FLASH_SIZE: usize = 0x2_0000;

fn cartridge(kind: i32, saved: Option<Vec<u8>>) -> MemoryBus {
    let mut bus = new_bus();
    bus.save = SaveChip::with_saved(kind, saved.map(Arc::new));
    bus.write32(PI_BASE, DRAM);
    bus
}

fn chip(kind: i32) -> MemoryBus {
    cartridge(kind, None)
}

/// A block that skips the four ports with zero lengths and puts one command on the fifth channel.
fn run_cartridge(bus: &mut MemoryBus, send: u8, receive: u8, command: &[u8]) -> [u8; 64] {
    let mut ram = [0u8; 64];
    ram[4] = send;
    ram[5] = receive;
    ram[6..6 + command.len()].copy_from_slice(command);
    ram[6 + send as usize + receive as usize] = joybus::END;
    joybus::run(&mut ram, &mut bus.si.controllers, &mut bus.save);
    ram
}

fn transfer(bus: &mut MemoryBus, to_cartridge: bool, cart: u32, length: u32) {
    bus.write32(PI_BASE, DRAM);
    bus.write32(PI_BASE + 0x04, cart);
    bus.write32(PI_BASE + if to_cartridge { 0x08 } else { 0x0C }, length - 1);
}

fn eeprom(bus: &MemoryBus) -> &Eeprom {
    match &bus.save.device {
        SaveDevice::Eeprom(e) => e,
        other => panic!("no EEPROM: {other:?}"),
    }
}

fn sram(bus: &MemoryBus) -> &Sram {
    match &bus.save.device {
        SaveDevice::Sram(s) => s,
        other => panic!("no SRAM: {other:?}"),
    }
}

fn flash(bus: &MemoryBus) -> &FlashRam {
    match &bus.save.device {
        SaveDevice::Flash(f) => f,
        other => panic!("no FlashRAM: {other:?}"),
    }
}

fn dram(bus: &MemoryBus, offset: u32) -> u8 {
    bus.rdram[(DRAM + offset) as usize]
}

// ---- EEPROM, on the joybus's fifth channel ----

#[test]
fn an_eeprom_names_its_size_in_the_info_reply() {
    for (kind, size) in [(save_type::EEPROM_4K, 0x80u8), (save_type::EEPROM_16K, 0xC0)] {
        let mut bus = chip(kind);
        let ram = run_cartridge(&mut bus, 1, 3, &[0x00]);
        assert_eq!(ram[5], 3, "kind {kind}");
        assert_eq!(ram[7..10], [0x00, size, 0x00], "kind {kind}");
    }
}

#[test]
fn a_cartridge_with_sram_does_not_answer_on_the_eeprom_channel() {
    let ram = run_cartridge(&mut chip(save_type::SRAM_256K), 1, 3, &[0x00]);
    assert_eq!(ram[5], joybus::NO_REPLY | 3);
}

#[test]
fn asking_an_undecided_chip_what_it_is_describes_the_smaller_eeprom_and_decides_nothing() {
    let mut bus = chip(save_type::UNKNOWN);
    let ram = run_cartridge(&mut bus, 1, 3, &[0x00]);
    assert_eq!(ram[7..10], [0x00, 0x80, 0x00]);
    assert_eq!(bus.save.kind, save_type::UNKNOWN);
}

#[test]
fn reading_an_undecided_chip_on_the_eeprom_channel_makes_it_an_eeprom() {
    let mut bus = chip(save_type::UNKNOWN);
    let ram = run_cartridge(&mut bus, 2, 8, &[0x04, 0x05]);
    assert_eq!(bus.save.kind, save_type::EEPROM_4K);
    assert_eq!(ram[8..16], [0xFF; 8]);
}

#[test]
fn a_written_block_reads_back_and_the_write_answers_zero() {
    let mut bus = chip(save_type::EEPROM_4K);
    let write = run_cartridge(&mut bus, 10, 1, &[0x05, 0x03, 1, 2, 3, 4, 5, 6, 7, 8]);
    let read = run_cartridge(&mut bus, 2, 8, &[0x04, 0x03]);
    assert_eq!(write[16], 0x00);
    assert_eq!(read[8..16], [1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(eeprom(&bus).data[24..28], [1, 2, 3, 4]);
    assert!(bus.save.dirty());
}

#[test]
fn a_write_carrying_more_than_a_block_wraps_inside_it() {
    let mut bus = chip(save_type::EEPROM_4K);
    run_cartridge(&mut bus, 12, 1, &[0x05, 0x01, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    assert_eq!(eeprom(&bus).data[8..17], [9, 10, 3, 4, 5, 6, 7, 8, 0xFF]);
}

#[test]
fn a_short_write_changes_only_the_bytes_it_carries() {
    let mut bus = chip(save_type::EEPROM_4K);
    run_cartridge(&mut bus, 5, 1, &[0x05, 0x02, 1, 2, 3]);
    assert_eq!(eeprom(&bus).data[16..20], [1, 2, 3, 0xFF]);
}

#[test]
fn a_small_eeprom_still_reaches_two_kilobytes() {
    let mut bus = chip(save_type::EEPROM_4K);
    run_cartridge(&mut bus, 10, 1, &[0x05, 0xFF, 1, 2, 3, 4, 5, 6, 7, 8]);
    assert_eq!(eeprom(&bus).data.len(), 2048);
    assert_eq!(eeprom(&bus).data[0x7F8], 1);
}

#[test]
fn a_sixth_channel_reaches_the_cartridge_too() {
    let mut bus = chip(save_type::EEPROM_16K);
    let mut ram = [0u8; 64];
    ram[..12].copy_from_slice(&[0, 0, 0, 0, 0, 1, 3, 0x00, 0, 0, 0, 0xFE]);
    joybus::run(&mut ram, &mut bus.si.controllers, &mut bus.save);
    assert_eq!(ram[8..11], [0x00, 0xC0, 0x00]);
}

// ---- SRAM, on the second domain ----

#[test]
fn sram_round_trips_through_a_transfer_each_way() {
    let mut bus = chip(save_type::SRAM_256K);
    for i in 0..16u32 {
        bus.rdram[(DRAM + i) as usize] = 0xA0 + i as u8;
    }
    transfer(&mut bus, true, SAVE_BASE + 0x100, 16);
    bus.rdram[DRAM as usize..DRAM as usize + 16].fill(0);
    transfer(&mut bus, false, SAVE_BASE + 0x100, 16);
    assert_eq!(dram(&bus, 0), 0xA0);
    assert_eq!(dram(&bus, 15), 0xAF);
    assert_eq!(bus.read32(SAVE_BASE + 0x104), 0xA4A5_A6A7);
}

#[test]
fn erased_sram_reads_all_ones() {
    assert_eq!(chip(save_type::SRAM_256K).read32(SAVE_BASE + 0x40), 0xFFFF_FFFF);
}

#[test]
fn sram_repeats_every_32_kilobytes() {
    let mut bus = chip(save_type::SRAM_256K);
    bus.store(SAVE_BASE + 0x10, 0x1234_5678, 4);
    assert_eq!(bus.read32(SAVE_BASE + 0x8010), 0x1234_5678);
}

#[test]
fn banked_sram_picks_its_bank_with_address_bits_18_and_19() {
    let mut bus = chip(save_type::SRAM_BANKED_768K);
    bus.store(SAVE_BASE + 0x4_0010, 0x1111_1111, 4);
    bus.store(SAVE_BASE + 0x8_0010, 0x2222_2222, 4);
    assert_eq!(bus.read32(SAVE_BASE + 0x10), 0xFFFF_FFFF);
    assert_eq!(bus.read32(SAVE_BASE + 0x4_0010), 0x1111_1111);
    assert_eq!(bus.read32(SAVE_BASE + 0x8_0010), 0x2222_2222);
    assert_eq!(sram(&bus).data[0x1_0010] as u32 * 0x0101_0101, 0x2222_2222);
    assert_eq!(bus.read32(SAVE_BASE + 0xC_0010), 0);
}

#[test]
fn a_byte_store_to_sram_writes_its_whole_bus_word() {
    let mut bus = chip(save_type::SRAM_256K);
    bus.store(SAVE_BASE + 0x21, 0x9ABC_DEF1, 1);
    assert_eq!(bus.read32(SAVE_BASE + 0x20), 0xDEF1_0000);
}

#[test]
fn a_store_to_the_save_chip_holds_the_bus_but_a_read_of_it_takes_nothing_back() {
    let mut bus = chip(save_type::SRAM_256K);
    bus.store(SAVE_BASE + 0x30, 0x5555_5555, 4);
    bus.store(SAVE_BASE + 0x34, 0x6666_6666, 4);
    assert!(bus.pi_io_busy());
    assert_eq!(bus.load(SAVE_BASE + 0x34, 4), 0x6666_6666);
    assert!(bus.pi_io_busy());
}

#[test]
fn an_empty_second_domain_reads_the_low_half_of_its_address_twice() {
    assert_eq!(chip(save_type::NONE).read32(SAVE_BASE + 0x1234), 0x1234_1234);
}

// ---- FlashRAM ----

#[test]
fn flash_identifies_itself_to_the_processor_and_to_a_transfer() {
    let mut bus = chip(save_type::FLASH_RAM);
    bus.store(FLASH_COMMAND, 0xE100_0000, 4);
    transfer(&mut bus, false, SAVE_BASE, 8);
    assert_eq!(bus.read32(SAVE_BASE), 0x1111_8001);
    assert_eq!(bus.read32(SAVE_BASE + 4), 0x00C2_001D);
    assert_eq!(bus.rdram[DRAM as usize..DRAM as usize + 8], [0x11, 0x11, 0x80, 0x01, 0x00, 0xC2, 0x00, 0x1D]);
}

#[test]
fn a_page_is_programmed_from_its_buffer_only_when_told_to_execute() {
    let mut bus = chip(save_type::FLASH_RAM);
    for i in 0..128u32 {
        bus.rdram[(DRAM + i) as usize] = i as u8;
    }
    bus.store(FLASH_COMMAND, 0xB400_0000, 4);
    transfer(&mut bus, true, SAVE_BASE, 128);
    bus.store(FLASH_COMMAND, 0xA500_0003, 4);
    assert_eq!(flash(&bus).data[3 * 128 + 5], 0xFF);

    bus.store(FLASH_COMMAND, 0xD200_0000, 4);
    assert_eq!(flash(&bus).data[3 * 128 + 5], 5);
    assert_eq!(bus.read32(SAVE_BASE), 0x1111_8004);
}

#[test]
fn an_erase_clears_one_page_not_a_sector() {
    let mut bus = cartridge(save_type::FLASH_RAM, Some(vec![0; FLASH_SIZE]));
    bus.store(FLASH_COMMAND, 0x4B00_0003, 4);
    bus.store(FLASH_COMMAND, 0x7800_0000, 4);
    bus.store(FLASH_COMMAND, 0xD200_0000, 4);
    assert_eq!(flash(&bus).data[3 * 128], 0xFF);
    assert_eq!(flash(&bus).data[4 * 128 - 1], 0xFF);
    assert_eq!(flash(&bus).data[4 * 128], 0x00);
    assert_eq!(bus.read32(SAVE_BASE), 0x1111_8008);
}

#[test]
fn execute_with_nothing_pending_changes_nothing() {
    let mut bus = cartridge(save_type::FLASH_RAM, Some(vec![0; FLASH_SIZE]));
    bus.store(FLASH_COMMAND, 0x4B00_0000, 4);
    bus.store(FLASH_COMMAND, 0xD200_0000, 4);
    assert_eq!(flash(&bus).data[0], 0x00);
    assert!(!bus.save.dirty());
}

#[test]
fn a_transfer_reads_the_array_only_in_read_mode() {
    let mut bus = chip(save_type::FLASH_RAM);
    transfer(&mut bus, false, SAVE_BASE + 0x80, 4);
    assert_eq!(bus.read32(DRAM), 0);

    bus.store(FLASH_COMMAND, 0xF000_0000, 4);
    transfer(&mut bus, false, SAVE_BASE + 0x80, 4);
    assert_eq!(bus.read32(DRAM), 0xFFFF_FFFF);
    assert_eq!(bus.read32(SAVE_BASE), 0x1111_8004);
    assert_eq!(bus.read32(SAVE_BASE + 4), 0xF000_001D);
}

#[test]
fn a_page_number_reaches_past_255() {
    let mut bus = cartridge(save_type::FLASH_RAM, Some(vec![0; FLASH_SIZE]));
    bus.store(FLASH_COMMAND, 0x4B00_0300, 4);
    bus.store(FLASH_COMMAND, 0x7800_0000, 4);
    bus.store(FLASH_COMMAND, 0xD200_0000, 4);
    assert_eq!(flash(&bus).data[0x300 * 128], 0xFF);
    assert_eq!(flash(&bus).data[0], 0x00);
}

#[test]
fn the_page_buffer_wraps_every_128_bytes() {
    let mut bus = chip(save_type::FLASH_RAM);
    for i in 0..256u32 {
        bus.rdram[(DRAM + i) as usize] = i as u8;
    }
    bus.store(FLASH_COMMAND, 0xB400_0000, 4);
    transfer(&mut bus, true, SAVE_BASE, 256);
    bus.store(FLASH_COMMAND, 0xA500_0001, 4);
    bus.store(FLASH_COMMAND, 0xD200_0000, 4);
    assert_eq!(flash(&bus).data[128], 128);
    assert_eq!(flash(&bus).data[255], 255);
}

#[test]
fn a_write_to_the_domains_first_word_is_not_a_command() {
    let mut bus = chip(save_type::FLASH_RAM);
    bus.store(SAVE_BASE, 0xE100_0000, 4);
    assert_eq!(bus.read32(SAVE_BASE), 0);
}

// ---- Naming the chip ----

#[test]
fn the_first_processor_touch_of_an_undecided_second_domain_makes_it_flash() {
    let mut bus = chip(save_type::UNKNOWN);
    bus.store(FLASH_COMMAND, 0xE100_0000, 4);
    assert_eq!(bus.save.kind, save_type::FLASH_RAM);
}

#[test]
fn the_first_transfer_on_an_undecided_second_domain_makes_it_sram() {
    let mut bus = chip(save_type::UNKNOWN);
    transfer(&mut bus, false, SAVE_BASE, 8);
    assert_eq!(bus.save.kind, save_type::SRAM_256K);
    assert_eq!(bus.read32(DRAM), 0xFFFF_FFFF);
}

#[test]
fn once_named_sram_the_eeprom_channel_goes_quiet() {
    let mut bus = chip(save_type::UNKNOWN);
    transfer(&mut bus, false, SAVE_BASE, 8);
    let ram = run_cartridge(&mut bus, 1, 3, &[0x00]);
    assert_eq!(ram[5], joybus::NO_REPLY | 3);
}

#[test]
fn an_earlier_save_reaches_whichever_chip_the_game_turns_out_to_use() {
    let mut saved = vec![0u8; SRAM_BANK];
    saved[0x40] = 0x77;
    let mut bus = cartridge(save_type::UNKNOWN, Some(saved));
    transfer(&mut bus, false, SAVE_BASE + 0x40, 2);
    assert_eq!(dram(&bus, 0), 0x77);
}

#[test]
fn an_earlier_save_names_its_chip_by_its_length() {
    let rows = [
        (0x200, save_type::EEPROM_4K),
        (0x800, save_type::EEPROM_4K),
        (0x8000, save_type::SRAM_256K),
        (0x1_8000, save_type::SRAM_BANKED_768K),
        (0x2_0000, save_type::FLASH_RAM),
        (0x1000, save_type::UNKNOWN),
    ];
    for (length, expected) in rows {
        assert_eq!(SaveChip::from_save_length(length), expected, "length {length:X}");
    }
}

#[test]
fn the_image_can_declare_its_own_chip() {
    assert_eq!(rom_image(&build_homebrew(save_type::FLASH_RAM)).declared_save_type(), save_type::FLASH_RAM);
}

#[test]
fn a_title_in_the_table_is_named_by_its_two_checksums() {
    let mut image = build_rom();
    image[0x10..0x18].copy_from_slice(&[0xB6, 0x95, 0x1A, 0x94, 0x63, 0xC8, 0x49, 0xAF]);
    assert_eq!(rom_image(&image).declared_save_type(), save_type::EEPROM_16K);
    assert_eq!(rom_image(&build_rom()).declared_save_type(), save_type::UNKNOWN);
}

// ---- The Controller Pak ----

#[test]
fn the_data_crc_is_crc8_with_polynomial_0x85() {
    let counting: Vec<u8> = (0..32).collect();
    assert_eq!(ControllerPak::data_crc(&counting), 0x33);
    assert_eq!(ControllerPak::data_crc(&[0xFF; 32]), 0x0A);
    assert_eq!(ControllerPak::data_crc(&[0; 32]), 0x00);
}

#[test]
fn a_new_pak_is_formatted_with_checksummed_ids_and_an_empty_index() {
    let pak = ControllerPak::new(None);
    let data = &pak.data[..];
    assert_eq!(data[0x38..0x40], [0x00, 0x01, 0x01, 0x00, 0x01, 0x01, 0xFE, 0xF1]);
    for copy in [3usize, 4, 6] {
        assert_eq!(data[0x20..0x40], data[copy * 32..copy * 32 + 32], "copy {copy}");
    }
    assert_eq!(data[0x101], 0x71);
    assert_eq!(data[0x10A..0x10C], [0x00, 0x03]);
    assert_eq!(data[0x100..0x200], data[0x200..0x300]);
    assert_eq!(data[0x300], 0);
}

#[test]
fn a_pak_write_answers_its_crc_and_a_read_returns_the_chunk_and_its_crc() {
    let mut bus = new_bus();
    bus.si.controllers[0].pak = Some(ControllerPak::new(None));
    let chunk: Vec<u8> = (0..32).collect();

    let mut write = [0u8; 64];
    write[..5].copy_from_slice(&[35, 1, 0x03, 0x04, 0x00]);
    write[5..37].copy_from_slice(&chunk);
    write[38] = 0xFE;
    joybus::run(&mut write, &mut bus.si.controllers, &mut bus.save);

    let mut read = [0u8; 64];
    read[..6].copy_from_slice(&[3, 33, 0x02, 0x04, 0x1F, 0xFE]);
    joybus::run(&mut read, &mut bus.si.controllers, &mut bus.save);

    assert_eq!(write[37], 0x33);
    assert_eq!(read[5..37], chunk[..]);
    assert_eq!(read[37], 0x33);
    assert!(bus.si.controllers[0].pak.as_ref().unwrap().dirty);
}

#[test]
fn above_the_pak_reads_zero_and_keeps_nothing() {
    let mut pak = ControllerPak::new(None);
    let mut into = [0xAAu8; 32];
    pak.write(0x8020, &[0; 32]);
    pak.read(0x8020, &mut into);
    assert_eq!(into, [0; 32]);
    assert!(!pak.dirty);
}
