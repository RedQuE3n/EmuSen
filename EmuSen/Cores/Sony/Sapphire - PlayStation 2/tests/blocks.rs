//! Stage S36's gate (Sapphire_Native.md §44): the EE recompiler graded against the interpreter by the oracles of
//! Mars_Native.md §5.8.5 that run on the project's own programs: the interpreter stepped beside the blocks and compared
//! after every instruction and after every block; random programs of integer instructions and branches, seeded and
//! compared; and the mechanisms one at a time. Every program here is the project's own, assembled for the purpose.

mod d780;
mod s6;

use sapphire::blocks::{self, Blocks, Verify};
use sapphire::board::System;

use s6::{DONE, DONE_AT, ORIGIN};

fn assemble(source: &str, origin: u32) -> Vec<u32> {
    emusen_mips::assemble(source, origin, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{e:?}\n{source}"))
}

/// The handler at 80000180h and 80000200h for the random programs: an interrupt from Count (IP7) moves Compare on by `PERIOD` cycles,
/// one from the INTC (IP2) acknowledges timer 0; any other exception returns past its instruction, or past the branch and
/// its slot.
const PERIOD: u32 = 1500;

fn handler() -> String {
    format!(
        "mfc0 k0, Cause
        andi k1, k0, 0x7c
        bne k1, zero, skip
        nop
        andi k1, k0, 0x8000
        beq k1, zero, intc
        nop
        mfc0 k1, Count
        addiu k1, k1, {PERIOD}
        mtc0 k1, Compare
        sync.p
        eret
        intc: lui k1, 0x1000
        ori k0, zero, 0x0d80
        sw k0, 0x10(k1)
        lui k1, 0x1000
        ori k1, k1, 0xf000
        ori k0, zero, 0x200
        sw k0, 0(k1)
        sync.p
        eret
        skip: mfc0 k1, EPC
        bltz k0, bd
        nop
        addiu k1, k1, 4
        beq zero, zero, out
        nop
        bd: addiu k1, k1, 8
        out: mtc0 k1, EPC
        sync.p
        eret"
    )
}

fn system(words: &[u32]) -> System {
    let mut sys = System::new(ORIGIN);
    sys.board.map.load(ORIGIN, words);
    sys.board.map.load(0x180, &assemble(&handler(), 0x8000_0180));
    // The interrupt vector, 80000200h, takes the same handler.
    sys.board.map.load(0x200, &assemble(&handler(), 0x8000_0200));
    sys
}

/// Blocks compiled at their first entry and waited for, so that short programs run compiled.
fn eager() -> Blocks {
    let mut b = Blocks::new();
    b.threshold = 1;
    b.wait = true;
    b
}

/// Runs `n` steps of the machine through the dispatcher.
fn run_steps(sys: &mut System, b: &mut Blocks, n: u64) {
    let mut left = n;
    while left > 0 {
        left -= blocks::run(sys, b, left).unwrap_or_else(|h| panic!("{h:?}"));
    }
}

/// The interpreter's machine and the blocks' run side by side for `steps` steps, compared every `stride` and whole at
/// the end; the blocks' counts.
fn twin(words: &[u32], steps: u64, stride: u64, b: &mut Blocks) -> blocks::Stats {
    let mut plain = system(words);
    let mut fast = system(words);
    let mut done = 0;
    while done < steps {
        let n = stride.min(steps - done);
        for _ in 0..n {
            plain.step().unwrap_or_else(|h| panic!("{h:?}"));
        }
        run_steps(&mut fast, b, n);
        done += n;
        if let Some(d) = blocks::compare(&plain, &fast, false) {
            panic!("after {done} steps: {d}");
        }
    }
    assert!(blocks::state_of(&plain) == blocks::state_of(&fast), "the whole states differ after {steps} steps");
    b.stats
}

/// The same with the verifier attached in the way given.
fn verified(words: &[u32], steps: u64, how: Verify) -> blocks::Stats {
    let mut b = eager();
    let mut fast = system(words);
    b.verify(how, Box::new(system(words)), false);
    run_steps(&mut fast, &mut b, steps);
    assert_eq!(b.verified(), Some(steps), "the interpreter stepped once for every step the blocks' machine took");
    b.check_all(&fast);
    b.stats
}

const LOOP: &str = "
    lui s0, 0x0012
    ori s1, zero, 300
    ori t0, zero, 1
    top: addu t0, t0, t0
    xori t1, t0, 0x55
    dsll32 t2, t1, 3
    dsra t2, t2, 7
    sltu t3, t1, t2
    mult t4, t1, t3
    mflo t5
    addiu s1, s1, -1
    bgtz s1, top
    or t6, t5, t0
    lui a0, 0x001f
    ori a0, a0, 0xfff0
    lui a1, 0x5a99
    ori a1, a1, 0xd00e
    sw a1, 0(a0)
    end: beq zero, zero, end
    nop
";

/// D-780's program (Sapphire_Native.md §39.18): byte and halfword stores to five units' registers, compiled, leave the
/// interpreter's machine; the record itself is graded in tests/dmac.rs.
#[test]
fn narrower_stores_to_registers_run_compiled_as_interpreted() {
    let (source, _) = d780::width_bare().finish();
    let words = assemble(&source, ORIGIN);
    let s = twin(&words, 6000, 250, &mut eager());
    assert!(s.compiled_steps > 1000, "the program ran compiled: {s:?}");
}

#[test]
fn a_hot_loop_runs_compiled_and_leaves_the_interpreters_machine() {
    let words = assemble(LOOP, ORIGIN);
    let s = twin(&words, 4000, 500, &mut eager());
    assert!(s.entered > 250 && s.compiled_steps > 2500, "the loop ran compiled: {s:?}");
}

#[test]
fn the_verifier_steps_beside_every_compiled_instruction_and_finds_nothing() {
    let words = assemble(LOOP, ORIGIN);
    let s = verified(&words, 4000, Verify::Steps);
    assert!(s.compiled_steps > 2500, "{s:?}");
    let s = verified(&words, 4000, Verify::Blocks);
    assert!(s.compiled_steps > 2500, "{s:?}");
}


/// A seeded generator (xorshift64*).
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len() as u64) as usize]
    }
}

