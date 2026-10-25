//! Stage S8's gate (Sapphire_Native.md §18.5): the project's programs over every multimedia encoding, run on the
//! interpreter and graded against the EE Core Instruction Set Manual's chapter 3; the multimedia rows of the timing
//! model pinned by sequences whose cycles follow from the EE Core User's Manual by hand.

mod s6;
mod s8;

use sapphire::ee::{self, Ee, bus::Ram};

use s6::{DONE, DONE_AT, ORIGIN, Program, RECORD};

fn assemble(source: &str, origin: u32) -> Vec<u32> {
    emusen_mips::assemble(source, origin, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{e:?}"))
}

fn run(source: &str, limit: u64) -> (Ee, Ram) {
    let mut ram = Ram::new();
    ram.load(ORIGIN, &assemble(source, ORIGIN));
    let mut ee = Ee::new(ORIGIN);
    let n = ee.run(&mut ram, limit, &mut |e, b| e.peek(b, DONE_AT, 4) as u32 == DONE).unwrap_or_else(|h| panic!("{h:?}"));
    assert!(n < limit, "no done word in {limit} instructions, pc {:08x}", ee.pc);
    (ee, ram)
}

fn grade(p: Program) {
    let name = p.name;
    let (source, expected) = p.finish();
    let (ee, mut ram) = run(&source, 20_000_000);
    let mut wrong = Vec::new();
    for (i, want) in expected.iter().enumerate() {
        let got = ee.peek(&mut ram, RECORD + 4 * i as u32, 4) as u32;
        if let Some(w) = want
            && *w != got
        {
            wrong.push(format!("word {i} ({:08x}): {got:08x}, the manual {w:08x}", RECORD + 4 * i as u32));
        }
    }
    assert!(wrong.is_empty(), "{name}: {} of {} words differ, first {:?}", wrong.len(), expected.len(), &wrong[..wrong.len().min(8)]);
}

#[test]
fn parallel_add_and_subtract_wrap_and_saturate_as_the_manual_says() {
    grade(s8::arithmetic());
}

#[test]
fn parallel_compare_maximum_minimum_and_absolute_value_equal_the_manual() {
    grade(s8::compare());
}

#[test]
fn logic_interleave_pack_and_rearrangement_equal_the_manual() {
    grade(s8::lanes());
}

#[test]
fn parallel_shifts_take_each_lane_widths_amount_as_the_manual_says() {
    grade(s8::shifts());
}

#[test]
fn moves_to_and_from_hi_and_lo_equal_the_manual() {
    grade(s8::hilo_moves());
}

#[test]
fn parallel_multiplies_write_gpr_rd_hi_and_lo_as_the_manual_says() {
    grade(s8::multiplies());
}

#[test]
fn parallel_divides_equal_the_manual() {
    grade(s8::divides());
}

#[test]
fn plzcw_and_qfsrv_equal_the_manual() {
    grade(s8::funnel_and_count());
}

#[test]
fn the_undefined_multimedia_inputs_give_the_interpreters_recorded_reading() {
    grade(s8::undefined());
}

#[test]
fn the_programs_execute_every_multimedia_encoding() {
    let seen: std::collections::BTreeSet<String> = s8::all().into_iter().flat_map(|p| s6::covered(&p.finish().0)).collect();
    let s8: Vec<_> = ee::names().skip(155).collect();
    assert_eq!(s8.len(), 91, "the multimedia instructions, PLZCW and QFSRV");
    let missing: Vec<_> = s8.iter().filter(|n| !seen.contains(**n)).collect();
    assert!(missing.is_empty(), "never executed: {missing:?}");
    let decoder: std::collections::BTreeSet<String> = (0..64u32 * 32)
        .flat_map(|k| [0x7000_0000 | (k >> 5) | (k & 31) << 6])
        .filter_map(|w| emusen_mips::r5900::operation(w).map(|o| o.mnemonic()))
        .collect();
    let named: std::collections::BTreeSet<String> = s8.iter().map(|n| n.to_string()).collect();
    let mmi_new: std::collections::BTreeSet<String> = decoder.into_iter().filter(|n| !ee::names().take(155).any(|o| o == n)).collect();
    assert_eq!(mmi_new, named, "the interpreter's S8 set is the decoder's MMI space less S6's entries");
}

// The timing model's multimedia rows (EE Core User's Manual §1.3.5, §1.3.6, §2.12).

fn machine(code: &str) -> (Ee, Ram) {
    let mut ram = Ram::new();
    ram.load(ORIGIN, &assemble(code, ORIGIN));
    (Ee::new(ORIGIN), ram)
}

fn steps(ee: &mut Ee, ram: &mut Ram, n: usize) {
    for _ in 0..n {
        ee.step(ram).unwrap();
    }
}

/// The cycles the code takes with both caches warm, as §2.12's table assumes: it runs once first.
fn cycles(code: &str) -> u64 {
    let n = code.lines().filter(|l| !l.trim().is_empty()).count();
    let (mut warm, mut ram) = machine(code);
    steps(&mut warm, &mut ram, n);
    let mut ee = Ee::new(ORIGIN);
    std::mem::swap(&mut ee.dcache, &mut warm.dcache);
    std::mem::swap(&mut ee.icache, &mut warm.icache);
    steps(&mut ee, &mut ram, n);
    ee.timing.cycles()
}

#[test]
fn wide_operations_issue_to_pipe_0_alone() {
    assert_eq!(cycles("paddw t0, t1, t2\npaddw t3, t4, t5"), 2, "two Wide Operate instructions cannot pair");
    assert_eq!(cycles("paddw t0, t1, t2\nlq t3, 0(s0)"), 1, "a load pairs beside one");
    assert_eq!(cycles("paddw t0, t1, t2\nlq t3, 0(s0)\naddu t6, t7, t8"), 2);
    assert_eq!(cycles("paddw t0, t1, t2\naddu t3, t4, t5\naddu t6, t7, t8"), 3, "Table 1-3's Y: ALU beside Wide Operate stalls a cycle");
    assert_eq!(cycles("paddw t0, t1, t2\nplzcw t3, t4\naddu t6, t7, t8"), 3, "LZC beside Wide Operate is Y too");
    assert_eq!(cycles("paddw t0, t1, t2\npaddw t3, t0, t5"), 2, "a dependent one a cycle later, latency 1");
}

#[test]
fn the_multimedia_multiply_takes_both_multipliers_four_cycles_and_two_between_issues() {
    assert_eq!(cycles("pmulth t0, t1, t2\npmfhi t3"), 5);
    assert_eq!(cycles("pmulth t0, t1, t2\npor t3, t0, zero"), 5, "GPR[rd] too is four cycles away");
    assert_eq!(cycles("pmulth t0, t1, t2\npmulth t3, t4, t5"), 3);
    assert_eq!(cycles("mult1 t0, t1, t2\npmultw t3, t4, t5"), 3, "MAC1 is busy for two cycles after MULT1");
    assert_eq!(cycles("pmaddh t0, t1, t2\npmaddh t3, t4, t5"), 3, "accumulation forwards at the throughput, as MADD's does");
}

#[test]
fn the_multimedia_divide_takes_both_dividers_thirty_seven_cycles() {
    assert_eq!(cycles("pdivw t0, t1\npmflo t2"), 38);
    assert_eq!(cycles("pdivw t0, t1\ndiv1 t2, t3\nmflo1 t4"), 75, "DIV1 waits for the divider PDIVW holds");
    assert_eq!(cycles("pdivbw t0, t1\nlw t2, 0(s0)\naddu t5, t6, t7"), 2, "other instructions run beside it");
}
