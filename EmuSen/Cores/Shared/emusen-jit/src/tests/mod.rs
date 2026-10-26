//! The crate's own tests, on a toy processor: four registers, sixteen words of memory, five instructions, its interpreter and its
//! lowering. They exercise the thread, publication, the W^X batches, the code budget and the verifier (EmuSen_Jit.md §5).

use std::mem::offset_of;
use std::panic::{AssertUnwindSafe, catch_unwind};

use cranelift_codegen::ir::types::{I32, I64};
use cranelift_codegen::ir::{self, AbiParam, InstBuilder, MemFlagsData};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use cranelift_jit::JITModule;
use cranelift_module::Module;

use crate::verify::{Reference, Verifier};
use crate::{FAILED, Jit, Lowering, Slot, exit_name};

/// The toy machine a block runs on.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Toy {
    regs: [u64; 4],
    mem: [u64; 16],
    pc: u32,
    steps: u64,
}

const REGS: i32 = offset_of!(Toy, regs) as i32;
const MEM: i32 = offset_of!(Toy, mem) as i32;
const PC: i32 = offset_of!(Toy, pc) as i32;
const STEPS: i32 = offset_of!(Toy, steps) as i32;

// The instructions: the operation in the top byte, register a, register b, an 8-bit immediate.
const ADDI: u32 = 1;
const ADD: u32 = 2;
const XOR: u32 = 3;
const STORE: u32 = 4;
const HALT: u32 = 5;
/// An operation the lowering refuses, so that a block publishes the sentinel.
const REFUSED: u32 = 0xEE;

const DONE: u32 = 0;
const HALTED: u32 = 1;
const UNKNOWN: u32 = 2;

fn word(op: u32, a: u32, b: u32, imm: u32) -> u32 {
    op << 24 | a << 16 | b << 8 | imm
}

fn fields(w: u32) -> (u32, usize, usize, u64) {
    (w >> 24, (w >> 16 & 3) as usize, (w >> 8 & 3) as usize, (w & 0xFF) as u64)
}

/// The interpreter: one instruction, the pc and the step count after it; false for an instruction that ends a block unrun.
fn step(toy: &mut Toy, w: u32) -> bool {
    let (op, a, b, imm) = fields(w);
    match op {
        ADDI => toy.regs[a] = toy.regs[a].wrapping_add(imm),
        ADD => toy.regs[a] = toy.regs[a].wrapping_add(toy.regs[b]),
        XOR => toy.regs[a] ^= toy.regs[b],
        STORE => toy.mem[(imm & 15) as usize] = toy.regs[a],
        _ => return false,
    }
    toy.pc += 4;
    toy.steps += 1;
    true
}

/// How the toy compiles a block: with the verifier's call after every instruction, and with ADDI deliberately subtracting.
#[derive(Clone, Copy, Default)]
struct Options {
    verify: bool,
    miscompile: bool,
}

#[derive(Clone)]
struct ToyLowering;

