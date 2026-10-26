//! Stage S33's programs (Sapphire_Native.md §47): the project's EE programs that wait in a loop for each kind of event
//! the board gives (V-blank, a timer reaching its compare, Count reaching Compare, a DMA transfer ending, seen in memory
//! and by BC0F, and the IOP's SIF0 transfer through its wake of the DMAC), and the loops idle skipping must refuse (a
//! timeout counted in a register, a read of IOP RAM). After each wait a program records Count and timer 0's count, so a
//! skip that charged one cycle too many or too few is seen in RAM as well as in the state. Each runs with idle skipping
//! off and on, on both tiers, compared after every frame in picture and whole state; with the recompiled tier's
//! verifier, which steps the interpreter through every skipped turn; and from a state saved while it waits.

use emusen_native::ffi::StateMachine;
use sapphire::blocks::{Blocks, Verify};
use sapphire::board::System;
use sapphire::idle::Refusal;
use sapphire::machine::{Machine, Market};
use sapphire::media::Media;

/// The program, entered in kseg0 so that its fetches go through the instruction cache, and its words in RAM: the
/// handler's period at 1000h, its base at 1004h and the flag it counts at 1010h, the done word, and the record of
/// Count and timer 0's count.
const ORIGIN: u32 = 0x8010_0000;
const HANDLER: u32 = 0x8000_0200;
const DONE: u32 = 0x1020;
const RECORD: u32 = 0x2000;
/// The IOP program's origin, the IOP's chain for SIF0 and its data, and where the EE receives it.
const IOP_ORIGIN: u32 = 0xA010_0000;
const IOP_CHAIN: u32 = 0x001C_1000;
const SIF_DEST: u32 = 0x0030_0000;

/// The interrupt handler at the level 1 interrupt vector: on Count's interrupt, Compare moved on by the period; on the
/// INTC's or the DMAC's, I_STAT, the DMAC's channel flags and timer 0's EQUF acknowledged; then the flag counted at the
/// base the program chose, kseg0 or kseg1.
const HANDLER_SRC: &str = "
    mfc0 k0, Cause
    andi k0, k0, 0x8000
    beq k0, zero, intc
    nop
    mfc0 k0, Count
    lui k1, 0xa000
    lw k1, 0x1000(k1)
    addu k0, k0, k1
    mtc0 k0, Compare
    sync.p
    beq zero, zero, count
    nop
    intc: lui k0, 0xb001
    lw k1, -0x1000(k0)
    sw k1, -0x1000(k0)
    lw k1, -0x1ff0(k0)
    andi k1, k1, 0x3ff
    sw k1, -0x1ff0(k0)
    lui k0, 0xb000
    lw k1, 0x10(k0)
    sw k1, 0x10(k0)
    count: lui k0, 0xa000
    lw k0, 0x1004(k0)
    lw k1, 0x1010(k0)
    addiu k1, k1, 1
    sw k1, 0x1010(k0)
    eret
";

/// The record after each wait: Count and timer 0's count at the record's next pair of words; s6 counts the waits, s5
/// is their number; then the done word and the idle loop every program ends in.
const RECORD_SRC: &str = "
    mfc0 t1, Count
    lui t2, 0xb000
    lw t3, 0(t2)
    sll t4, s6, 3
    addu t4, t4, s0
    sw t1, 0x2000(t4)
    sw t3, 0x2004(t4)
    addiu s6, s6, 1
    bne s6, s5, top
    nop
    ori t5, zero, 1
    sw t5, 0x1020(s0)
    end: beq zero, zero, end
    nop
";

/// A program: its setup, the waits' count, and the wait itself, which begins at `top` and falls into the record.
fn program(setup: &str, waits: u32, wait: &str) -> String {
    format!("lui s0, 0xa000\nlui s1, 0x8000\nori s5, zero, {waits}\nor s6, zero, zero\n{setup}\ntop:\n{wait}\n{RECORD_SRC}")
}

/// The flag waited on, read through `base` (s0 uncached, s1 cached) until the handler has counted it once more.
fn flag_wait(base: &str) -> String {
    format!("lw s2, 0x1010({base})\nloop: lw t0, 0x1010({base})\nbeq t0, s2, loop\nnop")
}

