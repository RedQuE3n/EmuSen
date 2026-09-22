//! The integer core: `MarsCpuArithmeticTests`, `MarsCpuTrapTests`, `MarsCpuUnalignedTests`, `MarsCpuExceptionTests`, `MarsCpuInterruptTests`.

use super::support::{Asm, ENTRY_POINT, Rig, build_rom, bus_with_cart, new_bus};
use crate::memory::bus::MemoryBus;
use crate::memory::bus_access::map::{MI_BASE, PI_BASE};
use crate::cpu::cop0::{
    BAD_VIRTUAL_ADDRESS, CAUSE, CAUSE_BRANCH_DELAY, CAUSE_INTERRUPT_RCP, CAUSE_INTERRUPT_TIMER, COMPARE, EXCEPTION_PC, STATUS,
    STATUS_BOOTSTRAP_VECTORS, STATUS_EXCEPTION_LEVEL, STATUS_INTERRUPT_ENABLE, VECTOR_BASE, VECTOR_BASE_BOOTSTRAP,
};
use crate::cpu::code;
use crate::memory::mi::interrupt;

const GENERAL_VECTOR: u64 = VECTOR_BASE + 0x180;
const REFILL_VECTOR: u64 = VECTOR_BASE;
const KSEG0: u64 = 0xFFFF_FFFF_8000_0000;

// ---- MarsCpuArithmeticTests ----

fn prepared(registers: impl FnOnce(&mut [u64; 32]), program: Asm) -> Rig {
    let mut rig = program.build_new();
    registers(&mut rig.cpu.gpr);
    rig.run(1);
    rig
}

const SHIFTS: [(u64, u32, u64); 4] = [
    (0x0000_0000_1234_5678, 4, 0x0000_0000_0123_4567),
    (0x0000_0000_8234_5678, 0, 0xFFFF_FFFF_8234_5678),
    (0x0123_4567_89AB_CDEF, 4, 0x0000_0000_789A_BCDE),
    (0x0000_0008_789A_BCDE, 4, 0xFFFF_FFFF_8789_ABCD),
];

#[test]
fn an_arithmetic_right_shift_shifts_the_whole_register_before_truncating() {
    for (source, amount, expected) in SHIFTS {
        let rig = prepared(|g| g[2] = source, Asm::new().sra(3, 2, amount));
        assert_eq!(rig.cpu.gpr[3], expected, "{source:016X} >> {amount}");
    }
}

#[test]
fn a_variable_arithmetic_right_shift_reads_the_same_width() {
    for (source, amount, expected) in SHIFTS {
        let rig = prepared(
            |g| {
                g[2] = source;
                g[4] = amount as u64;
            },
            Asm::new().srav(3, 2, 4),
        );
        assert_eq!(rig.cpu.gpr[3], expected, "{source:016X} >> {amount}");
    }
}

#[test]
fn a_variable_shift_takes_only_five_bits_of_its_count() {
    let rig = prepared(
        |g| {
            g[2] = 0x0123_4567_89AB_CDEF;
            g[4] = 0xFFFF_FFFF_FFFF_FFE4;
        },
        Asm::new().srav(3, 2, 4),
    );
    assert_eq!(rig.cpu.gpr[3], 0x0000_0000_789A_BCDE);
}

