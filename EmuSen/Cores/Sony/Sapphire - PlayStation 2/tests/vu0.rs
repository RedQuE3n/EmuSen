//! Stage S12's gate (Sapphire_Native.md §21.4): the project's programs over every macro instruction and transfer, run on
//! the interpreter with the board and graded against the VU User's Manual as `s12::manual` transcribes it; the
//! interlocks with VU0's micro mode on a scripted stand-in for S13's; the macro timing on sequences whose cycles follow
//! from §5.4's figures; and the console lines and the manual's own examples that decide a reading.

mod s6;
mod s9;
mod s12;

use sapphire::board::System;
use sapphire::ee::Halt;
use sapphire::vu::micro::Micro;
use sapphire::vu::{Units, ops};

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

fn grade(p: Program) {
    let name = p.name;
    let (source, expected) = p.finish();
    let mut sys = run(&source, 60_000_000);
    let mut wrong = Vec::new();
    for (i, want) in expected.iter().enumerate() {
        let got = sys.peek(RECORD + 4 * i as u32, 4) as u32;
        if let Some(w) = want
            && *w != got
        {
            wrong.push(format!("word {i} (record {}): {got:08x}, the manual {w:08x}", i / 2));
        }
    }
    assert!(wrong.is_empty(), "{name}: {} of {} words differ, first {:?}", wrong.len(), expected.len(), &wrong[..wrong.len().min(8)]);
}

#[test]
fn add_and_subtract_over_every_pair_of_input_classes_equal_the_manual() {
    grade(s12::add());
}

#[test]
fn multiply_over_every_pair_of_input_classes_equals_the_manual() {
    grade(s12::mul());
}

#[test]
fn the_multiply_adds_and_the_outer_product_equal_the_manual() {
    grade(s12::fused());
}

#[test]
fn the_flagless_instructions_and_clip_equal_the_manual() {
    grade(s12::misc());
}

#[test]
fn divide_square_root_and_reciprocal_root_equal_the_manual() {
    grade(s12::fdiv());
}

#[test]
fn the_integer_unit_memory_window_and_random_unit_equal_the_manual() {
    grade(s12::int());
}

#[test]
fn the_transfers_branches_and_exceptions_equal_the_manual() {
    grade(s12::transfer());
}

/// Every operation of the macro table and every COP2 transfer and branch, from the assembled words through the decoder;
/// VCALLMS and VCALLMSR, which start micro mode, are run by `vcallms_starts_micro_mode_and_the_ee_waits` instead.
#[test]
fn the_programs_execute_every_macro_instruction() {
    let mut covered = std::collections::BTreeSet::new();
    for p in s12::all() {
        let (source, _) = p.finish();
        covered.extend(s6::covered(&source));
    }
    let mut want: Vec<String> = ops::TABLE.iter().map(|e| format!("v{}", e.name)).collect();
    want.extend(["qmfc2", "qmtc2", "cfc2", "ctc2", "lqc2", "sqc2", "bc2f", "bc2t", "bc2fl", "bc2tl"].map(String::from));
    assert_eq!(want.len(), 120 + 10);
    let missing: Vec<_> = want.iter().filter(|n| !covered.contains(*n) && !n.starts_with("vcallms")).collect();
    assert!(missing.is_empty(), "not executed by any program: {missing:?}");
}

