//! The memory map and the processor's bus: `MarsBusTests`, `MarsMiTests` and `MarsCartridgeTests`.

use super::support::{Asm, SyntheticRom, build_rom, bus_with_cart, new_bus, new_bus_expanded};
use crate::bus::{MemoryBus, RDRAM_SIZE, RI_SELECT};
use crate::bus_access::map::{CART_DOMAIN1_ADDRESS2, IS_VIEWER_BASE, MI_BASE, PI_BASE, RDRAM_REGISTERS_BASE, SP_DMEM_BASE};
use crate::isviewer::{BUFFER_OFFSET, LENGTH_REGISTER_OFFSET};
use crate::mi::interrupt;
use crate::pi::STORE_DECAY_CYCLES;
use crate::rom::{HEADER_LENGTH, MAGIC, MINIMUM_LENGTH};
use crate::segments::{self, Mode, Segment};

const SP_IMEM_BASE: u32 = 0x0400_1000;

// ---- MarsBusTests ----

#[test]
fn the_two_untranslated_segments_strip_the_top_bits() {
    let rows: [(u64, u32); 4] = [
        (0xFFFF_FFFF_8000_0000, 0x0000_0000),
        (0xFFFF_FFFF_A000_0000, 0x0000_0000),
        (0xFFFF_FFFF_A400_0000, 0x0400_0000),
        (0xFFFF_FFFF_B3FF_0014, 0x13FF_0014),
    ];
    for (address, physical) in rows {
        assert_eq!(segments::decode(address, Mode::Kernel, false), Segment::Direct(physical), "{address:016X}");
    }
}

#[test]
fn every_other_segment_needs_the_tlb() {
    for address in [0x0000_0000_0000_0000u64, 0xFFFF_FFFF_C000_0000, 0xFFFF_FFFF_E000_0000] {
        assert_eq!(segments::decode(address, Mode::Kernel, false), Segment::Mapped, "{address:016X}");
    }
}

#[test]
fn rdram_round_trips_at_every_width() {
    let mut bus = new_bus();
    bus.write32(0x10, 0x1122_3344);
    assert_eq!(bus.read32(0x10), 0x1122_3344);
    assert_eq!(bus.read8(0x10), 0x11);
    assert_eq!(bus.read8(0x13), 0x44);
    assert_eq!(bus.read16(0x10), 0x1122);
    assert_eq!(bus.read16(0x12), 0x3344);

    bus.write64(0x20, 0x0102_0304_0506_0708);
    assert_eq!(bus.read64(0x20), 0x0102_0304_0506_0708);

    bus.write8(0x11, 0xFF);
    assert_eq!(bus.read32(0x10), 0x11FF_3344);

    bus.write16(0x12, 0xBEEF);
    assert_eq!(bus.read32(0x10), 0x11FF_BEEF);
}

#[test]
fn rdram_above_the_installed_size_reads_zero_rather_than_mirroring() {
    let mut bus = new_bus();
    bus.write32(0x10, 0x1122_3344);
    assert_eq!(bus.read32(RDRAM_SIZE as u32 + 0x10), 0);
}

#[test]
fn an_expansion_pak_makes_the_upper_half_real_memory() {
    let mut bus = new_bus_expanded();
    bus.write32(RDRAM_SIZE as u32 + 0x10, 0x1122_3344);
    assert_eq!(bus.read32(RDRAM_SIZE as u32 + 0x10), 0x1122_3344);
}

#[test]
fn the_two_signal_processor_memories_are_separate() {
    let mut bus = new_bus();
    bus.write32(SP_DMEM_BASE, 0xAAAA_AAAA);
    bus.write32(SP_IMEM_BASE, 0xBBBB_BBBB);
    assert_eq!(bus.read32(SP_DMEM_BASE), 0xAAAA_AAAA);
    assert_eq!(bus.read32(SP_IMEM_BASE), 0xBBBB_BBBB);
}