#[test]
fn a_signed_multiply_reads_thirty_five_bits_of_its_second_operand() {
    let rows: [(u64, u64, u64, u64); 27] = [
        (0x0000_0000_0000_0000, 0x0000_0000_0000_0000, 0x0000_0000_0000_0000, 0x0000_0000_0000_0000),
        (0x0000_0000_0000_0010, 0x0000_0000_0000_0000, 0x0000_0000_0000_0000, 0x0000_0000_0000_0000),
        (0x0000_0000_0000_0000, 0x0000_0000_0000_0010, 0x0000_0000_0000_0000, 0x0000_0000_0000_0000),
        (0x0000_0000_0000_0010, 0x0000_0000_0000_0010, 0x0000_0000_0000_0000, 0x0000_0000_0000_0100),
        (0x0000_0000_FFFF_FFFF, 0x0000_0000_0000_0010, 0x0000_0000_0000_000F, 0xFFFF_FFFF_FFFF_FFF0),
        (0x0000_00EE_FFFF_FFFF, 0x0000_0000_0000_0010, 0x0000_0000_0000_0EEF, 0xFFFF_FFFF_FFFF_FFF0),
        (0x0000_0000_0000_0010, 0x0000_00EE_FFFF_FFFF, 0xFFFF_FFFF_FFFF_FFEF, 0xFFFF_FFFF_FFFF_FFF0),
        (0x0000_0000_1234_5678, 0x0000_0000_0001_2334, 0x0000_0000_0000_14B5, 0x0000_0000_30EB_F860),
        (0x0000_0000_0001_2334, 0x0000_0000_1234_5678, 0x0000_0000_0000_14B5, 0x0000_0000_30EB_F860),
        (0x0000_0FF0_1234_5678, 0x0000_0000_0001_2334, 0x0000_0000_1221_2175, 0x0000_0000_30EB_F860),
        (0x0000_0000_0001_2334, 0x0000_0FF0_1234_5678, 0x0000_0000_0000_14B5, 0x0000_0000_30EB_F860),
        (0xFFFF_FFFF_FFFF_FFFF, 0x0000_0000_0000_0010, 0xFFFF_FFFF_FFFF_FFFF, 0xFFFF_FFFF_FFFF_FFF0),
        (0x0000_0000_0000_0010, 0xFFFF_FFFF_FFFF_FFFF, 0xFFFF_FFFF_FFFF_FFFF, 0xFFFF_FFFF_FFFF_FFF0),
        (0x0000_000F_7000_0001, 0x0000_0000_0000_0010, 0x0000_0000_0000_00F7, 0x0000_0000_0000_0010),
        (0x0000_0000_0000_0010, 0x0000_000F_7000_0001, 0xFFFF_FFFF_FFFF_FFF7, 0x0000_0000_0000_0010),
        (0x0000_000F_0000_0000, 0x0000_0000_0000_0010, 0x0000_0000_0000_00F0, 0x0000_0000_0000_0000),
        (0x0000_0000_0000_0010, 0x0000_000F_0000_0000, 0xFFFF_FFFF_FFFF_FFF0, 0x0000_0000_0000_0000),
        (0x0000_000C_0000_0000, 0x0000_0000_0000_0001, 0x0000_0000_0000_000C, 0x0000_0000_0000_0000),
        (0x0000_0000_0000_0001, 0x0000_000C_0000_0000, 0xFFFF_FFFF_FFFF_FFFC, 0x0000_0000_0000_0000),
        (0x0000_0002_0000_0000, 0x0000_0000_0000_0001, 0x0000_0000_0000_0002, 0x0000_0000_0000_0000),
        (0x0000_0000_0000_0001, 0x0000_0002_0000_0000, 0x0000_0000_0000_0002, 0x0000_0000_0000_0000),
        (0x0000_0004_0000_0000, 0x0000_0000_0000_0001, 0x0000_0000_0000_0004, 0x0000_0000_0000_0000),
        (0x0000_0000_0000_0001, 0x0000_0004_0000_0000, 0xFFFF_FFFF_FFFF_FFFC, 0x0000_0000_0000_0000),
        (0x0000_0008_0000_0000, 0x0000_0000_0000_0001, 0x0000_0000_0000_0008, 0x0000_0000_0000_0000),
        (0x0000_0000_0000_0001, 0x0000_0008_0000_0000, 0x0000_0000_0000_0000, 0x0000_0000_0000_0000),
        (0x0000_0040_0000_0000, 0x0000_0000_0000_0001, 0x0000_0000_0000_0040, 0x0000_0000_0000_0000),
        (0x0000_0000_0000_0001, 0x0000_0040_0000_0000, 0x0000_0000_0000_0000, 0x0000_0000_0000_0000),
    ];
    for (rs, rt, hi, lo) in rows {
        let rig = prepared(
            |g| {
                g[2] = rs;
                g[3] = rt;
            },
            Asm::new().mult(2, 3),
        );
        assert_eq!((rig.cpu.hi, rig.cpu.lo), (hi, lo), "{rs:016X} * {rt:016X}");
    }
}

#[test]
fn the_thirty_fifth_bit_of_the_second_operand_is_its_sign() {
    let rows: [(u64, u64, u64); 3] = [
        (0x0000_0008_0000_0000, 0x0000_0000_0000_0000, 0x0000_0000_0000_0000),
        (0x0000_0004_0000_0000, 0xFFFF_FFFF_FFFF_FFFC, 0x0000_0000_0000_0000),
        (0x0000_0002_0000_0000, 0x0000_0000_0000_0002, 0x0000_0000_0000_0000),
    ];
    for (rt, hi, lo) in rows {
        let rig = prepared(
            |g| {
                g[2] = 1;
                g[3] = rt;
            },
            Asm::new().mult(2, 3),
        );
        assert_eq!((rig.cpu.hi, rig.cpu.lo), (hi, lo), "1 * {rt:016X}");
    }
}

#[test]
fn a_signed_multiply_is_not_commutative() {
    let forward = prepared(
        |g| {
            g[2] = 0x0000_00EE_FFFF_FFFF;
            g[3] = 0x10;
        },
        Asm::new().mult(2, 3),
    );
    let reversed = prepared(
        |g| {
            g[2] = 0x10;
            g[3] = 0x0000_00EE_FFFF_FFFF;
        },
        Asm::new().mult(2, 3),
    );
    assert_eq!(forward.cpu.hi, 0x0000_0000_0000_0EEF);
    assert_eq!(reversed.cpu.hi, 0xFFFF_FFFF_FFFF_FFEF);
}

#[test]
fn an_unsigned_multiply_still_reads_thirty_two_bits_of_both_operands() {
    let rig = prepared(
        |g| {
            g[2] = 0x0000_00EE_FFFF_FFFF;
            g[3] = 0x10;
        },
        Asm::new().multu(2, 3),
    );
    assert_eq!(rig.cpu.hi, 0x0000_0000_0000_000F);
    assert_eq!(rig.cpu.lo, 0xFFFF_FFFF_FFFF_FFF0);
}

