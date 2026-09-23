//! The decoded table against the interpreter it is built from: random programs with branches into the middle of straight runs, delay
//! slots, breaks, status writes and transfers into IMEM, and the CPU writing IMEM between steps, run one tick at a time, many
//! cycles at once and whole to their events, the vector unit in host vectors and element by element. See Mars_Native.md §6.12.

use super::blocks::Rng;
use crate::machine::Machine;
use crate::memory::bus::RDRAM_SIZE;

const IMEM: u32 = 0x0400_1000;

/// Registers the programs lean on: an IMEM address for the transfer, a DRAM address, a length, a status word, addresses in IMEM.
const MEM_ADDRESS: u32 = 20;
const DRAM_ADDRESS: u32 = 21;
const LENGTH: u32 = 22;
const STATUS: u32 = 23;

/// A word of every class the table decodes differently, weighted toward what ends or enters a straight run.
fn instruction(r: &mut Rng) -> u32 {
    let reg = |r: &mut Rng| r.bits(5);
    match r.below(100) {
        // The vector unit, both selector kinds, and its loads and stores.
        0..22 => 0x4A00_0000 | r.bits(4) << 21 | reg(r) << 16 | reg(r) << 11 | reg(r) << 6 | r.bits(6),
        22..30 => (if r.below(2) == 0 { 0x32 } else { 0x3A }) << 26 | reg(r) << 21 | reg(r) << 16 | r.bits(4) << 11 | r.bits(4) << 7 | r.bits(7),
        30..34 => 0x4800_0000 | (r.below(4) as u32 * 2) << 21 | reg(r) << 16 | reg(r) << 11 | r.bits(4) << 7,
        // The scalar unit, `r0` among the destinations.
        34..46 => (r.range(0x08, 0x10) as u32) << 26 | reg(r) << 21 | (if r.below(4) == 0 { 0 } else { reg(r) }) << 16 | r.bits(16),
        46..56 => {
            const FUNCTIONS: [u32; 16] = [0, 2, 3, 4, 6, 7, 0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x2A, 0x2B];
            reg(r) << 21 | reg(r) << 16 | (if r.below(4) == 0 { 0 } else { reg(r) }) << 11 | reg(r) << 6 | FUNCTIONS[r.below(16) as usize]
        }
        56..62 => {
            const OPS: [u32; 9] = [0x20, 0x21, 0x23, 0x24, 0x25, 0x27, 0x28, 0x29, 0x2B];
            OPS[r.below(9) as usize] << 26 | reg(r) << 21 | reg(r) << 16 | r.bits(16)
        }
        // Branches both ways by a few words, into the middle of runs; jumps and links; register jumps to IMEM addresses.
        62..74 => (r.range(1, 8) as u32) << 26 | reg(r) << 21 | (if r.below(3) == 0 { r.below(4) as u32 | (r.below(2) as u32) << 4 } else { reg(r) }) << 16 | (r.range(-12, 12) as i16 as u16 as u32),
        74..77 => (if r.below(2) == 0 { 0x02 } else { 0x03 }) << 26 | r.bits(10),
        77..80 => reg(r) << 21 | (if r.below(2) == 0 { 0 } else { reg(r) }) << 11 | (if r.below(2) == 0 { 0x08 } else { 0x09 }),
        // The events: breaks, status and semaphore reads, transfers and status writes.
        80..83 => 0x0000_000D,
        83..87 => 0x4000_0000 | reg(r) << 16 | [4u32, 5, 6, 7, 8, 11][r.below(6) as usize] << 11,
        87..93 => {
            let (rt, rd) = match r.below(5) {
                0 => (MEM_ADDRESS, 0),
                1 => (DRAM_ADDRESS, 1),
                2 => (LENGTH, 2),
                3 => (LENGTH, 3),
                _ => (STATUS, 4),
            };
            0x4080_0000 | rt << 16 | rd << 11
        }
        93..95 => 0x4080_0000 | reg(r) << 16 | 7 << 11,
        95..97 => 0,
        _ => r.u64() as u32,
    }
}