#[test]
fn the_debug_port_reads_back_what_was_written_to_it() {
    let mut bus = new_bus();
    bus.write32(IS_VIEWER_BASE + BUFFER_OFFSET, 0x1234_5678);
    assert_eq!(bus.read32(IS_VIEWER_BASE + BUFFER_OFFSET), 0x1234_5678);
}

#[test]
fn a_length_write_emits_that_many_bytes_of_the_debug_buffer() {
    let mut bus = new_bus();
    let buffer = IS_VIEWER_BASE + BUFFER_OFFSET;
    bus.write32(buffer, 0x5061_7373);
    bus.write32(buffer + 4, 0x6564_0A00);
    bus.write32(IS_VIEWER_BASE + LENGTH_REGISTER_OFFSET, 7);
    assert_eq!(String::from_utf8_lossy(&bus.is_viewer.text), "Passed\n");
}

#[test]
fn nothing_is_emitted_until_the_length_register_is_written() {
    let mut bus = new_bus();
    bus.write32(IS_VIEWER_BASE + BUFFER_OFFSET, 0x5061_7373);
    assert!(bus.is_viewer.text.is_empty());
}

#[test]
fn the_select_register_comes_up_nonzero() {
    let mut bus = new_bus();
    assert_ne!(bus.read32(RI_SELECT), 0);
}

#[test]
fn the_cartridge_is_readable_at_its_domain_and_ignores_writes() {
    let mut bus = bus_with_cart(&build_rom());
    assert_eq!(bus.read32(CART_DOMAIN1_ADDRESS2), MAGIC);
    bus.write32(CART_DOMAIN1_ADDRESS2, 0);
    assert_eq!(bus.read32(CART_DOMAIN1_ADDRESS2), MAGIC);
}

#[test]
fn reading_past_the_end_of_the_cartridge_is_zero() {
    let mut bus = bus_with_cart(&build_rom());
    assert_eq!(bus.read32(CART_DOMAIN1_ADDRESS2 + MINIMUM_LENGTH as u32), 0);
}

#[test]
fn the_count_register_is_derived_from_the_machine_clock_at_half_rate() {
    let mut bus = new_bus();
    assert_eq!(bus.count(), 0);
    bus.tick(2);
    assert_eq!(bus.count(), 1);
    bus.tick(8);
    assert_eq!(bus.count(), 5);
}

#[test]
fn writing_the_count_register_rebases_it_without_stopping_the_clock() {
    let mut bus = new_bus();
    bus.tick(100);
    bus.set_count(0);
    assert_eq!(bus.count(), 0);
    bus.tick(10);
    assert_eq!(bus.count(), 5);
}

// ---- MarsMiTests ----

const MODE: u32 = MI_BASE;

fn arm(bus: &mut MemoryBus, length: u32) {
    bus.write32(MODE, 0x100 | (length - 1));
}

fn filled() -> MemoryBus {
    let mut bus = new_bus();
    for at in (0..0x1000).step_by(4) {
        bus.write32(at, 0xFFFF_FFFF);
    }
    bus
}

#[test]
fn the_mode_register_reads_back_its_length_and_its_three_flags() {
    let mut bus = new_bus();
    bus.write32(MODE, 0x100 | 0x2A);
    assert_eq!(bus.read32(MODE), 0xAA);
    bus.write32(MODE, 0x400 | 0x2000);
    assert_eq!(bus.read32(MODE), 0x380);
    bus.write32(MODE, 0x080 | 0x200 | 0x1000 | 0x05);
    assert_eq!(bus.read32(MODE), 0x05);
}

#[test]
fn a_write_carrying_a_flags_set_and_clear_bits_sets_it() {
    let mut bus = new_bus();
    bus.write32(MODE, 0x180 | 0x600 | 0x3000);
    assert_eq!(bus.read32(MODE), 0x380);
}

#[test]
fn the_mode_register_still_clears_the_display_processors_interrupt() {
    let mut bus = new_bus();
    bus.mi.raise(interrupt::DISPLAY_PROCESSOR);
    bus.write32(MODE, 0x800);
    assert_eq!(bus.mi.pending, 0);
    assert_eq!(bus.read32(MODE), 0);
}