impl Lowering for ToyLowering {
    type Options = Options;
    const NAME: &'static str = "toy";
    const EXITS: &'static [&'static str] = &["done", "halted", "unknown"];

    fn lower(&mut self, module: &JITModule, func: &mut ir::Function, functions: &mut FunctionBuilderContext, words: &[u32], start: u32, options: Options) -> Result<(), String> {
        if words.iter().any(|&w| w >> 24 == REFUSED) {
            return Err("an operation the toy refuses".into());
        }
        let ptr = module.target_config().pointer_type();
        func.signature.params.extend([AbiParam::new(ptr), AbiParam::new(ptr)]);
        func.signature.returns.push(AbiParam::new(I32));
        let mut hook = module.make_signature();
        hook.params.extend([AbiParam::new(ptr), AbiParam::new(ptr)]);
        let mut b = FunctionBuilder::new(func, functions);
        let head = b.create_block();
        b.append_block_params_for_function_params(head);
        b.switch_to_block(head);
        let (toy, harness) = (b.block_params(head)[0], b.block_params(head)[1]);
        let hook = b.import_signature(hook);
        let m = MemFlagsData::trusted();
        let leave = |b: &mut FunctionBuilder, exit: u32| {
            let v = b.ins().iconst(I32, exit as i64);
            b.ins().return_(&[v]);
        };
        let mut ended = false;
        for (k, &w) in words.iter().enumerate() {
            let (op, a, r, imm) = fields(w);
            let ra = REGS + 8 * a as i32;
            match op {
                ADDI => {
                    let v = b.ins().load(I64, m, toy, ra);
                    let v = if options.miscompile { b.ins().iadd_imm_s(v, -(imm as i64)) } else { b.ins().iadd_imm_s(v, imm as i64) };
                    b.ins().store(m, v, toy, ra);
                }
                ADD | XOR => {
                    let x = b.ins().load(I64, m, toy, ra);
                    let y = b.ins().load(I64, m, toy, REGS + 8 * r as i32);
                    let v = if op == ADD { b.ins().iadd(x, y) } else { b.ins().bxor(x, y) };
                    b.ins().store(m, v, toy, ra);
                }
                STORE => {
                    let v = b.ins().load(I64, m, toy, ra);
                    b.ins().store(m, v, toy, MEM + 8 * (imm & 15) as i32);
                }
                _ => {
                    leave(&mut b, if op == HALT { HALTED } else { UNKNOWN });
                    ended = true;
                    break;
                }
            }
            let pc = b.ins().iconst(I32, (start + 4 * (k as u32 + 1)) as i64);
            b.ins().store(m, pc, toy, PC);
            let s = b.ins().load(I64, m, toy, STEPS);
            let s = b.ins().iadd_imm_s(s, 1);
            b.ins().store(m, s, toy, STEPS);
            if options.verify {
                let f = b.ins().iconst(ptr, follow as *const () as usize as i64);
                b.ins().call_indirect(hook, f, &[toy, harness]);
            }
        }
        if !ended {
            leave(&mut b, DONE);
        }
        b.seal_all_blocks();
        b.finalize(module.target_config());
        Ok(())
    }
}

/// The toy's interpreter as the verifier's reference: its machine and the program it fetches from.
struct Interpreter {
    toy: Toy,
    program: Vec<u32>,
}

impl Reference for Interpreter {
    type Subject<'a> = &'a Toy;

    fn step(&mut self) {
        let w = self.program[(self.toy.pc / 4) as usize];
        assert!(step(&mut self.toy, w), "the reference stepped an instruction that ends a block");
    }

    fn differs(&self, subject: &Toy) -> Option<String> {
        let (a, b) = (&self.toy, subject);
        (a.regs != b.regs || a.pc != b.pc || a.steps != b.steps).then(|| format!("interpreter pc {:X} regs {:X?}; blocks pc {:X} regs {:X?}", a.pc, a.regs, b.pc, b.regs))
    }

    fn differs_whole(&self, subject: &Toy) -> Option<String> {
        (self.toy.mem != subject.mem).then(|| "in memory".to_string())
    }
}

/// What a compiled block's second argument points at: the verifier, and the first failure it found.
struct Harness {
    verifier: Option<Verifier<Interpreter>>,
    failure: Option<String>,
}

/// The verifier's call from compiled code; a failure is kept, since a panic may not cross the code's frames.
unsafe extern "C" fn follow(toy: *mut Toy, harness: *mut Harness) {
    // SAFETY: the block lends its machine and the harness for the call.
    let (toy, harness) = unsafe { (&*toy, &mut *harness) };
    let Some(v) = harness.verifier.as_mut() else { return };
    if harness.failure.is_none()
        && let Err(e) = catch_unwind(AssertUnwindSafe(|| v.follow(toy, "a compiled instruction")))
    {
        harness.failure = Some(e.downcast_ref::<String>().cloned().unwrap_or_default());
    }
}

type Code = unsafe extern "C" fn(*mut Toy, *mut Harness) -> u32;

fn run(code: *const u8, toy: &mut Toy, harness: &mut Harness) -> u32 {
    // SAFETY: the toy's lowering gives every block this signature, and its code reads nothing but the two pointers.
    unsafe { std::mem::transmute::<*const u8, Code>(code)(toy, harness) }
}

fn quiet() -> Harness {
    Harness { verifier: None, failure: None }
}

fn jit() -> Jit<ToyLowering> {
    Jit::new(ToyLowering, u64::MAX)
}

