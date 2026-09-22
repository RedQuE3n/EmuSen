//! Translation: `MarsTlbTests` and `MarsSegmentMapTests`.

use super::support::{Asm, Rig, new_bus};
use crate::bus::MemoryBus;
use crate::cop0::{CONTEXT, ENTRY_HI, ENTRY_LO0, ENTRY_LO1, INDEX, PAGE_MASK, STATUS, VECTOR_BASE, WIRED};
use crate::cpu::code;
use crate::tlb::{ENTRY_LO_DIRTY, ENTRY_LO_GLOBAL, ENTRY_LO_VALID, TlbEntry};

const MAPPED_PAGE: u64 = 0x0000_0000_0010_0000;
const FRAME: u32 = 0x2000;
const USABLE: u64 = ENTRY_LO_VALID | ENTRY_LO_DIRTY | ENTRY_LO_GLOBAL;

// ---- MarsTlbTests ----

fn machine(bus: MemoryBus) -> Rig {
    let mut rig = Asm::new().nop().run(0, bus);
    rig.at_entry();
    rig
}

/// One 4K pair mapping the test page.
fn map_test_page(rig: &mut Rig, valid: bool, writable: bool, global: bool, asid: u64) {
    let flags = (if valid { ENTRY_LO_VALID } else { 0 }) | (if writable { ENTRY_LO_DIRTY } else { 0 }) | if global { ENTRY_LO_GLOBAL } else { 0 };
    rig.cpu.tlb.entries[4] = TlbEntry {
        entry_hi: MAPPED_PAGE | asid,
        page_mask: 0,
        entry_lo0: (((FRAME >> 12) as u64) << 6) | flags,
        entry_lo1: ((((FRAME + 0x1000) >> 12) as u64) << 6) | flags,
    };
}

fn map_usable_test_page(rig: &mut Rig) {
    map_test_page(rig, true, true, true, 0);
}

fn map_pair(rig: &mut Rig, index: usize, page: u64, even: u32, odd: u32) {
    rig.cpu.tlb.entries[index] =
        TlbEntry { entry_hi: page, page_mask: 0, entry_lo0: (((even >> 12) as u64) << 6) | USABLE, entry_lo1: (((odd >> 12) as u64) << 6) | USABLE };
}

#[test]
fn the_two_halves_of_a_pair_are_four_kilobytes_apart_and_independent() {
    let mut bus = new_bus();
    bus.write32(0x5000 + 0x20, 0x0DD0_0DD0);
    bus.write32(0x9000 + 0x20, 0xBADF_00D5);
    let mut rig = machine(bus);
    map_pair(&mut rig, 0, MAPPED_PAGE, 0x9000, 0x5000);
    Asm::new().lui(1, 0x0010).lw(2, 1, 0x1020).load_into(&mut rig.bus);
    rig.run(2);
    assert_eq!(rig.last_code(), None);
    assert_eq!(rig.cpu.gpr[2], 0x0000_0000_0DD0_0DD0);
}

#[test]
fn a_pair_does_not_reach_into_the_pair_above_it() {
    let mut bus = new_bus();
    bus.write32(0x7000 + 0x20, 0xFEED_FACE);
    let mut rig = machine(bus);
    map_pair(&mut rig, 0, MAPPED_PAGE, 0x3000, 0x4000);
    map_pair(&mut rig, 1, MAPPED_PAGE + 0x2000, 0x7000, 0x8000);
    Asm::new().lui(1, 0x0010).lw(2, 1, 0x2020).load_into(&mut rig.bus);
    rig.run(2);
    assert_eq!(rig.last_code(), None);
    assert_eq!(rig.cpu.gpr[2], 0xFFFF_FFFF_FEED_FACE);
}

#[test]
fn a_mapped_page_translates_and_the_load_reaches_memory() {
    let mut bus = new_bus();
    bus.write32(FRAME + 0x20, 0xCAFE_BABE);
    let mut rig = machine(bus);
    map_usable_test_page(&mut rig);
    Asm::new().lui(1, 0x0010).lw(2, 1, 0x20).load_into(&mut rig.bus);
    rig.run(2);
    assert_eq!(rig.last_code(), None);
    assert_eq!(rig.cpu.gpr[2], 0xFFFF_FFFF_CAFE_BABE);
}

