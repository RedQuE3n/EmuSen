//! The project's own test programs in `programs/`, assembled with `emusen-mips` and run with no kernel: each one's
//! results as Diamond reads the documents, and as the witnesses of Diamond_Native.md §11.13 and the referees of
//! Diamond_Referees.md were seen to answer, or as a console's log shows.

use diamond::exe;
use diamond::kernel::{self, Kernel};
use diamond::machine::{Config, Gpu, Machine};
use diamond::media::{Market, Media};

fn run(source: &str) -> Machine {
    run_with(source, 10, Market::Americas, 0)
}

fn run_for(source: &str, frames: usize, market: Market) -> Machine {
    run_with(source, frames, market, 0)
}

/// The program run with port 1's pad holding `pressed`, the report's bits.
fn run_pressing(source: &str, pressed: u16) -> Machine {
    run_with(source, 10, Market::Americas, pressed)
}

fn run_with(source: &str, frames: usize, market: Market, pressed: u16) -> Machine {
    run_set_up(source, frames, market, |m| m.board.sio0.set_buttons(0, pressed as u32))
}

/// The program run on a machine `set_up` plugs or presses things into first.
fn run_set_up(source: &str, frames: usize, market: Market, set_up: impl FnOnce(&mut Machine)) -> Machine {
    let words = emusen_mips::assemble(source, 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).expect("assembles");
    let image = exe::build(&words);
    let media = Media::read(&image).unwrap();
    let mut m = Machine::new(media, Config { market, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None], &image, None);
    set_up(&mut m);
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

/// Diamond_Pad.md §4: port 1's pad answers 41h, 5Ah and its switches with Start and Cross pressed, pulling /ACK after
/// every byte but the last; port 2 and the card slot answer FFh with no /ACK, as empty.
#[test]
fn the_pad_answers_on_port_1_and_nothing_answers_elsewhere() {
    let m = run_pressing(include_str!("../programs/pad.s"), 1 << 3 | 1 << 14);
    let bytes: Vec<u8> = (0..30).map(|i| m.board.ram[0x10_0000 + i]).collect();
    let empty = [0xFF, 0, 0xFF, 0, 0xFF, 0, 0xFF, 0, 0xFF, 0];
    assert_eq!(bytes[..10], [0xFF, 1, 0x41, 1, 0x5A, 1, 0xF7, 1, 0xBF, 0], "port 1");
    assert_eq!((bytes[10..20].to_vec(), bytes[20..].to_vec()), (empty.to_vec(), empty.to_vec()), "port 2, and the card slot");
}

/// The GTE's interlock as BlackMoon_GTE.md D-13 reads psx-spx: a command's cycles from its fetch, so an access at once
/// waits one fewer; every access to coprocessor 2 waits, writes included; 00h as RTPS, the other unlisted numbers free.
#[test]
fn the_gte_holds_every_access_until_its_command_is_done() {
    let m = run(include_str!("../programs/gte-timing.s"));
    let commands = [0x01, 0x06, 0x0C, 0x10, 0x11, 0x12, 0x13, 0x14, 0x16, 0x1B, 0x1C, 0x1E, 0x20, 0x28, 0x29, 0x2A, 0x2D, 0x2E, 0x30, 0x3D, 0x3E, 0x3F];
    assert_eq!(results(&m, 0x10_0000, 22), commands.map(|c| blackmoon_gte::CYCLES[c] as u32 - 1));
    assert_eq!(results(&m, 0x10_0100, 21), (0..21).map(|n| 14u32.saturating_sub(n)).collect::<Vec<_>>());
    assert_eq!(results(&m, 0x10_0200, 7), [14; 7]);
    assert_eq!(results(&m, 0x10_0280, 6), [14, 0, 0, 0, 0, 0]);
    assert_eq!(results(&m, 0x10_0300, 8), [0xF80; 8], "IRGB reaches IR3 and IR1 at once (D-15)");
    assert_eq!(results(&m, 0x10_0320, 4), [15; 4], "and LZCS LZCR");
    assert_eq!(results(&m, 0x10_0330, 6), [0x100; 6], "and MTC2 and LWC2 the next command");
}

/// The fuzz program's 904 cases, written and read back through the processor, are what the engine computes alone: the
/// boundary of BlackMoon_GTE.md §2 moves every register by its number, through both delay slots.
#[test]
fn the_fuzz_program_reads_back_what_the_engine_computes() {
    let m = run(include_str!("../programs/gte-fuzz.s"));
    let n = m.board.ram_word(0xFFFF4);
    assert_eq!(n, 22 * 40 + 6 * 4);
    for i in 0..n {
        let at = 0x10_0000 + 516 * i;
        let case = results(&m, at, 129);
        let mut g = blackmoon_gte::Gte::new();
        for (r, &v) in case[1..65].iter().enumerate() {
            if r < 32 { g.set_data(r as u8, v) } else { g.set_control(r as u8 - 32, v) }
        }
        g.command(case[0]);
        assert_eq!(&case[65..], &(0..64).map(|r| g.register(r)).collect::<Vec<_>>()[..], "case {i}, command {:07X}", case[0]);
    }
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

/// D-10: GPUSTAT's bits 26 and 28 while a polygon, a line and a rectangle arrive and draw: bit 28 clear from a
/// polygon's or a line's command word to its last vertex, as Beetle PSX answers and amidog's expectations have it;
/// DuckStation and SwanStation keep it set (Diamond_Referees.md §4.8, Diamond_Disputes.md D-23).
#[test]
fn gpustat_while_each_shape_arrives_and_draws() {
    let m = run(include_str!("../programs/gpustat-shapes.s"));
    let bits: Vec<u32> = results(&m, 0x10_0000, 20).iter().map(|s| s >> 26 & 5).collect();
    assert_eq!(bits, [5, 0, 0, 0, 4, 4, 5, 0, 0, 4, 4, 5, 4, 4, 4, 4, 5, 0, 0, 5]);
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

fn bytes(m: &Machine, at: usize, n: usize) -> Vec<u8> {
    m.board.ram[at..at + n].to_vec()
}

/// Diamond_Pad.md §8: a sector written and read back through the protocol, the IDs, the flag's bit 3 cleared by the
/// write, a bad checksum refused, and /ACK after every byte but each command's last. With `DIAMOND_CARD_OUT` set, the
/// card the program wrote is kept there as its raw 128 KiB.
#[test]
fn a_card_is_written_and_read_through_the_protocol() {
    let m = run_set_up(include_str!("../programs/card.s"), 30, Market::Americas, |m| m.insert_card(0, None));
    let data: Vec<u8> = (0..128u32).map(|i| (3 * i) as u8).collect();
    let checksum = data.iter().fold(1u8, |c, &d| c ^ d);
    let write = bytes(&m, 0x10_0000, 138);
    assert_eq!((write[..6].to_vec(), write[134..].to_vec()), (vec![0xFF, 0x08, 0x5A, 0x5D, 0x00, 0x01], vec![data[127], 0x5C, 0x5D, 0x47]));
    let read = bytes(&m, 0x10_0100, 140);
    assert_eq!(read[..10], [0xFF, 0x00, 0x5A, 0x5D, 0x00, 0x01, 0x5C, 0x5D, 0x01, 0x00], "the flag cleared by the write");
    assert_eq!((read[10..138].to_vec(), read[138], read[139]), (data.clone(), checksum, 0x47));
    assert_eq!(bytes(&m, 0x10_0200, 10), [0xFF, 0x00, 0x5A, 0x5D, 0x5C, 0x5D, 0x04, 0x00, 0x00, 0x80]);
    assert_eq!(bytes(&m, 0x10_0300 + 137, 1), [0x4E], "a bad checksum");
    let acks = |at: usize, n: usize| bytes(&m, at + 0x400, n);
    assert!(acks(0x10_0000, 137).iter().all(|&a| a == 1) && acks(0x10_0000 + 137, 1) == [0]);
    assert!(acks(0x10_0100, 139).iter().all(|&a| a == 1), "the read's late /ACK within the timeout");
    let card = m.card_in(0).unwrap();
    assert_eq!((&card.data[0x100 * 128..0x101 * 128], card.changed), (&data[..], true));
    assert!(card.data[0x101 * 128..0x102 * 128].iter().all(|&b| b == 0), "the refused write stored nothing");
    if let Ok(path) = std::env::var("DIAMOND_CARD_OUT") {
        std::fs::write(path, &card.data).unwrap();
    }
}

/// Diamond_Pad.md §9: the multitap's long read after a request, slot C by its address, and slot A's card.
#[test]
fn the_multitap_answers_all_four_slots() {
    let m = run_set_up(include_str!("../programs/multitap.s"), 10, Market::Americas, |m| {
        m.insert_card(0, None);
        m.plug_multitap(0);
        m.board.sio0.set_buttons(3, 1 << 3);
    });
    assert_eq!(bytes(&m, 0x10_0000, 5), [0xFF, 0x41, 0x5A, 0xFF, 0xFF], "slot A, the request for next time");
    let long = bytes(&m, 0x10_0040, 35);
    assert_eq!(long[..3], [0xFF, 0x80, 0x5A]);
    let slot = |n: usize| long[3 + 8 * n..11 + 8 * n].to_vec();
    assert_eq!(slot(0), [0x41, 0x5A, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]);
    assert_eq!(slot(2), [0x41, 0x5A, 0xF7, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF], "port 1's slot C, host port 3, holds Start");
    assert_eq!(bytes(&m, 0x10_0080, 5), [0xFF, 0x41, 0x5A, 0xF7, 0xFF]);
    assert_eq!(bytes(&m, 0x10_00C0, 10), [0xFF, 0x08, 0x5A, 0x5D, 0x5C, 0x5D, 0x04, 0x00, 0x00, 0x80]);
}

/// The plan's stage 6 gate: the open kernel's pad functions read the pads over SIO0 on a program, a digital read on
/// port 1 and, once switched, the DualShock's analog read; port 2, empty, reads FFh.
#[test]
fn the_open_kernels_pad_functions_read_the_pads() {
    let words = emusen_mips::assemble(include_str!("../programs/kernel-pad.s"), 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).expect("assembles");
    let image = exe::build(&words);
    let media = Media::read(&image).unwrap();
    let rom = kernel::open_kernel().map(|k| (k, Kernel::Open));
    let mut m = Machine::new(media, Config { market: Market::Americas, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None], &image, rom);
    m.board.sio0.set_buttons(0, 1 << 3 | 1 << 14);
    for _ in 0..20 {
        m.advance();
    }
    assert_eq!(m.board.ram_word(0xFFFF0), 0xD1A0_D00E, "the program started the pads");
    assert_eq!((bytes(&m, 0x10_0000, 4), bytes(&m, 0x10_0040, 1)), (vec![0x00, 0x41, 0xF7, 0xBF], vec![0xFF]));
    m.board.sio0.set_buttons(0, 1 << 16);
    m.board.sio0.controller(0).unwrap().set_axis(0, 1.0);
    for _ in 0..3 {
        m.advance();
    }
    assert_eq!(bytes(&m, 0x10_0000, 8), [0x00, 0x73, 0xFF, 0xFF, 0x80, 0x80, 0xFF, 0x80], "the analog read, the left stick right");
}

/// D-11: the cycles from a primitive's last word to GPUSTAT bit 26, with a rectangle's and a fill's time charged by
/// the rows they draw; the referees read a 1x1 rectangle ready 10 to 16 cycles after its word (Diamond_GPU.md §8).
#[test]
fn small_rectangles_and_fills_cost_their_rows() {
    let m = run(include_str!("../programs/draw-time.s"));
    let busy: Vec<u32> = results(&m, 0x10_0000, 38).iter().map(|w| (w >> 16) - (w & 0xFFFF)).collect();
    assert_eq!(busy[..4], [7, 7, 7, 7], "the four settings");
    assert_eq!(busy[4..18], [7, 7, 7, 25, 70, 7, 25, 79, 322, 52, 610, 2284, 4444, 7], "flat rectangles 1x1 to 128x64, then GP0(68h)");
    assert_eq!(busy[18..26], [7, 7, 25, 106, 403, 7, 25, 448], "fills");
    assert_eq!(busy[26..30], [556, 574, 835, 4912], "triangles keep their setup");
}
