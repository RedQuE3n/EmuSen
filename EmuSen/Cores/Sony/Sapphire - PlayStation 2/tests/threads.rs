//! Stage S43, the GS's software rasteriser on worker threads (Sapphire_Native.md §50): generated streams of every
//! primitive, environment, texture (from the frame's own pages too), CLUT load and transfer drawn by the GS on one thread
//! and on 2, 3, 4 and 8 workers, the local memory and every value read back equal at every check; a state saved with
//! primitives in flight; and each wait site held open by holding the workers, the access shown to wait and to see what
//! the one-thread path leaves.

use std::time::{Duration, Instant};

use emusen_native::{State, StateReader, StateWriter};
use sapphire::gs::{Gs, HostInterface, privileged, reg};

mod gsgen;

use gsgen::{Step, busdir, run, stream};

fn save(gs: &Gs) -> Vec<u8> {
    let mut c = StateWriter::counter();
    gs.write_state(&mut c);
    let mut v = vec![0; c.len()];
    gs.write_state(&mut StateWriter::new(&mut v));
    v
}

fn threaded(n: usize) -> Gs {
    let mut gs = Gs::new();
    gs.set_workers(n);
    assert_eq!(gs.worker_count(), n);
    gs
}

#[test]
fn generated_streams_leave_the_memory_and_every_read_back_as_one_thread_does_at_every_worker_count() {
    let mut drawn = (0, 0);
    for seed in 0..16u64 {
        let steps = stream(seed, 700);
        let mut one = Gs::new();
        one.set_workers(1);
        let want = run(&mut one, &steps);
        for n in [2, 3, 4, 8] {
            let mut gs = threaded(n);
            let got = run(&mut gs, &steps);
            let first = want.iter().zip(&got).position(|(a, b)| a != b);
            assert!(first.is_none() && want.len() == got.len(), "seed {seed}, {n} workers: the first difference at entry {first:?} of {}", want.len());
            let c = gs.worker_counts().expect("workers");
            drawn = (drawn.0 + c.split, drawn.1 + c.alone);
        }
    }
    assert!(drawn.0 > 1000 && drawn.1 > 100, "both kinds of primitive drawn: split {} alone {}", drawn.0, drawn.1);
}

#[test]
fn a_state_saved_with_primitives_in_flight_loads_and_continues_as_one_thread_does() {
    let mut in_flight = 0;
    for seed in 10..13u64 {
        let steps = stream(seed, 600);
        let (a, b) = steps.split_at(300);
        // Primitives alone between the hold and the save, so that nothing but a primitive's own pages waits there.
        let tail: Vec<Step> = stream(seed + 100, 400).into_iter().filter(|s| matches!(s, Step::Regs(r) if r[0].0 == reg::PRIM)).collect();
        let mut one = Gs::new();
        one.set_workers(1);
        let mut want = run(&mut one, a);
        want.extend(run(&mut one, &tail));
        want.extend(run(&mut one, b));
        let mut gs = threaded(4);
        let mut got = run(&mut gs, a);
        let hold = gs.hold().expect("workers");
        hold.set(0b1010);
        // Released from another thread, since the GS's thread may wait for a held worker before a primitive.
        let releaser = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(100));
            hold.set(0);
        });
        got.extend(run(&mut gs, &tail));
        in_flight += gs.pending();
        let bytes = save(&gs);
        releaser.join().unwrap();
        let mut loaded = threaded(3);
        loaded.read_state(&mut StateReader::new(&bytes)).expect("the state loads");
        got.extend(run(&mut loaded, b));
        assert_eq!(want, got, "seed {seed}");
    }
    assert!(in_flight > 0, "some primitives were in flight when a state was saved");
}

const HELD: Duration = Duration::from_millis(150);

/// A frame at page 0 (64 × 64, PSMCT32) filled by a sprite of colour `c` while every worker is held; the access `f`
/// made, the workers released from another thread after HELD.
fn held_site(f: impl Fn(&mut Gs) -> u64, setup: impl Fn(&mut Gs)) -> (u64, u64, Duration) {
    let draw = |gs: &mut Gs| {
        setup(gs);
        gs.write(reg::FRAME_1, 1 << 16);
        gs.write(reg::SCISSOR_1, 63 << 16 | 63 << 48);
        gs.write(reg::TEST_1, 0);
        gs.write(reg::PRMODECONT, 1);
        gs.write(reg::PRIM, 6);
        gs.write(reg::RGBAQ, 0x8040_2010);
        gs.write(reg::XYZ2, 0);
        gs.write(reg::XYZ2, 64 * 16 | 64 * 16 << 16);
    };
    let mut one = Gs::new();
    one.set_workers(1);
    draw(&mut one);
    let want = f(&mut one);
    let mut gs = threaded(4);
    let hold = gs.hold().expect("workers");
    hold.set(0xF);
    draw(&mut gs);
    let t0 = Instant::now();
    let releaser = std::thread::spawn(move || {
        std::thread::sleep(HELD);
        hold.set(0);
    });
    let got = f(&mut gs);
    let waited = t0.elapsed();
    releaser.join().unwrap();
    (want, got, waited)
}