#[test]
fn the_second_half_of_the_pair_is_a_page_of_its_own() {
    let mut bus = new_bus();
    bus.write32(FRAME + 0x1000 + 0x20, 0x1234_5678);
    let mut rig = machine(bus);
    map_usable_test_page(&mut rig);
    Asm::new().lui(1, 0x0010).ori(1, 1, 0x1000).lw(2, 1, 0x20).load_into(&mut rig.bus);
    rig.run(3);
    assert_eq!(rig.cpu.gpr[2], 0x1234_5678);
}

#[test]
fn an_address_with_no_entry_at_all_takes_the_refill_vector() {
    let mut rig = machine(new_bus());
    Asm::new().lui(1, 0x0010).lw(2, 1, 0).load_into(&mut rig.bus);
    rig.run(2);
    let fault = rig.last().unwrap();
    assert_eq!(fault.code, code::TLB_LOAD);
    assert!(fault.refill);
    assert_eq!(rig.cpu.pc, VECTOR_BASE);
}

#[test]
fn an_entry_that_exists_but_is_invalid_takes_the_general_vector_instead() {
    let mut rig = machine(new_bus());
    map_test_page(&mut rig, false, true, true, 0);
    Asm::new().lui(1, 0x0010).lw(2, 1, 0).load_into(&mut rig.bus);
    rig.run(2);
    let fault = rig.last().unwrap();
    assert_eq!(fault.code, code::TLB_LOAD);
    assert!(!fault.refill);
    assert_eq!(rig.cpu.pc, VECTOR_BASE + 0x180);
}

#[test]
fn a_store_to_a_page_that_is_not_writable_is_its_own_exception() {
    let mut rig = machine(new_bus());
    map_test_page(&mut rig, true, false, true, 0);
    Asm::new().lui(1, 0x0010).addiu(2, 0, 5).sw(2, 1, 0).load_into(&mut rig.bus);
    rig.run(3);
    let fault = rig.last().unwrap();
    assert_eq!(fault.code, code::TLB_MODIFICATION);
    assert!(!fault.refill);
}

#[test]
fn a_load_from_the_same_page_is_still_allowed() {
    let mut bus = new_bus();
    bus.write32(FRAME, 0x99);
    let mut rig = machine(bus);
    map_test_page(&mut rig, true, false, true, 0);
    Asm::new().lui(1, 0x0010).lw(2, 1, 0).load_into(&mut rig.bus);
    rig.run(2);
    assert_eq!(rig.last_code(), None);
    assert_eq!(rig.cpu.gpr[2], 0x99);
}

#[test]
fn a_non_global_entry_only_matches_its_own_address_space() {
    let mut rig = machine(new_bus());
    map_test_page(&mut rig, true, true, false, 0x11);
    rig.cpu.cop0[ENTRY_HI] = 0x22;
    Asm::new().lui(1, 0x0010).lw(2, 1, 0).load_into(&mut rig.bus);
    rig.run(2);
    assert!(rig.last().unwrap().refill);
}

#[test]
fn the_same_entry_matches_once_the_address_space_agrees() {
    let mut bus = new_bus();
    bus.write32(FRAME, 0x77);
    let mut rig = machine(bus);
    map_test_page(&mut rig, true, true, false, 0x11);
    rig.cpu.cop0[ENTRY_HI] = 0x11;
    Asm::new().lui(1, 0x0010).lw(2, 1, 0).load_into(&mut rig.bus);
    rig.run(2);
    assert_eq!(rig.last_code(), None);
    assert_eq!(rig.cpu.gpr[2], 0x77);
}

