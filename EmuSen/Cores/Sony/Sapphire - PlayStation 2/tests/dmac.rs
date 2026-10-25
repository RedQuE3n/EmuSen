//! Stage S11's gate (Sapphire_Native.md §20.7): the project's programs on the DMAC, the INTC and the timers, run on the
//! interpreter with the board and graded by the records the EE User's Manual's rules give; and the timings and
//! interfaces no program can show a referee, checked on the units directly.

mod s6;
mod s11;

use sapphire::board::{Board, System};
use sapphire::clock::{self, BUS};
use sapphire::dmac::{self, Dmac, Peripherals};
use sapphire::ee::Bus;
use sapphire::ee::map::Map;
use sapphire::timer::Timer;

use s6::{DONE, DONE_AT, ORIGIN, Program, RECORD};

fn assemble(source: &str, origin: u32) -> Vec<u32> {
    emusen_mips::assemble(source, origin, &emusen_mips::Dialect::R5900).unwrap_or_else(|e| panic!("{e:?}"))
}

fn run(source: &str, limit: u64) -> System {
    let mut sys = System::new(ORIGIN);
    sys.board.map.load(ORIGIN, &assemble(source, ORIGIN));
    sys.board.ports.ports[dmac::VIF0].keep = true;
    let n = sys.run(limit, &mut |s| s.peek(DONE_AT, 4) as u32 == DONE).unwrap_or_else(|h| panic!("{h:?}"));
    assert!(n < limit, "no done word in {limit} instructions, pc {:08x}", sys.ee.pc);
    sys
}

/// Runs a program and returns the system with the record's words that differ.
fn grade_system(p: Program) -> System {
    let name = p.name;
    let (source, expected) = p.finish();
    let mut sys = run(&source, 60_000_000);
    let mut wrong = Vec::new();
    for (i, want) in expected.iter().enumerate() {
        let got = sys.peek(RECORD + 4 * i as u32, 4) as u32;
        if let Some(w) = want
            && *w != got
        {
            wrong.push(format!("word {i} (record {}): {got:08x}, expected {w:08x}", i / 2));
        }
    }
    assert!(wrong.is_empty(), "{name}: {} of {} words differ, first {:?}", wrong.len(), expected.len(), &wrong[..wrong.len().min(8)]);
    sys
}

fn record_word(sys: &mut System, record: usize) -> u32 {
    sys.peek(RECORD + 8 * record as u32, 4) as u32
}

#[test]
fn the_scratchpad_channels_and_the_common_registers_equal_chapter_5() {
    grade_system(s11::spr());
}

#[test]
fn source_and_destination_chains_walk_their_tags_as_chapter_5_says() {
    grade_system(s11::chain());
}

#[test]
fn call_and_ret_push_and_pop_the_address_stack() {
    let sys = grade_system(s11::callret());
    let vif0 = &sys.board.ports.ports[dmac::VIF0];
    assert_eq!(vif0.taken.len(), 3, "VIF0 receives the three packets' data and no tag");
    assert!(vif0.taken.iter().all(|&(q, tag)| q == 0 && !tag));
}

#[test]
fn interleave_moves_blocks_and_skips_as_the_console_text_records() {
    grade_system(s11::interleave());
}

/// ps2autotests' `dma/spr/interleave.expected`, the console's own lines for MADR after each upload and download
/// (the "WARNING: DMA transfer MADR did not increase as expected" lines), against the rule the programs are graded by.
#[test]
fn the_interleave_rule_gives_the_console_texts_madr_lines() {
    let console: [(u32, u32, u32, u32); 13] = [
        (1, 1, 3, 0x60),
        (1, 2, 3, 0x90),
        (1, 3, 3, 0xC0),
        (2, 1, 4, 0x60),
        (3, 1, 6, 0x80),
        (2, 2, 6, 0xC0),
        (3, 3, 9, 0x120),
        (1, 14, 2, 0x1E0),
        (1, 128, 2, 0x1020),
        (1, 257, 2, 0x40),
        (128, 1, 128, 0x810),
        (257, 1, 3, 0x60),
        (1, 255, 4, 0x4000),
    ];
    for (t, s, count, madr) in console {
        assert_eq!(s11::interleave_advance(t, s, count), madr, "transfer {t}, skip {s}, count {count}");
    }
    assert_eq!(s11::interleave_advance(1, 256, 2), 0x20, "skip 256 prints no warning: SQWC is eight bits");
}

#[test]
fn stall_control_holds_and_releases_the_drain() {
    grade_system(s11::stall());
}

#[test]
fn the_mfifo_drain_holds_while_empty_and_wraps_with_the_ring() {
    let sys = grade_system(s11::mfifo());
    assert_eq!(sys.board.ports.ports[dmac::VIF1].count, 70, "VIF1 receives every cnt packet's data and no tag");
}

