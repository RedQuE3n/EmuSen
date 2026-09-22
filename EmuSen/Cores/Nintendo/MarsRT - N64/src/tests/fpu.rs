//! Coprocessor one: `MarsFpuArithmeticTests` and `MarsFpuRegisterFileTests`.

use super::support::{Asm, ENTRY_POINT, Rig};
use crate::cop0::{CAUSE, CAUSE_COPROCESSOR, EXCEPTION_PC, STATUS, STATUS_COP1_USABLE, STATUS_FPU_FULL_MODE};
use crate::cop1::{FCSR_CAUSE_UNIMPLEMENTED, FPU_IMPLEMENTATION};
use crate::cpu::code;

const FULL_MODE: u64 = STATUS_COP1_USABLE | STATUS_FPU_FULL_MODE;
const HALF_MODE: u64 = STATUS_COP1_USABLE;
const IMPLEMENTATION_REGISTER: u32 = 0;
const CONTROL_STATUS_REGISTER: u32 = 31;

// ---- MarsFpuArithmeticTests ----

const INEXACT: u32 = 1 << 2;
const INVALID: u32 = 1 << 6;

fn compute(program: Asm, left: u32, right: u32) -> Rig {
    let mut rig = program.build_new();
    rig.cpu.cop0[STATUS] = FULL_MODE;
    rig.cpu.fpr[2] = left as u64;
    rig.cpu.fpr[3] = right as u64;
    rig.run(1);
    rig
}

fn compute_wide(program: Asm, operand: u64) -> Rig {
    let mut rig = program.build_new();
    rig.cpu.cop0[STATUS] = FULL_MODE;
    rig.cpu.fpr[2] = operand;
    rig.run(1);
    rig
}

#[test]
fn a_single_precision_addition_matches_the_measured_result_and_flags() {
    let rows: [(u32, u32, u32, u32); 8] = [
        (0x0000_0000, 0x4000_0000, 0x4000_0000, 0),
        (0x3F80_0000, 0x40A0_0000, 0x40C0_0000, 0),
        (0xFF7F_FFFF, 0xBF80_0000, 0xFF7F_FFFF, INEXACT),
        (0x7F7F_FFFF, 0xBF80_0000, 0x7F7F_FFFF, INEXACT),
        (0x7F7F_FFFF, 0x3F80_0000, 0x7F7F_FFFF, INEXACT),
        (0xFF7F_FFFF, 0x7F7F_FFFF, 0x0000_0000, 0),
        (0x7F7F_FFFF, 0x0080_0000, 0x7F7F_FFFF, INEXACT),
        (0x7F80_0000, 0xFF80_0000, 0x7FBF_FFFF, INVALID),
    ];
    for (left, right, expected, flags) in rows {
        let rig = compute(Asm::new().cop1_format(0x10, 0x00, 4, 2, 3), left, right);
        assert_eq!(rig.cpu.fpr[4] as u32, expected, "{left:08X} + {right:08X}");
        assert_eq!(rig.cpu.fcsr & 0x7C, flags, "{left:08X} + {right:08X}");
    }
}

#[test]
fn a_single_precision_square_root_matches_the_measured_result() {
    let rows: [(u32, u32, u32); 8] = [
        (0x4180_0000, 0x4080_0000, 0),
        (0x4000_0000, 0x3FB5_04F3, INEXACT),
        (0x0000_0000, 0x0000_0000, 0),
        (0x8000_0000, 0x8000_0000, 0),
        (0x7F80_0000, 0x7F80_0000, 0),
        (0xBF80_0000, 0x7FBF_FFFF, INVALID),
        (0x3F80_0000, 0x3F80_0000, 0),
        (0x4090_0000, 0x4007_C3B6, INEXACT),
    ];
    for (operand, expected, flags) in rows {
        let rig = compute(Asm::new().cop1_format(0x10, 0x04, 4, 2, 0), operand, 0);
        assert_eq!(rig.cpu.fpr[4] as u32, expected, "sqrt {operand:08X}");
        assert_eq!(rig.cpu.fcsr & 0x7C, flags, "sqrt {operand:08X}");
    }
}

#[test]
fn a_double_below_two_to_the_fifty_third_converts_to_a_long() {
    let rows: [(u64, i64); 4] = [
        (0x4010_0000_0000_0000, 4),
        (0x432C_6BF5_2634_0000, 4_000_000_000_000_000),
        (0x433F_FFFF_FFFF_FFFF, 9_007_199_254_740_991),
        (0xC33F_FFFF_FFFF_FFFF, -9_007_199_254_740_991),
    ];
    for (operand, expected) in rows {
        let rig = compute_wide(Asm::new().cop1_format(0x11, 0x25, 4, 2, 0), operand);
        assert_eq!(rig.last_code(), None, "{operand:016X}");
        assert_eq!(rig.cpu.fpr[4] as i64, expected, "{operand:016X}");
    }
}

