//! The recompiler against the interpreter it must equal: random programs, then each mechanism alone, compared whole. See Mars_Native.md §5.8.

use std::sync::Arc;

use super::support::{SyntheticRom, build_rom};
use crate::cpu::cop0::{COMPARE, ENTRY_HI, ENTRY_LO0, ENTRY_LO1, INDEX, PAGE_MASK, PAGE_MASK_WRITABLE, STATUS};
use crate::cpu::cop1::FCSR_FLUSH_TO_ZERO;
use crate::cpu::tlb::{ENTRY_LO_KEPT, Tlb};
use crate::cpu::blocks::Tier;
use crate::machine::Machine;
use crate::rom::RomImage;

/// SplitMix64: the tests' own generator, seeded, so a seed names one program.
pub(crate) struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0x2545_F491_4F6C_DD1D)
    }

    pub fn u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// C#'s `Next(n)`: below `n`.
    pub fn below(&mut self, n: u64) -> u64 {
        self.u64() % n
    }

    /// C#'s `Next(a, b)`: from `a` up to `b`.
    pub fn range(&mut self, a: i64, b: i64) -> i64 {
        a + self.below((b - a) as u64) as i64
    }

    pub fn bits(&mut self, n: u32) -> u32 {
        self.below(1 << n) as u32
    }
}

const EDGES: [u64; 20] = [
    0,
    1,
    2,
    3,
    0x7F,
    0x80,
    0xFF,
    0x7FFF,
    0x8000,
    0xFFFF,
    0x7FFF_FFFF,
    0x8000_0000,
    0xFFFF_FFFF,
    0x1_0000_0000,
    0x7FFF_FFFF_FFFF_FFFF,
    0x8000_0000_0000_0000,
    0xFFFF_FFFF_FFFF_FFFF,
    0xFFFF_FFFF_8000_0000,
    0xFFFF_FFFF_7FFF_FFFF,
    0xFFFF_FFFF_FFFF_FFFE,
];

const FLOAT_EDGES: [u64; 28] = [
    0x0000_0000,
    0x8000_0000,
    0x3F80_0000,
    0xBF80_0000,
    0x4040_0000,
    0x3EAA_AAAB,
    0x7F7F_FFFF,
    0x0080_0000,
    0x7F80_0000,
    0xFF80_0000,
    0x7FC0_0000,
    0x7FBF_FFFF,
    0x0000_0001,
    0x4F00_0000,
    0xCF00_0000,
    0x3F00_0000,
    0x3FF0_0000_0000_0000,
    0xBFF0_0000_0000_0000,
    0x4000_0000_0000_0000,
    0x7FEF_FFFF_FFFF_FFFF,
    0x0010_0000_0000_0000,
    0x7FF0_0000_0000_0000,
    0x7FF8_0000_0000_0000,
    0x7FF7_FFFF_FFFF_FFFF,
    0x0000_0000_0000_0001,
    0x43E0_0000_0000_0000,
    0xC3E0_0000_0000_0000,
    0x4330_0000_0000_0000,
];

fn value(r: &mut Rng) -> u64 {
    match r.below(3) {
        0 => EDGES[r.below(EDGES.len() as u64) as usize],
        1 => r.range(-40, 40) as i16 as i64 as u64,
        _ => (r.u64() >> 1) ^ (r.below(2) << 63),
    }
}

fn status_value(r: &mut Rng) -> u64 {
    0x1000_0000
        | if r.below(4) == 0 { 0 } else { 1 << 29 }
        | if r.below(2) == 0 { 1 << 26 } else { 0 }
        | if r.below(3) == 0 { 1 << 25 } else { 0 }
        | r.below(8) << 5
        | if r.below(6) == 0 { (r.range(1, 3) as u64) << 3 } else { 0 }
        | if r.below(2) == 0 { 0x8401 } else { 0 }
        | if r.below(8) == 0 { 1 << 30 } else { 0 }
}

pub(crate) const PROGRAM_AT: u32 = 0x0400;
pub(crate) const PROGRAM_WORDS: u32 = 0x2000;
const DATA_AT: u32 = 0x0010_0000;

const SPECIAL_FUNCTIONS: [u32; 48] = [
    0x00, 0x02, 0x03, 0x04, 0x06, 0x07, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21,
    0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x2F, 0x30, 0x31, 0x32, 0x33, 0x34, 0x36, 0x38, 0x3A, 0x3B, 0x3C, 0x3E, 0x3F,
];
const MEMORY_OPS: [u32; 28] =
    [0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x29, 0x2A, 0x2B, 0x2C, 0x2D, 0x2E, 0x1A, 0x1B, 0x37, 0x3F, 0x30, 0x34, 0x38, 0x3C, 0x31, 0x35, 0x39, 0x3D, 0x2F];
const COP0_REGISTERS: [u32; 23] = [0, 1, 2, 3, 4, 5, 6, 8, 9, 10, 11, 13, 14, 16, 17, 20, 26, 27, 28, 29, 30, 7, 31];

fn pick<T: Copy>(r: &mut Rng, from: &[T]) -> T {
    from[r.below(from.len() as u64) as usize]
}

