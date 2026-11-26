//! The GS drawn on the Vulkan device (`Sapphire_Gs.md`), compared against the software rasteriser.

use emusen_native::{State, StateReader, StateWriter};
use sapphire::gs::{Gs, HostInterface, reg};

use crate::gsgen;

use gsgen::{run, stream};

fn device() -> Gs {
    let mut gs = Gs::new();
    gs.set_workers(1);
    gs.set_device(false).expect("Vulkan hardware device required");
    gs.set_exact(true);
    gs
}

fn software() -> Gs {
    let mut gs = Gs::new();
    gs.set_workers(1);
    gs
}

fn save(gs: &Gs) -> Vec<u8> {
    let mut c = StateWriter::counter();
    gs.write_state(&mut c);
    let mut v = vec![0; c.len()];
    gs.write_state(&mut StateWriter::new(&mut v));
    v
}

mod vulkan_gpu {
    use super::*;

    #[test]
    fn generated_streams_leave_the_memory_and_every_read_back_as_the_software_rasteriser_does() {
        let (mut records, mut stretches, mut fallbacks) = (0, 0, 0);
        for seed in 0..16u64 {
            let steps = stream(seed, 700);
            let want = run(&mut software(), &steps);
            let mut gs = device();
            let got = run(&mut gs, &steps);
            let first = want.iter().zip(&got).position(|(a, b)| a != b);
            assert!(first.is_none() && want.len() == got.len(), "seed {seed}: the first difference at entry {first:?} of {}", want.len());
            let (c, _) = gs.device_counts().expect("the device's counts");
            records += c.plan.records;
            stretches += c.plan.stretches;
            fallbacks += c.plan.fallback.iter().sum::<u64>() + c.plan.snapshot[0];
        }
        assert!(records > 800 && stretches > 300, "records {records}, stretches {stretches}");
        assert!(fallbacks > 0, "some primitives were drawn by the processor or read a snapshot");
    }

    /// A host-to-local transfer cut by a V-sync and a drawing still reaches the device whole.
    #[test]
    fn a_transfer_across_a_vsync_and_a_drawing_reaches_the_device_whole() {
        let script = |gs: &mut Gs| {
            let sprite = |gs: &mut Gs, tex: bool, x0: u64, x1: u64| {
                gs.write(reg::PRIM, 6 | u64::from(tex) << 4 | 1 << 8);
                gs.write(reg::RGBAQ, 0x8080_8080);
                gs.write(reg::UV, 0);
                gs.write(reg::XYZ2, x0 * 16);
                gs.write(reg::UV, 64 * 16 | (64 * 16) << 16);
                gs.write(reg::XYZ2, x1 * 16 | (64 * 16) << 16);
            };
            gs.write(reg::FRAME_1, 64 | 8 << 16);
            gs.write(reg::SCISSOR_1, 511 << 16 | 63 << 48);
            gs.write(reg::TEST_1, 0);
            gs.write(reg::PRMODECONT, 1);
            gs.write(reg::TEX0_1, 64 * 32 | 1 << 14 | 6 << 26 | 6 << 30 | 1 << 35);
            gs.write(reg::BITBLTBUF, (64 * 32) << 32 | 1 << 48);
            gs.write(reg::TRXPOS, 0);
            gs.write(reg::TRXREG, 64 | 64 << 32);
            gs.write(reg::TRXDIR, 0);
            let unit = |k: u64| k.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (k << 17);
            for draw_between in [false, true] {
                gs.write(reg::TRXDIR, 0);
                (0..1024).for_each(|k| gs.image(unit(k + u64::from(draw_between))));
                gs.vsync(false);
                if draw_between {
                    sprite(gs, false, 128, 192);
                }
                (1024..2048).for_each(|k| gs.image(unit(k + u64::from(draw_between))));
                sprite(gs, true, 192 + 128 * u64::from(draw_between), 256 + 128 * u64::from(draw_between));
            }
            gs.local().words().to_vec()
        };
        assert!(script(&mut software()) == script(&mut device()), "the memories differ");
    }

