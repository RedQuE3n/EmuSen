//! The RCP's interfaces: `MarsDmaTests`, `MarsSerialTests`, `MarsEventTimingTests`, `MarsViTimingTests` and `MarsAudioTests`.

use super::support::{SyntheticRom, build_rom, bus_state, bus_with_cart, load_bus, new_bus};
use crate::ai::{DEFAULT_SAMPLE_RATE, MAX_BUFFERED_SAMPLES, PAGE_SIZE, SHORTEST_PERIOD, STATUS_ALWAYS_SET, STATUS_BUSY, STATUS_FULL};
use crate::bus::MemoryBus;
use crate::bus_access::map::{AI_BASE, CART_DOMAIN1_ADDRESS2, PI_BASE, SI_BASE, SP_DMEM_BASE, SP_REGISTERS_BASE, VI_BASE};
use crate::controller::ControllerPak;
use crate::joybus;
use crate::mi::interrupt;
use crate::rom::{HEADER_LENGTH, MAGIC};
use crate::si::TRANSFER_CYCLES;
use crate::sp::STATUS_HALT;

const SP_IMEM_BASE: u32 = 0x0400_1000;

// ---- MarsDmaTests ----

const SP_MEM: u32 = SP_REGISTERS_BASE;
const SP_DRAM: u32 = SP_REGISTERS_BASE + 0x04;
const SP_READ: u32 = SP_REGISTERS_BASE + 0x08;
const SP_WRITE: u32 = SP_REGISTERS_BASE + 0x0C;
const SP_STATUS: u32 = SP_REGISTERS_BASE + 0x10;
const PI_DRAM: u32 = PI_BASE;
const PI_CART: u32 = PI_BASE + 0x04;
const PI_WRITE: u32 = PI_BASE + 0x0C;
const PI_STATUS: u32 = PI_BASE + 0x10;
const PI_STATUS_DMA_BUSY: u32 = 0x01;
const PI_STATUS_IO_BUSY: u32 = 0x02;
const PI_STATUS_INTERRUPT: u32 = 0x08;

#[test]
fn a_transfer_moves_memory_into_the_data_bank() {
    let mut bus = new_bus();
    bus.write32(0x100, 0xCAFE_BABE);
    bus.write32(SP_MEM, 0);
    bus.write32(SP_DRAM, 0x100);
    bus.write32(SP_READ, 8 - 1);
    assert_eq!(bus.read32(SP_DMEM_BASE), 0xCAFE_BABE);
}

#[test]
fn the_address_bit_that_chooses_the_instruction_bank_is_honoured() {
    let mut bus = new_bus();
    bus.write32(0x100, 0xCAFE_BABE);
    bus.write32(SP_MEM, 0x1000);
    bus.write32(SP_DRAM, 0x100);
    bus.write32(SP_READ, 8 - 1);
    assert_eq!(bus.read32(SP_IMEM_BASE), 0xCAFE_BABE);
    assert_eq!(bus.read32(SP_DMEM_BASE), 0);
}

#[test]
fn a_transfer_runs_the_other_way_too() {
    let mut bus = new_bus();
    bus.write32(SP_DMEM_BASE, 0x1234_5678);
    bus.write32(SP_MEM, 0);
    bus.write32(SP_DRAM, 0x200);
    bus.write32(SP_WRITE, 8 - 1);
    assert_eq!(bus.read32(0x200), 0x1234_5678);
}

#[test]
fn a_transfer_off_the_end_of_the_instruction_bank_wraps_inside_it() {
    let mut bus = new_bus();
    for i in 0..0x20u32 {
        bus.write8(0x300 + i, 0xA0 + i as u8);
    }
    bus.write32(SP_MEM, 0x1000 | 0xFF8);
    bus.write32(SP_DRAM, 0x300);
    bus.write32(SP_READ, 0x10 - 1);
    assert_eq!(bus.read8(SP_IMEM_BASE + 0xFF8), 0xA0);
    assert_eq!(bus.read8(SP_IMEM_BASE), 0xA8);
    assert_eq!(bus.read8(SP_IMEM_BASE + 7), 0xAF);
    for i in 0..8 {
        assert_eq!(bus.read8(SP_DMEM_BASE + i), 0, "DMEM {i}");
    }
}

#[test]
fn a_rectangular_transfer_skips_between_rows() {
    let mut bus = new_bus();
    for i in 0..0x20u32 {
        bus.write8(0x400 + i, 0xB0 + i as u8);
    }
    bus.write32(SP_MEM, 0);
    bus.write32(SP_DRAM, 0x400);
    bus.write32(SP_READ, (8 - 1) | (1 << 12) | (8 << 20));
    assert_eq!(bus.read8(SP_DMEM_BASE), 0xB0);
    assert_eq!(bus.read8(SP_DMEM_BASE + 8), 0xC0);
}

#[test]
fn the_transfer_engine_is_never_busy_because_it_has_already_finished() {
    let mut bus = new_bus();
    assert_eq!(bus.read32(SP_REGISTERS_BASE + 0x18), 0);
    assert_eq!(bus.read32(SP_REGISTERS_BASE + 0x14), 0);
}

#[test]
fn the_signal_processor_comes_up_halted() {
    let mut bus = new_bus();
    assert_eq!(bus.read32(SP_STATUS) & STATUS_HALT, STATUS_HALT);
    bus.write32(SP_STATUS, 0x01);
    assert_eq!(bus.read32(SP_STATUS) & STATUS_HALT, 0);
}

#[test]
fn the_semaphore_is_taken_by_reading_it_and_released_by_writing_it() {
    let mut bus = new_bus();
    let semaphore = SP_REGISTERS_BASE + 0x1C;
    assert_eq!(bus.read32(semaphore), 0);
    assert_eq!(bus.read32(semaphore), 1);
    bus.write32(semaphore, 0);
    assert_eq!(bus.read32(semaphore), 0);
}