#[test]
fn a_miss_leaves_the_faulting_page_where_a_handler_expects_it() {
    let mut rig = machine(new_bus());
    Asm::new().lui(1, 0x0010).ori(1, 1, 0x0040).lw(2, 1, 0).load_into(&mut rig.bus);
    rig.run(3);
    assert_eq!(rig.cpu.cop0[ENTRY_HI] & 0xFFFF_E000, MAPPED_PAGE);
    assert_eq!(rig.cpu.cop0[CONTEXT] & 0x007F_FFF0, (MAPPED_PAGE >> 9) & 0x007F_FFF0);
}

#[test]
fn an_entry_written_by_a_program_is_the_one_the_lookup_uses() {
    let mut bus = new_bus();
    bus.write32(FRAME, 0x4242);
    let mut rig = machine(bus);
    rig.cpu.cop0[INDEX] = 7;
    rig.cpu.cop0[ENTRY_HI] = MAPPED_PAGE;
    rig.cpu.cop0[PAGE_MASK] = 0;
    rig.cpu.cop0[ENTRY_LO0] = (((FRAME >> 12) as u64) << 6) | ENTRY_LO_VALID | ENTRY_LO_GLOBAL;
    rig.cpu.cop0[ENTRY_LO1] = ENTRY_LO_GLOBAL;
    Asm::new().tlbwi().lui(1, 0x0010).lw(2, 1, 0).load_into(&mut rig.bus);
    rig.run(3);
    assert_eq!(rig.last_code(), None);
    assert_eq!(rig.cpu.gpr[2], 0x4242);
}

#[test]
fn a_probe_finds_a_written_entry_and_says_so_when_it_cannot() {
    let mut rig = machine(new_bus());
    map_usable_test_page(&mut rig);
    rig.cpu.cop0[ENTRY_HI] = MAPPED_PAGE;
    Asm::new().tlbp().load_into(&mut rig.bus);
    rig.step();
    assert_eq!(rig.cpu.cop0[INDEX], 4);

    let mut second = machine(new_bus());
    second.cpu.cop0[ENTRY_HI] = MAPPED_PAGE;
    Asm::new().tlbp().load_into(&mut second.bus);
    second.step();
    assert_eq!(second.cpu.cop0[INDEX], 0x8000_0000);
}

#[test]
fn an_entry_can_be_read_back_out_again() {
    let mut rig = machine(new_bus());
    map_usable_test_page(&mut rig);
    rig.cpu.cop0[INDEX] = 4;
    Asm::new().tlbr().load_into(&mut rig.bus);
    rig.step();
    assert_eq!(rig.cpu.cop0[ENTRY_HI], MAPPED_PAGE);
    assert_eq!(rig.cpu.cop0[ENTRY_LO0] & 0x7, USABLE);
}

#[test]
fn a_random_write_never_lands_on_a_wired_entry() {
    let mut rig = machine(new_bus());
    rig.cpu.cop0[WIRED] = 30;
    rig.cpu.cop0[ENTRY_HI] = MAPPED_PAGE;
    rig.cpu.cop0[ENTRY_LO0] = ENTRY_LO_VALID | ENTRY_LO_GLOBAL;
    let mut program = Asm::new();
    for _ in 0..8 {
        program = program.tlbwr();
    }
    program.load_into(&mut rig.bus);
    rig.run(8);
    for (i, entry) in rig.cpu.tlb.entries[..30].iter().enumerate() {
        assert_eq!(entry.entry_hi, 0, "entry {i}");
    }
    assert!(rig.cpu.tlb.entries[30].entry_hi == MAPPED_PAGE || rig.cpu.tlb.entries[31].entry_hi == MAPPED_PAGE);
}

// ---- MarsSegmentMapTests, through PrivilegeFixture ----

const KERNEL: u64 = 0;
const SUPERVISOR: u64 = 1;
const USER: u64 = 2;
const PROGRAM_PAGE: u64 = 0x0000_0000_0001_0000;
const EXTENDED_ADDRESSING: u64 = (1 << 5) | (1 << 6) | (1 << 7);