/// The registers a random program computes with; t8 counts an inner loop, t9 the outer, gp holds the scratch area,
/// fp the subroutine's address, k0 and k1 are the handler's.
const POOL: [&str; 21] =
    ["v0", "v1", "a0", "a1", "a2", "a3", "t0", "t1", "t2", "t3", "t4", "t5", "t6", "t7", "s0", "s1", "s2", "s3", "s4", "s5", "s6"];
const SUB: u32 = ORIGIN + 0x1_0000;
const SCRATCH: u32 = 0x0030_0000;

/// One instruction a block compiles, chosen at random, its operands from the pool; traps and overflowing adds rarely.
fn plain(r: &mut Rng) -> String {
    let (d, s, t) = (*r.pick(&POOL), *r.pick(&POOL), *r.pick(&POOL));
    let src = if r.below(8) == 0 { "ra" } else { s };
    let imm = r.next() as u16;
    let sh = r.below(32);
    match r.below(100) {
        0..=29 => format!("{} {d}, {src}, {t}", r.pick(&["addu", "subu", "daddu", "dsubu", "and", "or", "xor", "nor", "slt", "sltu", "movz", "movn", "sllv", "srlv", "srav", "dsllv", "dsrlv", "dsrav"])),
        30..=47 => match r.below(8) {
            0..=2 => format!("{} {d}, {src}, {}", r.pick(&["addiu", "daddiu", "slti", "sltiu"]), imm as i16),
            3..=5 => format!("{} {d}, {src}, 0x{imm:x}", r.pick(&["andi", "ori", "xori"])),
            _ => format!("lui {d}, 0x{imm:x}"),
        },
        48..=59 => format!("{} {d}, {t}, {sh}", r.pick(&["sll", "srl", "sra", "dsll", "dsrl", "dsra", "dsll32", "dsrl32", "dsra32"])),
        60..=67 => match r.below(2) {
            0 => format!("{} {d}", r.pick(&["mfhi", "mflo", "mfhi1", "mflo1"])),
            _ => format!("{} {s}", r.pick(&["mthi", "mtlo", "mthi1", "mtlo1"])),
        },
        68..=77 => {
            let name = *r.pick(&["mult", "multu", "mult1", "multu1", "madd", "maddu", "madd1", "maddu1", "div", "divu", "div1", "divu1"]);
            if name.starts_with("div") { format!("{name} {s}, {t}") } else { format!("{name} {d}, {s}, {t}") }
        }
        78..=81 => match r.below(4) {
            0 => format!("mfsa {d}"),
            1 => format!("mtsa {s}"),
            2 => format!("mtsab {s}, 0x{:x}", imm & 0xF),
            _ => format!("mtsah {s}, 0x{:x}", imm & 7),
        },
        82..=85 => format!("{} {d}, {s}, {t}", r.pick(&["add", "sub", "dadd", "dsub"])),
        86..=87 => format!("{} {d}, {s}, {}", r.pick(&["addi", "daddi"]), imm as i16),
        88..=89 => match r.below(3) {
            0 => format!("{} {s}, {t}", r.pick(&["teq", "tge", "tltu"])),
            1 => format!("teqi {s}, {}", imm as i16),
            _ => format!("tlti {s}, {}", -((imm & 0x7FFF) as i16)),
        },
        90..=91 => r.pick(&["sync", "sync.p"]).to_string(),
        _ => format!("{} {d}, {s}, {t}", r.pick(&["or", "addu", "xor"])),
    }
}

