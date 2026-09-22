//! The display processor's list on a drain against the same list run at once: C#'s `MarsThreadedRdpTests`, and a seeded stress of the same. See Mars_Native.md §5.6.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::Ordering::Relaxed;

use crate::machine::Machine;
use crate::memory::bus::RDRAM_SIZE;
use crate::memory::bus_access::map::{DP_COMMAND_BASE, MI_BASE};
use crate::memory::dp::SNAPSHOT_WORDS;
use crate::memory::dp_threads::{Threads, site};

const START: u32 = DP_COMMAND_BASE;
const END: u32 = DP_COMMAND_BASE + 4;
const MI_INTERRUPT: u32 = MI_BASE + 8;

pub(super) const LIST: u32 = 0x0010_0000;
pub(super) const FRAMEBUFFER: u32 = 0x0020_0000;
pub(super) const DEPTH: u32 = 0x0028_0000;
pub(super) const TEXTURE: u32 = 0x0030_0000;
pub(super) const WIDTH: u32 = 320;
pub(super) const ROWS: u32 = 240;

const SYNC_FULL: u64 = 0x29 << 56;
const FILL_CYCLE: u64 = (0x2F << 56) | (3 << 52);
const DEPTH_CYCLE: u64 = (0x2F << 56) | (3 << 4);

pub(super) fn at_once() -> Machine {
    Machine::new(RDRAM_SIZE).unwrap()
}

pub(super) fn threaded() -> Machine {
    let mut m = at_once();
    m.set_verify_rdp(true);
    m.set_threaded_rdp(true);
    assert!(m.bus.dp.threads.is_some());
    m
}

fn threads(m: &Machine) -> &Threads {
    m.bus.dp.threads.as_deref().unwrap()
}

fn mark(m: &Machine, at: u32) -> i64 {
    m.bus.dp.marks.marks[(at >> 12) as usize].load(Relaxed)
}

fn write_mark(m: &Machine, at: u32) -> i64 {
    m.bus.dp.marks.write_marks[(at >> 12) as usize].load(Relaxed)
}

pub(super) fn hand_over(m: &mut Machine, list: &[u64], at: u32) {
    for (i, &word) in list.iter().enumerate() {
        m.bus.write64(at + i as u32 * 8, word);
    }
    m.bus.write32(START, at);
    m.bus.write32(END, at + list.len() as u32 * 8);
}

pub(super) fn state(m: &Machine) -> Vec<u8> {
    m.save_state_vec(false).unwrap()
}

fn scissor(left: u32, top: u32, right: u32, bottom: u32) -> u64 {
    (0x2D << 56) | (((left << 2) as u64) << 44) | (((top << 2) as u64) << 32) | (((right << 2) as u64) << 12) | (bottom << 2) as u64
}

fn fill_rectangle(left: u32, top: u32, right: u32, bottom: u32) -> u64 {
    (0x36 << 56) | (((right << 2) as u64) << 44) | (((bottom << 2) as u64) << 32) | (((left << 2) as u64) << 12) | (top << 2) as u64
}

fn color_image(at: u32) -> u64 {
    (0x3F << 56) | (2 << 51) | (((WIDTH - 1) as u64) << 32) | at as u64
}

/// `Scene`: a full frame buffer of fill rectangles, a depth image set, and, when asked, a texture loaded from RDRAM.
pub(super) fn scene(color: u32, load: bool, load_rows: u64, depth: bool) -> Vec<u64> {
    let mut list = vec![if depth { DEPTH_CYCLE } else { FILL_CYCLE }, color_image(FRAMEBUFFER), (0x3E << 56) | DEPTH as u64, scissor(0, 0, WIDTH, ROWS), (0x37 << 56) | color as u64];
    for row in (0..ROWS).step_by(8) {
        list.push(fill_rectangle(0, row, WIDTH - 1, row + 7));
    }
    if load {
        list.push((0x3D << 56) | (2 << 51) | (63 << 32) | TEXTURE as u64);
        list.push((0x35 << 56) | (2 << 51) | (16 << 41));
        list.push((0x34 << 56) | ((31u64 << 2) << 12) | ((load_rows - 1) << 2));
    }
    list.push(SYNC_FULL);
    list
}

fn next(state: &mut u32) -> u32 {
    *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
    *state >> 8
}

fn corner(state: &mut u32, extent: u32, beyond: u32) -> f64 {
    (next(state) % (extent + 2 * beyond)) as f64 - beyond as f64
}