/// A program of `n` instructions drawn from the four that run, by a small generator.
fn program(seed: u64, n: usize) -> Vec<u32> {
    let mut x = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    (0..n)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            word(1 + (x % 4) as u32, (x >> 8 & 3) as u32, (x >> 10 & 3) as u32, (x >> 16 & 0xFF) as u32)
        })
        .collect()
}

fn interpreted(from: &Toy, words: &[u32]) -> Toy {
    let mut t = *from;
    for &w in words {
        if !step(&mut t, w) {
            break;
        }
    }
    t
}

#[test]
fn published_code_leaves_the_interpreters_machine_and_names_its_exits() {
    let mut jit = jit();
    let start = Toy { regs: [1, 2, 3, 4], ..Toy::default() };
    let cases = [
        (vec![word(ADDI, 0, 0, 5), word(ADD, 1, 0, 0), word(XOR, 2, 1, 0), word(STORE, 1, 0, 3), word(ADDI, 3, 0, 7)], DONE),
        (vec![word(ADDI, 0, 0, 9), word(HALT, 0, 0, 0), word(ADDI, 0, 0, 1)], HALTED),
        (vec![word(ADD, 2, 3, 0), word(0x09, 0, 0, 0)], UNKNOWN),
    ];
    for (words, exit) in cases {
        let code = jit.enqueue(words.clone().into(), 0, Options::default()).wait().expect("compiled");
        let mut toy = start;
        assert_eq!(run(code, &mut toy, &mut quiet()), exit);
        assert_eq!(toy, interpreted(&start, &words));
        assert_eq!(exit_name::<ToyLowering>(exit), Some(ToyLowering::EXITS[exit as usize]));
    }
    assert_eq!(exit_name::<ToyLowering>(3), None);
    let c = jit.counters().unwrap();
    assert_eq!((c.queued.load(std::sync::atomic::Ordering::Relaxed), c.compiled.load(std::sync::atomic::Ordering::Relaxed)), (3, 3));
}

/// Each block's code read from the dispatcher's thread as soon as its slot shows it: under ThreadSanitizer, the bytes the compiler's
/// thread wrote are ordered before this read by the slot's Release and Acquire alone (EmuSen_Jit.md §5).
#[test]
fn code_is_seen_whole_by_the_thread_that_reads_its_slot() {
    let mut jit = jit();
    for seed in 1..=200 {
        let words = program(seed, 1 + (seed % 24) as usize);
        let slot = jit.enqueue(words.clone().into(), 0x100, Options::default());
        let code = loop {
            if let Some(code) = slot.published() {
                break code;
            }
            std::hint::spin_loop();
        };
        // SAFETY: published code is at least its first instruction long.
        let first = unsafe { std::slice::from_raw_parts(code, 4) }.to_vec();
        assert!(first.iter().any(|&b| b != 0), "seed {seed}: the code's first bytes read as zero");
        let start = Toy { regs: [seed, seed << 8, !seed, 3], pc: 0x100, ..Toy::default() };
        let mut toy = start;
        assert_eq!(run(code, &mut toy, &mut quiet()), DONE);
        assert_eq!(toy, interpreted(&start, &words), "seed {seed}");
    }
}

/// The pages of `/proc/self/maps` that hold an address, as their permissions.
#[cfg(target_os = "linux")]
fn permissions(address: usize) -> String {
    let maps = std::fs::read_to_string("/proc/self/maps").unwrap();
    for line in maps.lines() {
        let mut parts = line.split_whitespace();
        let (range, perms) = (parts.next().unwrap(), parts.next().unwrap());
        let (lo, hi) = range.split_once('-').unwrap();
        let (lo, hi) = (usize::from_str_radix(lo, 16).unwrap(), usize::from_str_radix(hi, 16).unwrap());
        if (lo..hi).contains(&address) {
            return perms.to_string();
        }
    }
    panic!("no mapping holds {address:X}");
}