/// A random instruction that raises nothing, for the slots of the calls and returns: the handler's return past a branch
/// and its slot would skip a return and run the program into memory it never wrote.
fn quiet(r: &mut Rng) -> String {
    loop {
        let i = plain(r);
        let name = i.split_whitespace().next().unwrap_or("");
        if !matches!(name, "add" | "sub" | "dadd" | "dsub" | "addi" | "daddi") && !name.starts_with('t') {
            return i;
        }
    }
}

/// An instruction the blocks leave to the interpreter: a load or store in the scratch area.
fn other(r: &mut Rng) -> String {
    let (d, off) = (*r.pick(&POOL), 8 * r.below(256));
    format!("{} {d}, {off}(gp)", r.pick(&["sd", "ld", "sw", "lw", "lq", "sq"]))
}

fn constant(lines: &mut Vec<String>, reg: &str, v: u64) {
    lines.push(format!("lui {reg}, 0x{:x}", v >> 48 & 0xFFFF));
    lines.push(format!("ori {reg}, {reg}, 0x{:x}", v >> 32 & 0xFFFF));
    lines.push(format!("dsll {reg}, {reg}, 16"));
    lines.push(format!("ori {reg}, {reg}, 0x{:x}", v >> 16 & 0xFFFF));
    lines.push(format!("dsll {reg}, {reg}, 16"));
    lines.push(format!("ori {reg}, {reg}, 0x{:x}", v & 0xFFFF));
}