/// Status with IE, EIE and the interrupt masks `im`, and the handler's flag counted at `base`.
fn interrupts(im: u32, base: u32) -> String {
    format!(
        "lui t0, 0x{:x}\nori t0, t0, 0x{:x}\nsw t0, 0x1004(s0)\nlui t0, 0x{:x}\nori t0, t0, 0x{:x}\nmtc0 t0, Status\nsync.p",
        base >> 16,
        base & 0xFFFF,
        (0x1_0001 | im) >> 16,
        (0x1_0001 | im) & 0xFFFF
    )
}

/// I_MASK's bit for INTC source `n` turned on (a write of 1 toggles it).
fn intc_mask(n: u32) -> String {
    format!("lui t0, 0xb001\nori t1, zero, 0x{:x}\nsw t1, -0xff0(t0)", 1u32 << n)
}

struct Case {
    name: &'static str,
    ee: String,
    iop: Option<String>,
    frames: usize,
    /// Whether the program's waits are passed (some skip made), or the refusal its wait must meet.
    skips: bool,
    refusal: Option<Refusal>,
}

fn vblank() -> Case {
    let setup = format!("{}\n{}", intc_mask(2), interrupts(0x400, 0xA000_0000));
    Case { name: "vblank", ee: program(&setup, 3, &flag_wait("s0")), iop: None, frames: 5, skips: true, refusal: None }
}

fn timer() -> Case {
    let setup = format!(
        "lui t0, 0xb000\nori t1, zero, 30000\nsw t1, 0x20(t0)\nori t1, zero, 0x1c0\nsw t1, 0x10(t0)\n{}\n{}",
        intc_mask(9),
        interrupts(0x400, 0x8000_0000)
    );
    Case { name: "timer", ee: program(&setup, 12, &flag_wait("s1")), iop: None, frames: 2, skips: true, refusal: None }
}

fn count() -> Case {
    let setup = format!(
        "ori t0, zero, 50000\nsw t0, 0x1000(s0)\nmfc0 t1, Count\naddu t1, t1, t0\nmtc0 t1, Compare\nsync.p\n{}",
        interrupts(0x8000, 0xA000_0000)
    );
    Case { name: "count", ee: program(&setup, 10, &flag_wait("s0")), iop: None, frames: 2, skips: true, refusal: None }
}

/// fromSPR: 64 quadwords from the scratchpad to RAM at s3, the last word waited for in memory, by BC0F, or in the
/// channel's CHCR, a register only the DMAC changes.
fn dma(by: &'static str) -> Case {
    let setup = "
        lui t0, 0x7000
        ori t1, zero, 256
        fill: addiu t2, t1, 0x100
        sw t2, 0(t0)
        addiu t0, t0, 4
        addiu t1, t1, -1
        bne t1, zero, fill
        nop
        lui t0, 0xb001
        ori t1, zero, 1
        sw t1, -0x2000(t0)
        ori t1, zero, 0x100
        sw t1, -0x1fe0(t0)
        lui s3, 0x0004
    ";
    let start = "
        lui t0, 0xb001
        ori t1, zero, 0x100
        sw t1, -0x1ff0(t0)
        sw zero, -0x2f80(t0)
        sw s3, -0x2ff0(t0)
        ori t1, zero, 64
        sw t1, -0x2fe0(t0)
        ori t1, zero, 0x100
        sw t1, -0x3000(t0)
        lui t9, 0xa000
        or t9, t9, s3
    ";
    let wait = match by {
        "bc0f" => "loop: bc0f loop\nnop",
        "chcr" => "loop: lw t1, -0x3000(t0)\nandi t1, t1, 0x100\nbne t1, zero, loop\nnop",
        _ => "loop: lw t0, 0x3fc(t9)\nbeq t0, zero, loop\nnop",
    };
    let ee = program(setup, 4, &format!("{start}\n{wait}\nlui t8, 0x0001\naddu s3, s3, t8"));
    let name = match by {
        "bc0f" => "dma-bc0f",
        "chcr" => "dma-chcr",
        _ => "dma-memory",
    };
    Case { name, ee, iop: None, frames: 2, skips: true, refusal: None }
}