#[allow(clippy::too_many_arguments)]
fn combine(a: u64, b: u64, c: u64, d: u64, alpha_a: u64, alpha_b: u64, alpha_c: u64, alpha_d: u64) -> u64 {
    let high = (a << 20) | (c << 15) | (alpha_a << 12) | (alpha_c << 9) | (a << 5) | c;
    let low = ((b << 28) | (b << 24) | (alpha_a << 21) | (alpha_c << 18) | (d << 15) | (alpha_b << 12) | (alpha_d << 9) | (d << 6) | (alpha_b << 3) | alpha_d) & 0xFFFF_FFFF;
    (0x3C << 56) | (high << 32) | low
}

fn edge(x: f64, slope: f64) -> u64 {
    (((x * 65536.0).round_ties_even() as i32 as u32 as u64) << 32) | ((slope.clamp(-8192.0, 8191.0) * 65536.0).round_ties_even() as i32 as u32 as u64)
}

/// `Triangle`: a shaded, depth-tested triangle's words from three corners, its shade and depth from a seed.
fn triangle(id: u32, corners: [(f64, f64); 3], seed: u32) -> Vec<u64> {
    let mut sorted = corners;
    sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let (top, middle, bottom) = (sorted[0], sorted[1], sorted[2]);
    let major = if bottom.1 > top.1 { (bottom.0 - top.0) / (bottom.1 - top.1) } else { 0.0 };
    let upper = if middle.1 > top.1 { (middle.0 - top.0) / (middle.1 - top.1) } else { 0.0 };
    let lower = if bottom.1 > middle.1 { (bottom.0 - middle.0) / (bottom.1 - middle.1) } else { 0.0 };
    let (yh, ym, yl) = ((top.1 * 4.0).floor() as i32, (middle.1 * 4.0).floor() as i32, (bottom.1 * 4.0).floor() as i32);
    let row_top = top.1.floor();
    let (xh, xm, xl) = (top.0 + major * (row_top - top.1), top.0 + upper * (row_top - top.1), middle.0 + lower * (ym as f64 / 4.0 - middle.1));
    let major_on_left = middle.0 > top.0 + major * (middle.1 - top.1);

    let mut words = vec![0u64; crate::rdp::command_length(id) as usize];
    words[0] = ((id as u64) << 56) | if major_on_left { 1 << 55 } else { 0 } | (((yl & 0x3FFF) as u64) << 32) | (((ym & 0x3FFF) as u64) << 16) | (yh & 0x3FFF) as u64;
    words[1] = edge(xl, lower);
    words[2] = edge(xh, major);
    words[3] = edge(xm, upper);
    let mut s = seed;
    for word in &mut words[4..12] {
        *word = (((next(&mut s) & 0x007F_FFFF) as u64) << 32) | (next(&mut s) & 0x0003_FFFF) as u64;
    }
    if words.len() > 12 {
        words[12] = (((0x1000 + (next(&mut s) & 0xFFFF)) as u64) << 48) | (((next(&mut s) & 0xFFFF) as u64) << 32) | (((next(&mut s) & 0x3FF) as u64) << 16) | (next(&mut s) & 0xFFFF) as u64;
        words[13] = (((next(&mut s) & 0x3FF) as u64) << 48) | (((next(&mut s) & 0xFFFF) as u64) << 32) | (next(&mut s) & 0x03FF_FFFF) as u64;
    }
    words
}

/// `Shaded`: a cleared frame and depth buffer, then two dozen shaded, depth-tested triangles, some crossing the right edge.
pub(super) fn shaded(seed: u32, to_the_edge: bool, two_cycle: bool, memory_alpha_first: bool) -> Vec<u64> {
    let mut list = vec![
        FILL_CYCLE,
        color_image(FRAMEBUFFER),
        (0x3E << 56) | DEPTH as u64,
        scissor(0, 0, WIDTH, ROWS),
        (0x37 << 56) | 0x0001_0001,
        fill_rectangle(0, 0, WIDTH - 1, ROWS - 1),
        (0x37 << 56) | 0xFFFC_FFFC,
        color_image(DEPTH),
        fill_rectangle(0, 0, WIDTH - 1, ROWS - 1),
        color_image(FRAMEBUFFER),
    ];
    let blend = (1u64 << 22) | (1 << 20) | if memory_alpha_first { (1 << 18) | (1 << 16) } else { 0 };
    list.push((0x2F << 56) | ((two_cycle as u64) << 52) | (3 << 38) | (3 << 36) | blend | (1 << 4) | (1 << 5) | (1 << 3) | (1 << 6));
    list.push(combine(4, 8, 11, 7, 4, 7, 4, 7));
    let mut state = seed;
    for i in 0..24 {
        let mut c = [(0.0, 0.0); 3];
        for corner_at in &mut c {
            *corner_at = (corner(&mut state, WIDTH, 20), corner(&mut state, ROWS, 10));
        }
        if to_the_edge && i & 1 == 0 {
            c[1].0 = (WIDTH + 30) as f64;
        }
        let seed = next(&mut state);
        list.extend(triangle(0x0C, c, seed));
    }
    list.push(SYNC_FULL);
    list
}