/// One instruction of `MarsRtTests.Instruction`'s mix: every class, a third of the pairs naming one register twice.
pub(crate) fn instruction(r: &mut Rng) -> u32 {
    let imm = |r: &mut Rng| if r.below(2) == 0 { r.bits(16) } else { r.range(-9, 9) as i16 as u16 as u32 };
    let pair = |r: &mut Rng| {
        let rs = r.bits(5);
        rs << 21 | (if r.below(3) == 0 { rs } else { r.bits(5) }) << 16
    };
    let offset = |r: &mut Rng| (if r.below(32) == 0 { r.range(-8, 0) } else { r.range(1, 9) }) as i16 as u16 as u32;
    let roll = r.below(100);
    match roll {
        0..22 => pair(r) | r.bits(5) << 11 | r.bits(5) << 6 | pick(r, &SPECIAL_FUNCTIONS),
        22..34 => pick(r, &[0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x18, 0x19]) << 26 | r.bits(5) << 21 | r.bits(5) << 16 | imm(r),
        34..48 => {
            let op = pick(r, &MEMORY_OPS);
            let base = r.range(16, 24) as u32;
            let scale = if r.below(3) == 0 { 1 } else { 8 };
            op << 26 | base << 21 | r.bits(5) << 16 | (r.range(-16, 16) * scale) as i16 as u16 as u32
        }
        48..55 => pick(r, &[0x04, 0x05, 0x06, 0x07, 0x14, 0x15, 0x16, 0x17]) << 26 | pair(r) | offset(r),
        55..58 => 0x0400_0000 | r.bits(5) << 21 | pick(r, &[0, 1, 2, 3, 0x10, 0x11, 0x12, 0x13, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0E]) << 16 | offset(r),
        58..60 => {
            let kind = if r.below(2) == 0 { 0x0800_0000 } else { 0x0C00_0000 };
            kind | (((0x8000_0000 | (PROGRAM_AT + r.bits(13) * 4)) >> 2) & 0x03FF_FFFF)
        }
        60..61 => (r.range(24, 26) as u32) << 21 | r.bits(5) << 11 | if r.below(2) == 0 { 0x08 } else { 0x09 },
        61..76 => 0x4400_0000 | pick(r, &[0x10, 0x11, 0x14, 0x15, 0x10, 0x11]) << 21 | r.bits(5) << 16 | r.bits(5) << 11 | r.bits(5) << 6 | r.bits(6),
        76..81 => 0x4400_0000 | pick(r, &[0, 1, 2, 4, 5, 6, 8, 3, 7]) << 21 | r.bits(5) << 16 | (if r.below(3) == 0 { 31 } else { r.bits(5) }) << 11 | offset(r),
        81..86 => 0x4000_0000 | pick(r, &[0, 1, 4, 5, 2, 8]) << 21 | r.bits(5) << 16 | pick(r, &COP0_REGISTERS) << 11,
        86..87 => 0x4080_0000 | (r.range(2, 8) as u32) << 16 | 12 << 11,
        87..88 => pick(r, &[0x4200_0001, 0x4200_0002, 0x4200_0006, 0x4200_0008, 0x4200_0018, 0x4200_0010, 0x4200_0020]),
        88..90 => pick(r, &[0x0000_000C, 0x0000_000D, 0x0000_000F]),
        90..93 => 0x4800_0000 | pick(r, &[0, 1, 2, 4, 5, 6, 3]) << 21 | r.bits(5) << 16 | r.bits(5) << 11,
        93..95 => 0xBC00_0000 | (r.range(16, 24) as u32) << 21 | r.bits(5) << 16 | (r.range(-8, 8) * 4) as i16 as u16 as u32,
        _ => r.bits(31) ^ (r.bits(1) << 31),
    }
}

fn put(m: &mut Machine, at: u32, word: u32) {
    m.bus.rdram[at as usize..at as usize + 4].copy_from_slice(&word.to_be_bytes());
}