/// A random program and its subroutine: registers set, the interrupts of Count and (in `timer` programs) timer 0 armed,
/// then an outer loop of segments: straight runs, forward branches of every kind with their slots, inner loops, calls
/// through JALR, and loads and stores between them.
fn random_program(seed: u64, timer: bool) -> (Vec<u32>, Vec<u32>) {
    let mut r = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    let mut l = Vec::new();
    let special = [0, 1, 0x7FFF_FFFF, 0xFFFF_FFFF_8000_0000, u64::MAX, 0x8000_0000_0000_0000];
    for reg in POOL {
        let v = if r.below(4) == 0 { *r.pick(&special) } else if r.below(3) == 0 { (r.next() as i32) as i64 as u64 } else { r.next() };
        constant(&mut l, reg, v);
    }
    l.push(format!("lui gp, 0x{:x}", SCRATCH >> 16));
    l.push(format!("lui fp, 0x{:x}", SUB >> 16));
    l.push(format!("ori fp, fp, 0x{:x}", SUB & 0xFFFF));
    l.push(format!("ori t9, zero, {}", 40 + r.below(40)));
    l.push("mfc0 k0, Count".into());
    l.push(format!("addiu k0, k0, {PERIOD}"));
    l.push("mtc0 k0, Compare".into());
    if timer {
        // Timer 0 counting the bus clock, its compare interrupt through the INTC's mask on IM2.
        l.push("lui k1, 0x1000".into());
        l.push(format!("ori k0, zero, {}", 150 + r.below(300)));
        l.push("sw k0, 0x20(k1)".into());
        l.push("ori k0, zero, 0x1c0".into());
        l.push("sw k0, 0x10(k1)".into());
        l.push("lui k1, 0x1000".into());
        l.push("ori k1, k1, 0xf010".into());
        l.push("ori k0, zero, 0x200".into());
        l.push("sw k0, 0(k1)".into());
    }
    l.push("lui k0, 0x1".into());
    l.push(format!("ori k0, k0, 0x{:x}", if timer { 0x8401 } else { 0x8001 }));
    l.push("mtc0 k0, Status".into());
    l.push("sync.p".into());
    l.push("top:".into());
    let mut label = 0;
    for _ in 0..4 + r.below(8) {
        match r.below(10) {
            0..=2 => {
                for _ in 0..1 + r.below(12) {
                    l.push(plain(&mut r));
                }
            }
            3..=5 => {
                label += 1;
                let (s, t) = (*r.pick(&POOL), *r.pick(&POOL));
                let br = match r.below(14) {
                    0 => format!("beq {s}, {t}, f{label}"),
                    1 => format!("bne {s}, {t}, f{label}"),
                    2 => format!("beql {s}, {t}, f{label}"),
                    3 => format!("bnel {s}, {t}, f{label}"),
                    k => format!("{} {s}, f{label}", ["blez", "bgtz", "bltz", "bgez", "blezl", "bgtzl", "bltzl", "bgezl", "bltzal", "bgezal"][k as usize - 4]),
                };
                l.push(br);
                l.push(plain(&mut r));
                for _ in 0..1 + r.below(6) {
                    l.push(if r.below(5) == 0 { other(&mut r) } else { plain(&mut r) });
                }
                l.push(format!("f{label}:"));
            }
            6 => {
                label += 1;
                l.push(format!("ori t8, zero, {}", 2 + r.below(12)));
                l.push(format!("b{label}:"));
                for _ in 0..1 + r.below(8) {
                    l.push(plain(&mut r));
                }
                l.push("addiu t8, t8, -1".into());
                l.push(format!("{} t8, b{label}", if r.below(3) == 0 { "bgtzl" } else { "bgtz" }));
                l.push(plain(&mut r));
            }
            7 => {
                l.push(if r.below(2) == 0 { "jalr fp".into() } else { "jalr ra, fp".into() });
                l.push(quiet(&mut r));
            }
            8 => {
                label += 1;
                l.push(format!("j f{label}"));
                l.push(plain(&mut r));
                l.push(plain(&mut r));
                l.push(format!("f{label}:"));
            }
            _ => l.push(other(&mut r)),
        }
    }
    l.push("addiu t9, t9, -1".into());
    l.push("bgtz t9, top".into());
    l.push("nop".into());
    l.push(format!("lui a0, 0x{:x}", DONE_AT >> 16));
    l.push(format!("ori a0, a0, 0x{:x}", DONE_AT & 0xFFFF));
    l.push(format!("lui a1, 0x{:x}", DONE >> 16));
    l.push(format!("ori a1, a1, 0x{:x}", DONE & 0xFFFF));
    l.push("sw a1, 0(a0)".into());
    l.push("end: beq zero, zero, end".into());
    l.push("nop".into());
    let mut sub = Vec::new();
    for _ in 0..2 + r.below(8) {
        sub.push(plain(&mut r));
    }
    sub.push("jr ra".into());
    sub.push(quiet(&mut r));
    (assemble(&l.join("\n"), ORIGIN), assemble(&sub.join("\n"), SUB))
}