fn same_memory(a: &Machine, b: &Machine) {
    a.bus.dp.wait_all();
    b.bus.dp.wait_all();
    assert!(a.bus.rdram[..] == b.bus.rdram[..], "RDRAM differs");
    assert!(a.bus.rdram_hidden[..] == b.bus.rdram_hidden[..], "hidden RDRAM differs");
}

#[test]
fn a_list_on_the_thread_leaves_memory_and_the_state_as_the_list_at_once() {
    for list in [scene(0x1234_5678, false, 32, false), shaded(0x1122_3344, false, false, false), shaded(0x5566_7788, true, false, false), shaded(0x99AA_BBCC, true, true, true)] {
        let (mut once, mut drain) = (at_once(), threaded());
        hand_over(&mut once, &list, LIST);
        hand_over(&mut drain, &list, LIST);
        assert_eq!(once.bus.read32(MI_INTERRUPT), drain.bus.read32(MI_INTERRUPT));
        assert_eq!(once.bus.dp.status_word(), drain.bus.dp.status_word());
        drain.join_rdp();
        same_memory(&once, &drain);
        assert!(state(&once) == state(&drain));
    }
}

#[test]
fn a_state_written_while_the_thread_draws_holds_the_finished_drawing() {
    let (mut once, mut drain) = (at_once(), threaded());
    for pass in 0..6u32 {
        let list = if pass % 2 == 0 { scene(0x0101_0101u32.wrapping_mul(pass + 1), false, 32, false) } else { shaded(pass, true, pass == 3, false) };
        hand_over(&mut once, &list, LIST);
        hand_over(&mut drain, &list, LIST);
        assert!(state(&once) == state(&drain), "pass {pass}");
    }
}

#[test]
fn a_read_of_the_image_being_drawn_waits_for_the_drawing() {
    let mut drain = threaded();
    let at = FRAMEBUFFER + WIDTH * 2 * 100;
    for pass in 1..=20u32 {
        let color = 0x1111_1111u32.wrapping_mul(pass & 0xF) | 1;
        hand_over(&mut drain, &scene(color, false, 32, false), LIST);
        let narrowed = threads(&drain).counters.reads_narrowed;
        assert_ne!(mark(&drain, at), 0);
        assert_eq!(drain.bus.read32(at + 40), color);
        assert!(threads(&drain).counters.reads_narrowed > narrowed || mark(&drain, at) == 0);
    }
}

#[test]
fn pages_no_command_reaches_are_not_marked_and_the_images_and_texture_source_are() {
    let mut drain = threaded();
    drain.bus.dp.threads.as_ref().unwrap().hold();
    hand_over(&mut drain, &scene(0x8000_8000, true, 32, false), LIST);
    assert_ne!(mark(&drain, FRAMEBUFFER), 0);
    assert_ne!(mark(&drain, FRAMEBUFFER + WIDTH * 2 * (ROWS - 1)), 0);
    assert_ne!(mark(&drain, TEXTURE), 0);
    assert_eq!(mark(&drain, DEPTH + WIDTH * 2 * 120), 0, "fill mode never reaches depth");
    threads(&drain).resume();
    drain.join_rdp();
    threads(&drain).hold();
    hand_over(&mut drain, &scene(0x8000_8000, true, 32, true), LIST);
    assert_ne!(mark(&drain, DEPTH + WIDTH * 2 * 120), 0);
    assert_eq!(mark(&drain, 0x0018_0000), 0);
    assert_eq!(mark(&drain, 0x0038_0000), 0);
    assert_eq!(mark(&drain, LIST), 0);
    threads(&drain).resume();
    drain.join_rdp();
    assert_eq!(mark(&drain, FRAMEBUFFER), 0);
}

#[test]
fn a_page_only_a_load_reads_stops_writers_within_the_load_and_nobody_else() {
    let mut drain = threaded();
    threads(&drain).hold();
    hand_over(&mut drain, &scene(0x8000_8000, true, 16, false), LIST);
    assert_ne!(mark(&drain, TEXTURE), 0);
    assert_eq!(write_mark(&drain, TEXTURE), 0);
    assert_ne!(mark(&drain, FRAMEBUFFER), 0);
    assert_ne!(write_mark(&drain, FRAMEBUFFER), 0);

    // Held, so the load is surely pending when the writers come, which C#'s test leaves to the timing.
    drain.bus.write32(TEXTURE + 0x900, 0x1234_5678);
    assert_eq!(threads(&drain).counters.bystanders, 1);
    assert_ne!(mark(&drain, TEXTURE), 0);
    threads(&drain).resume();
    drain.bus.write32(TEXTURE + 0x100, 0x1234_5678);
    assert_eq!(threads(&drain).counters.bystanders, 1);
    assert_eq!(mark(&drain, TEXTURE), 0);
    drain.join_rdp();
}