/// `PrivilegeFixture.Run`: one instruction from a page every mode may execute.
fn privileged(mode: u64, wide: bool, address: u64, program: Asm, bus: MemoryBus) -> Rig {
    let mut rig = program.build(bus);
    rig.cpu.tlb.entries[0] = TlbEntry { entry_hi: PROGRAM_PAGE, page_mask: 0, entry_lo0: USABLE, entry_lo1: (1 << 6) | USABLE };
    rig.cpu.gpr[1] = address;
    rig.cpu.cop0[STATUS] = (mode << 3) | if wide { EXTENDED_ADDRESSING } else { 0 };
    rig.cpu.pc = PROGRAM_PAGE;
    rig.cpu.next_pc = PROGRAM_PAGE + 4;
    rig.cop0_written();
    rig.run(1);
    rig
}

fn describe(rig: &Rig) -> String {
    match rig.last_code() {
        None => "Sys".into(),
        Some(code::TLB_LOAD) => "TLBL".into(),
        Some(code::ADDRESS_ERROR_LOAD) => "AdEL".into(),
        Some(other) => other.to_string(),
    }
}

#[test]
fn a_load_lands_where_the_mode_says_it_can() {
    let rows: [(&str, u64, bool, u64, &str); 48] = [
        ("k32_kuseg_map", KERNEL, false, 0x0000_0000_0000_1000, "TLBL"),
        ("k32_kseg0_c", KERNEL, false, 0xFFFF_FFFF_8000_1000, "Sys"),
        ("k32_kseg1_d", KERNEL, false, 0xFFFF_FFFF_A000_1000, "Sys"),
        ("k32_ksseg_map", KERNEL, false, 0xFFFF_FFFF_C000_1000, "TLBL"),
        ("k32_kseg3_map", KERNEL, false, 0xFFFF_FFFF_E000_1000, "TLBL"),
        ("s32_suseg_map", SUPERVISOR, false, 0x0000_0000_0000_1000, "TLBL"),
        ("s32_low_unused", SUPERVISOR, false, 0xFFFF_FFFF_9000_1000, "AdEL"),
        ("s32_sseg_map", SUPERVISOR, false, 0xFFFF_FFFF_C000_1000, "TLBL"),
        ("s32_top_unused", SUPERVISOR, false, 0xFFFF_FFFF_E000_1000, "AdEL"),
        ("u32_useg_map", USER, false, 0x0000_0000_0000_1000, "TLBL"),
        ("u32_unused", USER, false, 0xFFFF_FFFF_9000_1000, "AdEL"),
        ("k64_xkuseg_map", KERNEL, true, 0x0000_0000_0000_1000, "TLBL"),
        ("k64_xkuseg_gap", KERNEL, true, 0x0000_0100_0000_0000, "AdEL"),
        ("k64_xksseg_map", KERNEL, true, 0x4000_0000_0000_1000, "TLBL"),
        ("k64_xksseg_gap", KERNEL, true, 0x4000_0100_0000_0000, "AdEL"),
        ("k64_xkphys0_c32", KERNEL, true, 0x8000_0000_0000_1000, "Sys"),
        ("k64_xkphys0_gap", KERNEL, true, 0x8000_0001_0000_0000, "AdEL"),
        ("k64_xkphys1_c32", KERNEL, true, 0x8800_0000_0000_1000, "Sys"),
        ("k64_xkphys1_gap", KERNEL, true, 0x8800_0001_0000_0000, "AdEL"),
        ("k64_xkphys2_d32", KERNEL, true, 0x9000_0000_0000_1000, "Sys"),
        ("k64_xkphys2_gap", KERNEL, true, 0x9000_0001_0000_0000, "AdEL"),
        ("k64_xkphys3_c32", KERNEL, true, 0x9800_0000_0000_1000, "Sys"),
        ("k64_xkphys3_gap", KERNEL, true, 0x9800_0001_0000_0000, "AdEL"),
        ("k64_xkphys4_c32", KERNEL, true, 0xA000_0000_0000_1000, "Sys"),
        ("k64_xkphys4_gap", KERNEL, true, 0xA000_0001_0000_0000, "AdEL"),
        ("k64_xkphys5_c32", KERNEL, true, 0xA800_0000_0000_1000, "Sys"),
        ("k64_xkphys5_gap", KERNEL, true, 0xA800_0001_0000_0000, "AdEL"),
        ("k64_xkphys6_c32", KERNEL, true, 0xB000_0000_0000_1000, "Sys"),
        ("k64_xkphys6_gap", KERNEL, true, 0xB000_0001_0000_0000, "AdEL"),
        ("k64_xkphys7_c32", KERNEL, true, 0xB800_0000_0000_1000, "Sys"),
        ("k64_xkphys7_gap", KERNEL, true, 0xB800_0001_0000_0000, "AdEL"),
        ("k64_xkseg_map", KERNEL, true, 0xC000_0000_0000_1000, "TLBL"),
        ("k64_xkseg_gap", KERNEL, true, 0xC000_0100_0000_0000, "AdEL"),
        ("k64_xkseg_top", KERNEL, true, 0xC000_00FF_7FFF_FFFC, "TLBL"),
        ("k64_xkseg_short", KERNEL, true, 0xC000_00FF_8000_0000, "AdEL"),
        ("k64_xkseg_short_high", KERNEL, true, 0xC000_00FF_F000_0000, "AdEL"),
        ("k64_ckseg0_c", KERNEL, true, 0xFFFF_FFFF_8000_1000, "Sys"),
        ("k64_ckseg1_d", KERNEL, true, 0xFFFF_FFFF_A000_1000, "Sys"),
        ("k64_ckseg2_map", KERNEL, true, 0xFFFF_FFFF_C000_1000, "TLBL"),
        ("k64_ckseg3_map", KERNEL, true, 0xFFFF_FFFF_E000_1000, "TLBL"),
        ("s64_xsuseg_map", SUPERVISOR, true, 0x0000_0000_0000_1000, "TLBL"),
        ("s64_xsuseg_gap", SUPERVISOR, true, 0x0000_0100_0000_0000, "AdEL"),
        ("s64_xsseg_map", SUPERVISOR, true, 0x4000_0000_0000_1000, "TLBL"),
        ("s64_xsseg_gap", SUPERVISOR, true, 0x4000_0100_0000_0000, "AdEL"),
        ("s64_csseg_map", SUPERVISOR, true, 0xFFFF_FFFF_C000_1000, "TLBL"),
        ("s64_top_unused", SUPERVISOR, true, 0xFFFF_FFFF_E000_1000, "AdEL"),
        ("u64_xuseg_map", USER, true, 0x0000_0000_0000_1000, "TLBL"),
        ("u64_xuseg_gap", USER, true, 0x0000_0100_0000_0000, "AdEL"),
    ];
    for (name, mode, wide, address, outcome) in rows {
        let rig = privileged(mode, wide, address, Asm::new().lw(2, 1, 0), new_bus());
        assert_eq!(format!("{name}: {}", describe(&rig)), format!("{name}: {outcome}"));
    }
}