fn random_system(program: &(Vec<u32>, Vec<u32>)) -> System {
    let mut sys = system(&program.0);
    sys.board.map.load(SUB, &program.1);
    sys
}

/// Random programs at every seed, the two machines compared every `stride` steps and whole at the end; the totals.
fn random_twins(seeds: std::ops::Range<u64>, steps: u64, stride: u64, timer: bool) -> blocks::Stats {
    let mut total = blocks::Stats::default();
    for seed in seeds {
        let p = random_program(seed, timer);
        let mut b = eager();
        let mut plain = random_system(&p);
        let mut fast = random_system(&p);
        let mut done = 0;
        while done < steps {
            let n = stride.min(steps - done);
            for _ in 0..n {
                plain.step().unwrap_or_else(|h| panic!("seed {seed}: {h:?}"));
            }
            run_steps(&mut fast, &mut b, n);
            done += n;
            if let Some(d) = blocks::compare(&plain, &fast, false) {
                panic!("seed {seed}, after {done} steps: {d}");
            }
        }
        assert!(blocks::state_of(&plain) == blocks::state_of(&fast), "seed {seed}: the whole states differ");
        let s = b.stats;
        total.entered += s.entered;
        total.compiled_steps += s.compiled_steps;
        total.rollbacks += s.rollbacks;
        total.rerun_steps += s.rerun_steps;
        total.interpreted += s.interpreted;
        total.reshaped += s.reshaped;
        total.refills += s.refills;
        total.waits += s.waits;
    }
    total
}

#[test]
fn random_programs_leave_the_interpreters_machine() {
    let s = random_twins(0..160, 24_000, 400, false);
    println!("random programs, Count's interrupt: {s:?}");
    assert!(s.compiled_steps > s.interpreted, "most steps ran compiled: {s:?}");
    assert!(s.rollbacks > 0, "an interrupt fell inside a block at least once: {s:?}");
}

#[test]
fn random_programs_with_timer_interrupts_through_the_board_leave_the_interpreters_machine() {
    let s = random_twins(1000..1080, 24_000, 400, true);
    println!("random programs, timer 0 and Count: {s:?}");
    assert!(s.rollbacks > 0, "{s:?}");
}

#[test]
fn random_programs_under_the_verifier_find_nothing() {
    for seed in 0..20 {
        let p = random_program(seed, seed % 2 == 0);
        for how in [Verify::Steps, Verify::Blocks] {
            let mut b = eager();
            let mut fast = random_system(&p);
            b.verify(how, Box::new(random_system(&p)), false);
            run_steps(&mut fast, &mut b, 10_000);
            assert_eq!(b.verified(), Some(10_000), "seed {seed}");
            b.check_all(&fast);
        }
    }
}

/// The steps the interpreter takes to a program's done word.
fn steps_to_done(words: &[u32]) -> u64 {
    let mut sys = system(words);
    sys.run(50_000_000, &mut |s| s.peek(DONE_AT, 4) as u32 == DONE).unwrap_or_else(|h| panic!("{h:?}"))
}