fn header_transfer(bus: &mut MemoryBus) {
    bus.write32(PI_DRAM, 0x1000);
    bus.write32(PI_CART, CART_DOMAIN1_ADDRESS2);
    bus.write32(PI_WRITE, 0x40 - 1);
}

#[test]
fn the_cartridge_engine_copies_the_header_into_memory() {
    let mut bus = bus_with_cart(&build_rom());
    header_transfer(&mut bus);
    assert_eq!(bus.read32(0x1000), MAGIC);
}

#[test]
fn the_cartridge_engine_reports_idle_so_a_polling_rom_makes_progress() {
    let mut bus = bus_with_cart(&build_rom());
    assert_eq!(bus.read32(PI_STATUS) & PI_STATUS_IO_BUSY, 0);
    assert_eq!(bus.read32(PI_STATUS) & PI_STATUS_DMA_BUSY, 0);
    header_transfer(&mut bus);
    assert_eq!(bus.read32(PI_STATUS) & PI_STATUS_IO_BUSY, 0);
    assert_eq!(bus.read32(PI_STATUS) & PI_STATUS_DMA_BUSY, 0);
}

#[test]
fn a_finished_transfer_raises_an_interrupt_flag_that_a_write_clears() {
    let mut bus = bus_with_cart(&build_rom());
    header_transfer(&mut bus);
    assert_eq!(bus.read32(PI_STATUS) & PI_STATUS_INTERRUPT, PI_STATUS_INTERRUPT);
    bus.write32(PI_STATUS, 0x02);
    assert_eq!(bus.read32(PI_STATUS) & PI_STATUS_INTERRUPT, 0);
}

#[test]
fn a_cartridge_transfer_advances_each_address_to_its_own_multiple() {
    let rows: [(u32, u32, u32); 8] = [(1, 2, 8), (2, 2, 8), (3, 4, 8), (7, 8, 8), (8, 8, 8), (9, 10, 16), (0x7C, 0x7C, 0x80), (0x7D, 0x7E, 0x80)];
    for (length, cart, dram) in rows {
        let mut bus = bus_with_cart(&build_rom());
        bus.write32(PI_DRAM, 0x0010_0000);
        bus.write32(PI_CART, 0x1000_0000);
        bus.write32(PI_WRITE, length - 1);
        assert_eq!(bus.read32(PI_CART), 0x1000_0000 + cart, "length {length}");
        assert_eq!(bus.read32(PI_DRAM), 0x0010_0000 + dram, "length {length}");
    }
}

#[test]
fn a_misaligned_cartridge_transfer_still_lands_on_a_block() {
    for (length, dram) in [(10u32, 0x10u32), (9, 0x10), (7, 0x08), (1, 0x08)] {
        let mut bus = bus_with_cart(&build_rom());
        bus.write32(PI_DRAM, 0x0010_0006);
        bus.write32(PI_CART, 0x1000_0000);
        bus.write32(PI_WRITE, length - 1);
        assert_eq!(bus.read32(PI_DRAM), 0x0010_0000 + dram, "length {length}");
    }
}

const BLOCK: u32 = 0x0010_0000;
const ROW_END: u32 = 0x0010_0800;
const PAINT: u8 = 0xAA;
const SPAN: usize = 0x180;

/// The corpus's arrangement: memory painted, the cartridge counting up, never through the paint value.
fn counting(offset: usize) -> u8 {
    (offset % PAINT as usize) as u8
}

fn counting_cartridge() -> MemoryBus {
    let bytes: Vec<u8> = (0..0x400).map(counting).collect();
    bus_with_cart(&SyntheticRom::patched(&bytes).build())
}

fn land(dram: u32, length: u32) -> Vec<u8> {
    land_on(&mut counting_cartridge(), dram, length)
}

fn land_on(bus: &mut MemoryBus, dram: u32, length: u32) -> Vec<u8> {
    for i in 0..SPAN as u32 {
        bus.write8(dram + i, PAINT);
    }
    bus.write32(PI_DRAM, dram);
    bus.write32(PI_CART, CART_DOMAIN1_ADDRESS2 + HEADER_LENGTH as u32);
    bus.write32(PI_WRITE, length - 1);
    (0..SPAN as u32).map(|i| bus.read8(dram + i)).collect()
}

/// Runs of destination offset, cartridge offset and count; everything else is still paint.
fn expect(runs: &[(usize, usize, usize)]) -> Vec<u8> {
    let mut expected = vec![PAINT; SPAN];
    for &(at, from, count) in runs {
        for i in 0..count {
            expected[at + i] = counting(from + i);
        }
    }
    expected
}

#[test]
fn a_short_misaligned_transfer_stores_its_misalignment_fewer_bytes() {
    for (length, stored) in [(1u32, 0usize), (2, 0), (6, 0), (7, 1), (8, 2), (16, 10), (119, 113), (120, 114)] {
        assert_eq!(land(BLOCK + 6, length), expect(&[(0, 0, stored)]), "length {length}");
    }
}

#[test]
fn a_first_block_one_byte_short_of_full_stores_its_last_pair_whole() {
    for (misaligned, length, stored) in [(0u32, 127u32, 128usize), (6, 121, 116)] {
        assert_eq!(land(BLOCK + misaligned, length), expect(&[(0, 0, stored)]), "misaligned {misaligned}");
    }
}

#[test]
fn the_first_misaligned_block_leaves_a_gap_before_the_next() {
    for length in [122u32, 128, 284] {
        assert_eq!(land(BLOCK + 6, length), expect(&[(0, 0, 116), (122, 122, length as usize - 122)]), "length {length}");
    }
}

#[test]
fn a_later_block_rounds_an_odd_remainder_up_to_a_pair() {
    for (length, stored) in [(129u32, 130usize), (131, 132), (133, 134)] {
        assert_eq!(land(BLOCK, length), expect(&[(0, 0, stored)]), "length {length}");
    }
}