/// The IOP's SIF0 transfer of 64 words to the EE, made by an IOP program after a delay, and then the IOP waiting in a
/// loop of its own; the EE waits for the last word in memory, the skip ending at the IOP side's wake of the DMAC.
fn sif0() -> Case {
    let setup = "
        lui t0, 0xb001
        ori t1, zero, 1
        sw t1, -0x2000(t0)
        sw zero, -0x3fe0(t0)
        ori t1, zero, 0x104
        sw t1, -0x4000(t0)
        lui t9, 0xa030
    ";
    let wait = "loop: lw t0, 0xfc(t9)\nbeq t0, zero, loop\nnop";
    let iop = format!(
        "
        lui t0, 0xbf80
        ori t1, zero, 0x800
        sw t1, 0x10f0(t0)
        ori t1, zero, 0x8800
        sw t1, 0x1570(t0)
        lui t1, 0x80
        sw t1, 0x10f4(t0)
        lui t1, 0x3f00
        sw t1, 0x1574(t0)
        sw zero, 0x1070(t0)
        ori t1, zero, 1
        sw t1, 0x1578(t0)
        ori t2, zero, 0x4000
        delay: addiu t2, t2, -1
        bne t2, zero, delay
        nop
        lui t1, 0x{:x}
        ori t1, t1, 0x{:x}
        sw t1, 0x152c(t0)
        lui t1, 0x0100
        ori t1, t1, 0x0701
        sw t1, 0x1528(t0)
        end: j end
        nop
        ",
        IOP_CHAIN >> 16,
        IOP_CHAIN & 0xFFFF
    );
    Case { name: "sif0", ee: program(setup, 1, wait), iop: Some(iop), frames: 2, skips: true, refusal: None }
}

/// A wait with a timeout counted down in a register and tested by BGTZ, timer 0's interrupt every 40,000 cycles: with
/// `turns` turns allowed the timeout never expires first, and the turns are passed up to the event; with a few hundred it
/// expires inside each wait, and the turns are passed up to the turn before it does.
fn timeout(turns: u32) -> Case {
    let setup = format!("{}\n{}", intc_mask(9), interrupts(0x400, 0xA000_0000));
    let setup = format!("lui t0, 0xb000\nori t1, zero, 20000\nsw t1, 0x20(t0)\nori t1, zero, 0x1c0\nsw t1, 0x10(t0)\n{setup}");
    let wait = format!(
        "lw s2, 0x1010(s0)\nlui t7, 0x{:x}\nori t7, t7, 0x{:x}\nloop: lw t0, 0x1010(s0)\nbne t0, s2, out\nnop\naddiu t7, t7, -1\nbgtz t7, loop\nnop\nout:",
        turns >> 16,
        turns & 0xFFFF
    );
    let name = if turns > 100_000 { "timeout-long" } else { "timeout-short" };
    Case { name, ee: program(&setup, 6, &wait), iop: None, frames: 2, skips: true, refusal: None }
}

/// A 64-bit count in memory and a register counting up by five, which an SLT compares with a bound the wait reaches in
/// some thousands of turns, ending it early: the turns passed stop where the comparison would change its answer.
fn counter64() -> Case {
    let setup = format!("{}\n{}", intc_mask(2), interrupts(0x400, 0xA000_0000));
    let wait = "lw s2, 0x1010(s0)\nor t8, zero, zero\nori s4, zero, 40000\nloop: ld t1, 0x1110(s1)\ndaddiu t1, t1, 1\nsd t1, 0x1110(s1)\ndaddiu t8, t8, 5\nslt t2, t8, s4\nbeq t2, zero, out\nnop\nlw t0, 0x1010(s0)\nbeq t0, s2, loop\nnop\nout:";
    Case { name: "counter64", ee: program(&setup, 3, wait), iop: None, frames: 3, skips: true, refusal: None }
}

/// A wait through a call: the flag read by a routine the loop calls, the return address the same each turn.
fn call() -> Case {
    let setup = format!("{}\n{}", intc_mask(2), interrupts(0x400, 0xA000_0000));
    let wait = "lw s2, 0x1010(s0)\nloop: jal get\nnop\nbeq v0, s2, loop\nnop\nbeq zero, zero, done\nnop\nget: lw v0, 0x1010(s0)\njr ra\nnop\ndone:";
    Case { name: "call", ee: program(&setup, 2, wait), iop: None, frames: 3, skips: true, refusal: None }
}

/// A wait that reads IOP RAM through the EE's window, which the IOP may write at any time: refused.
fn window() -> Case {
    let setup = format!(
        "ori t0, zero, 40000\nsw t0, 0x1000(s0)\nmfc0 t1, Count\naddu t1, t1, t0\nmtc0 t1, Compare\nsync.p\n{}",
        interrupts(0x8000, 0xA000_0000)
    );
    let wait = "lw s2, 0x1010(s0)\nlui t8, 0xbc00\nloop: lw t1, 0(t8)\nlw t0, 0x1010(s0)\nbeq t0, s2, loop\nnop";
    Case { name: "window", ee: program(&setup, 4, wait), iop: None, frames: 2, skips: false, refusal: Some(Refusal::Window) }
}

