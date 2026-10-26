//! Stage S43, the GS's software rasteriser on worker threads (Sapphire_Native.md §50): generated streams of every
//! primitive, environment, texture (from the frame's own pages too), CLUT load and transfer drawn by the GS on one thread
//! and on 2, 3, 4 and 8 workers, the local memory and every value read back equal at every check; a state saved with
//! primitives in flight; and each wait site held open by holding the workers, the access shown to wait and to see what
//! the one-thread path leaves.

use std::time::{Duration, Instant};

use emusen_native::{State, StateReader, StateWriter};
use sapphire::gs::{Gs, HostInterface, privileged, reg};

struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 16
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }

    fn chance(&mut self, k: u64) -> bool {
        self.below(k) == 0
    }
}

const FRAME_PSM: [u64; 8] = [0x00, 0x01, 0x02, 0x0A, 0x30, 0x31, 0x32, 0x3A];
const Z_PSM: [u64; 4] = [0x0, 0x1, 0x2, 0xA];
const TEX_PSM: [u64; 9] = [0x00, 0x01, 0x02, 0x0A, 0x13, 0x14, 0x1B, 0x24, 0x2C];
const Q: [u32; 6] = [0x3F80_0000, 0x3F00_0000, 0x4000_0000, 0x3E80_0000, 0x3FC0_0000, 0x3F40_0000];

/// One step of a stream: register writes, an image's units, or a read back of `n` units.
enum Step {
    Regs(Vec<(u8, u64)>),
    Image(Vec<u64>),
    Read(usize),
    Check,
}

fn float(r: &mut Rng) -> u32 {
    if r.chance(3) { Q[r.below(6) as usize] } else { 0x3E00_0000 + r.below(0x0180_0000) as u32 }
}