#[test]
fn a_block_stops_at_the_end_of_a_row() {
    for (length, first, rest) in [(50u32, 44usize, 0usize), (66, 52, 8), (284, 52, 226)] {
        assert_eq!(land(ROW_END - 58, length), expect(&[(0, 0, first), (58, 58, rest)]), "length {length}");
    }
}

#[test]
fn a_block_at_the_end_of_a_row_shortens_the_one_after_it() {
    assert_eq!(land(ROW_END - 6, 7), expect(&[(0, 0, 4), (6, 6, 2)]));
    assert_eq!(land(ROW_END - 4, 6), expect(&[(4, 4, 2)]));
    assert_eq!(land(ROW_END - 2, 4), expect(&[(2, 2, 2)]));
    assert_eq!(land(ROW_END - 6, 224), expect(&[(0, 0, 4), (6, 6, 126), (134, 132, 92)]));
    assert_eq!(land(ROW_END - 2, 224), expect(&[(2, 2, 122), (130, 124, 100)]));
}

#[test]
fn a_row_is_two_kilobytes_so_a_one_kilobyte_boundary_does_not_stop_a_block() {
    assert_eq!(land(BLOCK + 0x400 - 58, 66), expect(&[(0, 0, 60)]));
}

#[test]
fn a_block_eight_or_more_bytes_from_the_end_of_a_row_leaves_the_next_one_whole() {
    assert_eq!(land(ROW_END - 14, 224), expect(&[(0, 0, 12), (14, 14, 210)]));
}

#[test]
fn a_shortened_block_does_not_outlive_its_transfer() {
    let mut bus = counting_cartridge();
    land_on(&mut bus, ROW_END - 6, 6);
    assert_eq!(land_on(&mut bus, BLOCK, 128), expect(&[(0, 0, 128)]));
}

#[test]
fn a_first_block_one_byte_short_of_a_row_ends_on_a_lone_byte() {
    assert_eq!(land(ROW_END - 58, 57), expect(&[(0, 0, 51)]));
}

// ---- MarsSerialTests ----

const DRAM: u32 = 0x0010_0000;
const SI_DRAM: u32 = SI_BASE;
const SI_READ: u32 = SI_BASE + 0x04;
const SI_WRITE: u32 = SI_BASE + 0x10;
const SI_STATUS: u32 = SI_BASE + 0x18;
const SI_STATUS_DMA_BUSY: u32 = 0x0001;
const SI_STATUS_INTERRUPT: u32 = 0x1000;

fn with_block(block: &[u8]) -> MemoryBus {
    let mut bus = new_bus();
    for i in 0..64usize {
        bus.rdram[DRAM as usize + i] = block.get(i).copied().unwrap_or(0);
    }
    bus.rdram[DRAM as usize + 63] = 1;
    bus.write32(SI_DRAM, DRAM);
    bus
}

/// The block in, and then out, which is when the PIF runs it.
fn run_block(bus: &mut MemoryBus) {
    bus.write32(SI_WRITE, 0);
    bus.tick(TRANSFER_CYCLES);
    bus.write32(SI_STATUS, 0);
    bus.write32(SI_READ, 0);
}

fn finish(bus: &mut MemoryBus) {
    bus.tick(TRANSFER_CYCLES);
}

fn si_raised(bus: &MemoryBus) -> bool {
    bus.mi.pending & interrupt::SERIAL_INTERFACE != 0
}

fn dram(bus: &MemoryBus, offset: u32) -> u8 {
    bus.rdram[(DRAM + offset) as usize]
}

#[test]
fn a_transfer_carries_sixty_four_bytes_each_way_and_interrupts_when_its_time_has_passed() {
    let mut bus = with_block(&[0xFE]);
    run_block(&mut bus);
    assert!(!si_raised(&bus));
    assert_eq!(bus.read32(SI_STATUS), SI_STATUS_DMA_BUSY);

    bus.tick(TRANSFER_CYCLES - 1);
    assert!(!si_raised(&bus));
    bus.tick(1);
    assert!(si_raised(&bus));
    assert_eq!(bus.read32(SI_STATUS), SI_STATUS_INTERRUPT);

    bus.pif_ram[7] = 0xA5;
    bus.write32(SI_READ, 0);
    finish(&mut bus);
    assert_eq!(dram(&bus, 7), 0xA5);
}

#[test]
fn a_read_lands_in_memory_when_the_transfer_finishes_so_a_write_made_meanwhile_is_overwritten() {
    let mut bus = with_block(&[0xFF, 0x01, 0x04, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE]);
    bus.si.controllers[0].buttons = 0x1000;
    bus.write32(SI_WRITE, 0);
    bus.tick(TRANSFER_CYCLES);
    bus.write32(SI_STATUS, 0);

    bus.write32(SI_READ, 0);
    for i in (0..64).step_by(4) {
        bus.rdram[(DRAM + i + 3) as usize] = 0xFF;
    }
    bus.rdram[(DRAM + 4) as usize] = 0;

    bus.tick(TRANSFER_CYCLES - 1);
    assert_eq!(dram(&bus, 4), 0x00);
    bus.tick(1);
    assert_eq!(dram(&bus, 4), 0x10);
    assert_eq!(dram(&bus, 2), 0x04);
    assert!(si_raised(&bus));
}

#[test]
fn writing_the_status_clears_the_interrupt() {
    let mut bus = with_block(&[0xFE]);
    run_block(&mut bus);
    bus.tick(TRANSFER_CYCLES);
    assert!(si_raised(&bus));
    bus.write32(SI_STATUS, 0);
    assert!(!si_raised(&bus));
    assert_eq!(bus.read32(SI_STATUS), 0);
}