#[test]
fn a_list_beside_the_depth_image_is_read_without_waiting() {
    let list_at = DEPTH + WIDTH * 2 * ROWS + 0x100;
    assert_eq!((DEPTH + WIDTH * 2 * (ROWS - 1)) >> 12, list_at >> 12);
    let (mut once, mut drain) = (at_once(), threaded());
    let list = scene(0x7C1F_7C1F, false, 32, true);
    hand_over(&mut once, &list, list_at);
    hand_over(&mut drain, &list, list_at);
    assert_eq!(threads(&drain).counters.waits_per_site[site::TAKE], 0);
    assert!(threads(&drain).counters.bystanders > 0);
    drain.join_rdp();
    same_memory(&once, &drain);
    assert!(state(&once) == state(&drain));
}

fn fault_of(m: &mut Machine) -> String {
    let panic = catch_unwind(AssertUnwindSafe(|| m.join_rdp())).expect_err("the join raised no fault");
    panic.downcast_ref::<String>().cloned().unwrap_or_default()
}

#[test]
fn the_verifier_faults_a_write_outside_the_marked_pages_and_one_outside_the_marked_rows() {
    let mut drain = threaded();
    hand_over(&mut drain, &scene(0x0001_0001, false, 32, false), LIST);
    drain.bus.read32(FRAMEBUFFER);
    let word = threads(&drain).issued();
    threads(&drain).shared().verify(0x0018_0000, word, true);
    assert!(fault_of(&mut drain).contains("did not mark"));

    let mut drain = threaded();
    threads(&drain).hold();
    hand_over(&mut drain, &scene(0x0002_0002, false, 32, false), LIST);
    let beyond = FRAMEBUFFER + WIDTH * 2 * ROWS + 0x100;
    assert_ne!(write_mark(&drain, beyond), 0);
    let word = threads(&drain).issued();
    threads(&drain).shared().verify(beyond as usize, word, true);
    threads(&drain).resume();
    assert!(fault_of(&mut drain).contains("outside every range"));

    let mut drain = threaded();
    hand_over(&mut drain, &scene(0x0003_0003, false, 32, false), LIST);
    drain.join_rdp();
}

#[test]
fn a_snapshot_taken_while_the_thread_stands_loads_to_what_the_finished_list_leaves() {
    for list in [scene(0x2345_6789, false, 32, false), shaded(0x3456_789A, true, true, false)] {
        let (mut once, mut drain) = (at_once(), threaded());
        hand_over(&mut once, &list, LIST);
        threads(&drain).hold();
        hand_over(&mut drain, &list, LIST);
        assert_eq!(threads(&drain).shared().drain_words.load(Relaxed), 0);
        let snapshot = drain.save_state_vec(true).unwrap();
        let finished = state(&once);
        assert_eq!(snapshot.len(), finished.len() + 4 + 8 * SNAPSHOT_WORDS);
        assert_eq!(i32::from_le_bytes(snapshot[finished.len()..finished.len() + 4].try_into().unwrap()) as usize, list.len());

        let mut loaded = at_once();
        loaded.load_state(&snapshot).unwrap();
        loaded.bus.dp_replay_pending();
        loaded.loaded_version = once.loaded_version;
        same_memory(&once, &loaded);
        assert!(finished == state(&loaded));

        threads(&drain).resume();
        drain.join_rdp();
        assert!(finished == state(&drain));
    }
}

#[test]
fn a_snapshot_of_a_machine_with_nothing_in_flight_is_the_state_with_an_empty_tail() {
    let mut drain = threaded();
    hand_over(&mut drain, &scene(0x0F0F_0F0F, false, 32, false), LIST);
    drain.join_rdp();
    let (state, snapshot) = (state(&drain), drain.save_state_vec(true).unwrap());
    assert_eq!(snapshot.len(), state.len() + 4 + 8 * SNAPSHOT_WORDS);
    assert!(state[8..] == snapshot[8..state.len()], "the body after the version");
}