/// A wait whose turn stores a value the memory already holds, through the cache and past it: passed.
fn constant_store() -> Case {
    let setup = format!("ori s7, zero, 0x77\n{}\n{}", intc_mask(2), interrupts(0x400, 0xA000_0000));
    let wait = "lw s2, 0x1010(s0)\nloop: sw s7, 0x1100(s1)\nsw s7, 0x1104(s0)\nlw t0, 0x1010(s0)\nbeq t0, s2, loop\nnop";
    Case { name: "constant-store", ee: program(&setup, 2, wait), iop: None, frames: 3, skips: true, refusal: None }
}

/// A wait that counts its turns in memory, in a cached line, and in a register, neither count read by the loop's path:
/// passed, each count moved on by its turns.
fn counter() -> Case {
    let setup = format!("{}\n{}", intc_mask(2), interrupts(0x400, 0xA000_0000));
    let wait = "lw s2, 0x1010(s0)\nloop: lw t1, 0x1108(s1)\naddiu t1, t1, 1\nsw t1, 0x1108(s1)\naddiu t7, t7, 3\nlw t0, 0x1010(s0)\nbeq t0, s2, loop\nnop";
    Case { name: "counter", ee: program(&setup, 2, wait), iop: None, frames: 3, skips: true, refusal: None }
}

/// A count in memory the loop's path compares with zero: passed, up to the turn before the count would reach it.
fn counted() -> Case {
    let setup = format!("{}\n{}", intc_mask(2), interrupts(0x400, 0xA000_0000));
    let wait = "lw s2, 0x1010(s0)\nloop: lw t1, 0x110c(s1)\naddiu t1, t1, 1\nsw t1, 0x110c(s1)\nbeq t1, zero, out\nnop\nlw t0, 0x1010(s0)\nbeq t0, s2, loop\nnop\nout:";
    Case { name: "counted", ee: program(&setup, 2, wait), iop: None, frames: 3, skips: true, refusal: None }
}

/// A count doubled each turn, which is not a step: refused.
fn doubled() -> Case {
    let setup = format!("{}\n{}", intc_mask(2), interrupts(0x400, 0xA000_0000));
    let wait = "lw s2, 0x1010(s0)\nori t7, zero, 1\nloop: sll t7, t7, 1\nlw t0, 0x1010(s0)\nbeq t0, s2, loop\nnop";
    Case { name: "doubled", ee: program(&setup, 2, wait), iop: None, frames: 3, skips: false, refusal: Some(Refusal::InductionOp) }
}

/// Timer 0's waits while VU1 runs a long microprogram started by CMSAR1: the board is never in its quick path, so every
/// passed turn brings it through each of its times with `run_to`.
fn vu1_busy() -> Case {
    let setup = format!(
        "lui t0, 0x4000\nmtc0 t0, Status\nsync.p\nctc2 zero, vi31\nlui t0, 0xb000\nori t1, zero, 20000\nsw t1, 0x20(t0)\nori t1, zero, 0x1c0\nsw t1, 0x10(t0)\n{}\n{}",
        intc_mask(9),
        interrupts(0x400, 0xA000_0000)
    );
    Case { name: "vu1-busy", ee: program(&setup, 6, &flag_wait("s0")), iop: None, frames: 2, skips: true, refusal: None }
}

/// VU1's microprogram for `vu1_busy`: about 32,767 turns of a three-pair loop, then its end.
const VU1_LOOP: &str = "nop | iaddiu vi1, vi0, 0x7fff\ntop: nop | iaddi vi1, vi1, -1\nnop | ibne vi1, vi0, top\nnop | move vf0, vf0\n[e] nop | move vf0, vf0\nnop | move vf0, vf0";

/// A wait on timer 0's count, which changes with time: refused.
fn timer_poll() -> Case {
    let setup = "lui t0, 0xb000\nori t1, zero, 0x80\nsw t1, 0x10(t0)";
    let wait = "lui t0, 0xb000\nsw zero, 0(t0)\nloop: lw t1, 0(t0)\nandi t1, t1, 0x8000\nbeq t1, zero, loop\nnop";
    Case { name: "timer-poll", ee: program(setup, 3, wait), iop: None, frames: 2, skips: false, refusal: Some(Refusal::Timer) }
}