#[test]
fn a_double_of_two_to_the_fifty_third_or_more_refuses_to_convert() {
    for operand in [0x4340_0000_0000_0000u64, 0xC340_0000_0000_0000, 0x7FEF_FFFF_FFFF_FFFF] {
        let rig = compute_wide(Asm::new().cop1_format(0x11, 0x25, 4, 2, 0), operand);
        assert_eq!(rig.last_code(), Some(code::FLOATING_POINT), "{operand:016X}");
        assert_eq!(rig.cpu.fcsr & FCSR_CAUSE_UNIMPLEMENTED, FCSR_CAUSE_UNIMPLEMENTED, "{operand:016X}");
    }
}

#[test]
fn a_single_below_the_same_boundary_converts_to_a_long() {
    let rows: [(u32, i64); 2] = [(0x59FF_FFFE, 9_007_198_180_999_168), (0x59FF_FFFF, 9_007_198_717_870_080)];
    for (operand, expected) in rows {
        let rig = compute(Asm::new().cop1_format(0x10, 0x25, 4, 2, 0), operand, 0);
        assert_eq!(rig.last_code(), None, "{operand:08X}");
        assert_eq!(rig.cpu.fpr[4] as i64, expected, "{operand:08X}");
    }
}

#[test]
fn a_single_at_or_above_the_same_boundary_refuses_to_convert() {
    for operand in [0x5A00_0000u32, 0x5A00_0001] {
        let rig = compute(Asm::new().cop1_format(0x10, 0x25, 4, 2, 0), operand, 0);
        assert_eq!(rig.last_code(), Some(code::FLOATING_POINT), "{operand:08X}");
        assert_eq!(rig.cpu.fcsr & FCSR_CAUSE_UNIMPLEMENTED, FCSR_CAUSE_UNIMPLEMENTED, "{operand:08X}");
    }
}

#[test]
fn the_word_conversions_still_stop_at_the_word() {
    let rows: [(u64, i64); 2] = [(0x41DF_FFFF_FFC0_0000, 2_147_483_647), (0xC1E0_0000_0000_0000, -2_147_483_648)];
    for (operand, expected) in rows {
        let rig = compute_wide(Asm::new().cop1_format(0x11, 0x24, 4, 2, 0), operand);
        assert_eq!(rig.last_code(), None, "{operand:016X}");
        assert_eq!(rig.cpu.fpr[4] as i32 as i64, expected, "{operand:016X}");
    }
}

// ---- MarsFpuRegisterFileTests ----

fn prepared(status: u64, steps: usize, registers: impl FnOnce(&mut [u64; 32]), program: Asm) -> Rig {
    let mut rig = program.build_new();
    rig.cpu.cop0[STATUS] = status;
    registers(&mut rig.cpu.fpr);
    rig.run(steps);
    rig
}

fn run(status: u64, steps: usize, program: Asm) -> Rig {
    prepared(status, steps, |_| {}, program)
}

#[test]
fn a_wide_move_survives_a_round_trip_through_the_register_file() {
    let rig = run(FULL_MODE, 4, Asm::new().lui(1, 0x1234).ori(1, 1, 0x5678).dmtc1(1, 3).dmfc1(2, 3));
    assert_eq!(rig.cpu.fpr[3], 0x1234_5678);
    assert_eq!(rig.cpu.gpr[2], 0x1234_5678);
}

#[test]
fn in_full_mode_a_word_write_leaves_the_upper_half_standing() {
    let rig = prepared(FULL_MODE, 3, |f| f[0] = 0x0000_1111_2222_3333, Asm::new().lui(1, 0x3333).ori(1, 1, 0x2222).mtc1(1, 0));
    assert_eq!(rig.cpu.fpr[0], 0x0000_1111_3333_2222);
}

#[test]
fn in_full_mode_a_word_read_takes_the_lower_half() {
    let rig = prepared(FULL_MODE, 1, |f| f[1] = 0x4444_5555_6666_7777, Asm::new().mfc1(2, 1));
    assert_eq!(rig.cpu.gpr[2] as u32, 0x6666_7777);
}

#[test]
fn in_half_mode_an_odd_index_reads_the_upper_half_of_its_pair() {
    let rows: [(u32, u32); 4] = [(0, 0x2222_3333), (1, 0x0000_1111), (2, 0xAAAA_BBBB), (3, 0x8888_9999)];
    for (index, expected) in rows {
        let rig = prepared(
            HALF_MODE,
            1,
            |f| {
                f[0] = 0x0000_1111_2222_3333;
                f[2] = 0x8888_9999_AAAA_BBBB;
            },
            Asm::new().mfc1(2, index),
        );
        assert_eq!(rig.cpu.gpr[2] as u32, expected, "index {index}");
    }
}