#[test]
fn a_drain_stopped_and_started_again_carries_on_from_the_processor() {
    let (mut once, mut drain) = (at_once(), threaded());
    for pass in 0..4u32 {
        let list = shaded(0x0707_0707 ^ pass, true, pass & 1 == 1, false);
        let (first, rest) = list.split_at(list.len() / 3 + 1);
        hand_over(&mut once, first, LIST);
        hand_over(&mut drain, first, LIST);
        drain.set_threaded_rdp(pass % 2 == 0);
        hand_over(&mut once, rest, LIST + 0x8000);
        hand_over(&mut drain, rest, LIST + 0x8000);
        drain.set_threaded_rdp(true);
        assert!(state(&once) == state(&drain), "pass {pass}");
    }
}

/// A seeded mixture of lists, loads from what was drawn, and processor reads and writes of every image between and inside them.
fn stress_with(seed: u32, rounds: u32, workers: usize) {
    let (mut once, mut drain) = (at_once(), if workers == 1 { threaded() } else { split(workers) });
    let mut s = seed;
    for round in 0..rounds {
        let pick = next(&mut s) % 4;
        let list = match pick {
            0 => scene(next(&mut s), next(&mut s) & 1 == 0, 1 + (next(&mut s) % 32) as u64, next(&mut s) & 1 == 0),
            1 => shaded(next(&mut s), next(&mut s) & 1 == 0, next(&mut s) & 1 == 0, next(&mut s) & 1 == 0),
            _ => {
                let mut l = shaded(next(&mut s), true, false, false);
                l.pop();
                l.push((0x3D << 56) | (2 << 51) | (((WIDTH - 1) as u64) << 32) | FRAMEBUFFER as u64);
                l.push((0x35 << 56) | (2 << 51) | (16 << 41));
                l.push((0x34 << 56) | ((31u64 << 2) << 12) | (((next(&mut s) % 16) as u64) << 2));
                l.extend(shaded(next(&mut s), true, true, true));
                l
            }
        };
        let pieces = 1 + next(&mut s) % 3;
        let mut from = 0;
        let mut at = LIST + (round % 4) * 0x2_0000;
        for piece in 0..pieces {
            let to = if piece + 1 == pieces { list.len() } else { (from + (next(&mut s) as usize % list.len().max(1))).min(list.len()) };
            hand_over(&mut once, &list[from..to], at);
            hand_over(&mut drain, &list[from..to], at);
            at += ((to - from) as u32 * 8 + 0xFF) & !0xFF;
            from = to;
            for _ in 0..next(&mut s) % 6 {
                let image = [FRAMEBUFFER, DEPTH, TEXTURE, LIST][(next(&mut s) % 4) as usize];
                let address = (image + (next(&mut s) % (WIDTH * 2 * ROWS))) & !3;
                if next(&mut s) & 1 == 0 {
                    assert_eq!(once.bus.read32(address), drain.bus.read32(address), "seed {seed} round {round}: a read of {address:06X}");
                } else {
                    let value = next(&mut s);
                    once.bus.write32(address, value);
                    drain.bus.write32(address, value);
                }
            }
        }
        if next(&mut s).is_multiple_of(5) {
            assert!(state(&once) == state(&drain), "seed {seed} round {round}");
        }
    }
    drain.join_rdp();
    same_memory(&once, &drain);
    let (a, b) = (state(&once), state(&drain));
    if a != b {
        let layout = once.layout(false).unwrap();
        for line in layout.lines() {
            let p: Vec<&str> = line.split(' ').collect();
            let (at, n) = (p[0].parse::<usize>().unwrap(), p[1].parse::<usize>().unwrap());
            if a[at..at + n] != b[at..at + n] {
                eprintln!("differs: {} ({} bytes) {:?} vs {:?}", p[3], n, &a[at..at + n.min(16)], &b[at..at + n.min(16)]);
            }
        }
    }
    assert!(a == b, "seed {seed} at the end");
}

#[test]
fn seeded_lists_with_processor_accesses_between_leave_what_the_lists_at_once_leave() {
    let seeds: u32 = std::env::var("EMUSEN_MARSRT_STRESS").ok().and_then(|v| v.parse().ok()).unwrap_or(if cfg!(debug_assertions) { 3 } else { 24 });
    for seed in 1..=seeds {
        stress_with(seed, 12, 1);
    }
}