/// The manual's own examples (§4.2's FTOI and ITOF Remarks, §4.3's RGET range) and lines of ps2autotests' console texts
/// (consoles' outputs, Sapphire_References.md §2) that decide a reading.
#[test]
fn the_manual_s_examples_and_the_console_lines_hold() {
    use sapphire::float;
    let (m045, p045, p055, p12345) = (0xBEE6_6666u32, 0x3EE6_6666u32, 0x3F0C_CCCDu32, 0x42F6_E666u32);
    for (n, table) in [(0u32, [0i32, 0, 0, 123]), (4, [-7, 7, 8, 1975]), (12, [-1843, 1843, 2252, 505_651]), (15, [-14745, 14745, 18022, 4_045_209])] {
        for (x, want) in [m045, p045, p055, p12345].into_iter().zip(table) {
            assert_eq!(float::to_fixed(x, n) as i32, want, "ftoi{n} {x:08x}");
        }
    }
    // The Remarks print six places; the values are the exact quotients they round.
    for (n, table) in [(4u32, [-0.75f32, 0.0625, 7.6875, 115.1875]), (12, [-12.0 / 4096.0, 1.0 / 4096.0, 123.0 / 4096.0, 1843.0 / 4096.0])] {
        for (x, want) in [-12i32, 1, 123, 1843].into_iter().zip(table) {
            assert_eq!(f32::from_bits(float::from_fixed(x as u32, n)), want, "itof{n} {x}");
        }
    }
    // `vu/lower/random`: RNEXT's four steps from R as RINIT leaves it, and RXOR's chain (D-48).
    let mut r = 0x7F_FFFF;
    for want in [0x3FFF_FFFEu32, 0x3FFF_FFFC, 0x3FFF_FFF8, 0x3FFF_FFF0] {
        r = ops::next_r(r);
        assert_eq!(0x3F80_0000 | r, want);
    }
    for (seed, steps) in [
        (0x0123_4567u32, [0x3FC6_8ACEu32, 0x3F8D_159D, 0x3F9A_2B3B, 0x3FB4_5677]),
        (0x89AB_CDEF, [0x3FD7_9BDE, 0x3FAF_37BC, 0x3FDE_6F79, 0x3FBC_DEF2]),
        (0xFEDC_BA98, [0x3FB9_7530, 0x3FF2_EA61, 0x3FE5_D4C3, 0x3FCB_A987]),
        (0x7654_3210, [0x3FA8_6420, 0x3FD0_C840, 0x3FA1_9081, 0x3FC3_2102]),
    ] {
        let mut r = seed & 0x7F_FFFF;
        for want in steps {
            r = ops::next_r(r);
            assert_eq!(0x3F80_0000 | r, want, "from {seed:08x}");
        }
    }
    assert_eq!((0x0123_4567u32 ^ 0x89AB_CDEF) & 0x7F_FFFF | 0x3F80_0000, 0x3F88_8888, "RXOR's second line");
}

/// A stand-in for S13's micro mode: a microprogram that runs `length` cycles, sets M at `release` cycles, and at its
/// end writes VF05 and VI05; the EE's waits are read off what each instruction saw.
struct Scripted {
    started: Option<(u16, u64)>,
    length: u64,
    release: u64,
    starts: Vec<(u16, u64)>,
}

impl Scripted {
    fn new(length: u64, release: u64) -> Scripted {
        Scripted { started: None, length, release, starts: Vec::new() }
    }
    fn end(&mut self, units: &mut Units) -> Option<u64> {
        let (addr, at) = self.started.take()?;
        units.vu0.set_vf(5, [addr as u32, 2, 3, 4]);
        units.vu0.set_vi(5, 0x55);
        Some(at + self.length)
    }
}

impl Micro for Scripted {
    fn start(&mut self, _: &mut Units, addr: u16, at: u64) -> bool {
        self.started = Some((addr, at));
        self.starts.push((addr, at));
        true
    }
    fn run_to(&mut self, units: &mut Units, t: u64) {
        if self.started.is_some_and(|(_, at)| t >= at + self.length) {
            self.end(units);
        }
    }
    fn busy(&self) -> bool {
        self.started.is_some()
    }
    fn finish(&mut self, units: &mut Units, at: u64) -> u64 {
        self.end(units).unwrap_or(at).max(at)
    }
    fn release(&mut self, _: &mut Units, at: u64) -> u64 {
        self.started.map_or(at, |(_, s)| (s + self.release).max(at))
    }
    fn force_break(&mut self, _: &mut Units, _: u64) {
        self.started = None;
    }
    fn reset(&mut self) {
        self.started = None;
    }
    fn stat(&self) -> u32 {
        u32::from(self.started.is_some())
    }
}