// ---- MarsCpuTrapTests ----

/// The operands arrive by load, as the C# fixture puts them there.
fn trap_run(left: u64, right: u64, trap: impl FnOnce(Asm) -> Asm, steps: usize) -> Rig {
    let mut bus = new_bus();
    bus.write64(0x100, left);
    bus.write64(0x108, right);
    trap(Asm::new().lui(1, 0x8000).ld(2, 1, 0x100).ld(3, 1, 0x108)).run(steps, bus)
}

fn expect_trap(rows: &[(u64, u64, bool)], trap: fn(Asm) -> Asm) {
    for &(left, right, trapped) in rows {
        let rig = trap_run(left, right, trap, 4);
        let expected = if trapped { Some(code::TRAP) } else { None };
        assert_eq!(rig.last_code(), expected, "{left:016X}, {right:016X}");
    }
}

fn expect_trap_immediate(rows: &[(u64, i16, bool)], trap: fn(Asm, i16) -> Asm) {
    for &(value, immediate, trapped) in rows {
        let rig = trap_run(value, 0, |a| trap(a, immediate), 4);
        let expected = if trapped { Some(code::TRAP) } else { None };
        assert_eq!(rig.last_code(), expected, "{value:016X}, {immediate}");
    }
}

#[test]
fn trap_if_less_than_reads_both_operands_as_signed() {
    expect_trap(
        &[
            (0x0000_0000_0000_0095, 0x0000_0000_0000_0096, true),
            (0xFFFF_FFFF_FFFF_FFFF, 0x0000_0000_0000_0000, true),
            (0x0000_0095_0000_0096, 0x0000_0096_0000_0095, true),
            (0xFFFF_FFFF_0000_0000, 0x0000_0000_0000_0000, true),
            (0xFFFF_FFFF_0000_0000, 0xFFFF_FFFF_F000_0000, true),
            (0xBADD_ECAF_15C0_FFEE, 0xBADD_ECAF_15C0_FFEE, false),
            (0x0000_0000_0000_0000, 0xFFFF_FFFF_FFFF_FFFF, false),
            (0x0000_0000_0000_0000, 0xFFFF_FFFF_0000_0000, false),
            (0xFFFF_FFFF_F000_0000, 0xFFFF_FFFF_0000_0000, false),
        ],
        |a| a.tlt(2, 3),
    );
}

#[test]
fn trap_if_greater_or_equal_is_the_exact_inverse() {
    expect_trap(
        &[
            (0x0000_0000_0000_0095, 0x0000_0000_0000_0096, false),
            (0x0000_0095_0000_0096, 0x0000_0096_0000_0095, false),
            (0xFFFF_FFFF_0000_0000, 0xFFFF_FFFF_F000_0000, false),
            (0xBADD_ECAF_15C0_FFEE, 0xBADD_ECAF_15C0_FFEE, true),
            (0x0000_0000_0000_0000, 0xFFFF_FFFF_FFFF_FFFF, true),
            (0xFFFF_FFFF_F000_0000, 0xFFFF_FFFF_0000_0000, true),
        ],
        |a| a.tge(2, 3),
    );
}

#[test]
fn trap_if_less_than_unsigned_reads_both_operands_as_unsigned() {
    expect_trap(
        &[
            (0xFFFF_FFFF_0000_0122, 0x0FFF_FFFF_0000_0123, false),
            (0xFFFF_FFFF_0000_0000, 0xFFFF_FFFF_F000_0000, true),
            (0x0000_0000_0000_0100, 0x0000_0000_0000_0101, true),
            (0x0000_0100_0000_0100, 0x0000_0101_0000_00FF, true),
            (0x0000_0100_0000_0100, 0x0000_00FF_0000_0101, false),
        ],
        |a| a.tltu(2, 3),
    );
}

#[test]
fn trap_if_greater_or_equal_unsigned_is_the_exact_inverse() {
    expect_trap(
        &[
            (0x0FFF_FFFF_0000_0123, 0xFFFF_FFFF_0000_0122, false),
            (0x0000_0000_0000_0000, 0x0000_0000_0000_0000, true),
            (0xFFFF_FFFF_F000_0000, 0xFFFF_FFFF_0000_0000, true),
        ],
        |a| a.tgeu(2, 3),
    );
}

#[test]
fn trap_if_equal_compares_the_whole_register() {
    expect_trap(
        &[
            (0x0000_0000_0000_0000, 0x0000_0000_0000_0000, true),
            (0x0000_0100_0000_0000, 0x0000_0100_0000_0000, true),
            (0x0000_0100_0000_0000, 0x0000_0101_0000_0000, false),
            (0x0000_0100_0000_0100, 0x0000_00FF_0000_0101, false),
        ],
        |a| a.teq(2, 3),
    );
}

#[test]
fn trap_if_not_equal_compares_the_whole_register() {
    expect_trap(
        &[
            (0x0000_0000_0000_0000, 0x0000_0000_0000_0000, false),
            (0x0000_0100_0000_0000, 0x0000_0100_0000_0000, false),
            (0x0000_0100_0000_0000, 0x0000_0101_0000_0000, true),
            (0x0000_0100_0000_0100, 0x0000_00FF_0000_0101, true),
        ],
        |a| a.tne(2, 3),
    );
}