/// A status write that never halts the processor, so a program keeps running; or, now and then, one that sets single-step.
fn status(r: &mut Rng) -> u32 {
    let mut value = r.bits(25) & !0x3;
    if r.below(8) != 0 {
        value &= !(0x3 << 5);
    }
    value
}

fn randomise(m: &mut Machine, r: &mut Rng) {
    for at in (0..0x1000).step_by(4) {
        m.bus.sp_imem[at..at + 4].copy_from_slice(&instruction(r).to_be_bytes());
    }
    for byte in m.bus.sp_dmem.iter_mut() {
        *byte = r.bits(8) as u8;
    }
    // The first 64 KB of RDRAM hold code for the transfers into IMEM to bring in.
    for at in (0..0x1_0000u32).step_by(4) {
        m.bus.rdram.put_be32(at, instruction(r));
    }
    let p = &mut m.bus.sp.processor;
    for i in 1..32 {
        p.gpr[i] = match r.below(3) {
            0 => r.u64() as u32,
            1 => r.bits(12) & 0xFFC,
            _ => r.range(-8, 8) as i32 as u32,
        };
    }
    p.gpr[MEM_ADDRESS as usize] = 0x1000 | (r.bits(12) & 0xFF8);
    p.gpr[DRAM_ADDRESS as usize] = r.bits(16) & 0xFFF8;
    p.gpr[LENGTH as usize] = r.below(40) as u32 * 8 + r.below(8) as u32;
    p.gpr[STATUS as usize] = status(r);
    for register in p.vector.iter_mut() {
        for e in register.iter_mut() {
            *e = r.bits(16) as u16;
        }
    }
    for third in p.accumulator.iter_mut() {
        for e in third.iter_mut() {
            *e = r.bits(16) as u16;
        }
    }
    p.vco = r.bits(16) as u16;
    p.vcc = r.bits(16) as u16;
    p.vce = r.bits(8) as u8;
    p.start(r.bits(10) * 4);
    p.broke = false;
    m.bus.sp.single_step = false;
}

fn machines(simd: bool) -> (Machine, Machine) {
    let (mut plain, mut table) = (Machine::new(RDRAM_SIZE).unwrap(), Machine::new(RDRAM_SIZE).unwrap());
    plain.set_rsp_blocks(false);
    table.set_rsp_blocks(true);
    plain.set_rsp_simd(simd);
    table.set_rsp_simd(simd);
    (plain, table)
}

fn same(a: &Machine, b: &Machine) -> Option<String> {
    let (p, q) = (&a.bus.sp.processor, &b.bus.sp.processor);
    if (p.pc, p.next_pc, p.halted, p.broke) != (q.pc, q.next_pc, q.halted, q.broke) {
        return Some(format!("counters and halt {:03X?} against {:03X?}", (p.pc, p.next_pc, p.halted, p.broke), (q.pc, q.next_pc, q.halted, q.broke)));
    }
    if p != q {
        return Some("the processor's registers".into());
    }
    if a.bus.sp != b.bus.sp {
        return Some("the interface".into());
    }
    if a.bus.sp_dmem != b.bus.sp_dmem {
        return Some("DMEM".into());
    }
    if a.bus.sp_imem != b.bus.sp_imem {
        return Some("IMEM".into());
    }
    if a.bus.mi != b.bus.mi {
        return Some("the MI".into());
    }
    None
}

fn is_event(word: u32) -> bool {
    word >> 26 == 0x10 || (word >> 26 == 0 && word & 0x3F == 0x0D)
}

fn word_at(m: &Machine, pc: u32) -> u32 {
    let at = (pc & 0xFFC) as usize;
    u32::from_be_bytes(m.bus.sp_imem[at..at + 4].try_into().unwrap())
}