fn cases() -> Vec<Case> {
    vec![vu1_busy(), dma("chcr"), timer_poll(), vblank(), timer(), count(), dma("memory"), dma("bc0f"), sif0(), timeout(1 << 20), timeout(700), counter64(), call(), window(), constant_store(), counter(), counted(), doubled()]
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Tier {
    Interpreter,
    Recompiler,
}

fn machine(c: &Case, idle: bool, tier: Tier) -> Machine {
    let words = emusen_mips::assemble(&c.ee, ORIGIN, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{}: {e:?}", c.name));
    let handler = emusen_mips::assemble(HANDLER_SRC, HANDLER, &emusen_mips::Dialect::R5900).expect("the handler assembles");
    let mut m = Machine::new(Media::read(&sapphire::media::program(&[0], 0x0010_0000)).unwrap(), Market::Americas, [Some("ps2.dualshock2"), None]);
    let mut sys = System::new(ORIGIN);
    sys.board.map.load(ORIGIN & 0x1FFF_FFFF, &words);
    sys.board.map.load(HANDLER & 0x1FFF_FFFF, &handler);
    if let Some(src) = &c.iop {
        let iw = emusen_mips::assemble(src, IOP_ORIGIN, &blackmoon_r3000::dialect(None)).unwrap_or_else(|e| panic!("{}: {e:?}", c.name));
        sys.board.iop.bus.load(IOP_ORIGIN & 0x1F_FFFF, &iw);
        let tag = [IOP_CHAIN + 0x10 | 1 << 31, 64, 16 | 7 << 28, SIF_DEST];
        sys.board.iop.bus.load(IOP_CHAIN, &tag);
        sys.board.iop.bus.load(IOP_CHAIN + 0x10, &(0..64).map(|k| 0x6400_0000 + k).collect::<Vec<u32>>());
        sys.board.iop.enter(IOP_ORIGIN);
    }
    if c.name == "vu1-busy" {
        let prog = emusen_mips::vu::assemble(VU1_LOOP).expect("the microprogram assembles");
        for (i, pair) in prog.iter().enumerate() {
            sys.board.vpu.units.vu1.micro[i * 8..i * 8 + 8].copy_from_slice(&pair.to_le_bytes());
        }
    }
    m.sys = Box::new(sys);
    m.booted = true;
    m.idle.on = idle;
    if tier == Tier::Recompiler {
        let mut b = Blocks::new();
        b.wait = true;
        m.blocks = Some(Box::new(b));
    }
    m
}

fn state(m: &Machine) -> Vec<u8> {
    let mut v = vec![0; m.state_size()];
    let n = m.save_state(&mut v).expect("the state saves");
    v.truncate(n);
    v
}

fn word(m: &Machine, at: u32) -> u32 {
    u32::from_le_bytes(m.sys.board.map.rdram[at as usize..at as usize + 4].try_into().unwrap())
}

/// Runs the case with idle skipping off and on, frame by frame, the picture and the whole state compared after each.
fn equal(c: &Case, tier: Tier) -> Machine {
    let mut plain = machine(c, false, Tier::Interpreter);
    let mut fast = machine(c, true, tier);
    for f in 0..c.frames {
        plain.advance();
        fast.advance();
        assert!(plain.sys.board.pcrtc.picture.rgba == fast.sys.board.pcrtc.picture.rgba, "{} {tier:?}: the picture differs after frame {f}", c.name);
        assert!(state(&plain) == state(&fast), "{} {tier:?}: the state differs after frame {f}; {:?}; counts {:?}", c.name, (sapphire::blocks::compare(&plain.sys, &fast.sys, false), plain.sys.ee.timing.instructions, fast.sys.ee.timing.instructions, plain.sys.ee.modes, fast.sys.ee.modes, plain.sys.board.now, fast.sys.board.now), fast.idle.counts);
    }
    assert_eq!(word(&plain, DONE), 1, "{}: the program finished its waits; iop pc {:x} ch9 {:x?} ee ch5 {:x?} dctrl {:x}", c.name, plain.sys.board.iop.cpu.regs.pc, plain.sys.board.iop.bus.dmac.ch[9], plain.sys.board.dmac.ch[5], plain.sys.board.dmac.ctrl);
    let k = &fast.idle.counts;
    if c.skips && c.refusal.is_none() {
        assert!(k.ee_skips > 0 && k.ee_cycles > 0, "{} {tier:?}: no wait passed: {k:?}", c.name);
    }
    if let Some(r) = c.refusal {
        assert!(k.refused[r as usize] > 0, "{} {tier:?}: never refused as {r:?}: {k:?}", c.name);
    }
    fast
}

#[test]
fn every_wait_leaves_the_plain_runs_machine_on_the_interpreter() {
    for c in cases() {
        equal(&c, Tier::Interpreter);
    }
}

#[test]
fn every_wait_leaves_the_plain_runs_machine_on_the_recompiler() {
    for c in cases() {
        equal(&c, Tier::Recompiler);
    }
}

/// The records' values: each wait ends after its event, Count and the timer moving on between waits.
#[test]
fn the_records_count_on_from_wait_to_wait() {
    for c in [vblank(), timer(), count(), dma("memory"), dma("bc0f")] {
        let m = equal(&c, Tier::Interpreter);
        let counts: Vec<u32> = (0..3).map(|i| word(&m, RECORD + 8 * i)).collect();
        assert!(counts.iter().all(|&v| v != 0), "{}: {counts:x?}", c.name);
        assert!(counts.windows(2).all(|w| w[1] > w[0]), "{}: Count moves on from wait to wait: {counts:x?}", c.name);
    }
}

/// The recompiled tier's verifier steps the interpreter through every turn a skip passes and compares the two after
/// each skip, and the whole states at the end.
#[test]
fn the_verifier_steps_through_every_skipped_turn() {
    for c in cases() {
        let mut fast = machine(&c, true, Tier::Recompiler);
        let reference = machine(&c, false, Tier::Interpreter).sys;
        fast.blocks.as_mut().unwrap().verify(Verify::Blocks, reference, false);
        for _ in 0..c.frames {
            fast.advance();
        }
        let b = fast.blocks.as_ref().unwrap();
        b.check_all(&fast.sys);
        assert!(b.verified().unwrap() > 0, "{}", c.name);
    }
}

/// A state saved while a program waits, loaded into a fresh machine with idle skipping on, runs on as the plain run.
#[test]
fn a_state_saved_in_a_wait_loads_and_runs_on_alike() {
    for c in [vblank(), timer(), sif0()] {
        let mut plain = machine(&c, false, Tier::Interpreter);
        let mut first = machine(&c, true, Tier::Recompiler);
        plain.advance();
        first.advance();
        let mut second = machine(&c, true, Tier::Recompiler);
        second.load_state(&state(&first)).expect("the state loads");
        for f in 1..c.frames + 1 {
            plain.advance();
            second.advance();
            assert!(state(&plain) == state(&second), "{}: the loaded machine differs after frame {f}", c.name);
        }
    }
}

/// The SIF0 program: a skip of the EE's wait ends at the time the IOP side wakes the DMAC, and the IOP's own wait in its
/// loop is passed too, both tiers leaving the plain run's machine.
#[test]
fn the_iop_sides_wake_ends_a_skip_and_the_iops_own_wait_is_passed() {
    for tier in [Tier::Interpreter, Tier::Recompiler] {
        let m = equal(&sif0(), tier);
        let k = &m.idle.counts;
        assert!(k.woken > 0, "{tier:?}: no skip ended at the wake: {k:?}");
        assert!(k.iop_skips > 0 && k.iop_instructions > 0, "{tier:?}: the IOP's wait was not passed: {k:?}");
    }
}

/// Turns passed while a unit runs and across events are `run_to`'s at each time; a DMA transfer that writes what a wait
/// reads ends the skip there.
#[test]
fn turns_are_passed_while_a_unit_runs_and_end_where_the_dmac_writes_what_they_read() {
    let m = equal(&vu1_busy(), Tier::Interpreter);
    let k = &m.idle.counts;
    assert!(k.replayed > 0 && k.ee_skips > 0, "VU1 running: {k:?}");
    assert!(m.sys.board.vpu.pairs_run()[1] > 50_000, "VU1 ran its microprogram beside the waits");
    let m = equal(&dma("memory"), Tier::Interpreter);
    assert!(m.idle.counts.written > 0, "the transfer's last quadword: {:?}", m.idle.counts);
}