/// `MarsRtTests.Randomise`: registers, the FPU, the timer, the TLB and memory at random; s0-s7 into the data, t8 and t9 into the program.
pub(crate) fn random_machine(seed: u64) -> Machine {
    let mut m = Machine::boot(Arc::new(RomImage::from_image(&build_rom()).unwrap()), false);
    let r = &mut Rng::new(seed);
    let cpu = &mut m.cpu;
    for i in 1..32 {
        cpu.gpr[i] = if r.below(4) == 0 && i > 1 { cpu.gpr[r.range(1, i as i64) as usize] } else { value(r) };
    }
    for i in 16..24 {
        cpu.gpr[i] = 0xFFFF_FFFF_8000_0000 | (DATA_AT + (i as u32 - 16) * 0x1000 + r.bits(8) * 8) as u64;
    }
    for i in 2..8 {
        cpu.gpr[i] = status_value(r);
    }
    cpu.gpr[24] = 0xFFFF_FFFF_8000_0000 | (PROGRAM_AT + r.bits(13) * 4) as u64;
    cpu.gpr[25] = 0xFFFF_FFFF_8000_0000 | (PROGRAM_AT + r.bits(13) * 4) as u64;
    for i in 0..32 {
        cpu.fpr[i] = if r.below(3) == 0 { r.u64() >> 1 } else { pick(r, &FLOAT_EDGES) | if r.below(4) == 0 { (r.u64() >> 1) << 32 } else { 0 } };
    }
    cpu.fcsr = r.bits(2) | if r.below(3) == 0 { FCSR_FLUSH_TO_ZERO } else { 0 } | if r.below(6) == 0 { r.bits(5) << 7 } else { 0 };
    cpu.hi = value(r);
    cpu.lo = value(r);
    cpu.cop0[STATUS] = 0x3400_0000 | if r.below(2) == 0 { 0x8001 } else { 0 } | if r.below(4) == 0 { 0xE0 } else { 0 };
    cpu.cop0[COMPARE] = (m.bus.count() as u64 + r.range(1, 20_000) as u64) & 0xFFFF_FFFF;
    for (i, entry) in cpu.tlb.entries.iter_mut().enumerate() {
        entry.page_mask = Tlb::paired_page_mask(r.u64() & PAGE_MASK_WRITABLE);
        entry.entry_hi = (r.bits(6) as u64) << 13 | r.bits(2) as u64;
        entry.entry_lo0 = ((((0x100 + r.bits(8)) as u64) << 6 | (r.bits(3) as u64) << 3 | r.bits(3) as u64) & ENTRY_LO_KEPT) | (i as u64 & 1);
        entry.entry_lo1 = ((((0x100 + r.bits(8)) as u64) << 6 | (r.bits(3) as u64) << 3 | r.bits(3) as u64) & ENTRY_LO_KEPT) | (i as u64 & 1);
    }
    // Each vector steps over the faulting instruction and returns.
    let handler = [0x401A_7000, 0x275A_0004, 0x409A_7000, 0x4200_0018];
    for vector in [0x000, 0x080, 0x180] {
        for (i, &word) in handler.iter().enumerate() {
            put(&mut m, vector + i as u32 * 4, word);
        }
    }
    for i in 0..PROGRAM_WORDS {
        let word = instruction(r);
        put(&mut m, PROGRAM_AT + i * 4, word);
    }
    for i in (0..0x8000).step_by(4) {
        let word = value(r) as u32;
        put(&mut m, DATA_AT + i, word);
    }
    m.cpu.pc = 0xFFFF_FFFF_8000_0000 | PROGRAM_AT as u64;
    m.cpu.next_pc = m.cpu.pc + 4;
    m.cpu.cop0_written(&m.bus);
    m
}

/// The tiers under test: every one built, or those `EMUSEN_MARSRT_TIERS` names by number.
pub(crate) fn tiers() -> Vec<Tier> {
    match std::env::var("EMUSEN_MARSRT_TIERS") {
        Ok(v) => v.split(',').filter_map(|n| n.trim().parse().ok()).map(Tier::from_number).collect(),
        Err(_) => vec![Tier::Decoded, Tier::Compiled, Tier::Cached],
    }
}

/// A machine through the blocks at `tier`, compiling a block at its second entry and waiting for it, so short programs run compiled.
pub(crate) fn recompiled(m: &Machine, tier: Tier) -> Machine {
    let mut subject = m.clone();
    subject.set_recompiler(true);
    subject.blocks.tier = tier;
    subject.blocks.threshold = std::env::var("EMUSEN_MARSRT_TEST_THRESHOLD").ok().and_then(|v| v.parse().ok()).unwrap_or(2);
    subject.blocks.synchronous = true;
    subject
}

/// An interpreter and a recompiler at each tier, from one machine; the variant beside a running processor is compiled too,
/// since a test that never meets one would leave it uncovered.
pub(crate) fn pairs(m: &Machine) -> (Machine, Vec<Machine>) {
    (
        m.clone(),
        tiers()
            .into_iter()
            .map(|t| {
                let mut subject = recompiled(m, t);
                subject.blocks.beside_variant = true;
                subject
            })
            .collect(),
    )
}

/// Every machine stepped alike, the processor compared every `stride` steps and everything at the end; returns the last tier's.
pub(crate) fn lockstep(m: &Machine, steps: u64, stride: u64, what: &str) -> Machine {
    let (mut reference, mut subjects) = pairs(m);
    let mut done = 0;
    while done < steps {
        reference.run_steps(stride);
        done += stride;
        for subject in &mut subjects {
            subject.run_steps(stride);
            assert!(
                reference.cpu == subject.cpu && reference.bus.cycles == subject.bus.cycles,
                "{what}, {:?}: the processors part after {done} steps (interpreter pc {:X} cycles {}, blocks pc {:X} cycles {})",
                subject.blocks.tier,
                reference.cpu.current_pc,
                reference.bus.cycles,
                subject.cpu.current_pc,
                subject.bus.cycles
            );
        }
    }
    for subject in &subjects {
        assert!(reference == *subject, "{what}, {:?}: the machines part outside the processor after {steps} steps", subject.blocks.tier);
        assert_eq!(subject.blocks.counters()[12], 0, "{what}: Cranelift refused a block");
    }
    subjects.pop().unwrap()
}

