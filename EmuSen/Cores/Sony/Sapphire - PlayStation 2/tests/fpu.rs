//! Stage S9's gate (Sapphire_Native.md §19.4): the project's programs over every COP1 instruction and every class of
//! input, run on the interpreter and graded against chapter 5 of the EE Core Instruction Set Manual as `s9::manual`
//! transcribes it; the format's module against lines of ps2autotests' console texts that decide its readings; the
//! floating-point rows of the timing model on sequences whose cycles follow from the EE Core User's Manual by hand.

mod s6;
mod s9;

use sapphire::ee::{Ee, bus::Ram};
use sapphire::float;

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
fn add_and_subtract_over_every_pair_of_input_classes_equal_the_manual() {
    grade(s9::add());
}

#[test]
fn multiply_and_divide_over_every_pair_of_input_classes_equal_the_manual() {
    grade(s9::muldiv());
}

#[test]
fn the_accumulator_instructions_equal_the_manual() {
    grade(s9::accumulator());
}

#[test]
fn square_root_and_reciprocal_square_root_equal_the_manual() {
    grade(s9::roots());
}

#[test]
fn moves_conversions_picks_compares_loads_stores_and_branches_equal_the_manual() {
    grade(s9::misc());
}

#[test]
fn cop1_exceptions_record_cause_epc_and_badvaddr_as_chapter_4_says() {
    grade(s9::exceptions());
}

#[test]
fn the_programs_execute_every_cop1_instruction_the_interpreter_has() {
    let mut covered = std::collections::BTreeSet::new();
    for p in s9::all() {
        let (source, _) = p.finish();
        covered.extend(s6::covered(&source));
    }
    let cop1: Vec<&str> = sapphire::ee::names().skip(155 + 91).take(34).collect();
    assert_eq!(cop1.len(), 34, "{cop1:?}");
    let missing: Vec<_> = cop1.iter().filter(|n| !covered.contains(**n)).collect();
    assert!(missing.is_empty(), "not executed by any program: {missing:?}");
}

/// Lines of ps2autotests' `cpu/ee_fpu` console texts (consoles' outputs, Sapphire_References.md §2) that decide a
/// reading of the format: (operation, accumulator, fs, ft, result).
const CONSOLE: &[(&str, u32, u32, u32, u32)] = &[
    ("add", 0, 0xFFFF_FFFF, 0x3F80_0000, 0xFFFF_FFFF),
    ("sub", 0, 0x7FFF_FFFF, 0x3F80_0000, 0x7FFF_FFFF),
    ("add", 0, 0x7FFF_FFFF, 0x7FFF_FFFF, 0x7FFF_FFFF),
    ("add", 0, 0x7FFF_FFFF, 0xFFFF_FFFF, 0x0000_0000),
    ("add", 0, 0x0000_1337, 0xDEAD_BEEF, 0xDEAD_BEEF),
    ("add", 0, 0x3FFF_FFFF, 0x3F80_0000, 0x403F_FFFF),
    ("mul", 0, 0x3F80_0000, 0x3FFF_FFFF, 0x3FFF_FFFE),
    ("mul", 0, 0x3FFF_FFFF, 0x3F80_0000, 0x3FFF_FFFF),
    ("mul", 0, 0x3F80_0000, 0x7FFF_FFFF, 0x7FFF_FFFE),
    ("mul", 0, 0x3F80_0000, 0xFFFF_FFFF, 0xFFFF_FFFE),
    ("mul", 0, 0x7F80_0000, 0x7F80_0000, 0x7FFF_FFFF),
    ("div", 0, 0x3F80_0000, 0x3FFF_FFFF, 0x3F00_0001),
    ("div", 0, 0x0000_0000, 0x8000_0000, 0xFFFF_FFFF),
    ("div", 0, 0x3F80_0000, 0x0000_0001, 0x7FFF_FFFF),
    ("sqrt", 0, 0, 0x3FFF_FFFF, 0x3FB5_04F3),
    ("sqrt", 0, 0, 0x7FFF_FFFF, 0x5FB5_04F3),
    ("sqrt", 0, 0, 0xDEAD_BEEF, 0x4F15_2108),
    ("sqrt", 0, 0, 0x8000_0000, 0x0000_0000),
    ("rsqrt", 0, 0x7FFF_FFFF, 0x7FFF_FFFF, 0x5FB5_04F2),
    ("rsqrt", 0, 0x0000_0000, 0x8000_0000, 0x7FFF_FFFF),
    ("rsqrt", 0, 0x8000_0000, 0x0000_0000, 0xFFFF_FFFF),
    ("rsqrt", 0, 0x3F80_0000, 0x3FFF_FFFF, 0x3F35_04F3),
    ("rsqrt", 0, 0x3F80_0000, 0x7F80_0001, 0x1F80_0000),
    ("madd", 0x8000_0000, 0x8000_0000, 0x8000_0000, 0x0000_0000),
    ("madd", 0x0000_0000, 0x7F80_0001, 0x3F80_0000, 0x7F80_0001),
    ("madd", 0xFFFF_FFFF, 0x7FFF_FFFF, 0x7FFF_FFFF, 0x7FFF_FFFF),
    ("madd", 0x0000_0000, 0x3F80_0000, 0x3FFF_FFFF, 0x3FFF_FFFE),
    ("msub", 0x0000_0000, 0x7FFF_FFFF, 0x7FFF_FFFF, 0xFFFF_FFFF),
    ("msub", 0x8000_0000, 0x0000_0000, 0x8000_0000, 0x0000_0000),
    ("max", 0, 0x8000_0000, 0x0000_0000, 0x0000_0000),
    ("min", 0, 0x0000_0000, 0x8000_0000, 0x8000_0000),
    ("min", 0, 0x0000_0001, 0x3F80_0000, 0x0000_0001),
    ("cvt.w.s", 0, 0x4F00_0000, 0, 0x7FFF_FFFF),
    ("cvt.w.s", 0, 0xBFC0_0000, 0, 0xFFFF_FFFF),
    ("cvt.s.w", 0, 0x7FFF_FFFF, 0, 0x4EFF_FFFF),
    ("cvt.s.w", 0, 0x4EFF_FFFF, 0, 0x4E9D_FFFF),
];