/// W^X: every page holding published code is readable and executable and never writable, and a later batch neither makes it
/// writable again nor changes a byte of it.
#[cfg(target_os = "linux")]
#[test]
fn code_pages_are_never_writable_and_a_later_batch_leaves_them_alone() {
    let mut jit = jit();
    let compile = |jit: &mut Jit<ToyLowering>, seeds: std::ops::Range<u64>| -> Vec<(Vec<u32>, std::sync::Arc<Slot>)> {
        let slots: Vec<_> = seeds.map(|s| (program(s, 16), s)).map(|(w, _)| (w.clone(), jit.enqueue(w.into(), 0, Options::default()))).collect();
        jit.drain();
        slots
    };
    let check = |blocks: &[(Vec<u32>, std::sync::Arc<Slot>)]| {
        for (words, slot) in blocks {
            let code = slot.published().unwrap();
            let p = permissions(code as usize);
            assert!(p.starts_with("r-x"), "code at {:X} sits in a page marked {p}", code as usize);
            let mut toy = Toy::default();
            assert_eq!(run(code, &mut toy, &mut quiet()), DONE);
            assert_eq!(toy, interpreted(&Toy::default(), words));
        }
    };
    let first = compile(&mut jit, 1..40);
    check(&first);
    // SAFETY: each published function is at least 16 bytes of a page that stays mapped while the compiler lives.
    let bytes: Vec<Vec<u8>> = first.iter().map(|(_, s)| unsafe { std::slice::from_raw_parts(s.published().unwrap(), 16) }.to_vec()).collect();
    let second = compile(&mut jit, 40..120);
    check(&second);
    check(&first);
    for ((_, s), was) in first.iter().zip(&bytes) {
        // SAFETY: as above.
        assert_eq!(unsafe { std::slice::from_raw_parts(s.published().unwrap(), 16) }, &was[..], "a later batch rewrote published code");
    }
}

#[test]
fn a_block_the_lowering_refuses_publishes_the_sentinel_and_the_next_compiles() {
    let mut jit = jit();
    let refused = jit.enqueue(vec![word(ADDI, 0, 0, 1), word(REFUSED, 0, 0, 0)].into(), 0, Options::default());
    assert_eq!(refused.wait(), None);
    assert!(refused.settled());
    let next = jit.enqueue(vec![word(ADDI, 1, 0, 2)].into(), 0, Options::default());
    let code = next.wait().expect("the block after a refusal compiles");
    let mut toy = Toy::default();
    assert_eq!(run(code, &mut toy, &mut quiet()), DONE);
    assert_eq!(toy.regs[1], 2);
    let c = jit.counters().unwrap();
    let get = |a: &std::sync::atomic::AtomicU64| a.load(std::sync::atomic::Ordering::Acquire);
    assert_eq!((get(&c.failed), get(&c.compiled), get(&c.finished), get(&c.queued)), (1, 1, 2, 2));
    assert_eq!(FAILED, 1);
}

/// The budget: under it nothing happens; past it the module goes whole, the drop is counted, and the next block starts afresh.
#[test]
fn past_its_budget_the_module_goes_whole_and_compiling_starts_again() {
    let mut jit = jit();
    assert!(!jit.over_budget(), "a compiler that never started is over no budget");
    jit.enqueue(program(7, 20).into(), 0, Options::default()).wait().unwrap();
    let bytes = jit.counters().unwrap().bytes.load(std::sync::atomic::Ordering::Relaxed);
    assert!(bytes > 0);
    jit.budget = bytes;
    assert!(!jit.over_budget(), "code exactly at the budget is kept");
    assert_eq!(jit.flushes, 0);
    jit.budget = bytes - 1;
    assert!(jit.over_budget(), "code past the budget is dropped");
    assert_eq!(jit.flushes, 1);
    assert!(jit.counters().is_none(), "the compiler and its module are gone");
    let again = jit.enqueue(program(8, 4).into(), 0, Options::default());
    assert!(again.wait().is_some());
    assert_eq!(jit.counters().unwrap().queued.load(std::sync::atomic::Ordering::Relaxed), 1, "a new compiler counts from nothing");
    let copy = jit.clone();
    assert!(copy.counters().is_none() && copy.budget == jit.budget && copy.flushes == 0, "a copy keeps the settings and no code");
}