#[test]
fn the_version_register_reads_the_rcps_revision() {
    assert_eq!(new_bus().read32(MI_BASE + 4), 0x0202_0102);
}

#[test]
fn a_word_store_is_repeated_across_the_length() {
    let mut bus = filled();
    arm(&mut bus, 12);
    bus.store(0x100, 0x1234_5678_9ABC_DEF1, 4);
    assert_eq!(bus.read32(0x100), 0x9ABC_DEF1);
    assert_eq!(bus.read32(0x104), 0x9ABC_DEF1);
    assert_eq!(bus.read32(0x108), 0x9ABC_DEF1);
    assert_eq!(bus.read32(0x10C), 0xFFFF_FFFF);
}

#[test]
fn the_length_counts_from_the_stores_doubleword() {
    let mut bus = filled();
    arm(&mut bus, 12);
    bus.store(0x104, 0x9ABC_DEF1, 4);
    assert_eq!(bus.read32(0x100), 0xFFFF_FFFF);
    assert_eq!(bus.read32(0x104), 0x9ABC_DEF1);
    assert_eq!(bus.read32(0x108), 0x9ABC_DEF1);
    assert_eq!(bus.read32(0x10C), 0xFFFF_FFFF);
}

#[test]
fn a_store_whose_misalignment_uses_up_the_length_writes_nothing() {
    let mut bus = filled();
    arm(&mut bus, 1);
    bus.store(0x101, 0xF1, 1);
    assert_eq!(bus.read32(0x100), 0xFFFF_FFFF);
    assert!(!bus.mi.repeating);
}

#[test]
fn a_byte_store_repeats_its_whole_bus_word_not_its_byte() {
    let mut bus = filled();
    arm(&mut bus, 6);
    bus.store(0x101, 0x9ABC_DEF1, 1);
    assert_eq!(bus.read32(0x100), 0xFFF1_0000);
    assert_eq!(bus.read32(0x104), 0xDEF1_FFFF);
}

#[test]
fn a_halfword_store_repeats_its_whole_bus_word() {
    let mut bus = filled();
    arm(&mut bus, 8);
    bus.store(0x102, 0x9ABC_DEF1, 2);
    assert_eq!(bus.read32(0x100), 0xFFFF_DEF1);
    assert_eq!(bus.read32(0x104), 0x9ABC_DEF1);
}

#[test]
fn a_doubleword_store_repeats_all_eight_of_its_bytes() {
    let mut bus = filled();
    arm(&mut bus, 12);
    bus.store(0x200, 0x1234_5678_9ABC_DEF1, 8);
    assert_eq!(bus.read64(0x200), 0x1234_5678_9ABC_DEF1);
    assert_eq!(bus.read32(0x208), 0x1234_5678);
    assert_eq!(bus.read32(0x20C), 0xFFFF_FFFF);
}

#[test]
fn the_repeat_wraps_inside_its_two_kilobyte_row() {
    let mut bus = filled();
    arm(&mut bus, 128);
    bus.store(0x7F8, 0, 8);
    assert_eq!(bus.read64(0x7F8), 0);
    assert_eq!(bus.read64(0x000), 0);
    assert_eq!(bus.read64(0x070), 0);
    assert_eq!(bus.read64(0x078), 0xFFFF_FFFF_FFFF_FFFF);
    assert_eq!(bus.read64(0x800), 0xFFFF_FFFF_FFFF_FFFF);
}

#[test]
fn one_store_spends_the_repeat() {
    let mut bus = filled();
    arm(&mut bus, 16);
    bus.store(0x100, 0x1111_1111, 4);
    bus.store(0x300, 0x2222_2222, 4);
    assert_eq!(bus.read32(0x10C), 0x1111_1111);
    assert_eq!(bus.read32(0x300), 0x2222_2222);
    assert_eq!(bus.read32(0x304), 0xFFFF_FFFF);
    assert_eq!(bus.read32(MODE), 0x0F);
}