/// A read of a range waits for every pending draw on its pages, where C# frees one whose first eight bytes no pending draw holds (Mars_Native.md §5.6.2).
#[test]
fn a_range_read_whose_first_bytes_no_draw_holds_still_waits_for_the_draws_that_hold_the_rest() {
    let list = [FILL_CYCLE, color_image(FRAMEBUFFER), scissor(0, 0, WIDTH, ROWS), (0x37 << 56) | 0x1234_5678, fill_rectangle(0, 8, WIDTH - 1, 9), SYNC_FULL];
    let page = FRAMEBUFFER + 0x1000;
    let row = (FRAMEBUFFER + WIDTH * 2 * 8) as usize;
    assert!(page + 8 < FRAMEBUFFER + WIDTH * 2 * 7, "the page's first bytes lie above the drawn rows");
    let (mut once, mut drain) = (at_once(), threaded());
    hand_over(&mut once, &list, LIST);
    threads(&drain).hold();
    hand_over(&mut drain, &list, LIST);
    assert_ne!(write_mark(&drain, page), 0);

    let resume = threads(&drain).resumer();
    let watchdog = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(300));
        resume();
    });
    let started = std::time::Instant::now();
    drain.bus.dp.wait_read_range(page, 0x1000, site::VI);
    let waited = started.elapsed();
    let seen = drain.bus.rdram[row..row + 0x80].to_vec();
    watchdog.join().unwrap();
    assert!(waited >= std::time::Duration::from_millis(250), "the read went ahead of the draw after {waited:?}");
    assert!(seen[..] == once.bus.rdram[row..row + 0x80], "the read saw the rows before the draw");
}

pub(super) fn split(workers: usize) -> Machine {
    let mut m = threaded();
    m.set_rdp_workers(workers);
    assert_eq!(threads(&m).workers(), workers);
    m
}

/// A raw load, its words run, and the machine's threads started again as the options say, as C#'s `ReadState` with a snapshot leaves a threaded bus.
fn load_snapshot(m: &mut Machine, snapshot: &[u8]) {
    m.load_state(snapshot).unwrap();
    m.bus.dp_replay_pending();
    let threaded = m.options.threaded_rdp;
    m.set_threaded_rdp(threaded);
}

fn triangle_rows() -> u64 {
    (0x2F << 56) | (3 << 38) | (3 << 36) | (1 << 22) | (1 << 20) | (1 << 4) | (1 << 5) | (1 << 3) | (1 << 6)
}

#[test]
fn processors_sharing_a_list_leave_what_one_leaves() {
    for workers in [2, 3, 4] {
        for list in [scene(0x1234_5678, false, 32, false), shaded(0x1122_3344, false, false, false), shaded(0x5566_7788, true, false, false), shaded(0x99AA_BBCC, true, true, false)] {
            let (mut once, mut one, mut shared) = (at_once(), threaded(), split(workers));
            hand_over(&mut once, &list, LIST);
            hand_over(&mut one, &list, LIST);
            hand_over(&mut shared, &list, LIST);
            one.join_rdp();
            shared.join_rdp();
            same_memory(&once, &shared);
            assert!(state(&once) == state(&shared), "{workers} workers");
            assert!(state(&one) == state(&shared), "{workers} workers");
        }
    }
}

#[test]
fn a_load_from_the_drawn_image_and_a_live_carry_are_drawn_as_at_once() {
    let mut list = shaded(0x0F0F_0F0F, true, false, false);
    list.pop();
    list.push(combine(4, 0, 11, 7, 4, 7, 4, 7));
    let again = shaded(0x0F0F_0F0F, true, false, false);
    list.extend(&again[12..again.len() - 1]);
    list.push((0x3D << 56) | (2 << 51) | (((WIDTH - 1) as u64) << 32) | FRAMEBUFFER as u64);
    list.push((0x35 << 56) | (2 << 51) | (16 << 41));
    list.push((0x34 << 56) | ((31u64 << 2) << 12) | (15 << 2));
    list.extend(shaded(0xF0F0_F0F0, true, true, true));

    let (mut once, mut shared) = (at_once(), split(3));
    hand_over(&mut once, &list, LIST);
    hand_over(&mut shared, &list, LIST);
    shared.join_rdp();
    let s = &*shared.bus.dp.processor.split;
    assert!(s.hazard_loads >= 1 && s.serialised >= 1, "loads {} serialised {}", s.hazard_loads, s.serialised);
    same_memory(&once, &shared);
    assert!(state(&once) == state(&shared));
}

#[test]
fn the_read_past_a_rows_end_is_made_by_the_next_rows_owner_before_it_runs_on() {
    let list = [
        FILL_CYCLE,
        color_image(FRAMEBUFFER),
        (0x3E << 56) | DEPTH as u64,
        scissor(0, 0, WIDTH, ROWS),
        (0x37 << 56) | 0x0001_0001,
        fill_rectangle(0, 0, WIDTH - 1, ROWS - 1),
        triangle_rows(),
        combine(4, 8, 11, 7, 4, 7, 4, 7),
        (0x36 << 56) | (((WIDTH << 2) as u64) << 44) | (403 << 32) | 400,
        FILL_CYCLE,
        (0x37 << 56) | 0x7777_7777,
        fill_rectangle(0, 101, WIDTH - 1, 101),
        SYNC_FULL,
    ];
    for _ in 0..8 {
        let (mut once, mut shared) = (at_once(), split(2));
        hand_over(&mut once, &list, LIST);
        hand_over(&mut shared, &list, LIST);
        shared.join_rdp();
        let aliased = threads(&shared).aliased_reads();
        same_memory(&once, &shared);
        assert!(state(&once) == state(&shared));
        assert_eq!(aliased, 1, "the next row's owner made the read");
    }
}