#[test]
fn the_intc_latches_masks_and_drives_int0_and_the_dmac_int1() {
    let mut sys = grade_system(s11::intc());
    let span = u64::from(record_word(&mut sys, s11_intc_vblank_record()));
    let lines = clock::NTSC.lines - clock::NTSC.active;
    let want = lines * clock::NTSC.line * BUS;
    assert!(span.abs_diff(want) < 200, "V-blank lasted {span} cycles, {want} expected");
}

/// The record index of `s11-intc`'s V-blank span: its last record.
fn s11_intc_vblank_record() -> usize {
    s11::intc().expected.len() - 1
}

#[test]
fn the_timers_count_compare_overflow_and_gate_as_chapter_4_says() {
    let mut sys = grade_system(s11::timers());
    let (t0, t1, t2) = (record_word(&mut sys, 0), record_word(&mut sys, 1), record_word(&mut sys, 2));
    assert!((t0 / 16).abs_diff(t1) <= 1 && (t1 / 16).abs_diff(t2) <= 1, "{t0} {t1} {t2}");
    let p = s11::timers();
    let lines = record_word(&mut sys, p.expected.len() - 2);
    assert_eq!(lines, clock::NTSC.lines as u32, "H-blank edges between two V-blank starts");
}

/// A memory for the units' own tests: RDRAM alone.
fn memory() -> Map {
    Map::new()
}

struct Sink(Vec<u128>, usize);

impl Peripherals for Sink {
    fn request(&mut self, _ch: usize, want: usize) -> usize {
        want.min(self.1)
    }
    fn put(&mut self, _ch: usize, q: u128, _tag: bool) {
        self.0.push(q);
    }
    fn get(&mut self, _ch: usize) -> u128 {
        0
    }
}

/// Runs the DMAC alone until it is idle; the times its units ended.
fn drain(d: &mut Dmac, mem: &mut Map, spr: &mut [u8], dev: &mut dyn Peripherals) -> Vec<u64> {
    let mut ends = Vec::new();
    let mut guard = 0;
    while d.next_event() != u64::MAX {
        let t = d.next_event();
        d.step(t, mem, spr, dev);
        ends.push(t);
        guard += 1;
        assert!(guard < 10_000);
    }
    ends
}

#[test]
fn a_burst_moves_a_qword_a_bus_cycle_and_cycle_stealing_releases_every_128_bytes() {
    let mut mem = memory();
    let mut spr = vec![0u8; 16 << 10];
    let mut dev = Sink(Vec::new(), usize::MAX);
    let mut d = Dmac::default();
    d.write(dmac::D_CTRL, 1, 0);
    d.write(dmac::BASES[dmac::TO_SPR] + 0x20, 64, 0);
    d.write(dmac::BASES[dmac::TO_SPR], 0x100, 0);
    let ends = drain(&mut d, &mut mem, &mut spr, &mut dev);
    assert_eq!(ends.last().copied(), Some(64 * BUS), "64 qwords in one burst: 64 bus cycles");
    let mut d = Dmac::default();
    d.write(dmac::D_CTRL, 1 | 2 | 3 << 8, 0);
    d.write(dmac::BASES[dmac::TO_SPR] + 0x20, 64, 0);
    d.write(dmac::BASES[dmac::TO_SPR], 0x100, 0);
    let ends = drain(&mut d, &mut mem, &mut spr, &mut dev);
    let unit = 8 * BUS;
    let release = 64 * BUS;
    assert!(ends.contains(&(8 * unit + 7 * release)), "eight 128-byte units, RCYC 64 between them: {ends:?}");
    assert!(!ends.contains(&(64 * BUS)));
}

#[test]
fn a_slice_channel_moves_eight_qwords_to_an_eight_qword_boundary_and_rearbitrates() {
    let mut mem = memory();
    let mut spr = vec![0u8; 16 << 10];
    let mut dev = Sink(Vec::new(), usize::MAX);
    let mut d = Dmac::default();
    d.write(dmac::D_CTRL, 1, 0);
    let gif = dmac::BASES[dmac::GIF];
    d.write(gif + 0x10, 0x1000 + 3 * 16, 0);
    d.write(gif + 0x20, 20, 0);
    d.write(gif, 0x100, 0);
    let ends = drain(&mut d, &mut mem, &mut spr, &mut dev);
    let units: Vec<u64> = ends.windows(2).map(|w| (w[1] - w[0]) / BUS).filter(|&n| n > 0).collect();
    assert_eq!(units, vec![5, 8, 7], "20 qwords from qword 3 of a slice: 5, then 8, then the 7 left");
    assert_eq!(dev.0.len(), 20);
    assert!(d.stat & 1 << dmac::GIF != 0 && !d.ch[dmac::GIF].str());
}