/// A system entered at ORIGIN with COP2 usable, running `source` instruction by instruction; each step's issue cycle.
fn stepped(source: &str, micro: Option<Scripted>) -> (System, Vec<u64>) {
    let mut sys = System::new(ORIGIN);
    sys.board.map.load(ORIGIN, &assemble(source, ORIGIN));
    sys.ee.cop0[sapphire::ee::cop0::STATUS] |= sapphire::ee::status::CU2;
    if let Some(m) = micro {
        sys.board.vpu.micro0 = Box::new(m);
    }
    let n = source.lines().filter(|l| !l.trim().is_empty()).count();
    let mut at = Vec::new();
    for _ in 0..n {
        sys.step().unwrap_or_else(|h| panic!("{h:?}"));
        at.push(sys.ee.timing.cycles() - 1);
    }
    (sys, at)
}

#[test]
fn vcallms_starts_micro_mode_and_the_ee_waits() {
    // VCALLMS at 0x40, then QMFC2.NI reads VF05 at once (before the end), QMFC2.I waits for the end and reads it.
    let src = "vcallms 0x40\nqmfc2.ni t0, vf5\nqmfc2.i t1, vf5\nnop";
    let (sys, at) = stepped(src, Some(Scripted::new(30, 10)));
    assert_eq!(sys.ee.gpr[8] as u32, 0, "QMFC2.NI read before the microprogram's end");
    assert_eq!(sys.ee.gpr[9] as u32, 0x40, "QMFC2.I read the result");
    assert!(at[2] >= at[0] + 30, "QMFC2.I issued at {}, VCALLMS at {}", at[2], at[0]);
    assert!(at[1] <= at[0] + 2, "QMFC2.NI did not wait: {at:?}");
    // A macro instruction waits for the end as QMFC2.I does; CFC2.I too.
    let (sys, at) = stepped("vcallms 0x8\nviadd vi6, vi5, vi0\ncfc2 t0, vi6\nnop", Some(Scripted::new(25, 5)));
    assert!(at[1] >= at[0] + 25, "{at:?}");
    assert_eq!(sys.ee.gpr[8] as u32, 0x55, "the macro instruction saw the microprogram's VI05");
    // QMTC2.I waits for the pair with M set, QMTC2.NI does not; CTC2.I as QMTC2.I.
    let (_, at) = stepped("vcallms 0x8\nqmtc2.ni t0, vf6\nqmtc2.i t0, vf7\nctc2.i t0, vi7\nnop", Some(Scripted::new(40, 12)));
    assert!(at[1] < at[0] + 12 && at[2] >= at[0] + 12 && at[2] < at[0] + 40, "{at:?}");
    // VCALLMSR starts at CMSAR0; a second VCALLMS waits for the first to end.
    let (sys, at) = stepped("addiu t0, zero, 0x33\nctc2 t0, vi27\nvnop\nvnop\nvcallmsr vi27\nvcallms 0x9\nnop", Some(Scripted::new(20, 20)));
    assert!(at[5] >= at[4] + 20, "{at:?}");
    assert_eq!(sys.board.vpu.units.vu0.vf(5)[0], 0x33, "VCALLMSR started at CMSAR0");
}

#[test]
fn micro_mode_unbuilt_halts_the_interpreter_at_its_start() {
    let mut sys = System::new(ORIGIN);
    let words = assemble("vcallms 0x10\nnop", ORIGIN);
    sys.board.map.load(ORIGIN, &words);
    sys.ee.cop0[sapphire::ee::cop0::STATUS] |= sapphire::ee::status::CU2;
    assert_eq!(sys.step(), Err(Halt::Unimplemented { pc: ORIGIN, word: words[0] }), "VCALLMS is S13's");
    let words = assemble("addiu t0, zero, 0x100\nctc2 t0, vi31", ORIGIN);
    let mut sys = System::new(ORIGIN);
    sys.board.map.load(ORIGIN, &words);
    sys.ee.cop0[sapphire::ee::cop0::STATUS] |= sapphire::ee::status::CU2;
    sys.step().unwrap();
    assert_eq!(sys.step(), Err(Halt::Unimplemented { pc: ORIGIN + 4, word: words[1] }), "starting VU1 by CMSAR1 is S13's");
}