#[test]
fn the_format_reproduces_the_console_lines_that_decide_its_readings() {
    for &(op, acc, a, b, want) in CONSOLE {
        let got = match op {
            "add" => float::add(a, b).bits,
            "sub" => float::sub(a, b).bits,
            "mul" => float::mul(a, b).bits,
            "div" => float::div(a, b).value.bits,
            "sqrt" => float::sqrt(b).bits,
            "rsqrt" => float::rsqrt(a, b).value.bits,
            "madd" | "msub" => float::fused(acc, a, b, op == "msub").value.bits,
            "max" => float::max(a, b),
            "min" => float::min(a, b),
            "cvt.w.s" => float::to_word(a),
            "cvt.s.w" => float::from_word(a),
            _ => unreachable!(),
        };
        assert_eq!(got, want, "{op} ({acc:08x}) {a:08x}, {b:08x}: {got:08x}, the console {want:08x}");
    }
    assert_eq!(float::order(0x0000_0000), float::order(0x0000_0001), "eq 00000000, 00000001: true");
    assert!(float::order(0x7F80_0000) < float::order(0x7F80_0001), "lt 7f800000, 7f800001: true");
}

/// The cycles a run of COP1 code takes on the interpreter, warm, from its first instruction to its last.
fn cycles(body: &str) -> u64 {
    let text = format!("mfc0 t0, Status\nlui t1, 0x2000\nor t0, t0, t1\nmtc0 t0, Status\n{body}\n{body}\nnop\nnop");
    let words = assemble(&text, ORIGIN);
    let n = body.lines().count() as u64;
    let mut ram = Ram::new();
    ram.load(ORIGIN, &words);
    let mut ee = Ee::new(ORIGIN);
    for _ in 0..4 + n {
        ee.step(&mut ram).expect("runs");
    }
    let start = ee.timing.cycles();
    for _ in 0..n {
        ee.step(&mut ram).expect("runs");
    }
    ee.timing.cycles() - start
}

#[test]
fn the_floating_point_rows_give_their_latencies_and_repeat_rates() {
    // §2.12: ADD.S 4 to its result, one issue a cycle; a dependent ADD.S waits for it.
    assert_eq!(cycles("add.s f1, f2, f3\nadd.s f4, f5, f6"), 2);
    assert_eq!(cycles("add.s f1, f2, f3\nadd.s f4, f1, f6"), 5);
    assert_eq!(cycles("mul.s f1, f2, f3\nmul.s f4, f1, f6"), 5);
    // DIV.S 8 to its result and 7 between issues, SQRT.S on the same divider, RSQRT.S 14 and 13: a body repeated
    // runs at the divider's rate, so two divides take 14 cycles a pass, a divide and an independent add 7.
    assert_eq!(cycles("div.s f1, f2, f3\ndiv.s f4, f5, f6"), 14);
    assert_eq!(cycles("div.s f1, f2, f3\nsqrt.s f4, f6"), 14);
    assert_eq!(cycles("div.s f1, f2, f3\nadd.s f4, f5, f6"), 7);
    assert_eq!(cycles("div.s f1, f2, f3\nadd.s f4, f1, f6"), 9);
    assert_eq!(cycles("rsqrt.s f1, f2, f3\nrsqrt.s f4, f5, f6"), 26);
    // MTC1 2 to the FPR (the add beside the next MTC1 in a Y cell), MFC1 2 to the GPR (the next MFC1 beside the
    // addu), LWC1 2.
    assert_eq!(cycles("mtc1 t0, f1\nadd.s f2, f1, f1"), 3);
    assert_eq!(cycles("mfc1 t2, f1\naddu t3, t2, t2"), 2);
    assert_eq!(cycles("lwc1 f1, 0(zero)\nadd.s f2, f1, f1"), 3);
    // A COP1 move beside a COP1 operation is Table 1-3's Y cell: both issue, the next a cycle later.
    assert_eq!(cycles("add.s f1, f2, f3\nmtc1 t0, f4\naddu t3, t4, t5"), 3);
    // C.EQ.S writes C at the add row's latency; BC1T reads it.
    assert_eq!(cycles("c.eq.s f1, f2\nbc1f 0x100100\nnop"), 5);
}
