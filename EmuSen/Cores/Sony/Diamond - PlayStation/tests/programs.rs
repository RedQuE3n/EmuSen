//! The project's own test programs in `programs/`, assembled with `emusen-mips` and run with no kernel: each one's
//! results as Diamond reads the documents, and as the witnesses of Diamond_Native.md §11.13 and the referees of
//! Diamond_Referees.md were seen to answer, or as a console's log shows.

use diamond::exe;
use diamond::machine::{Config, Gpu, Machine};
use diamond::media::{Market, Media};

fn run(source: &str) -> Machine {
    run_for(source, 10, Market::Americas)
}

fn run_for(source: &str, frames: usize, market: Market) -> Machine {
    let words = emusen_mips::assemble(source, 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).expect("assembles");
    let image = exe::build(&words);
    let media = Media::read(&image).unwrap();
    let mut m = Machine::new(media, Config { market, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None], &image, None);
    for _ in 0..frames {
        m.advance();
    }
    assert_eq!(m.board.ram_word(0xFFFF0), 0xD1A0_D00E, "the program finished");
    m
}

fn results(m: &Machine, at: u32, n: u32) -> Vec<u32> {
    (0..n).map(|i| m.board.ram_word(at + 4 * i)).collect()
}

#[test]
fn every_exception_writes_ce_from_the_opcodes_low_bits() {
    let m = run(include_str!("../programs/exceptions.s"));
    assert_eq!(m.board.ram_word(0xFFFF4), 12);
    assert_eq!(
        results(&m, 0x10_0000, 12),
        [0x20, 0x24, 0x30, 0x3000_0010, 0x1000_0010, 0x1000_0014, 0x3000_0014, 0x3000_0028, 0x3000_002C, 0x2000_002C, 0x2000_002C, 0x30]
    );
}

#[test]
fn a_second_load_cancels_the_first() {
    let m = run(include_str!("../programs/loads.s"));
    assert_eq!(results(&m, 0x10_0000, 8), [0x55, 0x2222_2222, 0x77, 0x2222_2222, 0x3333_0000, 0, 0, 0x1111_1111]);
}

#[test]
fn the_garbage_registers_read_zero() {
    let m = run(include_str!("../programs/cop0-garbage.s"));
    assert_eq!(results(&m, 0x10_0000, 18), [0; 18]);
}