/// Every machine run frame by frame, compared whole after each; returns the last tier's.
pub(crate) fn frames(m: &Machine, frames: u32, what: &str) -> Machine {
    let (mut reference, mut subjects) = pairs(m);
    for n in 1..=frames {
        reference.run_frame();
        for subject in &mut subjects {
            subject.run_frame();
            assert!(reference == *subject, "{what}, {:?}: the machines part at frame {n} (pc {:X} and {:X})", subject.blocks.tier, reference.cpu.current_pc, subject.cpu.current_pc);
        }
    }
    subjects.pop().unwrap()
}

fn seeds() -> u64 {
    std::env::var("EMUSEN_MARSRT_SEEDS").ok().and_then(|v| v.parse().ok()).unwrap_or(160)
}

#[test]
fn a_random_program_through_the_blocks_leaves_the_state_the_interpreter_leaves() {
    let (mut instructions, mut entries, mut compiled) = (0, 0, 0);
    for seed in 1..=seeds() {
        let m = random_machine(seed);
        let subject = lockstep(&m, 24_000, 16, &format!("seed {seed}"));
        instructions += subject.blocks.stats.instructions;
        entries += subject.blocks.stats.entries;
        compiled += subject.blocks.stats.compiled_instructions;
    }
    eprintln!("{entries} entries, {instructions} instructions in blocks, {compiled} of them compiled");
    assert!(entries > 1000 && instructions > entries, "the blocks ran too little to compare: {entries} entries, {instructions} instructions");
    assert!(subjects_compile() || compiled > instructions / 4, "compiled code ran {compiled} of {instructions} instructions");
}

/// Whether the last tier under test is one that compiles; the counts above are then checked.
fn subjects_compile() -> bool {
    tiers().last().is_some_and(|&t| t == Tier::Decoded)
}

/// The verifier on: the interpreter stepped beside every instruction the blocks run, compiled or decoded, and compared.
#[test]
fn the_verifier_steps_beside_every_instruction_and_finds_nothing() {
    for seed in 1..=seeds().min(40) {
        let m = random_machine(seed);
        let mut reference = m.clone();
        reference.run_steps(24_000);
        for tier in tiers() {
            let mut subject = recompiled(&m, tier);
            subject.blocks.verify = true;
            subject.run_steps(24_000);
            assert!(reference == subject, "seed {seed}, {tier:?}: the machines part");
        }
    }
    let mut m = machine_with(0x1000, &counted_loop(30_000));
    with_devices(&mut m);
    let mut subject = recompiled(&m, Tier::BEST);
    subject.blocks.verify = true;
    for _ in 0..3 {
        subject.run_frame();
    }
    assert!(subject.blocks.stats.compiled_instructions > 100_000 || subjects_compile(), "{:?}", subject.blocks.stats);
}

// ---- The mechanisms one at a time ----

const KSEG0: u32 = 0x8000_0000;

fn r_type(rs: u32, rt: u32, rd: u32, sa: u32, funct: u32) -> u32 {
    rs << 21 | rt << 16 | rd << 11 | sa << 6 | funct
}

fn i_type(op: u32, rs: u32, rt: u32, immediate: i32) -> u32 {
    op << 26 | rs << 21 | rt << 16 | (immediate as u32 & 0xFFFF)
}

const NOP: u32 = 0;
fn addiu(rt: u32, rs: u32, v: i32) -> u32 {
    i_type(0x09, rs, rt, v)
}
fn addu(rd: u32, rs: u32, rt: u32) -> u32 {
    r_type(rs, rt, rd, 0, 0x21)
}
fn lui(rt: u32, v: u32) -> u32 {
    i_type(0x0F, 0, rt, v as i32)
}
fn ori(rt: u32, rs: u32, v: u32) -> u32 {
    i_type(0x0D, rs, rt, v as i32)
}
fn sw(rt: u32, rs: u32, offset: i32) -> u32 {
    i_type(0x2B, rs, rt, offset)
}
fn lw(rt: u32, rs: u32, offset: i32) -> u32 {
    i_type(0x23, rs, rt, offset)
}
fn bne(rs: u32, rt: u32, offset: i32) -> u32 {
    i_type(0x05, rs, rt, offset)
}
fn beq(rs: u32, rt: u32, offset: i32) -> u32 {
    i_type(0x04, rs, rt, offset)
}
fn beql(rs: u32, rt: u32, offset: i32) -> u32 {
    i_type(0x14, rs, rt, offset)
}
fn bnel(rs: u32, rt: u32, offset: i32) -> u32 {
    i_type(0x15, rs, rt, offset)
}
fn mtc0(rt: u32, rd: u32) -> u32 {
    0x4080_0000 | rt << 16 | rd << 11
}
fn mfc0(rt: u32, rd: u32) -> u32 {
    0x4000_0000 | rt << 16 | rd << 11
}
const ERET: u32 = 0x4200_0018;
fn j(target: u32) -> u32 {
    0x0800_0000 | ((target >> 2) & 0x03FF_FFFF)
}
fn jr(rs: u32) -> u32 {
    r_type(rs, 0, 0, 0, 0x08)
}