#[test]
fn the_stage_s6_programs_over_every_integer_encoding_leave_the_interpreters_machine() {
    let mut b = eager();
    let mut names = std::collections::BTreeSet::new();
    for p in s6::all() {
        let name = p.name;
        let (source, expected) = p.finish();
        let words = assemble(&source, ORIGIN);
        let n = steps_to_done(&words);
        let mut plain = system(&words);
        let mut fast = system(&words);
        b.forget();
        let mut done = 0;
        while done < n {
            let k = 500.min(n - done);
            for _ in 0..k {
                plain.step().unwrap();
            }
            run_steps(&mut fast, &mut b, k);
            done += k;
            if let Some(d) = blocks::compare(&plain, &fast, false) {
                panic!("{name}, after {done} steps: {d}");
            }
        }
        assert!(blocks::state_of(&plain) == blocks::state_of(&fast), "{name}: the whole states differ");
        for (i, want) in expected.iter().enumerate() {
            if let Some(w) = want {
                assert_eq!(fast.peek(s6::RECORD + 4 * i as u32, 4) as u32, *w, "{name}: record word {i} against the manual");
            }
        }
        names.extend(b.compiled_mnemonics());
    }
    let missing: Vec<_> = blocks::mnemonics().filter(|m| !names.contains(m)).collect();
    println!("compiled by the programs: {} of {} mnemonics; {:?}", names.len(), blocks::mnemonics().count(), b.stats);
    assert!(missing.is_empty(), "never compiled by the programs: {missing:?}");
}

/// A loop whose ADD overflows and whose delay-slot TEQ traps on some passes, through the handler.
const RAISING: &str = "
    lui s0, 0x7fff
    ori s0, s0, 0xfff0
    ori s1, zero, 40
    ori s2, zero, 0
    top: addiu s2, s2, 1
    andi t0, s2, 7
    add t1, s0, s2
    xor t2, t1, s2
    sll t3, t0, 2
    bne s1, zero, next
    teq t0, zero
    next: addiu s1, s1, -1
    bgtz s1, top
    sub t4, t2, s0
    end: beq zero, zero, end
    nop
";

#[test]
fn a_block_leaves_before_an_instruction_that_raises_in_a_line_and_in_a_delay_slot() {
    let words = assemble(RAISING, ORIGIN);
    let mut b = eager();
    let s = twin(&words, 3000, 300, &mut b);
    let mut fast = system(&words);
    run_steps(&mut fast, &mut eager(), 3000);
    let (overflow, trap) = (fast.ee.exceptions[12], fast.ee.exceptions[13]);
    assert!(overflow > 10 && trap > 3 && s.compiled_steps > 1000, "overflows {overflow}, traps {trap}, {s:?}");
}

/// A routine called in a loop, its first word replaced in memory after 30 passes and its line invalidated after 60.
const REPLACED: &str = "
    ori s1, zero, 90
    lui t5, 0x2442
    ori t5, t5, 0x0002
    top: jal 0x100200
    nop
    addiu s1, s1, -1
    ori t0, zero, 60
    bne s1, t0, keep
    nop
    lui t1, 0xa010
    sw t5, 0x200(t1)
    keep: ori t0, zero, 30
    bne s1, t0, go
    nop
    lui t2, 0x0010
    cache 0x0b, 0x200(t2)
    sync.p
    go: bgtz s1, top
    nop
    end: beq zero, zero, end
    nop
";

#[test]
fn code_replaced_runs_old_until_its_line_is_invalidated_and_the_block_is_shaped_again() {
    let mut words = assemble(REPLACED, ORIGIN);
    words.resize(0x200 / 4, 0);
    words.extend(assemble("addiu v0, v0, 1\naddu v1, v1, v0\njr ra\nxor a0, a0, v1", ORIGIN + 0x200));
    let mut b = eager();
    let s = twin(&words, 4000, 250, &mut b);
    let mut fast = system(&words);
    let mut fb = eager();
    run_steps(&mut fast, &mut fb, 4000);
    assert_eq!(fast.ee.gpr[2] as u64, 60 + 2 * 30, "the old word ran until the line was invalidated, then the new");
    assert!(fb.stats.reshaped >= 1 && s.compiled_steps > 1000, "{:?}", fb.stats);
}