#[test]
fn a_state_saved_during_a_transfer_still_interrupts_when_it_is_loaded() {
    let mut bus = with_block(&[0xFE]);
    run_block(&mut bus);
    bus.tick(100);
    bus.rdram[DRAM as usize] = 0x00;
    let state = bus_state(&mut bus);

    let mut loaded = new_bus();
    load_bus(&mut loaded, &state);
    assert_eq!(loaded.si.due, bus.si.due);
    loaded.tick(TRANSFER_CYCLES - 101);
    assert!(!si_raised(&loaded));
    loaded.tick(1);
    assert!(si_raised(&loaded));

    assert_eq!(dram(&loaded, 0), 0xFE);
    assert_eq!(loaded.si.pending_read, -1);

    let idle = bus_state(&mut loaded);
    assert_eq!(idle.len(), state.len() - 24);
}

#[test]
fn a_block_runs_on_a_read_whatever_its_last_byte_says() {
    let mut bus = with_block(&[0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0xFE]);
    bus.rdram[DRAM as usize + 63] = 0;
    run_block(&mut bus);
    assert_eq!(bus.pif_ram[3], 0x05);
}

#[test]
fn each_read_runs_the_block_again_with_the_buttons_as_they_are_now() {
    let mut bus = with_block(&[0xFF, 0x01, 0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0xFE]);
    run_block(&mut bus);
    finish(&mut bus);
    assert_eq!(dram(&bus, 4), 0x00);

    bus.si.controllers[0].buttons = 0x1000;
    bus.write32(SI_READ, 0);
    finish(&mut bus);
    assert_eq!(&bus.rdram[(DRAM + 4) as usize..(DRAM + 6) as usize], &[0x10, 0x00]);
}

#[test]
fn a_block_does_not_run_on_the_way_in() {
    let mut bus = with_block(&[0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0xFE]);
    bus.write32(SI_WRITE, 0);
    assert_eq!(bus.pif_ram[3], 0xFF);
}

#[test]
fn an_info_command_names_a_controller_with_no_pak() {
    let mut bus = with_block(&[0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0xFE]);
    run_block(&mut bus);
    assert_eq!(bus.pif_ram[1], 0x03);
    assert_eq!(bus.pif_ram[3], 0x05);
    assert_eq!(bus.pif_ram[4], 0x00);
    assert_eq!(bus.pif_ram[5], 0x02);
}

#[test]
fn a_controller_pak_shows_in_the_info_reply() {
    let mut bus = with_block(&[0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0xFE]);
    bus.si.controllers[0].pak = Some(ControllerPak::new(None));
    run_block(&mut bus);
    assert_eq!(bus.pif_ram[5], 0x01);
}

#[test]
fn a_state_command_reports_the_buttons_and_the_stick() {
    let mut bus = with_block(&[0x01, 0x04, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFE]);
    bus.si.controllers[0].buttons = 0x8021;
    bus.si.controllers[0].stick_x = 40;
    bus.si.controllers[0].stick_y = -40;
    run_block(&mut bus);
    assert_eq!(bus.pif_ram[3], 0x80);
    assert_eq!(bus.pif_ram[4], 0x21);
    assert_eq!(bus.pif_ram[5], 40);
    assert_eq!(bus.pif_ram[6], 0xD8);
}

#[test]
fn an_empty_port_sets_the_no_reply_bit() {
    let mut bus = with_block(&[0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0xFE]);
    bus.si.controllers[0].present = false;
    run_block(&mut bus);
    assert_eq!(bus.pif_ram[1], joybus::NO_REPLY | 0x03);
    assert_eq!(bus.pif_ram[3], 0xFF);
}

#[test]
fn a_reply_with_too_little_room_sets_the_over_run_bit() {
    let mut bus = with_block(&[0x01, 0x02, 0x00, 0xFF, 0xFF, 0xFE]);
    run_block(&mut bus);
    assert_eq!(bus.pif_ram[1], joybus::OVER_RUN | 0x02);
    assert_eq!(bus.pif_ram[3], 0x05);
    assert_eq!(bus.pif_ram[4], 0x00);
    assert_eq!(bus.pif_ram[5], joybus::END);
}

#[test]
fn a_controller_pak_read_is_not_answered() {
    let mut bus = with_block(&[0x03, 0x21, 0x02, 0x00, 0x00, 0xFE]);
    run_block(&mut bus);
    assert_eq!(bus.pif_ram[1], joybus::NO_REPLY | 0x21);
}

#[test]
fn a_zero_length_moves_to_the_next_channel() {
    let mut bus = with_block(&[0x00, 0x00, 0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0xFE]);
    bus.si.controllers[0].present = false;
    bus.si.controllers[2].present = true;
    run_block(&mut bus);
    assert_eq!(bus.pif_ram[3], 0x03);
    assert_eq!(bus.pif_ram[5], 0x05);
}

#[test]
fn a_command_moves_to_the_next_channel_as_well() {
    let mut bus = with_block(&[0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0xFE]);
    bus.si.controllers[1].present = false;
    run_block(&mut bus);
    assert_eq!(bus.pif_ram[1], 0x03);
    assert_eq!(bus.pif_ram[7], joybus::NO_REPLY | 0x03);
}

#[test]
fn the_two_lengths_are_six_bits_each() {
    let mut bus = with_block(&[0x41, 0x43, 0x00, 0xFF, 0xFF, 0xFF, 0xFE]);
    run_block(&mut bus);
    assert_eq!(bus.pif_ram[3], 0x05);
    assert_eq!(bus.pif_ram[5], 0x02);
}

#[test]
fn the_end_byte_is_not_read_as_a_send_length() {
    let mut bus = with_block(&[0xFE, 0x80, 0x00]);
    run_block(&mut bus);
    assert_eq!(bus.pif_ram[1], 0x80);
}

#[test]
fn the_end_byte_is_not_read_as_a_receive_length() {
    let mut bus = with_block(&[0x40, 0xFE, 0x00, 0x00, 0x00]);
    run_block(&mut bus);
    assert_eq!(bus.pif_ram[2], 0x00);
    assert_eq!(bus.pif_ram[3], 0x00);
}

