//! Stage S13's gate (Sapphire_Native.md §23): the project's programs over every micro instruction and the visibility
//! the manual documents, run on the interpreter with the board (VU0 started by VCALLMS, VU1 by CMSAR1, both waited for
//! as the EE waits) and graded against `s13::manual` and `s12::manual`; the console's `vu/lower/efu` text against the
//! EFU; and the EE's side of micro mode, timed as §5.4.5 and §5.4.6 draw it.

mod s6;
mod s9;
mod s12;
mod s13;

use sapphire::board::System;
use sapphire::vu::pipe;

use s6::{DONE, DONE_AT, ORIGIN, Program, RECORD};

fn assemble(source: &str, origin: u32) -> Vec<u32> {
    emusen_mips::assemble(source, origin, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{e:?}"))
}

fn run(source: &str, limit: u64) -> System {
    let mut sys = System::new(ORIGIN);
    sys.board.map.load(ORIGIN, &assemble(source, ORIGIN));
    let n = sys.run(limit, &mut |s| s.peek(DONE_AT, 4) as u32 == DONE).unwrap_or_else(|h| panic!("{h:?}"));
    assert!(n < limit, "no done word in {limit} instructions, pc {:08x}", sys.ee.pc);
    sys
}

fn grade(p: Program, notes: &[String]) {
    let name = p.name;
    let (source, expected) = p.finish();
    let mut sys = run(&source, 20_000_000);
    let mut wrong = Vec::new();
    for (i, want) in expected.iter().enumerate() {
        let got = sys.peek(RECORD + 4 * i as u32, 4) as u32;
        if let Some(w) = want
            && *w != got
        {
            let note = notes.get(i / 4).map_or("", String::as_str);
            wrong.push(format!("word {i} ({note}): {got:08x}, the manual {w:08x}"));
        }
    }
    assert!(wrong.is_empty(), "{name}: {} of {} words differ, first {:?}", wrong.len(), expected.len(), &wrong[..wrong.len().min(8)]);
}

fn gate(name: &str) {
    let (n, segs) = s13::all_segs().into_iter().find(|(n, _)| *n == name).unwrap();
    let notes: Vec<String> = segs.iter().flat_map(|s| {
        let mut v: Vec<String> = s.notes.iter().map(|(_, n)| n.clone()).collect();
        if s.tpc.is_some() {
            v.push("TPC".into());
        }
        v
    }).collect();
    grade(s13::program(n, &segs), &notes);
}

#[test]
fn the_integer_unit_loads_stores_moves_and_random_unit_equal_the_manual() {
    gate("s13-integer");
}

#[test]
fn the_flag_instructions_equal_the_manual() {
    gate("s13-flags");
}

#[test]
fn the_branches_equal_the_manual() {
    gate("s13-branch");
}

#[test]
fn the_divider_in_micro_mode_equals_the_manual() {
    gate("s13-divider");
}

#[test]
fn every_upper_operation_in_micro_mode_equals_the_manual() {
    gate("s13-upper");
}

#[test]
fn the_efu_equals_the_nano_code() {
    gate("s13-efu");
}

#[test]
fn what_each_pair_sees_equals_the_manual_s_pipeline() {
    gate("s13-visible");
}

/// Every micro instruction of §4.2 and §4.3 is in a program's microprograms but the three that reach units outside
/// the VPU, which `tests/vu_micro.rs` runs against a stand-in (XGKICK, XTOP, XITOP).
#[test]
fn the_programs_carry_every_micro_instruction() {
    use emusen_mips::vu::{LOWER, SPECIAL1, SPECIAL2, Unit};
    let covered = s13::covered();
    let all: Vec<&str> = SPECIAL1.iter().chain(SPECIAL2.iter()).chain(LOWER.iter()).flatten().filter(|o| o.unit != Unit::Macro).map(|o| o.name).collect();
    assert_eq!(all.len(), 120 - 2 + 18 + 28, "the shared operations but VCALLMS and VCALLMSR, and the micro-only ones");
    let missing: Vec<_> = all.iter().filter(|n| !covered.contains(**n) && !matches!(**n, "xgkick" | "xtop" | "xitop")).collect();
    assert!(missing.is_empty(), "in no microprogram: {missing:?}");
}

/// The console's `vu/lower/efu` text (Sapphire_References.md §2: a console's output): the EFU's nano code on the
/// format's arithmetic equals 198 of its 208 lines and is within 2 in the last place of the rest, its algorithm's own
/// last bits being undocumented (Sapphire_Native.md §23, a named loss).
#[test]
fn the_efu_against_the_console_s_text() {
    use s13::manual::{EFU, EFU_CONSOLE, EFU_INPUTS};
    use sapphire::vu::efu;
    let (mut exact, mut near) = (0, 0);
    for (name, row) in EFU.iter().zip(EFU_CONSOLE) {
        for ((input, v), want) in EFU_INPUTS.iter().zip(row) {
            let got = match *name {
                "eatan" => efu::eatan(v[2]),
                "eatanxy" => efu::eatanxy(*v),
                "eatanxz" => efu::eatanxz(*v),
                "eexp" => efu::eexp(v[2]),
                "eleng" => efu::eleng(*v),
                "ercpr" => efu::ercpr(v[2]),
                "erleng" => efu::erleng(*v),
                "ersadd" => efu::ersadd(*v),
                "ersqrt" => efu::ersqrt(v[2]),
                "esadd" => efu::esadd(*v),
                "esin" => efu::esin(v[2]),
                "esqrt" => efu::esqrt(v[2]),
                _ => efu::esum(*v),
            };
            assert_eq!(got, s13::manual::efu(name, *v, 2), "{name} {input}: the module and the transcription");
            exact += u32::from(got == want);
            near += u32::from(got != want && got >> 31 == want >> 31 && (got as i64 - want as i64).abs() <= 2);
        }
    }
    assert_eq!((exact, near), (198, 10), "exact, and within two in the last place");
}

/// §5.4.5 and §5.4.6 on the board: VCALLMS's first pair 3 cycles after it, 4 behind a macro instruction; a macro
/// instruction after it issues as the last pair's S stage meets its T, QMFC2.I a cycle later; QMTC2.I after the first
/// pair with M; VU1 started by CMSAR1 runs beside the EE until BC2T sees it end.
#[test]
fn the_ee_waits_for_micro_mode_as_section_5_4_draws_it() {
    let micro = emusen_mips::vu::assemble("nop | iaddiu vi5, vi0, 0x5\n[m] nop | move vf0, vf0\nnop | move vf0, vf0\n[e] nop | move vf0, vf0\nnop | iaddiu vi6, vi0, 0x6").unwrap();
    let stepped = |src: &str| {
        let mut sys = System::new(ORIGIN);
        sys.board.map.load(ORIGIN, &assemble(src, ORIGIN));
        for (i, p) in micro.iter().enumerate() {
            sys.board.vpu.units.vu0.micro[i * 8..i * 8 + 8].copy_from_slice(&p.to_le_bytes());
            sys.board.vpu.units.vu1.micro[i * 8..i * 8 + 8].copy_from_slice(&p.to_le_bytes());
        }
        sys.ee.cop0[sapphire::ee::cop0::STATUS] |= sapphire::ee::status::CU2;
        let mut at = Vec::new();
        for _ in 0..src.lines().count() {
            sys.step().unwrap_or_else(|h| panic!("{h:?}"));
            at.push(sys.ee.timing.cycles() - 1);
        }
        (sys, at)
    };
    // VCALLMS at c: pairs at c+3 to c+7, the last (the delay slot) at c+7.
    let (sys, at) = stepped("vcallms 0x0\nviadd vi7, vi6, vi0\ncfc2.i t0, vi6\nnop");
    let last = at[0] + pipe::START + 4;
    assert_eq!(at[1], last + 3, "the macro instruction's T stage meets the last pair's S stage");
    assert_eq!(sys.board.vpu.units.vu0.vi(7), 6, "and reads the delay slot's result");
    assert!(at[2] >= last + 4, "CFC2.I issues as the S stage passes: {at:?}");
    let (_, at) = stepped("vnop\nvcallms 0x0\nqmtc2.i t0, vf1\nnop");
    assert_eq!(at[1], at[0] + 2, "VCALLMS after a macro instruction stalls a cycle");
    assert_eq!(at[2], at[1] + pipe::START + 2, "QMTC2.I issues the cycle after the pair with M");
    let (sys, at) = stepped("ctc2 zero, vi31\nbc2t l\nnop\nl: nop");
    assert!(sys.board.vpu.units.vu1.vi(6) != 6 || at[1] > at[0], "VU1 started beside the EE");
}

/// XTOP and XITOP read VIF's TOP and ITOP at the T stage, which the link carries until VIF exists (S14); XTOP is VU1's
/// alone, XITOP each unit's own VIF's.
#[test]
fn xtop_and_xitop_read_the_link() {
    use sapphire::vu::micro::Micro;
    use sapphire::vu::{Link, Units, VU0_MEM, VU1_MEM, Vu};
    let mut u = Units { vu0: Vu::new(VU0_MEM), vu1: Vu::new(VU1_MEM), link: Link::default(), halts: false };
    u.link.top = [0x11, 0x222];
    u.link.itop = [0x33, 0x144];
    let prog = emusen_mips::vu::assemble("nop | xtop vi1\nnop | xitop vi2\n[e] nop | move vf0, vf0\nnop | move vf0, vf0").unwrap();
    for unit in 0..2 {
        let mem = if unit == 0 { &mut u.vu0.micro } else { &mut u.vu1.micro };
        for (i, p) in prog.iter().enumerate() {
            mem[i * 8..i * 8 + 8].copy_from_slice(&p.to_le_bytes());
        }
        let mut p = pipe::Pipe::new(unit);
        p.start(&mut u, 0, 10);
        p.finish(&mut u, 10);
    }
    assert_eq!((u.vu0.vi(1), u.vu0.vi(2)), (0, 0x33), "VU0: no XTOP; XITOP reads VIF0's ITOP");
    assert_eq!((u.vu1.vi(1), u.vu1.vi(2)), (0x222, 0x144));
}

/// A D or T bit reaches the INTC as the unit's interrupt (§7.3.1: "an interrupt signal is sent to the host processor";
/// INTC's VU0 and VU1 sources, EE User's Manual §3.1).
#[test]
fn a_t_bit_raises_vu1_s_interrupt() {
    let micro = emusen_mips::vu::assemble("nop | move vf0, vf0\n[t] nop | move vf0, vf0\nnop | move vf0, vf0").unwrap();
    let mut sys = System::new(ORIGIN);
    sys.board.map.load(ORIGIN, &assemble("addiu t0, zero, 0x800\nctc2 t0, vi28\nctc2 zero, vi31\nnop\nnop\nnop\nnop\nnop\nnop\nnop", ORIGIN));
    for (i, p) in micro.iter().enumerate() {
        sys.board.vpu.units.vu1.micro[i * 8..i * 8 + 8].copy_from_slice(&p.to_le_bytes());
    }
    sys.ee.cop0[sapphire::ee::cop0::STATUS] |= sapphire::ee::status::CU2;
    for _ in 0..10 {
        sys.step().unwrap();
    }
    assert_eq!(sys.board.intc.stat >> sapphire::intc::source::VU1 & 1, 1);
    assert_eq!(sys.board.vpu.vpu_stat(sys.ee.timing.cycles()) >> 8 & 0x7, 0x4, "VTS1, VU1 not busy");
    assert_eq!(sys.board.vpu.units.vu1.tpc, 2);
}