#[test]
fn a_state_saved_between_blocks_loads_into_an_empty_cache_and_runs_alike() {
    for seed in [3, 8, 13] {
        let p = random_program(seed, seed == 8);
        let (mut carried, mut carried_blocks) = (random_system(&p), eager());
        let mut plain = random_system(&p);
        run_steps(&mut carried, &mut carried_blocks, 7_777);
        for _ in 0..7_777 {
            plain.step().unwrap();
        }
        let state = blocks::state_of(&carried);
        let mut loaded = System::new(ORIGIN);
        loaded.read_state(&mut emusen_native::StateReader::new(&state)).expect("the state loads");
        // The exception counts are the hardware's reports, not the machine's state.
        (loaded.ee.exceptions, loaded.ee.exceptions2) = (carried.ee.exceptions, carried.ee.exceptions2);
        let mut loaded_blocks = eager();
        for _ in 0..20 {
            for _ in 0..500 {
                plain.step().unwrap();
            }
            run_steps(&mut carried, &mut carried_blocks, 500);
            run_steps(&mut loaded, &mut loaded_blocks, 500);
            assert_eq!(blocks::compare(&plain, &carried, false), None, "seed {seed}: the machine that carried its blocks");
            assert_eq!(blocks::compare(&plain, &loaded, false), None, "seed {seed}: the machine loaded into an empty cache");
        }
        assert!(blocks::state_of(&carried) == blocks::state_of(&loaded), "seed {seed}");
        assert!(loaded_blocks.stats.compiled_steps > 0, "seed {seed}");
    }
}

#[test]
fn blocks_refused_at_a_small_step_budget_leave_the_machine_exact() {
    let words = assemble(LOOP, ORIGIN);
    for stride in [1, 3, 7, 11] {
        let s = twin(&words, 1500, stride, &mut eager());
        assert!(stride < 11 || s.compiled_steps > 0, "{s:?}");
    }
}

#[test]
fn single_issue_blocks_leave_the_interpreters_machine() {
    let source = format!("mfc0 t0, Config\nlui t1, 0xfffb\nori t1, t1, 0xffff\nand t0, t0, t1\nmtc0 t0, Config\nsync.p\n{LOOP}");
    let words = assemble(&source, ORIGIN);
    let s = twin(&words, 4000, 500, &mut eager());
    assert!(s.compiled_steps > 2500, "{s:?}");
}

#[test]
fn dropping_every_block_at_the_codes_budget_changes_nothing() {
    for seed in 40..44 {
        let p = random_program(seed, false);
        let mut b = eager();
        b.set_budget(16 << 10);
        let mut plain = random_system(&p);
        let mut fast = random_system(&p);
        for _ in 0..40 {
            for _ in 0..500 {
                plain.step().unwrap();
            }
            run_steps(&mut fast, &mut b, 500);
            assert_eq!(blocks::compare(&plain, &fast, false), None, "seed {seed}");
        }
        assert!(b.flushes() > 0, "seed {seed}: the budget dropped the blocks");
    }
}

#[test]
fn a_block_ends_at_its_pages_end() {
    let jump = assemble(&format!("j 0x{:x}\nnop", ORIGIN + 0xFD0), ORIGIN);
    let mut words = jump;
    words.resize(0xFD0 / 4, 0);
    words.extend(assemble(LOOP, ORIGIN + 0xFD0));
    let s = twin(&words, 4000, 500, &mut eager());
    assert!(s.compiled_steps > 2000, "{s:?}");
}

#[test]
fn the_fixed_cost_compromise_changes_the_cycles_and_not_the_results() {
    let words = assemble(LOOP, ORIGIN);
    let mut plain = system(&words);
    let mut fast = system(&words);
    let mut b = eager();
    b.fixed_cost = true;
    for _ in 0..4000 {
        plain.step().unwrap();
    }
    run_steps(&mut fast, &mut b, 4000);
    assert_eq!(plain.ee.gpr, fast.ee.gpr, "a program that reads no time computes the same");
    let (exact, fixed) = (plain.ee.timing.cycles(), fast.ee.timing.cycles());
    println!("the loop's 4000 steps: {exact} cycles in the issue model, {fixed} at one cycle an instruction");
    assert_ne!(exact, fixed, "the compromise changes the time the program takes");
}