#[test]
fn trap_if_less_than_immediate_unsigned_sign_extends_then_compares_unsigned() {
    expect_trap_immediate(
        &[
            (0xFFFF_FFFF_FFFF_FFFF, -2, false),
            (0xFFFF_FFFF_FFFF_FFFE, -2, false),
            (0xFFFF_FFFF_FFFF_FFFD, -2, true),
            (0x0000_0000_0000_0000, -2, true),
            (0x0000_0000_FFFF_FFFF, -2, true),
            (0x8000_0000_0000_0000, 0, false),
            (0x0000_0000_0000_0001, 2, true),
            (0x0000_0000_0000_0002, 2, false),
        ],
        |a, i| a.tltiu(2, i),
    );
}

#[test]
fn trap_if_greater_or_equal_immediate_unsigned_is_the_exact_inverse() {
    expect_trap_immediate(&[(0xFFFF_FFFF_FFFF_FFFF, -2, true), (0xFFFF_FFFF_FFFF_FFFD, -2, false), (0x0000_0000_0000_0000, -2, false)], |a, i| {
        a.tgeiu(2, i)
    });
}

#[test]
fn trap_if_less_than_immediate_reads_both_as_signed() {
    expect_trap_immediate(
        &[
            (0x8000_0000_0000_0000, 0, true),
            (0x0000_0000_FFFF_FFFF, 0, false),
            (0x0000_0000_0000_0001, 2, true),
            (0xFFFF_FFFF_0000_0002, 2, true),
            (0xFFFF_FFFF_FFFF_FFFF, -2, false),
            (0x0000_0000_FFFF_FFFD, -2, false),
            (0xFFFF_FFFF_FFFF_FFFD, -2, true),
        ],
        |a, i| a.tlti(2, i),
    );
}

#[test]
fn trap_if_greater_or_equal_immediate_is_the_exact_inverse() {
    expect_trap_immediate(
        &[
            (0x8000_0000_0000_0000, 0, false),
            (0x0000_0000_FFFF_FFFF, 0, true),
            (0xFFFF_FFFF_FFFF_FFFF, -2, true),
            (0x0000_0000_FFFF_FFFD, -2, true),
            (0xFFFF_FFFF_FFFF_FFFD, -2, false),
        ],
        |a, i| a.tgei(2, i),
    );
}

#[test]
fn trap_if_equal_immediate_compares_the_sign_extended_immediate() {
    expect_trap_immediate(&[(0xFFFF_FFFF_FFFF_FFFE, -2, true), (0x0000_0000_FFFF_FFFE, -2, false), (0x0000_0000_0000_0002, 2, true)], |a, i| {
        a.teqi(2, i)
    });
}

#[test]
fn trap_if_not_equal_immediate_compares_the_sign_extended_immediate() {
    expect_trap_immediate(&[(0xFFFF_FFFF_FFFF_FFFE, -2, false), (0x0000_0000_FFFF_FFFE, -2, true), (0x0000_0000_0000_0002, 2, false)], |a, i| {
        a.tnei(2, i)
    });
}

#[test]
fn a_taken_trap_reports_itself_as_a_trap_and_saves_its_own_address() {
    let rig = trap_run(0, 0, |a| a.teq(2, 3), 4);
    assert_eq!(rig.last_code(), Some(code::TRAP));
    assert_eq!(rig.cpu.cop0[CAUSE] & 0xFF, 0x34);
    assert_eq!(rig.cpu.cop0[EXCEPTION_PC], KSEG0 + 12);
}

#[test]
fn an_untaken_trap_costs_nothing_and_execution_carries_on() {
    let rig = trap_run(1, 0, |a| a.teq(2, 3).addiu(4, 0, 0x1234), 5);
    assert_eq!(rig.last_code(), None);
    assert_eq!(rig.cpu.gpr[4], 0x1234);
}

#[test]
fn a_trap_in_a_delay_slot_saves_the_branch_and_flags_the_slot() {
    for taken in [true, false] {
        let rig = trap_run(0, 0, |a| if taken { a.beq(0, 0, 1).teq(2, 3) } else { a.bne(0, 0, 1).teq(2, 3) }, 5);
        assert_eq!(rig.last_code(), Some(code::TRAP), "taken {taken}");
        assert_eq!(rig.cpu.cop0[EXCEPTION_PC], KSEG0 + 12, "taken {taken}");
        assert_eq!(rig.cpu.cop0[CAUSE] & CAUSE_BRANCH_DELAY, CAUSE_BRANCH_DELAY, "taken {taken}");
    }
}

// ---- MarsCpuUnalignedTests ----

const DATA: u32 = 0x800;

/// 0xAAAAAAAA in the register beforehand, so every byte left alone is visible.
fn preloaded() -> Asm {
    Asm::new().lui(1, 0x8000).ori(1, 1, DATA as u16).lui(2, 0xAAAA).ori(2, 2, 0xAAAA)
}