#[test]
fn a_block_runs_on_the_way_out() {
    let mut bus = new_bus();
    let block = [0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0xFE];
    bus.pif_ram[..block.len()].copy_from_slice(&block);
    bus.pif_ram[63] = 1;
    bus.write32(SI_DRAM, DRAM);
    bus.write32(SI_READ, 0);
    finish(&mut bus);
    assert_eq!(bus.pif_ram[3], 0x05);
    assert_eq!(dram(&bus, 3), 0x05);
}

#[test]
fn padding_does_not_move_to_the_next_channel() {
    let mut bus = with_block(&[0xFF, 0xFD, 0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0xFE]);
    bus.si.controllers[0].present = true;
    run_block(&mut bus);
    assert_eq!(bus.pif_ram[5], 0x05);
}

#[test]
fn the_end_byte_stops_the_block() {
    let mut bus = with_block(&[0xFE, 0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF]);
    run_block(&mut bus);
    assert_eq!(bus.pif_ram[4], 0xFF);
}

#[test]
fn the_pif_leaves_the_byte_it_was_started_by() {
    let mut bus = with_block(&[0x01, 0x03, 0x00, 0xFF, 0xFF, 0xFF, 0xFE]);
    run_block(&mut bus);
    assert_eq!(bus.pif_ram[63], 0x01);
}

// ---- MarsEventTimingTests ----

const PROCESSOR_CLOCK: i64 = 93_750_000;
const NTSC_CLOCK: i64 = 48_681_818;
const PAL_CLOCK: i64 = 49_656_530;
const CURRENT_LINE: u32 = 0x10;
const VERTICAL_SYNC: u32 = 0x18;
const HORIZONTAL_SYNC: u32 = 0x1C;
const NTSC_SYNC: u32 = 525;
const PAL_SYNC: u32 = 625;
const LINE: u32 = 3093;
const THRESHOLD: i64 = LINE as i64 * PROCESSOR_CLOCK;
const RATE: u32 = 1520;
const PERIOD: i64 = (RATE as i64 + 1) * PROCESSOR_CLOCK;
const BUFFER: u32 = 0x0010_0000;
const AI_DRAM: u32 = AI_BASE;
const AI_LENGTH: u32 = AI_BASE + 0x04;
const AI_CONTROL: u32 = AI_BASE + 0x08;
const AI_STATUS: u32 = AI_BASE + 0x0C;
const AI_DAC_RATE: u32 = AI_BASE + 0x10;

fn signal(sync: u32) -> MemoryBus {
    let mut bus = new_bus();
    bus.write32(VI_BASE + VERTICAL_SYNC, sync);
    bus.write32(VI_BASE + HORIZONTAL_SYNC, LINE);
    bus
}

fn playing_with(bus: &mut MemoryBus, length: u32) {
    bus.write32(AI_DAC_RATE, RATE);
    bus.write32(AI_CONTROL, 1);
    queue_at(bus, BUFFER, length);
}

fn queue_at(bus: &mut MemoryBus, address: u32, length: u32) {
    bus.write32(AI_DRAM, address);
    bus.write32(AI_LENGTH, length);
}

fn current(bus: &mut MemoryBus) -> u32 {
    bus.read32(VI_BASE + CURRENT_LINE)
}

fn ceiling(owed: i64, rate: i64) -> i64 {
    (owed + rate - 1) / rate
}

fn cycles_to_next_line(bus: &mut MemoryBus) -> i64 {
    let start = current(bus);
    for n in 1..=20_000 {
        bus.tick(1);
        if current(bus) != start {
            return n;
        }
    }
    -1
}

fn cycles_to_next_sample(bus: &mut MemoryBus) -> i64 {
    let start = bus.ai.samples_played;
    for n in 1..=20_000 {
        bus.tick(1);
        if bus.ai.samples_played != start {
            return n;
        }
    }
    -1
}

/// Every change either device makes over one-cycle ticks, by the cycle it came on.
fn trace(bus: &mut MemoryBus, cycles: i64) -> Vec<String> {
    let mut seen = Vec::new();
    let mut line = current(bus);
    let mut samples = bus.ai.samples_played;
    for n in 1..=cycles {
        bus.tick(1);
        if current(bus) == line && bus.ai.samples_played == samples {
            continue;
        }
        line = current(bus);
        samples = bus.ai.samples_played;
        seen.push(format!("{n}: line {line}, samples {samples}, pending {}", bus.mi.pending));
    }
    seen
}

#[test]
fn a_line_turns_on_the_first_tick_whose_cycles_cover_it() {
    let mut bus = signal(NTSC_SYNC);
    assert_eq!(cycles_to_next_line(&mut bus), ceiling(2 * THRESHOLD, 2 * NTSC_CLOCK));
    assert_eq!(cycles_to_next_line(&mut bus), ceiling(4 * THRESHOLD, 2 * NTSC_CLOCK) - ceiling(2 * THRESHOLD, 2 * NTSC_CLOCK));
}

#[test]
fn a_switch_to_pal_mid_line_owes_the_cycles_before_it_at_the_ntsc_rate() {
    let mut bus = signal(NTSC_SYNC);
    bus.tick(1000);
    bus.write32(VI_BASE + VERTICAL_SYNC, PAL_SYNC);
    assert_eq!(cycles_to_next_line(&mut bus), ceiling(2 * THRESHOLD - 1000 * 2 * NTSC_CLOCK, 2 * PAL_CLOCK));
}

#[test]
fn a_signal_stopped_by_a_zero_sync_owes_nothing_for_the_time_it_stood() {
    let mut bus = signal(NTSC_SYNC);
    bus.tick(1000);
    bus.write32(VI_BASE + VERTICAL_SYNC, 0);
    bus.tick(1_000_000);
    assert_eq!(current(&mut bus), 0);
    bus.write32(VI_BASE + VERTICAL_SYNC, NTSC_SYNC);
    assert_eq!(cycles_to_next_line(&mut bus), ceiling(2 * THRESHOLD - 1000 * 2 * NTSC_CLOCK, 2 * NTSC_CLOCK));
}