/// D-2 as Diamond reads psx-spx's table: 6, 9 and 13 cycles by rs, 36 for a division, MFLO free after five more.
#[test]
fn the_multiplier_takes_psx_spxs_cycles() {
    let m = run(include_str!("../programs/multiply.s"));
    assert_eq!(results(&m, 0x10_0000, 22), [6, 6, 9, 9, 13, 13, 6, 9, 6, 9, 9, 13, 9, 13, 13, 6, 6, 36, 36, 36, 36, 36]);
    assert_eq!(results(&m, 0x10_0100, 16), [6, 5, 4, 3, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
}

/// GPUSTAT's bits 25-30 as ps1-tests' `gpu/gpustat` console log passes them (Diamond_GPU.md §2.3): ready for a command
/// and the FIFO empty while a rectangle drawn by the CPU is busy; nothing ready while a DMA's second rectangle waits.
#[test]
fn gpustat_reads_as_the_console_passes() {
    let m = run(include_str!("../programs/gpustat.s"));
    let bits: Vec<u32> = results(&m, 0x10_0000, 12).iter().map(|s| s & 0x7E00_0000).collect();
    assert_eq!(bits, [0x1400_0000, 0x1000_0000, 0x1400_0000, 0x5600_0000, 0x4000_0000, 0x5600_0000, 0x3600_0000, 0x2200_0000, 0x3600_0000, 0x5600_0000, 0x4000_0000, 0x5600_0000]);
}

/// D-0 as Beetle PSX times it to within 2 cycles a field: per GP1(08h) mode, the mean CPU cycles of 40 fields and the
/// hblanks of two in a row, on an NTSC console and on a PAL one (Diamond_Referees.md §4.1).
#[test]
fn the_video_timing_is_the_referees_line_on_both_clocks() {
    for (market, expected) in [
        (Market::Americas, [(566_121, [263, 263]), (674_415, [314, 314]), (565_045, [262, 263]), (671_194, [313, 312]), (565_045, [262, 263]), (671_194, [313, 312])]),
        (Market::Europe, [(571_332, [263, 263]), (680_624, [314, 314]), (570_246, [262, 263]), (677_372, [313, 312]), (570_246, [262, 263]), (677_372, [313, 312])]),
    ] {
        let m = run_for(include_str!("../programs/video-timing.s"), 400, market);
        for (k, (cycles, lines)) in expected.into_iter().enumerate() {
            let stamps = results(&m, 0x10_0000 + 0x200 * k as u32, 41);
            let hblanks = results(&m, 0x10_0100 + 0x200 * k as u32, 3);
            let mean = (stamps[40] - stamps[0]) as f64 / 40.0;
            assert!((mean - cycles as f64).abs() < 2.0, "{market:?} mode {k}: {mean}");
            assert_eq!([(hblanks[1] - hblanks[0]) & 0xFFFF, (hblanks[2] - hblanks[1]) & 0xFFFF], lines, "{market:?} mode {k}");
        }
    }
}

/// D-7: the read-back of `raster-order`, whose sixteen shapes DuckStation, Beetle PSX, SwanStation and PCSX-ReARMed
/// draw identically, by its FNV-1a hash over the words (Diamond_Referees.md §4.6).
#[test]
fn shaded_triangles_draw_in_every_vertex_order_as_the_referees_do() {
    let m = run_for(include_str!("../programs/raster-order.s"), 60, Market::Americas);
    assert_eq!(m.board.ram_word(0xFFFF0), 0xD1A0_D00E);
    let hash = results(&m, 0x10_0000, 18432).iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &w| (h ^ w as u64).wrapping_mul(0x0000_0100_0000_01B3));
    assert_eq!(hash, 0xDD74_3D10_26AB_B444);
}

/// D-8: an x-flipped rectangle starts from U with its low bit set, at even and odd columns alike, as Beetle PSX, the
/// one referee that flips, draws it; y-flipped ones start at V (Diamond_Referees.md §4.7).
#[test]
fn flipped_rectangles_start_where_the_referee_starts_them() {
    let m = run_for(include_str!("../programs/xflip.s"), 60, Market::Americas);
    let texel = |x: u32, y: u32| (m.board.ram_word(0x10_0000 + (y * 192 + x) / 2 * 4) >> (16 * (x & 1))) & 0xFFFF;
    let first = |x: u32| [texel(x, 0), texel(x + 1, 0), texel(x, 1)];
    assert_eq!(first(16), [0x8001, 0x8000, 0x8101], "U 0, an even column");
    assert_eq!(first(33), [0x8001, 0x8000, 0x8101], "U 0, an odd column");
    assert_eq!(first(48), [0x8001, 0x8000, 0x8101], "U 1 starts at 1");
    assert_eq!(first(80), [0x800B, 0x800A, 0x810B], "U 10 starts at 11");
    assert_eq!(first(97), [0x800B, 0x800A, 0x810B], "U 11 starts at 11");
    assert_eq!(first(128), [0x8100, 0x8101, 0x8000], "y-flipped from V 1");
}

/// D-10: GPUSTAT's bits 26 and 28 while a polygon, a line and a rectangle arrive and draw, as Diamond reads them and
/// DuckStation and SwanStation answer; Beetle PSX clears bit 28 between a polygon's or a line's command word and its
/// last vertex (Diamond_Referees.md §4.8).
#[test]
fn gpustat_while_each_shape_arrives_and_draws() {
    let m = run(include_str!("../programs/gpustat-shapes.s"));
    let bits: Vec<u32> = results(&m, 0x10_0000, 20).iter().map(|s| s >> 26 & 5).collect();
    assert_eq!(bits, [5, 4, 4, 4, 4, 4, 5, 4, 4, 4, 4, 5, 4, 4, 4, 4, 5, 0, 0, 5]);
}

/// D-11: after 2,048 words to VRAM in blocks of 1 to 128 and in a burst, the GPU reads ready at the first poll, on
/// Diamond and on every referee; the console's `chopping` log reads it busy after the 128-word blocks and the burst.
#[test]
fn the_gpu_is_ready_at_once_after_each_transfer() {
    let m = run(include_str!("../programs/gpu-busy.s"));
    let r = results(&m, 0x10_0000, 36);
    for case in 0..9 {
        let (end, b28, b26) = (r[4 * case], r[4 * case + 1], r[4 * case + 2]);
        assert!(b28 - end < 40 && b26 - end < 40, "case {case}: {end} {b28} {b26}");
    }
}

/// D-5: a RAM store drains in 8 cycles and a load waits for the queue, as Diamond fitted them; no referee models the
/// queue (Diamond_Referees.md §4.4).
#[test]
fn stores_drain_as_the_fit_has_them() {
    let m = run(include_str!("../programs/stores.s"));
    assert_eq!(results(&m, 0x10_0000, 8), [8, 16, 24, 32, 40, 48, 56, 64]);
    assert_eq!(results(&m, 0x10_0020, 8), [13, 21, 29, 37, 45, 53, 61, 69]);
    assert_eq!(results(&m, 0x10_0040, 8), [9, 16, 24, 32, 40, 48, 56, 64]);
}