/// A booted machine with `program` at physical `at` and the program counter on it through KSEG0, interrupts off.
fn machine_with(at: u32, program: &[u32]) -> Machine {
    let mut m = Machine::boot(Arc::new(RomImage::from_image(&SyntheticRom::default().build()).unwrap()), false);
    for (i, &word) in program.iter().enumerate() {
        put(&mut m, at + 4 * i as u32, word);
    }
    m.cpu.pc = (KSEG0 | at) as i32 as i64 as u64;
    m.cpu.next_pc = m.cpu.pc + 4;
    m.cpu.cop0[STATUS] = 0x3400_0000;
    m.cpu.cop0_written(&m.bus);
    m
}

/// The general vector's handler: count in k1, step over the instruction, return.
fn with_handler(m: &mut Machine) {
    for (i, &word) in [addiu(27, 27, 1), 0x401A_7000, 0x275A_0004, 0x409A_7000, ERET].iter().enumerate() {
        put(m, 0x180 + 4 * i as u32, word);
    }
}

/// Programs the VI so its fields end, and the AI so its samples are due, both inside the loops below.
fn with_devices(m: &mut Machine) {
    for (register, value) in [(0x0440_0018u32, 0x20D), (0x0440_001C, 0x0C15), (0x0440_000C, 0x100), (0x0440_0000, 0x3202)] {
        m.bus.write32(register, value);
    }
    for (register, value) in [(0x0450_0010u32, 0x2000), (0x0450_0014, 0xF), (0x0450_0008, 1), (0x0450_0000, 0x0020_0000), (0x0450_0004, 0x1000)] {
        m.bus.write32(register, value);
    }
    m.bus.reschedule();
}

/// A counted loop: t0 counts to t1, adding into t2 each turn.
fn counted_loop(turns: i32) -> Vec<u32> {
    vec![addiu(9, 0, turns), addiu(8, 0, 0), addiu(8, 8, 1), addu(10, 10, 8), bne(8, 9, -3), addiu(11, 11, 1), j(KSEG0 | 0x1000), NOP]
}

#[test]
fn a_hot_loop_with_the_devices_due_inside_it_leaves_the_interpreters_state() {
    let mut m = machine_with(0x1000, &counted_loop(30_000));
    with_devices(&mut m);
    let subject = lockstep(&m, 400_000, 997, "the counted loop");
    assert!(subject.blocks.stats.instructions > 300_000, "the loop did not run in blocks");
}

#[test]
fn frames_through_the_blocks_end_where_the_interpreters_end() {
    let mut m = machine_with(0x1000, &counted_loop(5_000));
    with_devices(&mut m);
    let subject = frames(&m, 12, "the loop by frames");
    assert!(subject.blocks.stats.entries > 1000);
    // No VI: every frame ends on the cycle cap.
    frames(&machine_with(0x1000, &counted_loop(5_000)), 3, "the loop to the cap");
}

#[test]
fn the_timer_interrupt_lands_where_the_interpreter_takes_it() {
    let mut m = machine_with(0x1000, &counted_loop(30_000));
    with_handler(&mut m);
    // Each interrupt moves Compare on, so it fires again; k0 is the handler's.
    for (i, &word) in [mfc0(26, 9), addiu(26, 26, 777), mtc0(26, 11), addiu(27, 27, 1), ERET].iter().enumerate() {
        put(&mut m, 0x180 + 4 * i as u32, word);
    }
    m.cpu.cop0[STATUS] = 0x3400_8001;
    m.cpu.cop0[COMPARE] = (m.bus.count() + 500) as u64;
    m.cpu.cop0_written(&m.bus);
    let subject = lockstep(&m, 300_000, 1_009, "the timer");
    assert!(subject.cpu.gpr[27] > 100, "the timer fired {} times", subject.cpu.gpr[27]);
}

#[test]
fn a_fault_in_the_middle_of_a_block_is_taken_on_its_instruction() {
    // A load from an address that is not aligned, every turn: the handler steps over it.
    let program = [lui(4, 0x8010), ori(4, 4, 2), addiu(8, 8, 1), lw(5, 4, 0), addiu(10, 10, 1), beq(0, 0, -4), addiu(11, 11, 1)];
    let mut m = machine_with(0x1000, &program);
    with_handler(&mut m);
    let subject = lockstep(&m, 50_000, 211, "the fault");
    assert!(subject.cpu.gpr[27] > 1000, "the fault was taken {} times", subject.cpu.gpr[27]);
}

#[test]
fn a_block_that_stores_into_its_own_words_runs_the_new_ones() {
    // The loop rewrites the immediate of its own add each turn: the next pass must add the new amount.
    let program = [
        lui(4, 0x8000),
        ori(4, 4, 0x1014),
        lw(5, 4, 0),
        addiu(5, 5, 1),
        sw(5, 4, 0),
        addiu(10, 10, 0),
        beq(0, 0, -5),
        NOP,
    ];
    let m = machine_with(0x1000, &program);
    let subject = lockstep(&m, 20_000, 223, "the rewriting loop");
    assert!(subject.cpu.gpr[10] > 1000 && subject.blocks.stats.discarded > 100, "{} discarded", subject.blocks.stats.discarded);
}