#[test]
fn a_store_to_the_rdram_registers_spends_the_repeat_without_repeating() {
    let mut bus = filled();
    arm(&mut bus, 16);
    bus.store(RDRAM_REGISTERS_BASE + 4, 0x1111_1111, 4);
    bus.store(0x100, 0x2222_2222, 4);
    assert_eq!(bus.read32(0x100), 0x2222_2222);
    assert_eq!(bus.read32(0x104), 0xFFFF_FFFF);
}

#[test]
fn a_store_outside_rdram_leaves_the_repeat_armed() {
    let mut bus = filled();
    arm(&mut bus, 8);
    bus.store(SP_DMEM_BASE, 0x3333_3333, 4);
    bus.store(0x100, 0x2222_2222, 4);
    assert_eq!(bus.read32(SP_DMEM_BASE), 0x3333_3333);
    assert_eq!(bus.read32(0x104), 0x2222_2222);
}

#[test]
fn a_transfer_into_rdram_neither_repeats_nor_spends_the_repeat() {
    let mut bus = filled();
    arm(&mut bus, 8);
    bus.write32(0x100, 0x4444_4444);
    assert_eq!(bus.read32(0x104), 0xFFFF_FFFF);
    assert!(bus.mi.repeating);
}

#[test]
fn the_rdram_registers_read_what_the_corpus_measured_every_64_bytes() {
    let rows: [(u32, u32); 10] = [
        (0x00, 0xB419_0010),
        (0x04, 0),
        (0x08, 0x2B3B_1A0B),
        (0x0C, 0),
        (0x18, 0x101C_0A04),
        (0x24, 0),
        (0x3C, 0),
        (0x40, 0xB419_0010),
        (0x1C8, 0x2B3B_1A0B),
        (0x1D8, 0x101C_0A04),
    ];
    for (offset, expected) in rows {
        assert_eq!(new_bus().read32(RDRAM_REGISTERS_BASE + offset), expected, "offset {offset:X}");
    }
}

#[test]
fn a_doubleword_load_of_the_first_rdram_register_carries_the_second() {
    assert_eq!(new_bus().load(RDRAM_REGISTERS_BASE, 8), 0xB419_0010_0000_0000);
}

#[test]
fn a_write_to_the_rdram_registers_is_not_kept() {
    let mut bus = new_bus();
    bus.write32(RDRAM_REGISTERS_BASE, 0x1234_5678);
    bus.write32(RDRAM_REGISTERS_BASE + 4, 0x1234_5678);
    assert_eq!(bus.read32(RDRAM_REGISTERS_BASE), 0xB419_0010);
    assert_eq!(bus.read32(RDRAM_REGISTERS_BASE + 4), 0);
}

#[test]
fn a_store_from_the_processor_is_repeated_too() {
    let mut bus = filled();
    arm(&mut bus, 12);
    let mut rig = Asm::new().lui(1, 0x8000).ori(1, 1, 0x100).lui(2, 0x9ABC).ori(2, 2, 0xDEF1).sw(2, 1, 0).build(bus);
    rig.run(5);
    assert_eq!(rig.bus.read32(0x100), 0x9ABC_DEF1);
    assert_eq!(rig.bus.read32(0x108), 0x9ABC_DEF1);
    assert_eq!(rig.bus.read32(0x10C), 0xFFFF_FFFF);
    assert!(!rig.bus.mi.repeating);
}

// ---- MarsCartridgeTests ----

const DATA: u32 = CART_DOMAIN1_ADDRESS2 + HEADER_LENGTH as u32;
const ROM: u32 = CART_DOMAIN1_ADDRESS2;
const PI_STATUS: u32 = PI_BASE + 0x10;
const STATUS_IO_BUSY: u32 = 0x02;

const BYTES: [u8; 24] =
    [0x01, 0x23, 0x45, 0x67, 0x89, 0xAB, 0xCD, 0xEF, 0x21, 0x43, 0x65, 0x87, 0x99, 0xBA, 0xDC, 0xFE, 0xA9, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22];