/// A stream of `n` steps over a few pages, so that frames, Z buffers, textures, CLUTs and transfers meet.
fn stream(seed: u64, n: usize) -> Vec<Step> {
    let mut r = Rng(seed);
    let mut out = Vec::new();
    for _ in 0..n {
        match r.below(20) {
            0..=5 => {
                let c = r.below(2) as u8;
                let mut regs = vec![
                    (reg::FRAME_1 + c, r.below(12) | (1 + r.below(4)) << 16 | FRAME_PSM[r.below(8) as usize] << 24 | if r.chance(4) { r.below(1 << 32) << 32 } else { 0 }),
                    (reg::ZBUF_1 + c, r.below(12) | Z_PSM[r.below(4) as usize] << 24 | u64::from(r.chance(4)) << 32),
                    (reg::TEST_1 + c, r.below(1 << 19)),
                    (reg::SCISSOR_1 + c, r.below(64) | (64 + r.below(260)) << 16 | r.below(64) << 32 | (64 + r.below(260)) << 48),
                    (reg::ALPHA_1 + c, r.below(256) | r.below(256) << 32),
                    (reg::FBA_1 + c, u64::from(r.chance(4))),
                    (reg::PABE, u64::from(r.chance(4))),
                    (reg::DTHE, u64::from(r.chance(3))),
                    (reg::DIMX, r.next() << 16 | r.below(1 << 16)),
                    (reg::COLCLAMP, r.below(2)),
                    (reg::SCANMSK, if r.chance(6) { 2 + r.below(2) } else { 0 }),
                    (reg::XYOFFSET_1 + c, if r.chance(3) { r.below(64) << 4 | r.below(64) << 36 } else { 0 }),
                    (reg::PRMODECONT, u64::from(!r.chance(8))),
                    (reg::PRMODE, r.below(256) << 3),
                    (reg::FOGCOL, r.below(1 << 24)),
                    (reg::TEXA, r.below(256) | r.below(2) << 15 | r.below(256) << 32),
                    (reg::TEXCLUT, 1 + r.below(4) | r.below(4) << 6 | r.below(16) << 12),
                    (reg::TEX1_1 + c, r.below(2) | r.below(7) << 2 | r.below(2) << 5 | r.below(6) << 6 | u64::from(r.chance(4)) << 9 | r.below(4) << 19 | r.below(1 << 12) << 32),
                    (reg::CLAMP_1 + c, r.below(16) | r.below(1024) << 4 | r.below(1024) << 14 | r.below(1024) << 24 | r.below(1024) << 34),
                    (reg::MIPTBP1_1 + c, r.next() & ((1 << 60) - 1)),
                    (reg::MIPTBP2_1 + c, r.next() & ((1 << 60) - 1)),
                ];
                let tex0 = r.below(12 * 32) | (1 + r.below(4)) << 14 | TEX_PSM[r.below(9) as usize] << 20 | (2 + r.below(7)) << 26 | (2 + r.below(7)) << 30 | r.below(2) << 34 | r.below(4) << 35 | r.below(12 * 32) << 37 | [0u64, 2, 0xA][r.below(3) as usize] << 51 | r.below(2) << 55 | r.below(32) << 56 | r.below(6) << 61;
                regs.push((reg::TEX0_1 + c, tex0));
                if r.chance(4) {
                    regs.push((reg::TEX2_1 + c, tex0 ^ (r.below(32) << 56)));
                }
                out.push(Step::Regs(regs));
            }
            6..=14 => {
                let kind = r.below(8);
                let attributes = r.below(256) << 3;
                let mut regs = vec![(reg::PRIM, kind | attributes)];
                for _ in 0..1 + r.below(6) {
                    let q = float(&mut r);
                    regs.push((reg::RGBAQ, r.below(1 << 32) | u64::from(q) << 32));
                    regs.push((reg::ST, u64::from(float(&mut r)) | u64::from(float(&mut r)) << 32));
                    regs.push((reg::UV, r.below(1 << 14) | r.below(1 << 14) << 16));
                    let (x, y) = (r.below(340 * 16), r.below(340 * 16));
                    let which = [reg::XYZ2, reg::XYZF2, reg::XYZ3, reg::XYZF3][if r.chance(6) { 2 + r.below(2) } else { r.below(2) } as usize];
                    regs.push((which, x | y << 16 | r.below(1 << 32) << 32));
                }
                out.push(Step::Regs(regs));
            }
            15 | 16 => {
                let psm = TEX_PSM[r.below(9) as usize];
                let (w, h) = (1 + r.below(48), 1 + r.below(48));
                let bits = match psm {
                    0x00 => 32,
                    0x01 => 24,
                    0x02 | 0x0A => 16,
                    0x13 | 0x1B => 8,
                    _ => 4,
                };
                out.push(Step::Regs(vec![
                    (reg::BITBLTBUF, (r.below(12 * 32)) << 32 | (1 + r.below(4)) << 48 | psm << 56),
                    (reg::TRXPOS, r.below(200) << 32 | r.below(200) << 48),
                    (reg::TRXREG, w | h << 32),
                    (reg::TRXDIR, 0),
                ]));
                // A quarter of the transfers are cut short, so that the next TRXDIR ends them.
                let units = (w * h * bits).div_ceil(64);
                let units = if r.chance(4) { r.below(units) } else { units };
                out.push(Step::Image((0..units).map(|_| r.next() << 16 ^ r.next()).collect()));
            }
            17 => {
                let psm = FRAME_PSM[r.below(4) as usize];
                let (w, h) = (1 + r.below(32), 1 + r.below(32));
                out.push(Step::Regs(vec![
                    (reg::BITBLTBUF, r.below(12 * 32) | (1 + r.below(4)) << 16 | psm << 24 | (r.below(12 * 32)) << 32 | (1 + r.below(4)) << 48 | psm << 56),
                    (reg::TRXPOS, r.below(200) | r.below(200) << 16 | r.below(200) << 32 | r.below(200) << 48 | r.below(4) << 59),
                    (reg::TRXREG, w | h << 32),
                    (reg::TRXDIR, 2),
                ]));
            }
            18 => {
                let psm = FRAME_PSM[r.below(8) as usize];
                let (w, h) = (1 + r.below(40), 1 + r.below(40));
                out.push(Step::Regs(vec![
                    (reg::BITBLTBUF, r.below(12 * 32) | (1 + r.below(4)) << 16 | psm << 24),
                    (reg::TRXPOS, r.below(260) | r.below(260) << 16),
                    (reg::TRXREG, w | h << 32),
                    (reg::TRXDIR, 1),
                ]));
                let bits = if psm & 2 != 0 { 16 } else if psm & 1 != 0 { 24 } else { 32 };
                out.push(Step::Read((w * h * bits).div_ceil(64) as usize));
            }
            _ => out.push(Step::Check),
        }
    }
    out.push(Step::Check);
    out
}