fn read_back(gs: &mut Gs) -> u64 {
    gs.write(reg::BITBLTBUF, 1 << 16);
    gs.write(reg::TRXPOS, 5 | 7 << 16);
    gs.write(reg::TRXREG, 2 | 1 << 32);
    gs.write(reg::TRXDIR, 1);
    busdir(gs, 1);
    let v = gs.read().unwrap();
    busdir(gs, 0);
    v
}

#[test]
fn a_local_to_host_transfer_waits_for_the_primitives_drawing_its_pixels() {
    let (want, got, waited) = held_site(read_back, |_| {});
    assert_eq!(want, 0x8040_2010_8040_2010);
    assert_eq!(got, want);
    assert!(waited >= HELD - Duration::from_millis(5), "waited {waited:?}");
}

#[test]
fn a_host_to_local_transfer_is_written_after_the_primitives_drawing_the_pixels_it_overwrites() {
    let (want, got, waited) = held_site(
        |gs| {
            gs.write(reg::BITBLTBUF, 1 << 48);
            gs.write(reg::TRXPOS, 5 << 32 | 7 << 48);
            gs.write(reg::TRXREG, 2 | 1 << 32);
            gs.write(reg::TRXDIR, 0);
            gs.image(0x1111_2222_3333_4444);
            read_back(gs)
        },
        |_| {},
    );
    assert_eq!(want, 0x1111_2222_3333_4444);
    assert_eq!((got, waited >= HELD - Duration::from_millis(5)), (want, true), "waited {waited:?}");
}

#[test]
fn a_clut_load_from_a_page_being_drawn_waits_and_loads_what_was_drawn() {
    let (want, got, waited) = held_site(
        |gs| {
            // A PSMT4 texture whose CLUT (CSM1, PSMCT32) is the frame's first 8 × 2 pixels, loaded with CLD 1.
            gs.write(reg::TEX0_2, 0x14 << 20 | 1 << 61);
            u64::from(gs.local().words().iter().take(16).fold(0u32, |h, &w| h.rotate_left(3) ^ w))
        },
        |_| {},
    );
    assert_eq!((got, waited >= HELD - Duration::from_millis(5)), (want, true), "waited {waited:?}");
}

#[test]
fn a_pcrtc_line_waits_for_the_primitives_drawing_its_pages() {
    let (want, got, waited) = held_site(
        |gs| {
            gs.settle(sapphire::gs::psm::Psm::Ct32, 0, 1, (0, 0x7FF), (9, 9));
            u64::from(gs.peek(sapphire::gs::psm::Psm::Ct32, 0, 1, 20, 9))
        },
        |_| {},
    );
    assert_eq!((want, got, waited >= HELD - Duration::from_millis(5)), (0x8040_2010, want, true), "waited {waited:?}");
}

#[test]
fn a_state_and_the_debugger_wait_for_every_primitive() {
    let digest = |gs: &mut Gs| save(gs).iter().fold(0u64, |h, &b| h.rotate_left(5) ^ u64::from(b));
    let (want, got, waited) = held_site(digest, |_| {});
    assert_eq!((got, waited >= HELD - Duration::from_millis(5)), (want, true), "waited {waited:?}");
    let (want, got, waited) = held_site(|gs| u64::from(gs.local().words()[3]), |_| {});
    assert_eq!((got, waited >= HELD - Duration::from_millis(5)), (want, true), "waited {waited:?}");
}

/// `program` run on one thread and on two workers with worker 1 held for HELD from the start: the memory equal.
fn with_worker_1_held(program: impl Fn(&mut Gs)) -> (Gs, Gs) {
    let mut one = Gs::new();
    one.set_workers(1);
    program(&mut one);
    one.join();
    let mut gs = threaded(2);
    let hold = gs.hold().expect("workers");
    hold.set(0b10);
    let releaser = std::thread::spawn(move || {
        std::thread::sleep(HELD);
        hold.set(0);
    });
    program(&mut gs);
    releaser.join().unwrap();
    assert!(*gs.local() == *one.local(), "the memory after the program");
    (one, gs)
}

/// A 64 × 64 PSMCT32 sprite of one colour at page 0: rows 0 to 31 worker 0's, 32 to 63 worker 1's.
fn fill_page_0(gs: &mut Gs) {
    gs.write(reg::FRAME_1, 1 << 16);
    gs.write(reg::SCISSOR_1, 63 << 16 | 63 << 48);
    gs.write(reg::TEST_1, 0);
    gs.write(reg::PRMODECONT, 1);
    gs.write(reg::PRIM, 6);
    gs.write(reg::RGBAQ, 0x8040_2010);
    gs.write(reg::XYZ2, 0);
    gs.write(reg::XYZ2, 64 * 16 | 64 * 16 << 16);
}

