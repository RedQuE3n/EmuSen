//! Stages S36's and S37's gates (Sapphire_Native.md §44, §48): the EE recompiler graded against the interpreter by the
//! oracles of Mars_Native.md §5.8.5 that run on the project's own programs: the interpreter stepped beside the blocks and
//! compared after every instruction and after every block; random programs of integer instructions, branches, loads and
//! stores, seeded and compared; and the mechanisms one at a time. Every program here is the project's own, assembled for
//! the purpose.

mod d780;
mod s6;
mod s7;

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
    // The refill vector, 80000000h, takes the same handler: a TLB refill returns past its instruction.
    sys.board.map.load(0x000, &assemble(&handler(), 0x8000_0000));
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

/// D-860 (Sapphire_Native.md §30.78): while a recognised interrupt waits for its two instructions no block starts, so the
/// blocks' machine takes it where the interpreter does, a hot loop's block already compiled.
#[test]
fn no_block_starts_while_a_recognised_interrupt_waits() {
    let words = assemble(LOOP, ORIGIN);
    let mut b = eager();
    let mut plain = system(&words);
    let mut fast = system(&words);
    for _ in 0..600 {
        plain.step().unwrap();
    }
    run_steps(&mut fast, &mut b, 600);
    assert!(b.stats.entered > 0, "the loop runs compiled: {:?}", b.stats);
    for m in [&mut plain, &mut fast] {
        m.ee.cop0[sapphire::ee::cop0::STATUS] |= sapphire::ee::status::IE | sapphire::ee::status::EIE | sapphire::ee::status::IM2;
        m.ee.interrupt_after = 2;
    }
    if let Some(d) = blocks::compare(&plain, &fast, false) {
        panic!("before: {d}");
    }
    let entered = b.stats.entered;
    for k in 1..=2 {
        plain.step().unwrap();
        assert_eq!(blocks::run(&mut fast, &mut b, 64).unwrap(), 1, "step {k}: one instruction, interpreted");
        if let Some(d) = blocks::compare(&plain, &fast, false) {
            panic!("after {k} steps: {d}");
        }
    }
    assert_eq!((fast.ee.pc, b.stats.entered), (0x8000_0200, entered), "taken at the second check, no block entered");
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

/// The registers a random program computes with; t8 counts an inner loop, t9 the outer, gp holds the scratch area cached,
/// s7 the same memory through kseg1, sp the scratchpad, fp the subroutine's address, k0 and k1 are the handler's.
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

const LOADS_STORES: [&str; 22] =
    ["lb", "lbu", "lh", "lhu", "lw", "lwu", "ld", "lq", "lwl", "lwr", "ldl", "ldr", "sb", "sh", "sw", "sd", "sq", "swl", "swr", "sdl", "sdr", "pref"];

/// The alignment a load or store needs to raise no address error (LQ and SQ ignore their low four bits).
fn alignment(name: &str) -> u64 {
    match name {
        "lh" | "lhu" | "sh" => 2,
        "lw" | "lwu" | "sw" => 4,
        "ld" | "sd" => 8,
        "lq" | "sq" => 16,
        _ => 1,
    }
}

/// A load, store or PREF of any width: mostly aligned in a span of three data caches of the scratch area, cached
/// through gp or past the caches through s7, or in the scratchpad through sp; now and then misaligned, unmapped (a TLB
/// refill), a timer's count read, or the scratch area through kseg0.
fn other(r: &mut Rng) -> String {
    let d = *r.pick(&POOL);
    let name = *r.pick(&LOADS_STORES);
    let a = alignment(name);
    let store = name.starts_with('s');
    let (base, span, setup) = match r.below(20) {
        0..=9 => ("gp", 0x6000, None),
        10..=13 => ("s7", 0x6000, None),
        14..=17 => ("sp", 0x4000, None),
        18 => ("k1", 0x6000, Some("lui k1, 0x8030")),
        _ => match r.below(2) {
            0 => ("k1", 0x1000, Some("lui k1, 0x4000")),
            _ if !store => ("k1", 0x10, Some("lui k1, 0xb000")),
            _ => ("gp", 0x6000, None),
        },
    };
    let off = if r.below(16) == 0 { r.below(span) } else { a * r.below(span / a) };
    let off = if base == "k1" && span == 0x10 { 0 } else { off };
    let access = if name == "pref" { format!("pref 0, {off}({base})") } else { format!("{name} {d}, {off}({base})") };
    match setup {
        Some(s) => format!("{s}\n{access}"),
        None => access,
    }
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
    l.push(format!("lui s7, 0x{:x}", (SCRATCH | 0xA000_0000) >> 16));
    l.push("lui sp, 0x7000".into());
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
        total.dmac_stops += s.dmac_stops;
        total.kept_past += s.kept_past;
        total.fills += s.fills;
        total.generations += s.generations;
        for k in 0..5 {
            total.left[k] += s.left[k];
        }
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
    // Each program runs its code once: its blocks are shaped from the lines fetched so far, as at S36, where a load or
    // store ended a block before it ran into a line not yet fetched.
    b.wait_lines = false;
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
    // PREF is the memory programs' of S37 (`every_load_and_store_runs_compiled_on_every_region`).
    let missing: Vec<_> = blocks::mnemonics().filter(|m| !names.contains(m) && *m != "pref").collect();
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


/// The bases of the regions a block's loads and stores reach: the scratch area cached through kuseg and kseg0, past the
/// caches through kseg1 and through the TLB's uncached pair at 20000000h, the accelerated pages, and the scratchpad.
const REGIONS: [(&str, u32); 6] =
    [("kuseg", 0x0030_0000), ("kseg0", 0x8030_0000), ("kseg1", 0xA030_0000), ("uncached", 0x2030_0000), ("accelerated", 0x3040_0000), ("scratchpad", 0x7000_0000)];

/// Every load, store and PREF at every byte of a quadword its alignment allows, in each region, forty passes, the
/// data cache's lines written in one region and read through another.
fn regions_program() -> String {
    let mut l = vec!["ori t9, zero, 40".to_string(), "top:".into()];
    for (n, &(_, base)) in REGIONS.iter().enumerate() {
        l.push(format!("lui a0, 0x{:x}", base >> 16));
        l.push(format!("ori a0, a0, 0x{:x}", 0x40 * n as u32));
        for name in LOADS_STORES {
            for k in (0..16).step_by(alignment(name) as usize) {
                l.push(if name == "pref" { format!("pref 0, {k}(a0)") } else { format!("{name} t{}, {k}(a0)", k % 8) });
                l.push("daddu t8, t8, t0".into());
            }
        }
        l.push("sll t9, t9, 0".into());
    }
    l.extend(["addiu t9, t9, -1".into(), "bgtz t9, top".into(), "nop".into(), "end: beq zero, zero, end".into(), "nop".into()]);
    l.join("\n")
}

#[test]
fn every_load_and_store_runs_compiled_on_every_region() {
    let words = assemble(&regions_program(), ORIGIN);
    let mut b = eager();
    let s = twin(&words, 60_000, 500, &mut b);
    let compiled = b.compiled_mnemonics();
    let missing: Vec<_> = LOADS_STORES.iter().filter(|m| !compiled.contains(*m)).collect();
    println!("every load and store on every region: {s:?}");
    assert!(missing.is_empty(), "never compiled: {missing:?}");
    assert!(s.compiled_steps > s.interpreted, "{s:?}");
}

#[test]
fn the_stage_s7_memory_programs_leave_the_interpreters_machine() {
    for p in s7::all() {
        let name = p.name;
        let (source, expected) = p.finish();
        let words = assemble(&source, ORIGIN);
        let n = steps_to_done(&words);
        let mut b = eager();
        let s = twin(&words, n, 250, &mut b);
        let mut fast = system(&words);
        run_steps(&mut fast, &mut eager(), n);
        for (i, want) in expected.iter().enumerate() {
            if let Some(w) = want {
                assert_eq!(fast.peek(s6::RECORD + 4 * i as u32, 4) as u32, *w, "{name}: record word {i} against the manual");
            }
        }
        println!("{name}: {n} steps, {s:?}");
    }
}

/// The DMAC moving between RDRAM and the scratchpad, cycle-stealing in 128-byte units, while a loop reads and writes
/// both through its caches, past them and in the scratchpad, polling the channels between passes.
const DMA: &str = "
    lui s0, 0xb001
    ori t0, zero, 0x303
    sw t0, -0x2000(s0)
    lui s7, 0xa030
    lui gp, 0x0030
    lui sp, 0x7000
    ori t9, zero, 60
    top: lui t0, 0x0030
    sw t0, -0x2bf0(s0)
    sw zero, -0x2b80(s0)
    ori t0, zero, 48
    sw t0, -0x2be0(s0)
    ori t0, zero, 0x100
    sw t0, -0x2c00(s0)
    lui t0, 0x0038
    sw t0, -0x2ff0(s0)
    ori t0, zero, 0x400
    sw t0, -0x2f80(s0)
    ori t0, zero, 32
    sw t0, -0x2fe0(s0)
    ori t0, zero, 0x100
    sw t0, -0x3000(s0)
    ori t8, zero, 24
    inner: lq t1, 0x10(sp)
    sq t1, 0x400(sp)
    lw t2, 0x20(gp)
    addu t2, t2, t9
    sw t2, 0x20(s7)
    sd t2, 0x30(gp)
    lw t3, 0x404(sp)
    sw t3, 0x40(s7)
    ld t4, 0x48(sp)
    daddu t5, t5, t4
    addiu t8, t8, -1
    bgtz t8, inner
    sw t5, 0x50(sp)
    wait: lw t0, -0x2c00(s0)
    lw t1, -0x3000(s0)
    or t0, t0, t1
    andi t0, t0, 0x100
    bne t0, zero, wait
    nop
    addiu t9, t9, -1
    bgtz t9, top
    nop
    end: beq zero, zero, end
    nop
";

#[test]
fn loads_and_stores_beside_the_dmac_leave_the_interpreters_machine() {
    let words = assemble(DMA, ORIGIN);
    let mut b = eager();
    let s = twin(&words, 40_000, 333, &mut b);
    println!("beside the DMAC: {s:?}");
    assert!(s.compiled_steps > 8_000 && s.left[0] > 0, "compiled, and accesses left at the DMAC's events: {s:?}");
    let s = verified(&words, 20_000, Verify::Steps);
    assert!(s.compiled_steps > 5_000, "{s:?}");
}

/// A page at 04000000h, past the wired entries, of TLB entry 40 mapped and remapped under the same virtual address, made clean and invalid, its ASID changed;
/// then the scratch area through kseg0 in kernel mode and again in user mode, where it raises.
const REMAPPED: &str = "
    lui v0, 0x0400
    ori t9, zero, 30
    top: ori t0, zero, 40
    mtc0 t0, Index
    mtc0 zero, PageMask
    mtc0 v0, EntryHi
    ori t1, zero, 0xc01e
    mtc0 t1, EntryLo0
    ori t1, zero, 0xc05e
    mtc0 t1, EntryLo1
    sync.p
    tlbwi
    sync.p
    lw t2, 0x10(v0)
    addu t2, t2, t9
    sw t2, 0x10(v0)
    sd t2, 0x1008(v0)
    ori t1, zero, 0xc41e
    mtc0 t1, EntryLo0
    sync.p
    tlbwi
    sync.p
    lw t3, 0x10(v0)
    sw t3, 0x18(v0)
    ori t1, zero, 0xc41a
    mtc0 t1, EntryLo0
    sync.p
    tlbwi
    sync.p
    lw t4, 0x10(v0)
    sw t4, 0x14(v0)
    ori t1, zero, 0xc418
    mtc0 t1, EntryLo0
    sync.p
    tlbwi
    sync.p
    lw t5, 0x10(v0)
    ori t1, zero, 0x5
    or t1, t1, v0
    mtc0 t1, EntryHi
    sync.p
    lw t6, 0x1010(v0)
    mtc0 v0, EntryHi
    sync.p
    lui a0, 0x8030
    lw t7, 0x20(a0)
    sw t7, 0x24(a0)
    addiu t9, t9, -1
    bgtz t9, top
    nop
    mfc0 t0, Status
    ori t0, t0, 0x10
    mtc0 t0, Status
    sync.p
    user: lw t7, 0x20(a0)
    lw t6, 0x20(gp)
    addu t6, t6, t7
    sw t6, 0x28(a0)
    beq zero, zero, user
    sw t6, 0x2c(gp)
";

#[test]
fn a_tlb_write_or_a_mode_change_is_seen_by_the_next_access() {
    let words = assemble(&format!("lui gp, 0x0030\n{REMAPPED}"), ORIGIN);
    let mut b = eager();
    let s = twin(&words, 12_000, 7, &mut b);
    let mut fast = system(&words);
    let mut fb = eager();
    run_steps(&mut fast, &mut fb, 12_000);
    let (modified, invalid, address) = (fast.ee.exceptions[1], fast.ee.exceptions[2] + fast.ee.exceptions[3], fast.ee.exceptions[4] + fast.ee.exceptions[5]);
    println!("remapped: {s:?}; TLB modified {modified}, invalid {invalid}, address errors {address}");
    assert!(modified >= 30 && invalid >= 30 && address > 100, "the exceptions were taken");
    assert!(fb.stats.generations > 60, "the translation cache moved to a new generation at each write: {:?}", fb.stats);
}

#[test]
fn the_board_stops_a_guarded_run_before_the_dmac_runs() {
    // A transfer to the scratchpad due at once: within the guard the run stands before the event's time.
    let mut sys = System::new(ORIGIN);
    {
        use sapphire::ee::Bus;
        let b = &mut sys.board;
        b.write(0x1000_E000, 4, 1);
        b.write(0x1000_D410, 4, 0x0030_0000);
        b.write(0x1000_D480, 4, 0);
        b.write(0x1000_D420, 4, 4);
        b.write(0x1000_D400, 4, 0x100);
    }
    let due = sys.board.next_event();
    let times: Vec<u64> = (1..=8).map(|k| due.saturating_sub(4) + k).collect();
    let before = times.iter().position(|&t| t >= due).expect("the event falls inside the times");
    let mut copy = System::new(ORIGIN);
    copy.read_state(&mut emusen_native::StateReader::new(&blocks::state_of(&sys))).unwrap();
    let System { ee, board } = &mut sys;
    let (stood, tail) = board.run_through(&times, &mut ee.spram[..], times.len(), &mut |_, _| true);
    assert_eq!((stood, tail), (before, None), "the run stood before the time the event falls in");
    let System { ee, board } = &mut copy;
    let (stood, _) = board.run_through(&times, &mut ee.spram[..], 0, &mut |_, _| true);
    assert_eq!(stood, times.len(), "unguarded, every time runs");
}


/// A loop of stores whose blocks the Count interrupt falls inside, and a handler that reads what the loop stored last:
/// a rollback must leave memory as the instructions that stand left it.
const STORING: &str = "
    lui gp, 0x0030
    mfc0 k0, Count
    addiu k0, k0, 700
    mtc0 k0, Compare
    lui k0, 0x1
    ori k0, k0, 0x8001
    mtc0 k0, Status
    sync.p
    top: addiu s0, s0, 1
    sw s0, 0(gp)
    addiu s0, s0, 1
    sw s0, 0(gp)
    addiu s0, s0, 1
    sd s0, 8(gp)
    addiu s0, s0, 1
    sw s0, 0(gp)
    sll s1, s0, 3
    sw s1, 0x40(sp)
    addiu s0, s0, 1
    sw s0, 0(gp)
    addiu s0, s0, 1
    sb s0, 4(gp)
    addiu s0, s0, 1
    sw s0, 0(gp)
    beq zero, zero, top
    sw s0, 0x44(sp)
";

/// The handler for `STORING`: what the loop stored last, at 0(gp) and in the scratchpad, summed into 10h(gp).
const SEEING: &str = "
    lw k1, 0(gp)
    lw k0, 0x10(gp)
    addu k0, k0, k1
    lw k1, 0x40(sp)
    addu k0, k0, k1
    lbu k1, 4(gp)
    addu k0, k0, k1
    sw k0, 0x10(gp)
    mfc0 k1, Count
    addiu k1, k1, 700
    mtc0 k1, Compare
    sync.p
    eret
";

#[test]
fn a_rollback_leaves_memory_as_the_instructions_that_stand_left_it() {
    let words = assemble(&format!("lui sp, 0x7000\n{STORING}"), ORIGIN);
    let handler = assemble(SEEING, 0x8000_0200);
    let make = || {
        let mut sys = system(&words);
        sys.board.map.load(0x200, &handler);
        sys
    };
    let (mut plain, mut fast, mut b) = (make(), make(), eager());
    for _ in 0..30 {
        for _ in 0..1000 {
            plain.step().unwrap();
        }
        run_steps(&mut fast, &mut b, 1000);
        assert_eq!(blocks::compare(&plain, &fast, false), None);
    }
    assert!(blocks::state_of(&plain) == blocks::state_of(&fast), "the whole states differ");
    println!("stores beside interrupts: {:?}", b.stats);
    assert!(b.stats.rollbacks > 5, "interrupts fell inside the blocks: {:?}", b.stats);
}

/// A line refilled by a load, a block's store into it, then two lines of the same set read so that it is replaced, and
/// RDRAM read past the caches: the store reached memory only if it set the line's D bit.
const DIRTYING: &str = "
    lui gp, 0x0030
    lui s7, 0xa030
    top: lw t0, 0(gp)
    addiu t0, t0, 3
    sw t0, 0(gp)
    sw t0, 0x20(gp)
    lw t1, 0x2000(gp)
    lw t2, 0x4000(gp)
    lw t3, 0(s7)
    addu s0, s0, t3
    lw t4, 0x20(s7)
    addu s0, s0, t4
    sw s0, 0x100(s7)
    beq zero, zero, top
    addiu s1, s1, 1
";

#[test]
fn a_store_into_a_clean_line_dirties_it_and_reaches_memory_when_it_is_replaced() {
    let words = assemble(DIRTYING, ORIGIN);
    let s = twin(&words, 6000, 100, &mut eager());
    println!("stores into clean lines: {s:?}");
    assert!(s.compiled_steps > 2000, "{s:?}");
}