fn busdir(gs: &mut Gs, v: u64) {
    gs.write_privileged(privileged::address(privileged::BUSDIR), 8, v);
}

/// The stream run on `gs`; every value read back and the memory's digest at each check, in order.
fn run(gs: &mut Gs, steps: &[Step]) -> Vec<u64> {
    let mut out = Vec::new();
    for s in steps {
        match s {
            Step::Regs(list) => list.iter().for_each(|&(r, v)| gs.write(r, v)),
            Step::Image(units) => units.iter().for_each(|&u| gs.image(u)),
            Step::Read(n) => {
                busdir(gs, 1);
                out.extend((0..*n).map(|_| gs.read().unwrap_or(0xDEAD)));
                busdir(gs, 0);
            }
            Step::Check => out.push(gs.local().words().iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &w| (h ^ u64::from(w)).wrapping_mul(0x100_0000_01B3))),
        }
    }
    out
}

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
fn a_host_to_local_transfer_waits_for_the_primitives_drawing_the_pixels_it_overwrites() {
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

#[test]
fn a_primitive_textured_from_pages_another_worker_draws_waits_for_them() {
    // The sprite at page 0 drawn split while held; then a sprite into page 8 textured from page 0 (PSMCT32, 64 × 64).
    let (want, got, waited) = held_site(
        |gs| {
            gs.write(reg::FRAME_1, 8 | 1 << 16);
            gs.write(reg::TEX0_1, 1 << 14 | 6 << 26 | 6 << 30 | 1 << 35);
            gs.write(reg::PRIM, 6 | 1 << 4 | 1 << 8);
            gs.write(reg::UV, 0);
            gs.write(reg::XYZ2, 0);
            gs.write(reg::UV, 64 * 16 | 64 * 16 << 16);
            gs.write(reg::XYZ2, 64 * 16 | 64 * 16 << 16);
            u64::from(gs.local().words()[8 * 2048 + 100])
        },
        |_| {},
    );
    assert_eq!((want, got), (0x8040_2010, 0x8040_2010));
    assert!(waited >= HELD - Duration::from_millis(5), "waited {waited:?}");
}

#[test]
fn a_clut_loaded_while_primitives_still_read_the_last_does_not_change_what_they_draw() {
    // Two PSMT4 sprites, each after its own CLUT, the first drawn while every worker is held.
    let program = |gs: &mut Gs| {
        for (k, colour) in [0x1111_1111u64, 0x2222_2222].into_iter().enumerate() {
            gs.write(reg::BITBLTBUF, 64 << 32 | 1 << 48);
            gs.write(reg::TRXPOS, 0);
            gs.write(reg::TRXREG, 8 | 2 << 32);
            gs.write(reg::TRXDIR, 0);
            (0..8).for_each(|_| gs.image(colour | colour << 32));
            gs.write(reg::TEX0_1, 128 | 1 << 14 | 0x14 << 20 | 4 << 26 | 4 << 30 | 1 << 35 | 64 << 37 | 1 << 61);
            gs.write(reg::FRAME_1, (8 + k as u64) | 1 << 16);
            gs.write(reg::SCISSOR_1, 63 << 16 | 63 << 48);
            gs.write(reg::PRMODECONT, 1);
            gs.write(reg::PRIM, 6 | 1 << 4 | 1 << 8);
            gs.write(reg::UV, 0);
            gs.write(reg::XYZ2, 0);
            gs.write(reg::UV, 16 * 16 | 16 * 16 << 16);
            gs.write(reg::XYZ2, 16 * 16 | 16 * 16 << 16);
        }
    };
    let mut one = Gs::new();
    one.set_workers(1);
    program(&mut one);
    let want = one.local().clone();
    let mut gs = threaded(2);
    let hold = gs.hold().expect("workers");
    hold.set(0b11);
    let releaser = std::thread::spawn(move || {
        std::thread::sleep(HELD);
        hold.set(0);
    });
    program(&mut gs);
    releaser.join().unwrap();
    assert!(*gs.local() == want, "the memory after both sprites");
    assert_eq!(want.words()[8 * 2048] & 0xFF, 0x11);
    assert_eq!(want.words()[9 * 2048] & 0xFF, 0x22);
}