fn with_word() -> MemoryBus {
    let mut bus = new_bus();
    bus.write32(DATA, 0x0102_0304);
    bus
}

fn with_double_word() -> MemoryBus {
    let mut bus = new_bus();
    bus.write64(DATA, 0x0102_0304_0506_0708);
    bus
}

fn preloaded_wide(bus: &mut MemoryBus) -> Asm {
    bus.write64(DATA + 0x10, 0xAAAA_AAAA_AAAA_AAAA);
    Asm::new().lui(1, 0x8000).ori(1, 1, DATA as u16).ld(2, 1, 0x10)
}

/// A register whose upper half is neither zero nor all ones, which is what makes the pair separable.
fn measured(load: impl FnOnce(Asm) -> Asm) -> Rig {
    let mut bus = new_bus();
    bus.write64(DATA, 0x0123_4567_89AB_CDEF);
    bus.write64(DATA + 0x10, 0xFEDC_BA98_7654_3210);
    load(Asm::new().lui(1, 0x8000).ori(1, 1, DATA as u16).ld(2, 1, 0x10)).run(4, bus)
}

#[test]
fn a_left_load_fills_from_the_addressed_byte_upwards() {
    let rows: [(i16, u64); 4] = [(0, 0x0000_0000_0102_0304), (1, 0x0000_0000_0203_04AA), (2, 0x0000_0000_0304_AAAA), (3, 0x0000_0000_04AA_AAAA)];
    for (offset, expected) in rows {
        let rig = preloaded().lwl(2, 1, offset).run(5, with_word());
        assert_eq!(rig.cpu.gpr[2], expected, "offset {offset}");
    }
}

#[test]
fn a_right_load_fills_from_the_addressed_byte_downwards() {
    let rows: [(i16, u64); 4] = [(3, 0x0000_0000_0102_0304), (2, 0xFFFF_FFFF_AA01_0203), (1, 0xFFFF_FFFF_AAAA_0102), (0, 0xFFFF_FFFF_AAAA_AA01)];
    for (offset, expected) in rows {
        let rig = preloaded().lwr(2, 1, offset).run(5, with_word());
        assert_eq!(rig.cpu.gpr[2], expected, "offset {offset}");
    }
}

#[test]
fn a_left_load_sign_extends_however_little_of_the_word_it_took() {
    let rows: [(i16, u64); 8] = [
        (0, 0x0000_0000_0123_4567),
        (1, 0x0000_0000_2345_6710),
        (2, 0x0000_0000_4567_3210),
        (3, 0x0000_0000_6754_3210),
        (4, 0xFFFF_FFFF_89AB_CDEF),
        (5, 0xFFFF_FFFF_ABCD_EF10),
        (6, 0xFFFF_FFFF_CDEF_3210),
        (7, 0xFFFF_FFFF_EF54_3210),
    ];
    for (offset, expected) in rows {
        let rig = measured(|a| a.lwl(2, 1, offset));
        assert_eq!(rig.cpu.gpr[2], expected, "offset {offset}");
    }
}

#[test]
fn a_right_load_sign_extends_only_when_it_took_the_whole_word() {
    let rows: [(i16, u64); 8] = [
        (0, 0xFEDC_BA98_7654_3201),
        (1, 0xFEDC_BA98_7654_0123),
        (2, 0xFEDC_BA98_7601_2345),
        (3, 0x0000_0000_0123_4567),
        (4, 0xFEDC_BA98_7654_3289),
        (5, 0xFEDC_BA98_7654_89AB),
        (6, 0xFEDC_BA98_7689_ABCD),
        (7, 0xFFFF_FFFF_89AB_CDEF),
    ];
    for (offset, expected) in rows {
        let rig = measured(|a| a.lwr(2, 1, offset));
        assert_eq!(rig.cpu.gpr[2], expected, "offset {offset}");
    }
}

#[test]
fn the_pair_together_reads_a_word_that_straddles_the_boundary() {
    let mut bus = with_word();
    bus.write32(DATA + 4, 0x0506_0708);
    let rig = preloaded().lwl(2, 1, 1).lwr(2, 1, 4).run(6, bus);
    assert_eq!(rig.cpu.gpr[2], 0x0000_0000_0203_0405);
}

#[test]
fn a_left_double_load_merges_with_the_register_it_finds() {
    let mut bus = with_double_word();
    let rig = preloaded_wide(&mut bus).ldl(2, 1, 2).run(4, bus);
    assert_eq!(rig.cpu.gpr[2], 0x0304_0506_0708_AAAA);
}

#[test]
fn a_right_double_load_merges_from_the_other_end() {
    let mut bus = with_double_word();
    let rig = preloaded_wide(&mut bus).ldr(2, 1, 5).run(4, bus);
    assert_eq!(rig.cpu.gpr[2], 0xAAAA_0102_0304_0506);
}

#[test]
fn the_double_pair_reads_a_straddling_doubleword() {
    let mut bus = with_double_word();
    bus.write64(DATA + 8, 0x090A_0B0C_0D0E_0F10);
    let rig = preloaded().ldl(2, 1, 1).ldr(2, 1, 8).run(6, bus);
    assert_eq!(rig.cpu.gpr[2], 0x0203_0405_0607_0809);
}