#[test]
fn code_replaced_under_a_block_from_outside_is_run_as_replaced() {
    let m = machine_with(0x1000, &counted_loop(100));
    let (mut reference, mut subject) = (m.clone(), recompiled(&m, Tier::BEST));
    for turn in 0..40u32 {
        reference.run_steps(3_001);
        subject.run_steps(3_001);
        assert!(reference == subject, "turn {turn}");
        // A new amount for the loop's add, written as the host's poke or a DMA would write it.
        for machine in [&mut reference, &mut subject] {
            put(machine, 0x100C, addu(10, 10, 8 + (turn % 3)));
            *machine.bus.written += 1;
        }
    }
    assert!(subject.blocks.stats.discarded > 10);
}

#[test]
fn a_likely_branch_not_taken_leaves_its_slot_unrun() {
    let program = [addiu(8, 8, 1), andi_(9, 8, 3), bnel(9, 0, 2), addiu(10, 10, 1), addiu(11, 11, 1), beql(0, 0, -6), addiu(12, 12, 1)];
    let m = machine_with(0x1000, &program);
    lockstep(&m, 30_000, 227, "the likely branches");
}

fn andi_(rt: u32, rs: u32, v: u32) -> u32 {
    i_type(0x0C, rs, rt, v as i32)
}

#[test]
fn a_branch_in_a_delay_slot_is_left_to_the_interpreter() {
    let program = [addiu(8, 8, 1), beq(0, 0, 3), bne(8, 0, -2), addiu(10, 10, 1), addiu(11, 11, 1), addiu(12, 12, 1), j(KSEG0 | 0x1000), NOP];
    let m = machine_with(0x1000, &program);
    lockstep(&m, 20_000, 229, "the branch in a slot");
}

#[test]
fn the_multiply_and_divide_stalls_are_ticked() {
    let program = [
        addiu(4, 4, 7),
        0x0085_0018,
        0x0085_001A,
        0x0085_001C,
        0x0085_001E,
        0x0000_2012,
        addu(5, 5, 4),
        beq(0, 0, -8),
        NOP,
    ];
    let m = machine_with(0x1000, &program);
    lockstep(&m, 40_000, 233, "the stalls");
}

#[test]
fn a_store_to_the_vi_inside_a_loop_is_seen_at_once() {
    // Writing VI_V_INTR each turn moves the next event; the block must leave after the store.
    let program = [lui(4, 0xA440), addiu(5, 5, 1), andi_(5, 5, 0x1FF), sw(5, 4, 0x0C), addiu(10, 10, 1), beq(0, 0, -5), NOP];
    let mut m = machine_with(0x1000, &program);
    with_devices(&mut m);
    lockstep(&m, 200_000, 269, "the VI store");
}

/// A loop across two pages whose frames are not neighbours, with a decoy after the first frame: `Mars_Recompiler.md` §17.
#[test]
fn a_mapped_loop_across_two_pages_leaves_the_interpreters_state() {
    let mut m = machine_with(0x0020_0000, &[]);
    // Virtual 0x0040_0000 and 0x0040_1000, a pair, mapped to frames 0x0020_0000 and 0x0030_0000.
    let entry = &mut m.cpu.tlb.entries[3];
    entry.page_mask = 0;
    entry.entry_hi = 0x0040_0000;
    entry.entry_lo0 = (0x0020_0000 >> 12) << 6 | 0x1E | 1;
    entry.entry_lo1 = (0x0030_0000 >> 12) << 6 | 0x1E | 1;
    let first = [addiu(8, 8, 1), addiu(10, 10, 2)];
    for (i, &word) in first.iter().enumerate() {
        put(&mut m, 0x0020_0FF8 + 4 * i as u32, word);
    }
    // The decoy, in the frame's own next page, which a block not held to its page would run.
    put(&mut m, 0x0020_1000, addiu(12, 12, 1));
    put(&mut m, 0x0020_1004, addiu(12, 12, 1));
    let second = [addiu(11, 11, 1), beq(0, 0, -4), NOP];
    for (i, &word) in second.iter().enumerate() {
        put(&mut m, 0x0030_0000 + 4 * i as u32, word);
    }
    m.cpu.pc = 0x0040_0FF8;
    m.cpu.next_pc = 0x0040_0FFC;
    m.cpu.cop0_written(&m.bus);
    let subject = lockstep(&m, 30_000, 239, "the mapped loop");
    assert!(subject.blocks.stats.mapped > 1000, "{} mapped entries", subject.blocks.stats.mapped);
    assert_eq!(subject.cpu.gpr[12], 0);
}