fn cartridge() -> MemoryBus {
    bus_with_cart(&SyntheticRom::patched(&BYTES).build())
}

fn busy(bus: &mut MemoryBus) -> bool {
    bus.read32(PI_STATUS) & STATUS_IO_BUSY != 0
}

#[test]
fn a_word_read_reaches_every_word() {
    let mut bus = cartridge();
    assert_eq!(bus.load(DATA, 4), 0x0123_4567);
    assert_eq!(bus.load(DATA + 4, 4), 0x89AB_CDEF);
    assert_eq!(bus.load(DATA + 8, 4), 0x2143_6587);
}

#[test]
fn a_halfword_read_cannot_reach_every_other_halfword() {
    let mut bus = cartridge();
    let expected: [u64; 8] = [0x0123, 0x89AB, 0x89AB, 0x2143, 0x2143, 0x99BA, 0x99BA, 0xA988];
    for (i, &halfword) in expected.iter().enumerate() {
        assert_eq!(bus.load(DATA + 2 * i as u32, 2), halfword, "halfword {i}");
    }
}

#[test]
fn a_byte_read_cannot_reach_every_other_halfword() {
    let mut bus = cartridge();
    let expected: [u64; 16] = [0x01, 0x23, 0x89, 0xAB, 0x89, 0xAB, 0x21, 0x43, 0x21, 0x43, 0x99, 0xBA, 0x99, 0xBA, 0xA9, 0x88];
    for (i, &byte) in expected.iter().enumerate() {
        assert_eq!(bus.load(DATA + i as u32, 1), byte, "byte {i}");
    }
}

#[test]
fn a_store_is_what_the_next_read_returns_once() {
    for offset in [0u32, 4] {
        let mut bus = cartridge();
        bus.store(DATA + offset, 0xBADC_0FFE, 4);
        bus.store(DATA, 0xDECAF, 4);
        assert_eq!(bus.load(DATA, 4), 0xBADC_0FFE, "offset {offset}");
        assert_eq!(bus.load(DATA, 4), 0x0123_4567, "offset {offset}");
        assert_eq!(bus.load(DATA, 4), 0x0123_4567, "offset {offset}");
    }
}

#[test]
fn the_window_ends_where_the_pif_begins() {
    for (physical, first) in [(0x1000_0000u32, 0xBADC_0FFEu64), (0x1FBF_FFFC, 0xBADC_0FFE), (0x1FC0_0000, 0x000D_ECAF)] {
        let mut bus = cartridge();
        bus.store(physical, 0xBADC_0FFE, 4);
        bus.store(DATA, 0xDECAF, 4);
        assert_eq!(bus.load(DATA, 4), first, "{physical:08X}");
        assert_eq!(bus.load(DATA, 4), 0x0123_4567, "{physical:08X}");
    }
}

#[test]
fn a_store_of_any_size_is_latched_as_a_whole_word() {
    let rows: [(u32, u64, u32, u64); 4] =
        [(0, 0xBA, 1, 0xBA00_0000), (1, 0x1234_56BA, 1, 0x56BA_0000), (0, 0xBADC, 2, 0xBADC_0000), (0, 0x9876_5432_1AF1_231A, 8, 0x9876_5432)];
    for (offset, value, size, word) in rows {
        let mut bus = cartridge();
        bus.store(ROM + offset, value, size);
        bus.store(DATA, 0x0101_0101_2323_2323, 8);
        assert_eq!(bus.load(DATA, 4), word, "{value:X} at {offset}, size {size}");
    }
}

#[test]
fn a_narrow_read_of_the_stored_word_starts_at_its_top() {
    for (size, expected) in [(1u32, 0xBAu64), (2, 0xBADC)] {
        let mut bus = cartridge();
        bus.store(ROM, 0xBADC_0FFE, 4);
        assert_eq!(bus.load(DATA, size), expected, "size {size}");
    }
}