#[test]
fn an_image_one_row_into_the_last_is_drawn_after_the_last_is_finished() {
    let mut list = vec![FILL_CYCLE, color_image(FRAMEBUFFER), (0x3E << 56) | DEPTH as u64, scissor(0, 0, WIDTH, ROWS), triangle_rows(), combine(4, 8, 11, 7, 4, 7, 4, 7)];
    for _ in 0..20 {
        list.push((0x36 << 56) | ((((WIDTH - 1) << 2) as u64) << 44) | (7 << 32) | 4);
    }
    list.extend([FILL_CYCLE, (0x37 << 56) | 0x1111_1111, fill_rectangle(0, 0, WIDTH - 1, ROWS - 1), color_image(FRAMEBUFFER + WIDTH * 2), (0x37 << 56) | 0x2222_2222, fill_rectangle(0, 0, WIDTH - 1, ROWS - 2), SYNC_FULL]);
    let (mut once, mut shared) = (at_once(), split(2));
    hand_over(&mut once, &list, LIST);
    hand_over(&mut shared, &list, LIST);
    shared.join_rdp();
    same_memory(&once, &shared);
    assert!(state(&once) == state(&shared));
}

#[test]
fn a_snapshot_while_every_processor_waits_inside_a_command_is_written_and_loads() {
    let list = shaded(0x1357_9BDF, true, false, false);
    let cut = list.len() - 12;
    let (mut once, mut shared, mut loaded) = (at_once(), split(2), split(2));
    hand_over(&mut once, &list[..cut], LIST);
    hand_over(&mut once, &list[cut..cut + 5], LIST + cut as u32 * 8);
    hand_over(&mut once, &list[cut + 5..], LIST + (cut + 5) as u32 * 8);
    hand_over(&mut shared, &list[..cut], LIST);
    shared.join_rdp();
    hand_over(&mut shared, &list[cut..cut + 5], LIST + cut as u32 * 8);

    let snapshot = shared.save_state_vec(true).unwrap();
    hand_over(&mut shared, &list[cut + 5..], LIST + (cut + 5) as u32 * 8);
    shared.join_rdp();
    same_memory(&once, &shared);
    assert!(state(&once) == state(&shared), "the resumed machine's state differs");

    load_snapshot(&mut loaded, &snapshot);
    hand_over(&mut loaded, &list[cut + 5..], LIST + (cut + 5) as u32 * 8);
    loaded.join_rdp();
    same_memory(&once, &loaded);
    assert!(state(&once) == state(&loaded), "the loaded machine's state differs");
}

#[test]
fn a_snapshot_with_several_processors_stands_them_at_one_boundary() {
    for seed in 0..6u32 {
        let list = shaded(0x2468_ACE0 ^ seed, true, seed & 1 == 1, false);
        let (mut once, mut shared, mut loaded) = (at_once(), split(4), split(2));
        hand_over(&mut once, &list, LIST);
        if seed < 3 {
            threads(&shared).hold();
        }
        hand_over(&mut shared, &list, LIST);
        let snapshot = shared.save_state_vec(true).unwrap();
        if seed < 3 {
            threads(&shared).resume();
        }
        load_snapshot(&mut loaded, &snapshot);
        loaded.join_rdp();
        same_memory(&once, &loaded);
        assert!(state(&once) == state(&loaded), "seed {seed}: the loaded machine's state differs");
        shared.join_rdp();
        assert!(state(&once) == state(&shared), "seed {seed}");
    }
}

#[test]
fn seeded_lists_shared_by_several_processors_leave_what_the_lists_at_once_leave() {
    let seeds: u32 = std::env::var("EMUSEN_MARSRT_STRESS").ok().and_then(|v| v.parse().ok()).unwrap_or(if cfg!(debug_assertions) { 2 } else { 12 });
    for seed in 1..=seeds {
        for workers in [2, 3, 4] {
            stress_with(seed * 7 + workers as u32, 8, workers);
        }
    }
}