fn store_program() -> Asm {
    Asm::new().lui(1, 0x8000).ori(1, 1, DATA as u16).lui(2, 0x5566).ori(2, 2, 0x7788)
}

#[test]
fn a_left_store_writes_the_high_bytes_and_leaves_the_rest() {
    let mut bus = new_bus();
    bus.write32(DATA, 0x1122_3344);
    let mut rig = store_program().swl(2, 1, 1).run(5, bus);
    assert_eq!(rig.last_code(), None);
    assert_eq!(rig.bus.read32(DATA), 0x1155_6677);
}

#[test]
fn a_right_store_writes_the_low_bytes_and_leaves_the_rest() {
    let mut bus = new_bus();
    bus.write32(DATA, 0x1122_3344);
    let mut rig = store_program().swr(2, 1, 2).run(5, bus);
    assert_eq!(rig.last_code(), None);
    assert_eq!(rig.bus.read32(DATA), 0x6677_8844);
}

#[test]
fn the_store_pair_writes_a_word_across_the_boundary() {
    let mut bus = new_bus();
    bus.write32(DATA, 0x1122_3344);
    bus.write32(DATA + 4, 0x99AA_BBCC);
    let mut rig = store_program().swl(2, 1, 1).swr(2, 1, 4).run(6, bus);
    assert_eq!(rig.last_code(), None);
    assert_eq!(rig.bus.read32(DATA), 0x1155_6677);
    assert_eq!(rig.bus.read32(DATA + 4), 0x88AA_BBCC);
}

#[test]
fn a_double_store_merges_at_both_ends() {
    let mut bus = new_bus();
    bus.write64(DATA, 0x1122_3344_5566_7788);
    let mut rig = Asm::new().lui(1, 0x8000).ori(1, 1, DATA as u16).addiu(2, 0, -1).sdl(2, 1, 6).run(4, bus);
    assert_eq!(rig.last_code(), None);
    assert_eq!(rig.bus.read64(DATA), 0x1122_3344_5566_FFFF);
}

#[test]
fn none_of_them_fault_on_an_address_an_ordinary_load_would_refuse() {
    let rig = preloaded().lwl(2, 1, 1).lwr(3, 1, 2).ldl(4, 1, 3).swl(2, 1, 1).run(8, with_word());
    assert_eq!(rig.last_code(), None);
}

// ---- MarsCpuExceptionTests ----

#[test]
fn a_system_call_records_its_cause_and_jumps_to_the_handler() {
    let rig = Asm::new().nop().syscall().run_new(2);
    assert_eq!(rig.cpu.cop0[CAUSE] & 0x7C, (code::SYSCALL as u64) << 2);
    assert_eq!(rig.cpu.cop0[STATUS] & STATUS_EXCEPTION_LEVEL, STATUS_EXCEPTION_LEVEL);
    assert_eq!(rig.cpu.pc, GENERAL_VECTOR);
}

#[test]
fn the_saved_address_points_at_the_instruction_that_faulted() {
    let rig = Asm::new().nop().nop().syscall().run_new(3);
    assert_eq!(rig.cpu.cop0[EXCEPTION_PC], ENTRY_POINT + 8);
}

#[test]
fn a_fault_in_a_delay_slot_saves_the_branch_and_says_so() {
    let rig = Asm::new().beq(0, 0, 4).syscall().run_new(2);
    assert_eq!(rig.cpu.cop0[EXCEPTION_PC], ENTRY_POINT);
    assert_eq!(rig.cpu.cop0[CAUSE] & CAUSE_BRANCH_DELAY, CAUSE_BRANCH_DELAY);
}

#[test]
fn an_ordinary_fault_clears_the_delay_slot_flag_again() {
    let mut rig = Asm::new().beq(0, 0, 4).syscall().run_new(2);
    rig.cpu.cop0[STATUS] &= !STATUS_EXCEPTION_LEVEL;
    let second = Asm::new().nop().syscall().run_new(2);
    assert_eq!(second.cpu.cop0[CAUSE] & CAUSE_BRANCH_DELAY, 0);
}

#[test]
fn an_address_error_records_the_address_that_caused_it() {
    let rig = Asm::new().lui(1, 0x8000).ori(1, 1, 0x0801).lw(2, 1, 0).run_new(3);
    assert_eq!(rig.last_code(), Some(code::ADDRESS_ERROR_LOAD));
    assert_eq!(rig.cpu.cop0[BAD_VIRTUAL_ADDRESS], 0xFFFF_FFFF_8000_0801);
}

#[test]
fn an_untranslated_segment_uses_the_refill_vector() {
    let rig = Asm::new().lui(1, 0x0010).lw(2, 1, 0).run_new(2);
    assert_eq!(rig.last_code(), Some(code::TLB_LOAD));
    assert_eq!(rig.cpu.pc, REFILL_VECTOR);
}

