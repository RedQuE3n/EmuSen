//! Stage S7's gate (Sapphire_Native.md §17.8): the project's programs on the TLB, the scratchpad, the caches, the
//! counters, the breakpoints and the bus errors, run on the interpreter and graded by the records the manuals' rules
//! give; and the rules no program can show a referee, checked on the interpreter directly.

mod s6;
mod s7;

use sapphire::ee::cop0::{self, config};
use sapphire::ee::map::{self, Map, Region};
use sapphire::ee::{self, Ee, Exception, Level2, status, timing};

use s6::{DONE, DONE_AT, ORIGIN, Program, RECORD};

fn assemble(source: &str, origin: u32) -> Vec<u32> {
    emusen_mips::assemble(source, origin, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{e:?}"))
}

fn run(source: &str, limit: u64) -> (Ee, Map) {
    let mut bus = Map::new();
    bus.load(ORIGIN, &assemble(source, ORIGIN));
    let mut ee = Ee::new(ORIGIN);
    let n = ee.run(&mut bus, limit, &mut |e, b| e.peek(b, DONE_AT, 4) as u32 == DONE).unwrap_or_else(|h| panic!("{h:?}"));
    assert!(n < limit, "no done word in {limit} instructions, pc {:08x}", ee.pc);
    (ee, bus)
}

fn grade(p: Program) {
    let name = p.name;
    let (source, expected) = p.finish();
    let (ee, mut bus) = run(&source, 5_000_000);
    let mut wrong = Vec::new();
    for (i, want) in expected.iter().enumerate() {
        let got = ee.peek(&mut bus, RECORD + 4 * i as u32, 4) as u32;
        if let Some(w) = want
            && *w != got
        {
            wrong.push(format!("word {i}: {got:08x}, expected {w:08x}"));
        }
    }
    assert!(wrong.is_empty(), "{name}: {} of {} words differ, first {:?}", wrong.len(), expected.len(), &wrong[..wrong.len().min(8)]);
}

#[test]
fn tlb_refills_pages_and_their_exceptions_equal_the_manual() {
    grade(s7::tlb());
}

#[test]
fn the_scratchpad_is_reached_through_s_entries_alone() {
    grade(s7::scratchpad());
}

#[test]
fn the_data_cache_writes_back_replaces_and_locks_as_chapter_6_says() {
    grade(s7::data_cache());
}

#[test]
fn the_instruction_cache_keeps_replaced_code_until_invalidated() {
    grade(s7::instruction_cache());
}

#[test]
fn count_compare_and_the_performance_counters_equal_chapter_7() {
    grade(s7::counters());
}

#[test]
fn the_breakpoints_raise_the_debug_exception_as_chapter_9_says() {
    grade(s7::breakpoints());
}

#[test]
fn bus_errors_set_badpaddr_and_bem() {
    grade(s7::bus_errors());
}

#[test]
fn the_programs_execute_every_instruction_s7_adds() {
    let mut seen: std::collections::BTreeSet<String> = s7::all().into_iter().flat_map(|p| s6::covered(&p.finish().0)).collect();
    for (at, words) in s7::handlers() {
        seen.extend(words.iter().filter_map(|&w| emusen_mips::r5900::operation(w)).map(|o| o.mnemonic()));
        assert!(at >= 0x8000_0000);
    }
    let s7: Vec<_> = ee::names().skip(128).collect();
    assert_eq!(s7.len(), 27, "the TLB, debug, performance, BC0 and CACHE instructions");
    let unexecuted: Vec<_> = s7.iter().filter(|n| !seen.contains(**n)).collect();
    let by_unit = ["mfiab", "mfiabm", "mfdab", "mfdabm", "mfdvb", "mfdvbm", "bc0f", "bc0t", "bc0fl", "bc0tl"];
    assert!(unexecuted.iter().all(|n| by_unit.contains(n)), "never executed: {unexecuted:?}");
}

// What no program shows a referee: each rule below on the interpreter directly.

fn machine(code: &str) -> (Ee, Map) {
    let mut bus = Map::new();
    bus.load(ORIGIN, &assemble(code, ORIGIN));
    (Ee::new(ORIGIN), bus)
}

fn steps(ee: &mut Ee, bus: &mut Map, n: usize) {
    for _ in 0..n {
        ee.step(bus).unwrap();
    }
}

#[test]
fn reset_leaves_the_state_section_4_2_1_gives() {
    let e = Ee::reset();
    assert_eq!(e.pc, 0xBFC0_0000);
    assert_eq!(e.cop0[cop0::STATUS], status::ERL | status::BEV);
    assert_eq!(e.cop0[cop0::CONFIG] & (config::DIE | config::ICE | config::DCE | config::NBE | config::BPE), 0);
    assert_eq!(e.cop0[cop0::CONFIG] >> 6 & 0x3F, 0b010_001, "IC 16 KiB, DC 8 KiB");
    assert_eq!((e.cop0[cop0::RANDOM], e.cop0[cop0::WIRED], e.pccr, e.debug[0]), (47, 0, 0, 0));
    assert!(e.dcache.tags.iter().flatten().all(|t| !t.valid && !t.dirty && !t.lrf && !t.lock));
    assert!(e.icache.tags.iter().flatten().all(|t| !t.valid && !t.lrf));
}

#[test]
fn random_decrements_once_an_issue_cycle_between_wired_and_47() {
    let (mut e, mut bus) = machine(&"nop\n".repeat(200));
    e.cop0[cop0::WIRED] = 44;
    e.cop0[cop0::RANDOM] = 47;
    let mut seen = Vec::new();
    for _ in 0..12 {
        e.step(&mut bus).unwrap();
        seen.push(e.cop0[cop0::RANDOM]);
    }
    assert!(seen.iter().all(|r| (44..=47).contains(r)), "{seen:?}");
    assert!(seen.windows(2).any(|w| w[0] == 44 && w[1] == 47), "wraps from Wired to 47: {seen:?}");
    let (mut e, mut bus) = machine("addiu t0, zero, 5\nmtc0 t0, Wired");
    e.cop0[cop0::RANDOM] = 20;
    steps(&mut e, &mut bus, 2);
    assert_eq!(e.cop0[cop0::RANDOM], 47, "writing Wired sets Random to 47");
}

#[test]
fn bc0_branches_on_cpcond0() {
    let code = "bc0t 0x00100010\naddiu t0, zero, 1\naddiu t1, zero, 1\nnop\naddiu t2, zero, 1";
    for (cond, taken) in [(false, false), (true, true)] {
        let (mut e, mut bus) = machine(code);
        e.cpcond0 = cond;
        steps(&mut e, &mut bus, 3);
        assert_eq!((e.gpr[8], e.gpr[9] as u64, e.gpr[10]), (1, u64::from(!taken), u128::from(taken)), "BC0T, CPCOND0 {cond}");
    }
    let (mut e, mut bus) = machine("bc0fl 0x00100010\naddiu t0, zero, 1\nnop\nnop\nnop");
    e.cpcond0 = true;
    steps(&mut e, &mut bus, 2);
    assert_eq!((e.gpr[8], e.pc), (0, ORIGIN + 12), "BC0FL not taken nullifies its slot");
    let (mut e, mut bus) = machine("bc0f 0x00100010\nnop");
    e.cop0[cop0::STATUS] = 0x10;
    steps(&mut e, &mut bus, 1);
    assert_eq!(e.cop0[cop0::CAUSE] >> 2 & 31, 11, "COP0 unusable in user mode");
}

#[test]
fn the_debug_moves_reach_their_registers_with_their_fixed_bits() {
    let mut code = String::new();
    for (i, r) in ["iab", "iabm", "dab", "dabm", "dvb", "dvbm"].iter().enumerate() {
        code += &format!("lui t0, 0x{:x}\nori t0, t0, 0xfff3\nmt{r} t0\nmf{r} t{}\n", 0x1111 * (i + 1), i + 1);
    }
    let (mut e, mut bus) = machine(&code);
    steps(&mut e, &mut bus, 24);
    for i in 0..6usize {
        let v = (0x1111 * (i as u32 + 1)) << 16 | 0xFFF3;
        let want = if i < 2 { v & !3 } else { v };
        assert_eq!(e.gpr[8 + i + 1] as u32, want, "register {i}");
    }
}

#[test]
fn the_index_operations_read_back_what_they_store() {
    let code = "lui t0, 0x1234\nori t0, t0, 0x5678\nmtc0 t0, TagLo\nori t1, zero, 0x2b\nmtc0 t1, TagHi\nsync.p\n\
                lui t2, 0\nori t2, t2, 0x1245\ncache 0x05, 0(t2)\nmtc0 zero, TagLo\nmtc0 zero, TagHi\ncache 0x01, 0(t2)\n\
                mfc0 t3, TagLo\nmfc0 t4, TagHi\n\
                lui t0, 0x0007\nori t0, t0, 0xf030\nmtc0 t0, TagLo\ncache 0x04, 0(t2)\nmtc0 zero, TagLo\ncache 0x00, 0(t2)\nmfc0 t5, TagLo\n\
                lui t0, 0xabcd\nori t0, t0, 0x0009\nmtc0 t0, TagLo\nlui t1, 0x8765\nori t1, t1, 0x4320\nmtc0 t1, TagHi\ncache 0x06, 0(t2)\n\
                mtc0 zero, TagLo\nmtc0 zero, TagHi\ncache 0x02, 0(t2)\nmfc0 t6, TagLo\nmfc0 t7, TagHi\ncache 0x0c, 0(t2)\ncache 0x02, 0(t2)\nmfc0 t8, TagLo";
    let (mut e, mut bus) = machine(code);
    steps(&mut e, &mut bus, 36);
    assert_eq!((e.gpr[11] as u32, e.gpr[12] as u32 & 0x3F), (0x1234_5678, 0x2B), "IXSDT then IXLDT");
    assert_eq!(e.gpr[13] as u32, 0x0007_F030, "IXSTG then IXLTG: PFN 7Fh, V, R");
    assert_eq!((e.gpr[14] as u32, e.gpr[15] as u32), (0xABCD_0009, 0x8765_4320), "BXSBT then BXLBT");
    assert_eq!(e.gpr[24] as u32, 0xABCD_0008, "BFH clears the BTAC's V bit");
}

#[test]
fn a_fetch_past_20000000h_takes_the_instruction_bus_error() {
    let (mut e, mut bus) = machine("nop");
    e.tlb.write(0, 0, 0x4000_0000, 0x2_0000 << 6 | 2 << 3 | 7, 0x2_0001 << 6 | 2 << 3 | 7);
    e.pc = 0x4000_0008;
    steps(&mut e, &mut bus, 1);
    assert_eq!(e.cop0[cop0::CAUSE] >> 2 & 31, 6);
    assert_eq!((e.cop0[cop0::EPC], e.cop0[cop0::BADPADDR]), (0x4000_0008, 0x2000_0000));
    assert_ne!(e.cop0[cop0::STATUS] & status::BEM, 0);
}

#[test]
fn the_physical_map_is_the_ee_users_manuals() {
    let cases = [
        (0x0000_0000, Region::Rdram),
        (0x01FF_FFFC, Region::Rdram),
        (0x0200_0000, Region::Unpopulated),
        (0x1000_0000, Region::Registers),
        (0x1200_0000, Region::Registers),
        (0x1400_0000, Region::Reserved),
        (0x1FC0_0000, Region::BootRom),
        (0x2000_0000, Region::BusError),
        (0x3FFF_FFF0, Region::BusError),
        (0x4000_0000, Region::Extend),
        (0x8000_0000, Region::BusError),
        (0xFFFF_FFF0, Region::BusError),
    ];
    for (pa, r) in cases {
        assert_eq!(map::region(pa), r, "{pa:08x}");
    }
}

#[test]
fn an_interrupt_waits_for_ie_eie_and_im7_and_vectors_to_200h() {
    let (mut e, mut bus) = machine(&"nop\n".repeat(64));
    e.cop0[cop0::COMPARE] = 10;
    steps(&mut e, &mut bus, 40);
    assert_ne!(e.cop0[cop0::CAUSE] & cop0::cause::IP7, 0, "IP7 latched at Compare");
    assert!(e.pc < 0x8000_0000, "masked: no exception");
    e.cop0[cop0::STATUS] = status::IE | status::EIE | status::IM7;
    let pc = e.pc;
    steps(&mut e, &mut bus, 1);
    assert_eq!((e.pc, e.cop0[cop0::EPC], e.cop0[cop0::CAUSE] >> 2 & 31), (0x8000_0200, pc, 0));
    e.int0 = true;
    e.cop0[cop0::STATUS] = status::IE | status::EIE | status::IM2;
    e.cop0[cop0::CAUSE] &= !cop0::cause::IP7;
    e.pc = ORIGIN;
    steps(&mut e, &mut bus, 1);
    assert_eq!((e.pc, e.cop0[cop0::CAUSE] & cop0::cause::IP2), (0x8000_0200, cop0::cause::IP2), "Int0 on IP2 (D-15)");
}

#[test]
fn level_2_exceptions_use_errorepc_bd2_and_the_dev_vectors() {
    let (mut e, mut bus) = machine("nop");
    e.cop0[cop0::STATUS] = status::DEV;
    e.raise2(Level2::Counter, 0x0010_0004, true);
    assert_eq!((e.pc, e.cop0[cop0::ERROREPC]), (0xBFC0_0280, 0x0010_0000));
    assert_eq!(e.cop0[cop0::CAUSE] & (cop0::cause::BD2 | 7 << 16), cop0::cause::BD2 | 2 << 16);
    assert_ne!(e.cop0[cop0::STATUS] & status::ERL, 0);
    e.cop0[cop0::STATUS] = 0;
    e.raise2(Level2::Debug, 0x0010_0008, false);
    assert_eq!((e.pc, e.cop0[cop0::CAUSE] >> 16 & 7), (0x8000_0100, 3));
    steps(&mut e, &mut bus, 0);
    let _ = Exception::Interrupt;
}

#[test]
fn a_miss_costs_its_refill_and_a_dirty_victim_its_write_back() {
    // The cycles with the instruction cache warm, and the data cache warm too or cold.
    let run = |code: &str, n: usize, warm_data: bool| {
        let (mut e, mut bus) = machine(code);
        steps(&mut e, &mut bus, n);
        let mut f = Ee::new(ORIGIN);
        f.icache = e.icache.clone();
        if warm_data {
            f.dcache = e.dcache.clone();
        }
        steps(&mut f, &mut bus, n);
        f.timing.cycles()
    };
    let load = "lui s0, 0x4\nlw t0, 0(s0)\naddu t1, t0, t0";
    assert_eq!(run(load, 3, false), run(load, 3, true) + timing::REFILL, "a load miss waits for the refill");
    let three = |first: &str| format!("lui s0, 0x4\nlui s1, 0x5\nlui s2, 0x6\n{first}\nlw t0, 0(s1)\nlw t1, 0(s2)\naddu t2, t1, t1");
    let dirty = run(&three("sw zero, 0(s0)"), 7, false);
    let clean = run(&three("lw zero, 0(s0)"), 7, false);
    assert_eq!(dirty, clean + timing::WRITEBACK, "the third line of set 0 evicts the first, dirty or clean");
}