#[test]
fn a_peripheral_without_dreq_holds_its_channel_until_woken() {
    let mut mem = memory();
    let mut spr = vec![0u8; 16 << 10];
    let mut dev = Sink(Vec::new(), 0);
    let mut d = Dmac::default();
    d.write(dmac::D_CTRL, 1, 0);
    let vif1 = dmac::BASES[dmac::VIF1];
    d.write(vif1 + 0x20, 4, 0);
    d.write(vif1, 0x101, 0);
    d.step(0, &mut mem, &mut spr, &mut dev);
    assert_eq!(d.next_event(), u64::MAX, "no DREQ, no event: the DMAC is not polled");
    dev.1 = usize::MAX;
    d.wake(100);
    let ends = drain(&mut d, &mut mem, &mut spr, &mut dev);
    assert_eq!(ends.last().copied(), Some(100 + 4 * BUS));
    assert_eq!(d.ch[dmac::VIF1].qwc, 0);
}

#[test]
fn group_a_is_served_before_group_c() {
    let mut mem = memory();
    let mut spr = vec![0u8; 16 << 10];
    let mut d = Dmac::default();
    struct Order(Vec<usize>);
    impl Peripherals for Order {
        fn request(&mut self, _ch: usize, want: usize) -> usize {
            want
        }
        fn put(&mut self, ch: usize, _q: u128, _tag: bool) {
            self.0.push(ch);
        }
        fn get(&mut self, _ch: usize) -> u128 {
            0
        }
    }
    let mut dev = Order(Vec::new());
    d.write(dmac::D_CTRL, 1, 0);
    for c in [dmac::GIF, dmac::VIF0] {
        d.write(dmac::BASES[c] + 0x20, 16, 0);
        d.write(dmac::BASES[c], 0x100, 0);
    }
    drain(&mut d, &mut mem, &mut spr, &mut dev);
    assert_eq!(dev.0[..16].iter().filter(|&&c| c == dmac::VIF0).count(), 16, "VIF0's two slices first: {:?}", dev.0);
}

#[test]
fn a_bus_error_ends_the_transfer_with_beis_and_int1_unmasked() {
    let mut mem = memory();
    let mut spr = vec![0u8; 16 << 10];
    let mut dev = Sink(Vec::new(), usize::MAX);
    let mut d = Dmac::default();
    d.write(dmac::D_CTRL, 1, 0);
    let s = dmac::BASES[dmac::TO_SPR];
    d.write(s + 0x10, 0x2000_0000, 0);
    d.write(s + 0x20, 2, 0);
    d.write(s, 0x100, 0);
    drain(&mut d, &mut mem, &mut spr, &mut dev);
    assert_eq!(d.stat & (1 << 15 | 1 << dmac::TO_SPR), 1 << 15 | 1 << dmac::TO_SPR);
    assert!(d.int1(), "BEIS has no mask (§5.8's formula)");
}

#[test]
fn a_timer_is_evaluated_at_its_flag_and_not_before() {
    let mut t = Timer::default();
    t.write(0x20, 0x100, false, false);
    t.write(0x10, 0x180, false, false);
    assert_eq!(t.next_event(), 0x100 * BUS, "the compare's tick: COMP bus cycles from 0");
    assert_eq!(t.sync(0x100 * BUS - 1), 0);
    assert_eq!(t.sync(0x100 * BUS), 1, "EQUF rises at its tick");
    assert_eq!(t.next_event(), u64::MAX, "EQUF set: nothing left to schedule");
    let mut t = Timer::default();
    t.write(0x00, 0xFFF0, false, false);
    t.write(0x10, 0x281, false, false);
    assert_eq!(t.next_event(), 16 * 16 * BUS, "overflow after 16 ticks of BUSCLK/16");
}

#[test]
fn zret_returns_to_zero_at_the_tick_that_reaches_comp() {
    let mut t = Timer::default();
    t.write(0x20, 3, false, false);
    t.write(0x10, 0xC0, false, false);
    let seen: Vec<u32> = (1..=9).map(|k| { t.sync(k * BUS); t.read(0) }).collect();
    assert_eq!(seen, vec![1, 2, 0, 1, 2, 0, 1, 2, 0], "the tick that reaches COMP gives 0 (D-38)");
    t.write(0x10, 0x1C0, false, false);
    assert_eq!(t.next_event(), 12 * BUS, "EQUF at the tick that would reach COMP");
}

#[test]
fn the_board_holds_no_h_blank_event_unless_a_timer_needs_it() {
    let mut b = Board::new(clock::NTSC);
    let mut spr = vec![0u8; 16 << 10];
    b.run_to(10 * clock::NTSC.line * BUS, &mut spr);
    assert_eq!(b.intc.stat, 0, "no V-blank edge in ten lines");
    b.write(sapphire::timer::BASE + 0x1810, 4, 0x83);
    b.run_to(20 * clock::NTSC.line * BUS, &mut spr);
    assert_eq!(b.timers[3].count, 10, "H-blank as a clock: one tick a line");
    b.run_to(clock::NTSC.lines * clock::NTSC.line * BUS, &mut spr);
    assert_eq!(b.intc.stat, 0b1100, "VB_ON at line 240 and VB_OFF at the field's end");
}