#[test]
fn a_fault_while_already_handling_one_uses_the_general_vector_and_keeps_the_first_address() {
    let mut bus = new_bus();
    bus.write32(0x180, Asm::new().syscall().words()[0]);
    let rig = Asm::new().nop().syscall().run(3, bus);
    assert_eq!(rig.cpu.cop0[EXCEPTION_PC], ENTRY_POINT + 4);
    assert_eq!(rig.cpu.pc, GENERAL_VECTOR);
}

#[test]
fn the_bootstrap_flag_moves_the_handler_somewhere_else_entirely() {
    let mut rig = Asm::new().nop().run_new(1);
    rig.cpu.cop0[STATUS] |= STATUS_BOOTSTRAP_VECTORS;
    rig.at_entry();
    rig.bus.write32(0, Asm::new().syscall().words()[0]);
    rig.step();
    assert_eq!(rig.cpu.pc, VECTOR_BASE_BOOTSTRAP + 0x180);
}

#[test]
fn a_handler_can_return_to_where_the_fault_happened() {
    let mut bus = new_bus();
    let handler = Asm::new().addiu(5, 0, 0x77).mfc0(6, EXCEPTION_PC).addiu(6, 6, 4).mtc0(6, EXCEPTION_PC).eret();
    for (i, &word) in handler.words().iter().enumerate() {
        bus.write32(0x180 + i as u32 * 4, word);
    }
    let rig = Asm::new().syscall().addiu(4, 0, 0x42).run(7, bus);
    assert_eq!(rig.cpu.gpr[5], 0x77);
    assert_eq!(rig.cpu.gpr[4], 0x42);
    assert_eq!(rig.cpu.cop0[STATUS] & STATUS_EXCEPTION_LEVEL, 0);
}

// ---- MarsCpuInterruptTests ----

const TIMER_MASK: u64 = 1 << 15;
const RCP_MASK: u64 = 1 << 10;

fn machine(bus: MemoryBus) -> Rig {
    let mut rig = Asm::new().nop().run(0, bus);
    rig.at_entry();
    rig
}

/// Execution continues into the handler, so the vector is only on the program counter for one step.
fn step_until_interrupted(rig: &mut Rig, limit: usize) -> bool {
    for _ in 0..limit {
        rig.step();
        if rig.last().is_some() {
            return true;
        }
    }
    false
}

fn timer(rig: &Rig) -> u64 {
    rig.cpu.cop0[CAUSE] & CAUSE_INTERRUPT_TIMER
}

#[test]
fn the_counter_raises_its_line_when_it_reaches_the_comparison_value() {
    let mut rig = machine(new_bus());
    rig.cpu.cop0[COMPARE] = 3;
    rig.cop0_written();
    rig.run(10);
    assert_eq!(timer(&rig), CAUSE_INTERRUPT_TIMER);
}

#[test]
fn a_counter_that_jumps_over_the_comparison_value_still_raises_it() {
    let mut rig = Asm::new().addiu(1, 0, 7).addiu(2, 0, 3).mult(1, 2).run(0, new_bus());
    rig.at_entry();
    rig.cpu.cop0[COMPARE] = 2;
    rig.cop0_written();
    rig.run(3);
    assert_eq!(rig.bus.count(), 4);
    assert_eq!(timer(&rig), CAUSE_INTERRUPT_TIMER);
}

#[test]
fn a_raised_line_does_nothing_while_it_is_masked_off() {
    let mut rig = machine(new_bus());
    rig.cpu.cop0[COMPARE] = 3;
    rig.cpu.cop0[STATUS] = STATUS_INTERRUPT_ENABLE;
    rig.cop0_written();
    rig.run(10);
    assert_eq!(timer(&rig), CAUSE_INTERRUPT_TIMER);
    assert_eq!(rig.last_code(), None);
}

#[test]
fn a_masked_in_line_interrupts_once_the_enable_bit_is_set() {
    let mut rig = machine(new_bus());
    rig.cpu.cop0[COMPARE] = 3;
    rig.cpu.cop0[STATUS] = STATUS_INTERRUPT_ENABLE | TIMER_MASK;
    rig.cop0_written();
    assert!(step_until_interrupted(&mut rig, 10));
    assert_eq!(rig.last_code(), Some(code::INTERRUPT));
    assert_eq!(rig.cpu.pc, GENERAL_VECTOR);
}

#[test]
fn the_enable_bit_alone_gates_it() {
    let mut rig = machine(new_bus());
    rig.cpu.cop0[COMPARE] = 3;
    rig.cpu.cop0[STATUS] = TIMER_MASK;
    rig.cop0_written();
    rig.run(10);
    assert_eq!(rig.last_code(), None);
}

#[test]
fn an_exception_already_in_progress_blocks_the_next_interrupt() {
    let mut rig = machine(new_bus());
    rig.cpu.cop0[COMPARE] = 3;
    rig.cpu.cop0[STATUS] = STATUS_INTERRUPT_ENABLE | TIMER_MASK | STATUS_EXCEPTION_LEVEL;
    rig.cop0_written();
    rig.run(10);
    assert_eq!(rig.last_code(), None);
}