#[test]
fn a_page_remapped_by_a_tlb_write_alone_is_fetched_from_its_new_frame() {
    // Two frames hold loops that add 1 and 2; every 200 turns the loop rewrites the entry to point at the other frame.
    let mut m = machine_with(0x0010_0000, &[]);
    let body = |amount: i32| [addiu(10, 10, amount), addiu(8, 8, 1), andi_(9, 8, 0xFF), bne(9, 0, -4), NOP, lui(7, 0x8010), jr(7), NOP];
    for (frame, amount) in [(0x0020_0000u32, 1), (0x0030_0000, 2)] {
        for (i, &word) in body(amount).iter().enumerate() {
            put(&mut m, frame + 4 * i as u32, word);
        }
    }
    // The switch, in KSEG0: flip EntryLo0 between the frames, write entry 0, and jump back into the mapped page.
    let switch = [
        mfc0(4, ENTRY_LO0 as u32),
        ori(5, 0, (0x0020_0000u32 >> 12 << 6) ^ (0x0030_0000u32 >> 12 << 6)),
        0x0085_2026,
        mtc0(4, ENTRY_LO0 as u32),
        mtc0(0, INDEX as u32),
        0x4200_0002,
        lui(6, 0x0040),
        jr(6),
        NOP,
    ];
    for (i, &word) in switch.iter().enumerate() {
        put(&mut m, 0x0010_0000 + 4 * i as u32, word);
    }
    m.cpu.cop0[ENTRY_HI] = 0x0040_0000;
    m.cpu.cop0[PAGE_MASK] = 0;
    m.cpu.cop0[ENTRY_LO0] = (0x0020_0000 >> 12) << 6 | 0x1E | 1;
    m.cpu.cop0[ENTRY_LO1] = 0x1E | 1;
    let entry = &mut m.cpu.tlb.entries[0];
    entry.entry_hi = 0x0040_0000;
    entry.entry_lo0 = (0x0020_0000 >> 12) << 6 | 0x1E | 1;
    entry.entry_lo1 = 0x1F;
    m.cpu.pc = 0x0040_0000;
    m.cpu.next_pc = 0x0040_0004;
    m.cpu.cop0_written(&m.bus);
    let subject = lockstep(&m, 60_000, 241, "the remapped page");
    assert!(subject.cpu.gpr[10] > 10_000 && subject.blocks.stats.mapped > 100, "{} added, {} mapped", subject.cpu.gpr[10], subject.blocks.stats.mapped);
}

#[test]
fn a_jump_to_an_unaligned_address_or_out_of_the_direct_segments_faults_as_the_interpreter_faults() {
    let mut m = machine_with(0x1000, &[lui(6, 0x8000), ori(6, 6, 0x1012), addiu(10, 10, 1), jr(6), NOP]);
    with_handler(&mut m);
    // The handler returns to the loop's head rather than past a fault it cannot step over.
    for (i, &word) in [addiu(27, 27, 1), lui(26, 0x8000), ori(26, 26, 0x1008), mtc0(26, 14), ERET].iter().enumerate() {
        put(&mut m, 0x180 + 4 * i as u32, word);
        put(&mut m, 4 * i as u32, word);
    }
    lockstep(&m, 20_000, 251, "the misaligned jump");
    put(&mut m, 0x1004, ori(6, 6, 0x2010));
    put(&mut m, 0x1000, lui(6, 0));
    lockstep(&m, 20_000, 257, "the jump into kuseg");
}

#[test]
fn a_machine_loaded_from_a_state_keeps_no_block_that_disagrees_with_memory() {
    let m = machine_with(0x1000, &counted_loop(2_000));
    let (mut reference, mut subject) = (m.clone(), recompiled(&m, Tier::BEST));
    reference.run_steps(50_000);
    subject.run_steps(50_000);
    let state = reference.save_state_vec(false).unwrap();
    // The subject returns to an earlier state, whose loop is the same, then to a different program.
    let mut other = machine_with(0x1000, &counted_loop(7));
    other.run_steps(10);
    let earlier = other.save_state_vec(false).unwrap();
    other.restore_state(&earlier).unwrap();
    subject.restore_state(&earlier).unwrap();
    subject.run_steps(20_000);
    other.run_steps(20_000);
    assert!(other.cpu == subject.cpu && other.bus == subject.bus);
    subject.restore_state(&state).unwrap();
    reference.restore_state(&state).unwrap();
    reference.run_steps(20_000);
    subject.run_steps(20_000);
    assert!(reference == subject);
}

/// The signal processor running its own loop while the CPU's blocks run: the variant that steps it after every instruction.
#[test]
fn blocks_run_beside_a_running_processor_leave_the_interpreters_state() {
    let mut m = machine_with(0x1000, &counted_loop(30_000));
    with_devices(&mut m);
    for (i, &word) in [addiu(2, 2, 1), bne(2, 0, -2), NOP].iter().enumerate() {
        m.bus.write32(0x0400_1000 + 4 * i as u32, word);
    }
    m.bus.write32(0x0408_0000, 0);
    m.bus.write32(0x0404_0010, 0x0005);
    assert!(!m.bus.sp.processor.halted, "the processor did not start");

    let subject = lockstep(&m, 200_000, 337, "the processor beside");
    assert!(subject.bus.sp.processor.gpr[2] > 10_000, "the processor ran {} steps", subject.bus.sp.processor.gpr[2]);
    assert!(subject.blocks.stats.compiled_beside > 100 || subjects_compile(), "{:?}", subject.blocks.stats);
}