/// Two samples and half a period more in one tick: the half period is left over when the buffer runs out.
fn ran_out_with_half_a_period_owed() -> MemoryBus {
    let mut bus = new_bus();
    playing_with(&mut bus, 8);
    bus.tick(ceiling(5 * PERIOD / 2, NTSC_CLOCK));
    assert_eq!(bus.ai.samples_played, 2);
    bus
}

#[test]
fn a_buffer_queued_after_the_interface_stood_idle_starts_a_whole_period_later() {
    let mut bus = ran_out_with_half_a_period_owed();
    bus.tick(10_000);
    queue_at(&mut bus, BUFFER, 8);
    assert_eq!(cycles_to_next_sample(&mut bus), ceiling(PERIOD, NTSC_CLOCK));
}

#[test]
fn a_buffer_queued_on_the_cycle_the_last_one_ran_out_keeps_what_was_owed() {
    let mut bus = ran_out_with_half_a_period_owed();
    let leftover = ceiling(5 * PERIOD / 2, NTSC_CLOCK) * NTSC_CLOCK - 2 * PERIOD;
    queue_at(&mut bus, BUFFER, 8);
    assert_eq!(cycles_to_next_sample(&mut bus), ceiling(PERIOD - leftover, NTSC_CLOCK));
}

#[test]
fn a_state_is_the_same_bytes_however_the_ticks_that_reached_it_were_cut() {
    let mut whole = signal(NTSC_SYNC);
    let mut cut = signal(NTSC_SYNC);
    playing_with(&mut whole, 0x1000);
    playing_with(&mut cut, 0x1000);
    whole.tick(12_345);
    for _ in 0..12_345 {
        cut.tick(1);
    }
    assert!(bus_state(&mut whole) == bus_state(&mut cut));
}

#[test]
fn a_state_loaded_into_a_fresh_bus_keeps_step_with_the_one_that_saved_it() {
    let mut saver = signal(NTSC_SYNC);
    playing_with(&mut saver, 0x1000);
    for _ in 0..12_345 {
        saver.tick(1);
    }
    let state = bus_state(&mut saver);
    let mut loader = new_bus();
    load_bus(&mut loader, &state);
    assert_eq!(trace(&mut saver, 30_000), trace(&mut loader, 30_000));
}

#[test]
fn a_state_loaded_back_into_the_bus_that_saved_it_replays_what_followed() {
    let mut bus = signal(NTSC_SYNC);
    playing_with(&mut bus, 0x1000);
    bus.tick(12_345);
    let state = bus_state(&mut bus);
    let first = trace(&mut bus, 30_000);
    bus.tick(100_000);
    load_bus(&mut bus, &state);
    assert_eq!(trace(&mut bus, 30_000), first);
}

// ---- MarsViTimingTests ----

const VI_CONTROL: u32 = 0x00;
const VI_INTERRUPT: u32 = 0x0C;
const NTSC_LINE: u32 = 3093;
const PAL_LINE: u32 = 3177;
const NTSC_HALF_LINE: i64 = 2979;

fn programmed(sync: u32, line: u32, interrupt_at: u32, serrate: bool) -> MemoryBus {
    let mut bus = new_bus();
    bus.write32(VI_BASE + VI_CONTROL, if serrate { 1 << 6 } else { 0 });
    bus.write32(VI_BASE + VERTICAL_SYNC, sync);
    bus.write32(VI_BASE + HORIZONTAL_SYNC, line);
    bus.write32(VI_BASE + VI_INTERRUPT, interrupt_at);
    bus.mi.clear(interrupt::VIDEO_INTERFACE);
    bus
}

fn ntsc(interrupt_at: u32) -> MemoryBus {
    programmed(NTSC_SYNC, NTSC_LINE, interrupt_at, false)
}

fn vi_raised(bus: &MemoryBus) -> bool {
    bus.mi.pending & interrupt::VIDEO_INTERFACE != 0
}

#[test]
fn the_half_line_advances_at_the_rate_the_registers_ask_for() {
    let mut bus = ntsc(0x3FF);
    bus.tick(NTSC_HALF_LINE * 2);
    assert_eq!(current(&mut bus), 2);
    bus.tick(NTSC_HALF_LINE * 8);
    assert_eq!(current(&mut bus), 10);
}

#[test]
fn a_pal_signal_counts_at_its_own_rate() {
    let mut ntsc_bus = ntsc(0x3FF);
    let mut pal = programmed(PAL_SYNC, PAL_LINE, 0x3FF, false);
    ntsc_bus.tick(1_000_000);
    pal.tick(1_000_000);
    assert_eq!(current(&mut ntsc_bus), 334);
    assert_eq!(current(&mut pal), 332);
}

#[test]
fn a_progressive_signal_never_reports_a_field() {
    let mut bus = ntsc(0x3FF);
    for i in 0..99 {
        bus.tick(NTSC_HALF_LINE);
        assert_eq!(current(&mut bus) & 1, 0, "half line {i}");
    }
    assert_eq!(current(&mut bus), 98);
}

#[test]
fn an_interlaced_signal_changes_field_when_the_count_wraps() {
    let mut bus = programmed(NTSC_SYNC, NTSC_LINE, 0x3FF, true);
    bus.tick(NTSC_HALF_LINE * (NTSC_SYNC as i64 - 1));
    assert_eq!(current(&mut bus) & 1, 0);
    bus.tick(NTSC_HALF_LINE);
    assert_eq!(current(&mut bus) & 1, 1);
    assert_eq!(current(&mut bus), 1);
    bus.tick(NTSC_HALF_LINE * NTSC_SYNC as i64);
    assert_eq!(current(&mut bus) & 1, 0);
}