    /// A sprite drawn, then a sprite in other tiles textured from it: the second must see the first's pixels.
    #[test]
    fn a_record_reading_what_an_earlier_one_drew_in_other_tiles_sees_it() {
        let script = |gs: &mut Gs| {
            gs.write(reg::FRAME_1, 64 | 8 << 16);
            gs.write(reg::SCISSOR_1, 511 << 16 | 127 << 48);
            gs.write(reg::TEST_1, 0);
            gs.write(reg::PRMODECONT, 1);
            for k in 0..4u64 {
                for j in 0..64u64 {
                    gs.write(reg::PRIM, 6 | 1 << 3);
                    gs.write(reg::RGBAQ, 0x8000_0000 | (0x10 + 0x30 * k + j) * 0x0001_0101);
                    gs.write(reg::XYZ2, 0);
                    gs.write(reg::XYZ2, (128 * 16) | (128 * 16) << 16);
                }
                gs.write(reg::TEX0_1, (64 * 32) | 8 << 14 | 7 << 26 | 7 << 30 | 1 << 35);
                gs.write(reg::PRIM, 6 | 1 << 4 | 1 << 8);
                gs.write(reg::UV, 0);
                gs.write(reg::XYZ2, ((256 + 64 * k) * 16) | 0);
                gs.write(reg::UV, (64 * 16) | (128 * 16) << 16);
                gs.write(reg::XYZ2, ((320 + 64 * k) * 16) | (128 * 16) << 16);
            }
            gs.local().words().to_vec()
        };
        assert!(script(&mut software()) == script(&mut device()), "the memories differ");
    }

    /// Perspective-textured triangles with single-precision coordinates (`Sapphire_Gs.md §10.2`).
    #[test]
    fn single_precision_coordinates_leave_nearly_every_pixel_as_the_reference() {
        let script = |gs: &mut Gs| {
            gs.write(reg::FRAME_1, 64 | 8 << 16);
            gs.write(reg::SCISSOR_1, 511 << 16 | 255 << 48);
            gs.write(reg::TEST_1, 0);
            gs.write(reg::PRMODECONT, 1);
            gs.write(reg::BITBLTBUF, (8 * 32) << 32 | 2 << 48);
            gs.write(reg::TRXPOS, 0);
            gs.write(reg::TRXREG, 128 | 128 << 32);
            gs.write(reg::TRXDIR, 0);
            (0..8192u64).for_each(|k| gs.image(k.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ k << 29));
            gs.write(reg::TEX0_1, (8 * 32) | 2 << 14 | 7 << 26 | 7 << 30 | 1 << 35);
            gs.write(reg::PRIM, 3 | 1 << 4);
            let vertex = |gs: &mut Gs, x: u64, y: u64, s: f32, t: f32, q: f32| {
                gs.write(reg::ST, u64::from((s * q).to_bits()) | u64::from((t * q).to_bits()) << 32);
                gs.write(reg::RGBAQ, 0x8080_8080 | u64::from(q.to_bits()) << 32);
                gs.write(reg::XYZ2, x * 16 | (y * 16) << 16);
            };
            for k in 0..6u64 {
                let (x0, q0, q1) = (k * 80, 1.0 + k as f32 * 0.37, 0.3 + k as f32 * 0.21);
                vertex(gs, x0, 0, 0.0, 0.0, q0);
                vertex(gs, x0 + 79, 10, 0.97, 0.05, q1);
                vertex(gs, x0 + 7, 250, 0.11, 0.99, 1.5);
            }
            gs.local().words().to_vec()
        };
        let want = script(&mut software());
        let mut gs = device();
        gs.set_exact(false);
        let got = script(&mut gs);
        let differing = want.iter().zip(&got).filter(|(a, b)| a != b).count();
        assert!(differing < want.len() / 2000, "{differing} words differ");
    }

    #[test]
    fn a_state_saved_mid_stream_loads_into_the_device_and_continues_as_the_software_rasteriser_does() {
        for seed in 20..23u64 {
            let steps = stream(seed, 600);
            let (a, b) = steps.split_at(300);
            let mut one = software();
            let mut want = run(&mut one, a);
            want.extend(run(&mut one, b));
            let mut gs = device();
            let mut got = run(&mut gs, a);
            let bytes = save(&gs);
            let mut loaded = device();
            loaded.read_state(&mut StateReader::new(&bytes)).expect("the state loads");
            got.extend(run(&mut loaded, b));
            assert_eq!(want, got, "seed {seed}");
        }
    }

    #[test]
    fn with_snapshots_two_runs_leave_the_same_memory_and_read_backs() {
        let mut snapshots = 0;
        for seed in 0..6u64 {
            let steps = stream(seed, 700);
            let runs: Vec<(Vec<u64>, u64)> = (0..2)
                .map(|_| {
                    let mut gs = device();
                    gs.set_exact(false);
                    let got = run(&mut gs, &steps);
                    (got, gs.device_counts().expect("counts").0.plan.snapshot[0])
                })
                .collect();
            assert_eq!(runs[0], runs[1], "seed {seed}");
            snapshots += runs[0].1;
        }
        assert!(snapshots > 0, "some records read a snapshot");
    }
}
