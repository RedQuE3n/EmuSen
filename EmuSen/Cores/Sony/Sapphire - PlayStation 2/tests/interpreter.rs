//! Stage S6's gate (Sapphire_Native.md §16.5): the project's programs over every integer encoding, run on the
//! interpreter and graded against the manual's definitions; the timing model's table and its issue rules pinned.

mod s6;

use sapphire::ee::timing::{self, Category, FLOAT, INTEGER, Issue, Pair, Pipe, Timing};
use sapphire::ee::{self, Ee, Halt, bus::Ram, cop0};


use s6::{DONE, DONE_AT, ORIGIN, Program, RECORD};

fn assemble(source: &str, origin: u32) -> Vec<u32> {
    emusen_mips::assemble(source, origin, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{e:?}"))
}

/// Runs a program as the referees do: loaded at 00100000h and entered there, until its done word.
fn run(source: &str, limit: u64) -> (Ee, Ram) {
    let mut ram = Ram::new();
    ram.load(ORIGIN, &assemble(source, ORIGIN));
    let mut ee = Ee::new(ORIGIN);
    let n = ee.run(&mut ram, limit, &mut |b| b.read(DONE_AT, 4) as u32 == DONE).unwrap_or_else(|h| panic!("{h:?}"));
    assert!(n < limit, "no done word in {limit} instructions, pc {:08x}", ee.pc);
    (ee, ram)
}

fn grade(p: Program) {
    let name = p.name;
    let (source, expected) = p.finish();
    let (_, ram) = run(&source, 20_000_000);
    let mut wrong = Vec::new();
    for (i, want) in expected.iter().enumerate() {
        let got = ram.word(RECORD + 4 * i as u32);
        if let Some(w) = want
            && *w != got
        {
            wrong.push(format!("word {i} ({:08x}): {got:08x}, the manual {w:08x}", RECORD + 4 * i as u32));
        }
    }
    assert!(wrong.is_empty(), "{name}: {} of {} words differ, first {:?}", wrong.len(), expected.len(), &wrong[..wrong.len().min(8)]);
}

#[test]
fn three_register_operations_equal_the_manual() {
    grade(s6::three());
}

#[test]
fn immediate_and_shift_forms_equal_the_manual() {
    grade(s6::immediates());
}

#[test]
fn both_multipliers_and_dividers_equal_the_manual() {
    grade(s6::muldiv());
}

#[test]
fn the_shift_amount_register_reads_back_as_written() {
    grade(s6::shift_amount());
}

#[test]
fn every_load_and_store_at_every_byte_equals_the_manual() {
    grade(s6::memory());
}

#[test]
fn branches_jumps_and_links_equal_the_manual() {
    grade(s6::branches());
}

#[test]
fn exceptions_record_cause_epc_badvaddr_and_status_as_chapter_4_says() {
    grade(s6::exceptions());
}

#[test]
fn bits_127_to_64_survive_every_narrower_write() {
    grade(s6::upper());
}

#[test]
fn the_undefined_inputs_give_the_interpreters_recorded_reading() {
    grade(s6::undefined());
}

#[test]
fn the_programs_execute_every_integer_encoding_the_interpreter_has() {
    let mut seen: std::collections::BTreeSet<String> = s6::all().into_iter().flat_map(|p| s6::covered(&p.finish().0)).collect();
    seen.extend(assemble(&s6::handler(), s6::VECTOR).iter().filter_map(|&w| emusen_mips::r5900::operation(w)).map(|o| o.mnemonic()));
    let missing: Vec<_> = ee::names().filter(|n| !seen.contains(*n)).collect();
    assert!(missing.is_empty(), "never executed: {missing:?}");
    assert_eq!(ee::names().count(), 128, "the S6 set");
}

#[test]
fn record_s_gives_the_twelve_words_the_referees_gave() {
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/programs/record.s")).unwrap();
    let (ee, ram) = run(&source, 100_000);
    let words: Vec<u32> = (0..12).map(|i| ram.word(RECORD + 4 * i)).collect();
    assert_eq!(words, [0x8000_0000, 0xF800_0000, 0x0800_0000, 1, 0, 1, 0x3FFF_FFFF, 0xFFFF_FFFD, 0xFFFF_FFFF, 0x13BA, 1, 0xEEDD_BB80]);
    assert!(ee.timing.cycles() > 0 && ee.timing.cycles() < 2 * ee.timing.instructions);
}

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

#[test]
fn a_jump_to_an_unaligned_address_faults_at_the_fetch_with_the_target_in_epc_and_badvaddr() {
    let (mut ee, mut ram) = machine("lui t0, 0x10\nori t0, t0, 0x102\njr t0\naddiu t1, zero, 1");
    steps(&mut ee, &mut ram, 5);
    assert_eq!(ee.gpr[9], 1, "the delay slot ran");
    assert_eq!(ee.cop0[cop0::EPC], 0x0010_0102);
    assert_eq!(ee.cop0[cop0::BADVADDR], 0x0010_0102);
    assert_eq!(ee.cop0[cop0::CAUSE] >> 2 & 31, 4);
    assert_eq!(ee.pc, 0x8000_0180);
}

#[test]
fn bev_moves_the_vector_and_an_exception_inside_a_handler_keeps_epc() {
    let (mut ee, mut ram) = machine("syscall");
    ee.cop0[cop0::STATUS] = ee::status::BEV;
    steps(&mut ee, &mut ram, 1);
    assert_eq!((ee.pc, ee.cop0[cop0::EPC]), (0xBFC0_0380, ORIGIN));
    let (mut ee, mut ram) = machine("break");
    ee.cop0[cop0::STATUS] = ee::status::EXL;
    ee.cop0[cop0::EPC] = 0x1234;
    steps(&mut ee, &mut ram, 1);
    assert_eq!((ee.pc, ee.cop0[cop0::EPC], ee.cop0[cop0::CAUSE] >> 2 & 31), (0x8000_0180, 0x1234, 9));
}

#[test]
fn eret_returns_through_errorepc_when_erl_is_set() {
    let (mut ee, mut ram) = machine("eret");
    ee.cop0[cop0::STATUS] = ee::status::ERL | ee::status::EXL;
    ee.cop0[cop0::ERROREPC] = 0x0010_0040;
    steps(&mut ee, &mut ram, 1);
    assert_eq!((ee.pc, ee.cop0[cop0::STATUS]), (0x0010_0040, ee::status::EXL));
}

#[test]
fn a_later_stages_instruction_halts_with_its_word_and_a_usable_coprocessor() {
    let (mut ee, mut ram) = machine("nop\nmfc1 t0, f1");
    ee.cop0[cop0::STATUS] = 1 << 29;
    steps(&mut ee, &mut ram, 1);
    assert_eq!(ee.step(&mut ram), Err(Halt::Unimplemented { pc: ORIGIN + 4, word: 0x4408_0800 }));
    assert_eq!(ee.pc, ORIGIN + 4, "the machine is left before the instruction");
    let (mut ee, mut ram) = machine("paddw t0, t1, t2");
    assert!(matches!(ee.step(&mut ram), Err(Halt::Unimplemented { .. })), "MMI is S8's");
}

// The timing model (EE Core User's Manual §1.3.5, §1.3.6, §2.12).

#[test]
fn the_latency_table_is_section_2_12s() {
    let t: Vec<_> = INTEGER.iter().map(|r| (r.name, r.latency, r.throughput)).collect();
    assert_eq!(
        t,
        [
            ("Add/Sub, and logical operations", 1, 1),
            ("Transfer to HI / LO registers", 1, 1),
            ("Shift / LUI", 1, 1),
            ("Branch / Jump", 1, 1),
            ("Conditional Move", 1, 1),
            ("MULT / MULTU", 4, 2),
            ("MULT1 / MULTU1", 4, 2),
            ("DIV / DIVU", 37, 37),
            ("DIV1 / DIVU1", 37, 37),
            ("MADD / MADDU", 4, 2),
            ("MADD1 / MADDU1", 4, 2),
            ("Load", 1, 1),
            ("Store", 0, 1),
            ("Multimedia Multiply", 4, 2),
            ("Multimedia Divide", 37, 37),
        ]
    );
    assert_eq!(INTEGER[5].pipes, [Pipe::I0]);
    assert_eq!(INTEGER[6].pipes, [Pipe::I1]);
    let f: Vec<_> = FLOAT.iter().map(|r| (r.latency, r.throughput)).collect();
    assert_eq!(f, [(2, 1), (4, 1), (4, 1), (4, 1), (2, 1), (4, 1), (8, 7), (8, 7), (14, 13), (4, 1), (2, 1)]);
}

#[test]
fn the_pairing_tables_are_tables_1_2_and_1_3() {
    use Category::*;
    assert_eq!(timing::logical(LoadStore), (false, true));
    assert_eq!(timing::logical(Mac0), (true, false));
    assert_eq!(timing::logical(Mac1), (false, true));
    assert_eq!(timing::logical(Alu), (true, true));
    assert_eq!(timing::pair(Branch, Branch), Some(Pair::X));
    assert_eq!(timing::pair(Eret, Branch), Some(Pair::X));
    assert_eq!(timing::pair(Lzc, WideOperate), Some(Pair::Y));
    assert_eq!(timing::pair(Alu, WideOperate), Some(Pair::Y));
    assert_eq!(timing::pair(Mac1, WideOperate), Some(Pair::Y));
    assert_eq!(timing::pair(Cop1Move, Cop1Operate), Some(Pair::Y));
    assert_eq!(timing::pair(Cop2Move, Cop2Operate), Some(Pair::Y));
    let xs = timing::PIPE1.iter().flat_map(|(_, r)| r.iter()).filter(|&&c| c == Pair::X).count();
    let ys = timing::PIPE1.iter().flat_map(|(_, r)| r.iter()).filter(|&&c| c == Pair::Y).count();
    assert_eq!((xs, ys), (2, 5), "Table 1-3 holds two X and five Y");
}

fn cycles(code: &str) -> u64 {
    cycles_of(code, code.lines().filter(|l| !l.trim().is_empty()).count())
}

fn cycles_of(code: &str, n: usize) -> u64 {
    let (mut ee, mut ram) = machine(code);
    steps(&mut ee, &mut ram, n);
    ee.timing.cycles()
}

#[test]
fn independent_alu_instructions_pair_and_a_dependent_one_waits_a_cycle() {
    assert_eq!(cycles("addu t0, t1, t2\naddu t3, t4, t5"), 1);
    assert_eq!(cycles("addu t0, t1, t2\naddu t3, t0, t5"), 2);
    assert_eq!(cycles("addu t0, t1, t2\naddu t3, t4, t5\naddu t6, t7, t8\naddu s2, s3, s4"), 2);
}

#[test]
fn two_loads_cannot_pair_and_a_load_pairs_with_an_alu_instruction() {
    assert_eq!(cycles("lw t0, 0(s0)\nlw t1, 4(s0)"), 2);
    assert_eq!(cycles("lw t0, 0(s0)\naddu t1, t2, t3"), 1);
    assert_eq!(cycles("lw t0, 0(s0)\naddu t1, t0, t3"), 2, "load latency 1 (§2.12): the pair is lost");
}

#[test]
fn the_multipliers_are_four_cycles_to_their_result_and_two_between_issues() {
    assert_eq!(cycles("mult t0, t1, t2\nmflo t3"), 5);
    assert_eq!(cycles("mult t0, t1, t2\nmult t3, t4, t5"), 3);
    assert_eq!(cycles("mult t0, t1, t2\nmult1 t3, t4, t5"), 1, "MAC0 and MAC1 issue together");
    assert_eq!(cycles("mult t0, t1, t2\naddu t3, t0, t4"), 5, "rd too is four cycles away");
    assert_eq!(cycles("madd t0, t1, t2\nmadd t3, t4, t5"), 3, "accumulation forwards at the throughput");
}

#[test]
fn the_dividers_are_thirty_seven_cycles() {
    assert_eq!(cycles("div t0, t1\nmflo t2"), 38);
    assert_eq!(cycles("div t0, t1\ndiv1 t2, t3\nmflo1 t4"), 38);
    assert_eq!(cycles("div t0, t1\naddu t2, t3, t4\naddu t5, t6, t7"), 2, "other instructions run beside a divide");
}

#[test]
fn a_taken_branch_pairs_with_its_slot_and_the_target_starts_a_new_cycle() {
    assert_eq!(cycles_of("beq zero, zero, 0x00100010\naddu t0, t1, t2\nnop\nnop\naddu t3, t4, t5", 3), 2);
}

#[test]
fn a_y_pair_issues_together_and_costs_a_cycle() {
    let mut t = Timing::default();
    let i = |category, row| Issue { category, row, reads: 0, writes: 0, taken: false };
    assert_eq!(t.issue(i(Category::WideOperate, timing::rows::ALU)), 0);
    assert_eq!(t.issue(i(Category::Alu, timing::rows::ALU)), 0);
    assert_eq!(t.issue(i(Category::Alu, timing::rows::ALU)), 2);
}