/// Rows that compute no level of detail stamp no fraction, so a primitive drawn alone assembles the raster order's, where C#'s split assembles a stale one (Mars_Native.md §5.6.6).
#[test]
fn a_fraction_carried_through_rows_that_compute_none_is_assembled_as_raster_order_leaves_it() {
    for seed in 0..16u32 {
        let mut list = scene(0x1234_5679 + seed, false, 32, true);
        list.extend(shaded(0x2468_1357 + seed, false, false, false));
        list.extend(shaded(0x1357_2468 + seed, true, true, true));
        let (mut once, mut shared) = (at_once(), split(2));
        hand_over(&mut once, &list, LIST);
        hand_over(&mut shared, &list, LIST);
        shared.join_rdp();
        same_memory(&once, &shared);
        assert_eq!(once.bus.dp.processor.lod_fraction, shared.bus.dp.processor.lod_fraction, "seed {seed}");
        assert!(state(&once) == state(&shared), "seed {seed}");
    }
}

/// A list of image changes, each a barrier, with fills between them.
fn barriers(count: usize) -> Vec<u64> {
    let mut list = vec![FILL_CYCLE, scissor(0, 0, WIDTH, ROWS), (0x37 << 56) | 0x0F0F_0F0F];
    for i in 0..count {
        list.push(color_image(FRAMEBUFFER + (i as u32 % 4) * WIDTH * 2));
        list.push(fill_rectangle(0, (i as u32 * 3) % (ROWS - 2), WIDTH - 1, (i as u32 * 3) % (ROWS - 2) + 1));
    }
    list.push(SYNC_FULL);
    list
}

/// A snapshot taken while the workers are at barriers is answered: a worker waiting at one raises the pause point past it, where C#'s workers wait for ones standing short of it.
#[test]
fn a_snapshot_taken_while_workers_wait_at_barriers_is_answered_and_loads() {
    let run = std::thread::spawn(move || {
        let list = barriers(3000);
        let (mut once, mut shared, mut loaded) = (at_once(), split(4), at_once());
        for attempt in 0..40u32 {
            hand_over(&mut once, &list, LIST + (attempt % 2) * 0x10000);
            hand_over(&mut shared, &list, LIST + (attempt % 2) * 0x10000);
            for _ in 0..(attempt * 997) % 20_000 {
                std::hint::spin_loop();
            }
            let snapshot = shared.save_state_vec(true).unwrap();
            if attempt % 8 == 0 {
                load_snapshot(&mut loaded, &snapshot);
                assert!(state(&once) == state(&loaded), "attempt {attempt}: the loaded snapshot differs");
            }
        }
        shared.join_rdp();
        assert!(state(&once) == state(&shared));
        threads(&shared).shared().barriers()
    });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    while !run.is_finished() {
        assert!(std::time::Instant::now() < deadline, "a snapshot was never answered: the workers deadlocked");
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let barriers = run.join().expect("the run failed");
    assert!(barriers >= 40 * 3000, "{barriers} barriers passed");
}

/// A load from the colour image before the image's first draw, then draws over its rows: every processor loads before any draws, where C#'s rule lets a slow one load what a fast one drew (Mars_Native.md §5.6.6).
#[test]
fn a_load_from_the_current_image_before_its_first_draw_is_run_by_every_processor_together() {
    let mut list = vec![FILL_CYCLE, scissor(0, 0, WIDTH, ROWS), (0x37 << 56) | 0x0102_0304, color_image(FRAMEBUFFER + 0x4_0000)];
    list.push(fill_rectangle(0, 0, WIDTH - 1, 63));
    list.push(color_image(FRAMEBUFFER));
    list.push((0x3D << 56) | (2 << 51) | (((WIDTH - 1) as u64) << 32) | FRAMEBUFFER as u64);
    list.push((0x35 << 56) | (2 << 51) | (16 << 41));
    list.push((0x34 << 56) | ((31u64 << 2) << 12) | (15 << 2));
    list.push((0x37 << 56) | 0x7E7E_7E7E);
    for row in 0..16 {
        list.push(fill_rectangle(0, row, WIDTH - 1, row));
    }
    list.push(SYNC_FULL);
    for attempt in 0..40 {
        let (mut once, mut shared) = (at_once(), split(2 + attempt % 3));
        for m in [&mut once, &mut shared] {
            for i in 0..(WIDTH * 2 * 16) / 4 {
                m.bus.write32(FRAMEBUFFER + i * 4, 0x1111_1111u32.wrapping_mul(i));
            }
        }
        hand_over(&mut once, &list, LIST);
        hand_over(&mut shared, &list, LIST);
        shared.join_rdp();
        assert!(shared.bus.dp.processor.split.hazard_loads >= 1, "the load was not joined");
        same_memory(&once, &shared);
        assert!(state(&once) == state(&shared), "attempt {attempt}");
    }
}