#[test]
fn the_interrupt_is_raised_when_the_half_line_reaches_its_register() {
    let mut bus = ntsc(4);
    bus.tick(NTSC_HALF_LINE * 3);
    assert!(!vi_raised(&bus));
    bus.tick(NTSC_HALF_LINE);
    assert!(vi_raised(&bus));
}

#[test]
fn an_odd_interrupt_register_is_never_reached() {
    let mut bus = ntsc(5);
    bus.tick(NTSC_HALF_LINE * NTSC_SYNC as i64 * 2);
    assert!(!vi_raised(&bus));
}

#[test]
fn writing_the_current_line_clears_the_interrupt() {
    let mut bus = ntsc(2);
    bus.tick(NTSC_HALF_LINE * 2);
    assert!(vi_raised(&bus));
    bus.write32(VI_BASE + CURRENT_LINE, 0);
    assert!(!vi_raised(&bus));
}

#[test]
fn another_register_does_not_clear_the_interrupt() {
    let mut bus = ntsc(2);
    bus.tick(NTSC_HALF_LINE * 2);
    bus.write32(VI_BASE + VI_INTERRUPT, 2);
    assert!(vi_raised(&bus));
}

#[test]
fn the_count_wraps_at_the_vertical_sync() {
    let mut bus = ntsc(0x3FF);
    bus.tick(NTSC_HALF_LINE * (NTSC_SYNC as i64 - 2));
    assert_eq!(current(&mut bus), NTSC_SYNC - 3);
    bus.tick(NTSC_HALF_LINE * 2);
    assert_eq!(current(&mut bus), 0);
}

#[test]
fn an_unprogrammed_interface_does_not_count() {
    let mut bus = new_bus();
    bus.write32(VI_BASE + VI_INTERRUPT, 0);
    bus.tick(10_000_000);
    assert_eq!(current(&mut bus), 0);
    assert!(!vi_raised(&bus));
}

#[test]
fn a_signal_with_a_line_but_no_vertical_sync_does_not_count() {
    let mut bus = programmed(0, NTSC_LINE, 0, false);
    bus.tick(NTSC_HALF_LINE * 100);
    assert_eq!(current(&mut bus), 0);
    assert!(!vi_raised(&bus));
}

#[test]
fn an_interrupt_register_above_nine_bits_is_still_reached() {
    let mut bus = ntsc(0x208);
    bus.tick(NTSC_HALF_LINE * 0x207);
    assert!(!vi_raised(&bus));
    bus.tick(NTSC_HALF_LINE);
    assert!(vi_raised(&bus));
}

#[test]
fn a_line_of_no_length_does_not_count() {
    let mut bus = programmed(NTSC_SYNC, 0, 0x3FF, false);
    bus.tick(10_000_000);
    assert_eq!(current(&mut bus), 0);
}

// ---- MarsAudioTests ----

fn playing(dac_rate: u32) -> MemoryBus {
    let mut bus = new_bus();
    bus.write32(AI_DAC_RATE, dac_rate);
    bus.write32(AI_CONTROL, 1);
    bus
}

/// Exactly the cycles `samples` DAC periods take, from a standing start.
fn play_samples(bus: &mut MemoryBus, samples: i64, dac_rate: u32) {
    let period = (dac_rate + 1).max(SHORTEST_PERIOD) as i64;
    let clock = bus.vi.video_clock();
    bus.tick((samples * period * PROCESSOR_CLOCK + clock - 1) / clock);
}

fn ai_status(bus: &mut MemoryBus) -> u32 {
    bus.read32(AI_STATUS)
}

fn ai_raised(bus: &MemoryBus) -> bool {
    bus.mi.pending & interrupt::AUDIO_INTERFACE != 0
}

fn sample_rate(bus: &MemoryBus) -> i32 {
    bus.ai.sample_rate(bus.vi.video_clock())
}

/// `Math.Round`, which rounds a tie to even.
fn rounded(clock: i64, period: i64) -> i32 {
    (clock as f64 / period as f64).round_ties_even() as i32
}

#[test]
fn the_status_reads_as_the_fpga_core_builds_it() {
    let mut bus = new_bus();
    assert_eq!(ai_status(&mut bus), STATUS_ALWAYS_SET);
    bus.write32(AI_CONTROL, 1);
    queue_at(&mut bus, BUFFER, 0x100);
    assert_eq!(ai_status(&mut bus), 0x4310_0000);
    queue_at(&mut bus, BUFFER + 0x100, 0x100);
    assert_eq!(ai_status(&mut bus), 0xC310_0001);
}

#[test]
fn a_length_written_while_idle_begins_a_buffer_and_raises_the_interrupt() {
    let mut bus = playing(RATE);
    assert!(!ai_raised(&bus));
    queue_at(&mut bus, BUFFER, 0x100);
    assert!(ai_raised(&bus));
    assert_eq!(bus.read32(AI_LENGTH), 0x100);
    bus.write32(AI_STATUS, 0);
    assert!(!ai_raised(&bus));
}

#[test]
fn a_third_buffer_is_dropped_while_two_are_held() {
    let mut bus = playing(RATE);
    queue_at(&mut bus, BUFFER, 8);
    queue_at(&mut bus, BUFFER + 8, 8);
    queue_at(&mut bus, BUFFER + 16, 0x800);
    play_samples(&mut bus, 4, RATE);
    assert_eq!(ai_status(&mut bus) & STATUS_BUSY, 0);
    assert_eq!(bus.ai.samples_played, 4);
}

#[test]
fn the_remaining_length_counts_down_as_samples_play() {
    let mut bus = playing(RATE);
    queue_at(&mut bus, BUFFER, 0x100);
    play_samples(&mut bus, 10, RATE);
    assert_eq!(bus.read32(AI_LENGTH), 0x100 - 40);
}