#[test]
fn in_half_mode_two_word_writes_fill_one_register_between_them() {
    let rig = prepared(HALF_MODE, 6, |_| {}, Asm::new().lui(1, 0x3333).ori(1, 1, 0x2222).mtc1(1, 0).lui(1, 0x7777).ori(1, 1, 0x6666).mtc1(1, 1));
    assert_eq!(rig.cpu.fpr[0], 0x7777_6666_3333_2222);
    assert_eq!(rig.cpu.fpr[1], 0);
}

#[test]
fn in_half_mode_a_wide_write_to_an_odd_index_lands_on_the_even_register() {
    let rig = prepared(HALF_MODE, 3, |f| f[5] = 0xDEAD_BEEF_DEAD_BEEF, Asm::new().lui(1, 0x0123).ori(1, 1, 0x4567).dmtc1(1, 5));
    assert_eq!(rig.cpu.fpr[4], 0x0123_4567);
    assert_eq!(rig.cpu.fpr[5], 0xDEAD_BEEF_DEAD_BEEF);
}

#[test]
fn in_half_mode_a_wide_read_of_an_odd_index_returns_the_even_register() {
    let rig = prepared(HALF_MODE, 1, |f| f[4] = 0x0011_0011_2233_2233, Asm::new().dmfc1(2, 5));
    assert_eq!(rig.cpu.gpr[2], 0x0011_0011_2233_2233);
}

#[test]
fn a_program_can_change_the_register_file_mode_for_itself() {
    let rig = prepared(FULL_MODE, 4, |f| f[0] = 0x0000_1111_2222_3333, Asm::new().lui(8, 0x2000).mtc0(8, STATUS).nop().mfc1(2, 1));
    assert_eq!(rig.cpu.gpr[2] as u32, 0x0000_1111);
}

#[test]
fn the_implementation_register_reads_its_fixed_value_and_refuses_every_write() {
    let rig = run(
        FULL_MODE,
        4,
        Asm::new().cfc1(2, IMPLEMENTATION_REGISTER).lui(1, 0x0123).ctc1(1, IMPLEMENTATION_REGISTER).cfc1(3, IMPLEMENTATION_REGISTER),
    );
    assert_eq!(rig.cpu.gpr[2] as u32, FPU_IMPLEMENTATION);
    assert_eq!(rig.cpu.gpr[3] as u32, FPU_IMPLEMENTATION);
}

#[test]
fn the_control_word_drops_the_bits_that_are_not_writable() {
    for (written, expected) in [(0xFFFD_F07Fu32, 0x0181_F07Fu32), (0xFFFC_0F83, 0x0180_0F83)] {
        let rig = run(FULL_MODE, 3, Asm::new().lui(1, (written >> 16) as u16).ori(1, 1, written as u16).ctc1(1, CONTROL_STATUS_REGISTER));
        assert_eq!(rig.cpu.fcsr, expected, "{written:08X}");
        assert_eq!(rig.last_code(), None, "{written:08X}");
    }
}

#[test]
fn a_cause_without_its_enable_is_recorded_and_not_raised() {
    let rig = run(FULL_MODE, 3, Asm::new().lui(1, 0x0000).ori(1, 1, 0x4000).ctc1(1, CONTROL_STATUS_REGISTER));
    assert_eq!(rig.cpu.fcsr, 0x4000);
    assert_eq!(rig.last_code(), None);
}

#[test]
fn a_cause_written_beside_its_enable_raises_a_floating_point_exception() {
    let rig = run(FULL_MODE, 3, Asm::new().lui(1, 0x0000).ori(1, 1, 0x4200).ctc1(1, CONTROL_STATUS_REGISTER));
    assert_eq!(rig.last_code(), Some(code::FLOATING_POINT));
    assert_eq!(rig.cpu.cop0[CAUSE] & 0x7C, (code::FLOATING_POINT as u64) << 2);
    assert_eq!(rig.cpu.cop0[EXCEPTION_PC], ENTRY_POINT + 8);
}

#[test]
fn the_control_word_a_raising_write_left_behind_is_the_one_that_was_written() {
    let rig = run(FULL_MODE, 3, Asm::new().lui(1, 0x0000).ori(1, 1, 0x4200).ctc1(1, CONTROL_STATUS_REGISTER));
    assert_eq!(rig.cpu.fcsr, 0x4200);
}

#[test]
fn an_exception_that_names_no_coprocessor_leaves_the_cause_field_at_zero() {
    let rig = Asm::new().syscall().run_new(1);
    assert_eq!(rig.cpu.cop0[CAUSE] & CAUSE_COPROCESSOR, 0);
}