#[test]
fn a_narrow_read_of_the_stored_word_takes_the_lane_it_names() {
    for (offset, size, expected) in [(1u32, 1u32, 0xDCu64), (3, 1, 0xFE), (2, 2, 0x0FFE)] {
        let mut bus = cartridge();
        bus.store(ROM, 0xBADC_0FFE, 4);
        assert_eq!(bus.load(DATA + offset, size), expected, "offset {offset}, size {size}");
    }
}

#[test]
fn the_stored_word_decays() {
    let mut bus = cartridge();
    bus.store(DATA, 0xBADC_0FFE, 4);
    bus.tick(STORE_DECAY_CYCLES - 1);
    assert_eq!(bus.load(DATA, 4), 0xBADC_0FFE);

    bus.store(DATA, 0xBADC_0FFE, 4);
    bus.tick(STORE_DECAY_CYCLES);
    assert_eq!(bus.load(DATA, 4), 0x0123_4567);
}

/// The decay by its own number, the FPGA core's 150 cycles at 62.5MHz counted in the processor's, not by the constant.
#[test]
fn the_stored_word_lasts_225_cycles() {
    let mut bus = cartridge();
    bus.store(DATA, 0xBADC_0FFE, 4);
    bus.tick(224);
    assert_eq!(bus.load(DATA, 4), 0xBADC_0FFE);

    bus.store(DATA, 0xBADC_0FFE, 4);
    bus.tick(225);
    assert_eq!(bus.load(DATA, 4), 0x0123_4567);
}

/// `MiInterface.Write32` clears before it sets, so a write carrying both bits of a device leaves it unmasked.
#[test]
fn a_mask_write_carrying_both_bits_of_a_device_sets_it() {
    let mut bus = new_bus();
    for source in 0..6u32 {
        bus.mi.write32(0x0C, 0x0FFF);
        assert_eq!(bus.mi.mask, 0x3F, "every device's pair written at once");
        bus.mi.write32(0x0C, 1 << (source * 2));
        assert_eq!(bus.mi.mask, 0x3F & !(1 << source));
        bus.mi.write32(0x0C, 3 << (source * 2));
        assert_eq!(bus.mi.mask, 0x3F, "device {source}'s pair written together");
    }
}

#[test]
fn the_decay_falls_inside_the_window_the_corpus_allows() {
    let mut bus = cartridge();
    bus.store(DATA, 0xBADC_0FFE, 4);
    bus.tick(33);
    assert_eq!(bus.load(DATA, 4), 0xBADC_0FFE);

    bus.store(DATA, 0xBADC_0FFE, 4);
    bus.tick(333);
    assert_eq!(bus.load(DATA, 4), 0x0123_4567);
}

#[test]
fn the_bus_is_busy_until_a_read_or_the_decay_frees_it() {
    let mut bus = cartridge();
    assert!(!busy(&mut bus));

    bus.store(DATA, 0x0101_0101_2323_2323, 8);
    assert!(busy(&mut bus));
    bus.load(DATA, 4);
    assert!(!busy(&mut bus));

    bus.store(DATA, 0xBADC_0FFE, 4);
    bus.tick(STORE_DECAY_CYCLES);
    assert!(!busy(&mut bus));
}

#[test]
fn a_transfer_neither_sees_nor_frees_the_stored_word() {
    let mut bus = cartridge();
    bus.store(DATA, 0xBADC_0FFE, 4);
    bus.write32(PI_BASE, 0x1000);
    bus.write32(PI_BASE + 0x04, DATA);
    bus.write32(PI_BASE + 0x0C, 8 - 1);
    assert_eq!(bus.read32(0x1000), 0x0123_4567);
    assert!(busy(&mut bus));
    assert_eq!(bus.load(DATA, 4), 0xBADC_0FFE);
}

#[test]
fn the_debug_port_is_outside_the_latch() {
    let mut bus = cartridge();
    bus.store(IS_VIEWER_BASE + 0x20, 0x1234_5678, 4);
    assert!(!busy(&mut bus));
    assert_eq!(bus.load(DATA, 4), 0x0123_4567);
    assert_eq!(bus.load(IS_VIEWER_BASE + 0x20, 4), 0x1234_5678);
}