#[test]
fn writing_the_comparison_value_is_what_lowers_the_line() {
    let mut rig = machine(new_bus());
    rig.cpu.cop0[COMPARE] = 3;
    rig.cop0_written();
    rig.run(10);

    Asm::new().addiu(1, 0, 0x7000).mtc0(1, COMPARE).load_into(&mut rig.bus);
    rig.at_entry();
    rig.cpu.cop0[STATUS] = 0;
    rig.cop0_written();
    rig.run(2);
    assert_eq!(timer(&rig), 0);
}

#[test]
fn the_line_rises_at_the_end_of_the_instruction_whose_cycles_reach_the_value() {
    let mut rig = Asm::new().addiu(1, 0, 3).mtc0(1, COMPARE).nop().nop().nop().nop().build_new();
    rig.run(5);
    assert_eq!(timer(&rig), 0);
    rig.run(1);
    assert_eq!(timer(&rig), CAUSE_INTERRUPT_TIMER);
}

#[test]
fn a_comparison_value_equal_to_the_count_waits_a_whole_wrap() {
    let mut program = Asm::new();
    for _ in 0..9 {
        program = program.nop();
    }
    let mut rig = program.addiu(1, 0, 5).mtc0(1, COMPARE).build_new();
    rig.run(11);
    assert_eq!(rig.bus.count(), 5);
    rig.run(200);
    assert_eq!(timer(&rig), 0);
}

#[test]
fn returning_with_the_line_still_raised_takes_the_interrupt_again_at_once() {
    let mut rig = Asm::new().nop().nop().build_new();
    rig.bus.write32((GENERAL_VECTOR & 0x1FFF_FFFF) as u32, 0x4200_0018);
    rig.cpu.cop0[STATUS] = STATUS_INTERRUPT_ENABLE | RCP_MASK;
    rig.cop0_written();
    rig.bus.mi.mask = interrupt::VIDEO_INTERFACE;
    rig.bus.mi.raise(interrupt::VIDEO_INTERFACE);

    rig.step();
    assert_eq!(rig.cpu.pc, GENERAL_VECTOR);
    rig.step();
    assert_eq!(rig.cpu.pc, ENTRY_POINT);
    rig.step();
    assert_eq!(rig.cpu.pc, GENERAL_VECTOR);
}

#[test]
fn a_device_reaches_the_cpu_through_the_aggregator() {
    let mut bus = new_bus();
    bus.mi.mask = interrupt::PERIPHERAL_INTERFACE;
    bus.mi.raise(interrupt::PERIPHERAL_INTERFACE);
    let mut rig = machine(bus);
    rig.cpu.cop0[STATUS] = STATUS_INTERRUPT_ENABLE | RCP_MASK;
    rig.step();
    assert_eq!(rig.last_code(), Some(code::INTERRUPT));
    assert_eq!(rig.cpu.cop0[CAUSE] & CAUSE_INTERRUPT_RCP, CAUSE_INTERRUPT_RCP);
}

#[test]
fn an_unmasked_device_never_reaches_the_line_at_all() {
    let mut bus = new_bus();
    bus.mi.raise(interrupt::PERIPHERAL_INTERFACE);
    let mut rig = machine(bus);
    rig.cpu.cop0[STATUS] = STATUS_INTERRUPT_ENABLE | RCP_MASK;
    rig.run(3);
    assert_eq!(rig.last_code(), None);
    assert_eq!(rig.cpu.cop0[CAUSE] & CAUSE_INTERRUPT_RCP, 0);
}

#[test]
fn clearing_the_device_lowers_the_line_again() {
    let mut bus = new_bus();
    bus.mi.mask = interrupt::PERIPHERAL_INTERFACE;
    bus.mi.raise(interrupt::PERIPHERAL_INTERFACE);
    let mut rig = machine(bus);
    rig.step();
    assert_eq!(rig.cpu.cop0[CAUSE] & CAUSE_INTERRUPT_RCP, CAUSE_INTERRUPT_RCP);

    rig.bus.mi.clear(interrupt::PERIPHERAL_INTERFACE);
    rig.step();
    assert_eq!(rig.cpu.cop0[CAUSE] & CAUSE_INTERRUPT_RCP, 0);
}

#[test]
fn a_finished_cartridge_transfer_is_one_of_those_devices() {
    let mut bus = bus_with_cart(&build_rom());
    bus.mi.mask = interrupt::PERIPHERAL_INTERFACE;
    bus.write32(PI_BASE, 0x1000);
    bus.write32(PI_BASE + 0x04, 0x1000_0000);
    bus.write32(PI_BASE + 0x0C, 0x40 - 1);

    let mut rig = machine(bus);
    rig.cpu.cop0[STATUS] = STATUS_INTERRUPT_ENABLE | RCP_MASK;
    rig.step();
    assert_eq!(rig.last_code(), Some(code::INTERRUPT));
}

#[test]
fn the_mask_register_takes_a_pair_of_bits_for_each_device() {
    let mut bus = new_bus();
    bus.write32(MI_BASE + 0x0C, 1 << 9);
    assert_eq!(bus.mi.mask, interrupt::PERIPHERAL_INTERFACE);
    bus.write32(MI_BASE + 0x0C, 1 << 8);
    assert_eq!(bus.mi.mask, 0);
}