#[test]
fn a_store_outside_the_mode_is_an_address_error_of_its_own_kind() {
    for (mode, wide, address) in
        [(USER, false, 0xFFFF_FFFF_9000_1000u64), (SUPERVISOR, false, 0xFFFF_FFFF_E000_1000), (KERNEL, true, 0x0000_0100_0000_0000)]
    {
        let rig = privileged(mode, wide, address, Asm::new().sw(2, 1, 0), new_bus());
        assert_eq!(rig.last_code(), Some(code::ADDRESS_ERROR_STORE), "{address:016X}");
    }
}

#[test]
fn every_direct_segment_strips_to_the_same_physical_address() {
    for address in [0xFFFF_FFFF_8000_1000u64, 0xFFFF_FFFF_A000_1000, 0x8000_0000_0000_1000, 0x9000_0000_0000_1000, 0xB800_0000_0000_1000] {
        let mut bus = new_bus();
        bus.write32(0x1000, 0xC0FF_EE00);
        let rig = privileged(KERNEL, true, address, Asm::new().lw(2, 1, 0), bus);
        assert_eq!(rig.last_code(), None, "{address:016X}");
        assert_eq!(rig.cpu.gpr[2], 0xFFFF_FFFF_C0FF_EE00, "{address:016X}");
    }
}