#[test]
fn a_primitive_textured_from_rows_another_worker_draws_waits_for_them() {
    // Page 0 filled while worker 1 is held, then a sprite into page 8 textured from page 0 upside down, so that
    // worker 0's rows read the rows worker 1 has still to draw.
    let (one, _) = with_worker_1_held(|gs| {
        fill_page_0(gs);
        gs.write(reg::FRAME_1, 8 | 1 << 16);
        gs.write(reg::TEX0_1, 1 << 14 | 6 << 26 | 6 << 30 | 1 << 35);
        gs.write(reg::PRIM, 6 | 1 << 4 | 1 << 8);
        gs.write(reg::UV, 64 * 16 << 16);
        gs.write(reg::XYZ2, 0);
        gs.write(reg::UV, 64 * 16);
        gs.write(reg::XYZ2, 64 * 16 | 64 * 16 << 16);
    });
    assert!(one.local().words()[8 * 2048..9 * 2048].iter().all(|&w| w == 0x8040_2010));
}

#[test]
fn a_clut_loaded_while_primitives_still_read_the_last_does_not_change_what_they_draw() {
    // Two CLUTs (PSMCT32, CSM1) uploaded and drawn in, then two PSMT4 sprites, each after loading its CLUT, the first
    // still being drawn by the held worker when the second is loaded.
    let program = |gs: &mut Gs| {
        for (cbp, colour) in [(64u64, 0x1111_1111u64), (96, 0x2222_2222)] {
            gs.write(reg::BITBLTBUF, cbp << 32 | 1 << 48);
            gs.write(reg::TRXPOS, 0);
            gs.write(reg::TRXREG, 8 | 2 << 32);
            gs.write(reg::TRXDIR, 0);
            (0..8).for_each(|_| gs.image(colour | colour << 32));
        }
        gs.join();
        gs.write(reg::SCISSOR_1, 63 << 16 | 63 << 48);
        gs.write(reg::TEST_1, 0);
        gs.write(reg::PRMODECONT, 1);
        for (k, cbp) in [64u64, 96].into_iter().enumerate() {
            gs.write(reg::TEX0_1, 128 | 1 << 14 | 0x14 << 20 | 6 << 26 | 6 << 30 | 1 << 35 | cbp << 37 | 1 << 61);
            gs.write(reg::FRAME_1, (8 + k as u64) | 1 << 16);
            gs.write(reg::PRIM, 6 | 1 << 4 | 1 << 8);
            gs.write(reg::UV, 0);
            gs.write(reg::XYZ2, 0);
            gs.write(reg::UV, 64 * 16 | 64 * 16 << 16);
            gs.write(reg::XYZ2, 64 * 16 | 64 * 16 << 16);
        }
    };
    let (one, gs) = with_worker_1_held(program);
    assert_eq!(one.local().words()[8 * 2048 + 2047] & 0xFF, 0x11);
    assert_eq!(one.local().words()[9 * 2048 + 2047] & 0xFF, 0x22);
    assert_eq!(gs.worker_counts().expect("workers").clut_copies, 2);
}

#[test]
fn a_clut_slot_is_reused_only_after_the_worker_drawing_with_it_is_done() {
    // Three CLUTs loaded in turn before 300 PSMT4 sprites on worker 1's rows, each in its own page: more loads than the
    // CLUT slots, so a slot comes round again while the held worker has yet to draw the sprite that read it.
    let program = |gs: &mut Gs| {
        for (cbp, colour) in [(64u64, 0x1111_1111u64), (96, 0x2222_2222), (160, 0x3333_3333)] {
            gs.write(reg::BITBLTBUF, cbp << 32 | 1 << 48);
            gs.write(reg::TRXPOS, 0);
            gs.write(reg::TRXREG, 8 | 2 << 32);
            gs.write(reg::TRXDIR, 0);
            (0..8).for_each(|_| gs.image(colour | colour << 32));
        }
        gs.join();
        gs.write(reg::SCISSOR_1, 63 << 16 | 63 << 48);
        gs.write(reg::TEST_1, 0);
        gs.write(reg::PRMODECONT, 1);
        for k in 0..300u64 {
            let cbp = [64u64, 96, 160][k as usize % 3];
            gs.write(reg::TEX0_1, 128 | 1 << 14 | 0x14 << 20 | 6 << 26 | 6 << 30 | 1 << 35 | cbp << 37 | 1 << 61);
            gs.write(reg::FRAME_1, (16 + k) | 1 << 16);
            gs.write(reg::PRIM, 6 | 1 << 4 | 1 << 8);
            gs.write(reg::UV, 0);
            gs.write(reg::XYZ2, (32 * 16) << 16);
            gs.write(reg::UV, 8 * 16 | 32 * 16 << 16);
            gs.write(reg::XYZ2, 8 * 16 | (64 * 16) << 16);
        }
    };
    let (one, gs) = with_worker_1_held(program);
    assert_eq!(one.local().words()[17 * 2048] & 0xFF, 0x11);
    assert!(gs.worker_counts().expect("workers").clut_copies >= 300);
}