/// The macro timing of §5.4.2 to §5.4.4 and §5.4.7, each cycle count read off the figures.
#[test]
fn the_macro_timing_follows_section_5_4() {
    let gap = |src: &str, a: usize, b: usize| {
        let (_, at) = stepped(src, None);
        at[b] - at[a]
    };
    assert_eq!(gap("vadd.xyzw vf1, vf2, vf3\nvadd.xyzw vf4, vf1, vf1", 0, 1), 4, "an FMAC result to a macro instruction (§5.4.3)");
    assert_eq!(gap("vadd.xyzw vf1, vf2, vf3\nqmfc2 t0, vf1", 0, 1), 5, "to QMFC2 (§5.4.4: four Ts after the S stage)");
    assert_eq!(gap("vadd.xyzw vf1, vf2, vf3\nvadd.xyzw vf4, vf2, vf2", 0, 1), 1, "no hazard, no stall");
    assert_eq!(gap("vadd.x vf1, vf2, vf3\nvadd.y vf4, vf1, vf1", 0, 1), 4, "hazards by register number, not field (§5.4.2)");
    assert_eq!(gap("vadd.xyzw vf1, vf2, vf3\nviadd vi4, vi1, vi1", 0, 1), 4, "VF01 and VI01 are one register for the hazard (§5.4.2)");
    assert_eq!(gap("viadd vi1, vi2, vi3\nviadd vi4, vi1, vi1", 0, 1), 1, "an integer result the next cycle");
    assert_eq!(gap("qmtc2 t0, vf1\nvadd.xyzw vf4, vf2, vf2", 0, 1), 2, "the macro instruction after QMTC2 always stalls one cycle (§5.4.4)");
    assert_eq!(gap("qmtc2 t0, vf1\nqmfc2 t1, vf1", 0, 1), 3, "QMFC2 of QMTC2's register: two cycles (§5.4.4)");
    assert_eq!(gap("qmtc2 t0, vf1\nqmfc2 t1, vf2", 0, 1), 1, "no hazard between transfers");
    assert_eq!(gap("vdiv q, vf1x, vf2x\nvdiv q, vf1x, vf2x", 0, 1), 7, "the divider's resource hazard (§5.4.2)");
    assert_eq!(gap("vrsqrt q, vf1x, vf2x\nvsqrt q, vf2x", 0, 1), 13, "VRSQRT's 13");
    assert_eq!(gap("vdiv q, vf1x, vf2x\nvwaitq\nvaddq.x vf3, vf0, q", 0, 1), 7, "VWAITQ issues once Q lands (§5.4.7)");
    assert_eq!(gap("vdiv q, vf1x, vf2x\nvwaitq\nvaddq.x vf3, vf0, q", 0, 2), 8, "and the reader the cycle after");
    assert_eq!(gap("vadd.xyzw vf1, vf2, vf3\ncfc2 t0, vi16", 0, 1), 5, "CFC2 of the status flag waits for the S stage");
}

#[test]
fn the_board_maps_the_vu_memories_at_11000000h() {
    use sapphire::ee::Bus;
    let mut sys = System::new(ORIGIN);
    let b = &mut sys.board;
    b.write(0x1100_4010, 16, 0x1234);
    assert_eq!(b.vpu.units.vu0.mem_q(1), 0x1234);
    assert_eq!(b.read(0x1100_5010, 4), 0x1234, "VU0's data memory mirrors every 4 KiB to 11008000h");
    b.write(0x1100_0008, 8, 0xABCD);
    assert_eq!(b.vpu.units.vu0.micro[8], 0xCD);
    b.write(0x1100_FFF0, 16, 7);
    assert_eq!(b.vpu.units.vu1.mem_q(0x3FF), 7, "VU1's data memory is 16 KiB");
    b.write(0x1100_8000, 4, 9);
    assert_eq!(b.vpu.units.vu1.micro[0], 9);
}

#[test]
fn vpu_stat_shows_the_macro_divide_busy() {
    let (mut sys, at) = stepped("vdiv q, vf1x, vf2x\ncfc2 t0, vi29\nnop", None);
    assert_eq!(sys.ee.gpr[8] as u32 & 0x20, 0x20, "DIV0 while the divide runs");
    let t = at[0] + 7;
    assert_eq!(sys.board.vpu.vpu_stat(t) & 0x20, 0, "idle once it has landed");
}