#[derive(Default)]
struct Seen {
    steps: u64,
    calls: [u64; 3],
    events: u64,
    breaks: u64,
    into_middle: u64,
    imem_transfers: u64,
    cpu_writes: u64,
    cpu_transfers: u64,
    single_steps: u64,
}

/// One seed: `calls` calls of a kind drawn each time, the same call made of both machines, compared after every one; the whole
/// states at the end. A tick call is one step, so in the tick-only mode the machines are compared after every step.
fn one_seed(seed: u64, simd: bool, calls: usize, ticks_only: bool, seen: &mut Seen) {
    let (mut plain, mut table) = machines(simd);
    let mut r = Rng::new(seed);
    randomise(&mut plain, &mut r);
    let mut again = Rng::new(seed);
    randomise(&mut table, &mut again);
    for call in 0..calls {
        for m in [&mut plain, &mut table] {
            if m.bus.sp.processor.halted {
                let pc = m.bus.sp.processor.pc;
                m.bus.sp.processor.start(pc);
                m.bus.sp.processor.broke = false;
            }
        }
        // The CPU's side between two calls: a word written into IMEM, often just ahead of the processor, or a transfer into it.
        match r.below(40) {
            0..3 => {
                let pc = plain.bus.sp.processor.pc;
                let at = if r.below(2) == 0 { (pc + 4 * r.bits(4)) & 0xFFC } else { r.bits(12) & 0xFFC };
                let word = instruction(&mut r);
                plain.bus.write32(IMEM + at, word);
                table.bus.write32(IMEM + at, word);
                seen.cpu_writes += 1;
            }
            3 => {
                let (mem, dram, length) = (0x1000 | (r.bits(12) & 0xFF8), r.bits(16) & 0xFFF8, r.below(64) as u32 * 8);
                for m in [&mut plain, &mut table] {
                    m.bus.write32(0x0404_0000, mem);
                    m.bus.write32(0x0404_0004, dram);
                    m.bus.write32(0x0404_0008, length);
                }
                seen.cpu_transfers += 1;
            }
            _ => {}
        }
        let before = word_at(&plain, plain.bus.sp.processor.pc);
        let (mem, length) = (plain.bus.sp.mem_address, plain.bus.sp.processor.gpr[LENGTH as usize]);
        let kind = if ticks_only { 0 } else { r.below(3) as usize };
        seen.calls[kind] += 1;
        let ran = match kind {
            0 => {
                plain.bus.sp_step(1);
                table.bus.sp_step(1);
                1
            }
            1 => {
                let cycles = r.range(2, 60);
                plain.bus.sp_step(cycles);
                table.bus.sp_step(cycles);
                cycles as u64
            }
            _ => {
                let budget = r.range(1, 300);
                let (a, b) = (plain.bus.rsp_run_to_event(budget), table.bus.rsp_run_to_event(budget));
                assert_eq!(a, b, "seed {seed}, call {call}: the whole runs ran {a} and {b} steps");
                a as u64
            }
        };
        seen.steps += ran;
        if kind == 0 {
            if is_event(before) {
                seen.events += 1;
                seen.breaks += (before == 0x0D) as u64;
                seen.imem_transfers += (before >> 21 == 0x204 && (before >> 11) & 0x1F == 2 && mem & 0x1000 != 0 && length > 0) as u64;
            }
            let p = &plain.bus.sp.processor;
            let target = p.next_pc;
            if target != (p.pc + 4) & 0xFFC {
                let prior = word_at(&plain, target.wrapping_sub(4));
                let branch = matches!(prior >> 26, 1..=7) || (prior >> 26 == 0 && matches!(prior & 0x3F, 8 | 9));
                seen.into_middle += (!branch && !is_event(prior)) as u64;
            }
        }
        seen.single_steps += plain.bus.sp.single_step as u64;
        if let Some(part) = same(&plain, &table) {
            panic!("seed {seed} (vector unit in host vectors: {simd}), call {call} of kind {kind}, {before:08X} first: {part}");
        }
    }
    plain.settle();
    table.settle();
    assert!(plain.save_state_vec(false).unwrap() == table.save_state_vec(false).unwrap(), "seed {seed}: the machines' states part");
}