#[test]
fn a_finished_buffer_hands_over_to_the_waiting_one_with_an_interrupt() {
    let mut bus = playing(RATE);
    queue_at(&mut bus, BUFFER, 8);
    queue_at(&mut bus, BUFFER + 0x40, 8);
    bus.write32(AI_STATUS, 0);

    play_samples(&mut bus, 2, RATE);
    assert!(ai_raised(&bus));
    assert_eq!(ai_status(&mut bus) & (STATUS_BUSY | STATUS_FULL), STATUS_BUSY);

    bus.write32(AI_STATUS, 0);
    play_samples(&mut bus, 2, RATE);
    assert!(!ai_raised(&bus));
    assert_eq!(ai_status(&mut bus) & STATUS_BUSY, 0);
}

#[test]
fn a_sample_is_two_big_endian_halves_left_first() {
    let mut bus = playing(RATE);
    bus.write32(BUFFER, 0x1234_FEDC);
    bus.write32(BUFFER + 4, 0x8000_7FFF);
    queue_at(&mut bus, BUFFER, 8);
    play_samples(&mut bus, 2, RATE);
    assert_eq!(bus.ai.drain(usize::MAX), [0x1234, 0xFEDC_u16 as i16, i16::MIN, i16::MAX]);
}

#[test]
fn samples_play_at_the_rate_the_dac_divides_the_video_clock_by() {
    let mut bus = playing(RATE);
    queue_at(&mut bus, BUFFER, 0x1000);
    let clock = bus.vi.video_clock();
    bus.tick((100 * PERIOD + clock - 1) / clock - 1);
    assert_eq!(bus.ai.samples_played, 99);
    bus.tick(1);
    assert_eq!(bus.ai.samples_played, 100);
    assert_eq!(sample_rate(&bus), rounded(clock, RATE as i64 + 1));
}

#[test]
fn a_period_shorter_than_the_floor_plays_at_the_floor() {
    for (dac_rate, period) in [(0x010u32, 512i64), (0x1FE, 512), (0x1FF, 512), (0x200, 513)] {
        let bus = playing(dac_rate);
        assert_eq!(sample_rate(&bus), rounded(bus.vi.video_clock(), period), "rate {dac_rate:X}");
    }
}

#[test]
fn an_address_drops_its_low_three_bits() {
    let mut bus = playing(RATE);
    bus.write32(BUFFER, 0x0102_0304);
    bus.write32(BUFFER + 4, 0x0506_0708);
    queue_at(&mut bus, BUFFER + 4, 8);
    play_samples(&mut bus, 1, RATE);
    assert_eq!(bus.ai.drain(usize::MAX), [0x0102, 0x0304]);
}

#[test]
fn a_length_drops_its_low_three_bits() {
    let mut bus = playing(RATE);
    queue_at(&mut bus, BUFFER, 0x0F);
    assert_eq!(bus.read32(AI_LENGTH), 8);
    play_samples(&mut bus, 3, RATE);
    assert_eq!(bus.ai.samples_played, 2);
}

#[test]
fn before_a_game_sets_a_rate_the_frontends_default_stands() {
    assert_eq!(sample_rate(&new_bus()), DEFAULT_SAMPLE_RATE);
}

#[test]
fn nothing_moves_while_the_dma_is_off() {
    let mut bus = playing(RATE);
    bus.write32(AI_CONTROL, 0);
    queue_at(&mut bus, BUFFER, 0x100);
    play_samples(&mut bus, 10, RATE);
    assert_eq!(bus.ai.samples_played, 0);
    assert_eq!(bus.read32(AI_LENGTH), 0x100);
}

#[test]
fn a_buffer_ending_on_a_page_starts_the_next_one_a_page_further_on() {
    let mut bus = playing(RATE);
    bus.write32(BUFFER + PAGE_SIZE, 0x1111_2222);
    bus.write32(BUFFER + 2 * PAGE_SIZE, 0x3333_4444);
    queue_at(&mut bus, BUFFER + PAGE_SIZE - 8, 8);
    queue_at(&mut bus, BUFFER + PAGE_SIZE, 8);
    play_samples(&mut bus, 3, RATE);
    let played = bus.ai.drain(usize::MAX);
    assert_eq!(played[4..6], [0x3333, 0x4444]);
}

#[test]
fn inside_a_buffer_crossing_a_page_is_continuous() {
    let mut bus = playing(RATE);
    let page = BUFFER + PAGE_SIZE;
    bus.write32(page - 8, 0x0101_0202);
    bus.write32(page - 4, 0x0303_0404);
    bus.write32(page, 0x0505_0606);
    bus.write32(page + 4, 0x0707_0808);
    queue_at(&mut bus, page - 8, 16);
    play_samples(&mut bus, 4, RATE);
    assert_eq!(bus.ai.drain(usize::MAX), [0x0101, 0x0202, 0x0303, 0x0404, 0x0505, 0x0606, 0x0707, 0x0808]);
}

#[test]
fn draining_takes_whole_pairs_and_leaves_the_rest() {
    let mut bus = playing(RATE);
    queue_at(&mut bus, BUFFER, 16);
    play_samples(&mut bus, 4, RATE);
    assert_eq!(bus.ai.drain(1).len(), 2);
    assert_eq!(bus.ai.drain(usize::MAX).len(), 6);
    assert!(bus.ai.drain(usize::MAX).is_empty());
}

#[test]
fn an_undrained_queue_keeps_only_the_newest_samples() {
    let mut bus = playing(0);
    queue_at(&mut bus, BUFFER, 0x3_FFF8);
    play_samples(&mut bus, MAX_BUFFERED_SAMPLES as i64 / 2 + 10, 0);
    assert_eq!(bus.ai.samples.len(), MAX_BUFFERED_SAMPLES);
    assert_eq!(bus.ai.samples_played, MAX_BUFFERED_SAMPLES as i64 / 2 + 10);
}