/// `JALR` whose link is its own target register: the interpreter reads the target before it writes the link.
/// Written for the mutant that reversed the two, which nothing else caught (Mars_Native.md §5.8.6).
#[test]
fn a_register_call_whose_link_is_its_own_register_leaves_the_interpreters_state() {
    let program = [lui(31, 0x8000), ori(31, 31, 0x1014), r_type(31, 0, 31, 0, 0x09), NOP, addiu(10, 10, 1), addiu(11, 11, 1), j(KSEG0 | 0x1000), NOP];
    let m = machine_with(0x1000, &program);
    let subject = lockstep(&m, 20_000, 263, "the register call");
    assert!(subject.cpu.gpr[11] > 1000 && subject.cpu.gpr[10] == 0, "r10 {} r11 {}", subject.cpu.gpr[10], subject.cpu.gpr[11]);
}

/// A compiled block that rewrites its own last word now and then: the pass that writes must run the new word, not the
/// one its code holds. The word alternates with the counter's bit that turns over as often as the address comes back
/// to the block, so the block stands still long enough to be compiled between writes (§5.8.6).
#[test]
fn a_compiled_block_that_rewrites_its_own_word_runs_the_new_one() {
    let program = [
        addiu(9, 9, 1),
        r_type(0, 9, 3, 8, 0x02),
        andi_(3, 3, 1),
        r_type(0, 3, 2, 16, 0x00),
        r_type(0, 3, 1, 21, 0x00),
        r_type(2, 1, 3, 0, 0x25),
        lui(6, 0x25CE),
        ori(6, 6, 1),
        r_type(6, 3, 5, 0, 0x26),
        addiu(8, 8, 4),
        andi_(8, 8, 0x3FC),
        lui(4, 0x8000),
        r_type(4, 8, 4, 0, 0x21),
        addiu(4, 4, 0x1044),
        sw(5, 4, 0),
        addiu(10, 10, 1),
        beq(0, 0, -17),
        NOP,
    ];
    let m = machine_with(0x1000, &program);
    let subject = lockstep(&m, 200_000, 281, "the block that rewrites its own word");
    assert!(subject.cpu.gpr[14] > 4 && subject.cpu.gpr[15] > 4, "the word was never rewritten both ways: r14 {} r15 {}", subject.cpu.gpr[14], subject.cpu.gpr[15]);
    assert!(subject.blocks.stats.discarded > 4, "{} discarded", subject.blocks.stats.discarded);
}

/// The signal processor's DMA writing over a block again and again, each time with another word: the block leaves at
/// the write it did not make, and the word it runs is the one in memory. The transfer's address is eight-byte aligned,
/// so the word it lands on is the block's last (§5.8.6).
#[test]
fn a_block_the_processors_dma_writes_over_runs_the_new_words() {
    let program = [
        addiu(10, 10, 1),
        addiu(11, 11, 1),
        addiu(12, 12, 1),
        addiu(13, 13, 1),
        addiu(8, 8, 1),
        addiu(9, 9, 1),
        addiu(10, 10, 1),
        addiu(11, 11, 1),
        addiu(12, 12, 1),
        addiu(13, 13, 1),
        addiu(8, 8, 1),
        addiu(9, 9, 1),
        addiu(10, 10, 1),
        beq(0, 0, -14),
        NOP,
    ];
    let mut m = machine_with(0x1000, &program);
    with_devices(&mut m);
    // The processor toggles a word in its own memory and hands it to the transfer, which lands on the loop's slot.
    let rsp = [
        0x3C05_25CE,
        0x34A5_0001,
        0x3C06_0021,
        0x3402_1038,
        0x3403_0007,
        0x00A6_2826,
        0xAC05_0000,
        0x4081_0000,
        0x4082_0800,
        0x4083_1800,
        0x0800_0005,
        NOP,
    ];
    for (i, &word) in rsp.iter().enumerate() {
        m.bus.write32(0x0400_1000 + 4 * i as u32, word);
    }
    m.bus.write32(0x0408_0000, 0);
    m.bus.write32(0x0404_0010, 0x0005);

    let subject = lockstep(&m, 200_000, 277, "the transfer over the block");
    assert!(subject.cpu.gpr[14] > 4 && subject.cpu.gpr[15] > 4, "the transfer never landed both ways: r14 {} r15 {}", subject.cpu.gpr[14], subject.cpu.gpr[15]);
    assert!(subject.blocks.stats.discarded > 4, "{} discarded", subject.blocks.stats.discarded);
}

/// Past the code's bound every block is dropped and the compiler's memory with it, and the machine computes the same.
#[test]
fn dropping_every_block_at_the_codes_bound_changes_nothing() {
    let mut m = machine_with(0x1000, &counted_loop(30_000));
    with_devices(&mut m);
    let mut reference = m.clone();
    let mut subject = recompiled(&m, Tier::BEST);
    subject.blocks.code_bound = 64;
    for n in 1..=8 {
        reference.run_steps(20_000);
        subject.run_steps(20_000);
        assert!(reference == subject, "the machines part after {n} runs");
    }
    assert!(subject.blocks.stats.flushes > 2, "the bound dropped the blocks {} times", subject.blocks.stats.flushes);
}