fn seeds() -> u64 {
    std::env::var("EMUSEN_MARSRT_RSP_SEEDS").ok().and_then(|v| v.parse().ok()).unwrap_or(150)
}

/// 150 seeds, every call a single tick, compared after every step: the lock-step path.
#[test]
fn the_decoded_table_steps_as_the_interpreter_steps_one_tick_at_a_time() {
    let mut seen = Seen::default();
    for seed in 1..=seeds() {
        one_seed(seed, seed % 2 == 0 && crate::rsp::simd_supported(), 3000, true, &mut seen);
    }
    eprintln!(
        "{} seeds, {} steps compared one at a time: {} events, {} breaks, {} transfers into IMEM, {} taken branches into the middle of a straight run, {} CPU writes and {} CPU transfers into IMEM, {} calls under single-step",
        seeds(), seen.steps, seen.events, seen.breaks, seen.imem_transfers, seen.into_middle, seen.cpu_writes, seen.cpu_transfers, seen.single_steps
    );
    assert!(seen.imem_transfers > 0 && seen.into_middle > 0 && seen.breaks > 0);
}

/// 150 seeds of ticks, calls of many cycles and whole runs to the next event, drawn at random, compared after every call.
#[test]
fn the_decoded_table_runs_as_the_interpreter_runs_in_every_entry() {
    let mut seen = Seen::default();
    for seed in 1..=seeds() {
        one_seed(1000 + seed, seed % 2 == 0 && crate::rsp::simd_supported(), 800, false, &mut seen);
    }
    eprintln!(
        "{} seeds, {} steps in {:?} calls (tick, many, whole), compared after every call: {} CPU writes and {} CPU transfers into IMEM, {} calls under single-step",
        seeds(), seen.steps, seen.calls, seen.cpu_writes, seen.cpu_transfers, seen.single_steps
    );
}

/// A word rewritten under a running table: a run of no-operations that a CPU store turns into a break halfway, and a transfer that
/// replaces the whole memory, each taken by the table as the interpreter takes it.
#[test]
fn a_word_rewritten_under_the_table_is_run_as_rewritten() {
    for simd in [false, true] {
        let (mut plain, mut table) = machines(simd && crate::rsp::simd_supported());
        for m in [&mut plain, &mut table] {
            m.bus.sp.processor.start(0);
            assert_eq!(m.bus.rsp_run_to_event(40), 40, "forty no-operations run straight");
            m.bus.write32(IMEM + 0x40, 0x0000_000D);
            m.bus.sp.processor.start(0);
        }
        assert_eq!(plain.bus.rsp_run_to_event(100), table.bus.rsp_run_to_event(100));
        assert!(table.bus.sp.processor.halted && table.bus.sp.processor.broke && table.bus.sp.processor.pc == 0x44);
        assert!(same(&plain, &table).is_none());
        // A transfer from RDRAM that fills IMEM with `addiu r1, r1, 1`, then runs from the start.
        for m in [&mut plain, &mut table] {
            for at in (0..0x1000u32).step_by(4) {
                m.bus.rdram.put_be32(0x2000 + at, 0x2421_0001);
            }
            m.bus.write32(0x0404_0000, 0x1000);
            m.bus.write32(0x0404_0004, 0x2000);
            m.bus.write32(0x0404_0008, 0xFFF);
            m.bus.sp.processor.start(0);
        }
        assert_eq!(plain.bus.rsp_run_to_event(500), table.bus.rsp_run_to_event(500));
        assert_eq!(table.bus.sp.processor.gpr[1], 500);
        assert!(same(&plain, &table).is_none());
    }
}