/// A dispatcher's loop that keeps to its budget, as MarsRT's does before each run: no run starts with more code than the budget.
#[test]
fn a_dispatcher_that_checks_its_budget_each_run_starts_no_run_past_it() {
    let budget = 4096;
    let mut jit = Jit::new(ToyLowering, budget);
    for run in 0..30u64 {
        jit.over_budget();
        let held = jit.counters().map_or(0, |c| c.bytes.load(std::sync::atomic::Ordering::Relaxed));
        assert!(held <= budget, "run {run} starts with {held} bytes of code");
        for k in 0..4 {
            jit.enqueue(program(run * 4 + k, 24).into(), 0, Options::default());
        }
        jit.drain();
    }
    assert!(jit.flushes >= 3, "the budget dropped the module {} times", jit.flushes);
}

fn verified(start: &Toy, words: &[u32]) -> Harness {
    Harness { verifier: Some(Verifier::new(Interpreter { toy: *start, program: words.to_vec() })), failure: None }
}

#[test]
fn the_verifier_steps_beside_every_compiled_instruction_and_finds_nothing() {
    let mut jit = jit();
    for seed in 1..=40 {
        let words = program(seed, 1 + (seed % 30) as usize);
        let code = jit.enqueue(words.clone().into(), 0, Options { verify: true, ..Options::default() }).wait().unwrap();
        let start = Toy { regs: [seed, 1, 2, 3], ..Toy::default() };
        let mut toy = start;
        let mut h = verified(&start, &words);
        assert_eq!(run(code, &mut toy, &mut h), DONE);
        assert_eq!(h.failure, None, "seed {seed}");
        let v = h.verifier.unwrap();
        assert_eq!(v.steps, words.len() as u64, "seed {seed}: the verifier stepped {} of {} instructions", v.steps, words.len());
        v.check_all(&toy);
    }
}

#[test]
fn the_verifier_finds_a_miscompiled_instruction_at_that_instruction() {
    let mut jit = jit();
    let words = vec![word(ADD, 0, 1, 0), word(ADDI, 2, 0, 5), word(XOR, 3, 2, 0)];
    let code = jit.enqueue(words.clone().into(), 0, Options { verify: true, miscompile: true }).wait().unwrap();
    let start = Toy { regs: [1, 2, 3, 4], ..Toy::default() };
    let mut h = verified(&start, &words);
    run(code, &mut start.clone(), &mut h);
    let failure = h.failure.expect("the verifier let a miscompiled ADDI through");
    assert!(failure.contains("parted from the interpreter after 2 steps"), "{failure}");
}

/// A block run with no call to the verifier leaves the reference behind, and the next instruction it follows says so.
#[test]
fn the_verifier_finds_a_block_it_did_not_follow() {
    let mut jit = jit();
    let skipped = vec![word(ADDI, 0, 0, 1), word(ADDI, 1, 0, 2)];
    let followed = vec![word(ADD, 2, 0, 0)];
    let a = jit.enqueue(skipped.clone().into(), 0, Options::default()).wait().unwrap();
    let b = jit.enqueue(followed.clone().into(), 8, Options { verify: true, ..Options::default() }).wait().unwrap();
    let start = Toy::default();
    let mut program = skipped.clone();
    program.extend(&followed);
    let mut h = Harness { verifier: Some(Verifier::new(Interpreter { toy: start, program })), failure: None };
    let mut toy = start;
    run(a, &mut toy, &mut h);
    run(b, &mut toy, &mut h);
    assert!(h.failure.is_some(), "a block the verifier never stepped went unseen");
}

/// The end of a run compares the rest of the machine: a word of memory written outside the blocks is found there.
#[test]
fn the_verifiers_last_check_finds_memory_the_steps_do_not_compare() {
    let words = vec![word(STORE, 0, 0, 2)];
    let mut v = Verifier::new(Interpreter { toy: Toy::default(), program: words.clone() });
    let mut toy = Toy::default();
    step(&mut toy, words[0]);
    v.follow(&toy, "a step");
    v.check_all(&toy);
    toy.mem[9] = 1;
    v.check(&toy, "a step");
    let e = catch_unwind(AssertUnwindSafe(|| v.check_all(&toy))).expect_err("memory written outside the blocks went unseen");
    assert!(e.downcast_ref::<String>().unwrap().contains("in memory"));
}
