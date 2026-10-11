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
    assert_eq!(m.board.ram_word(0xFFFF0), 0xD1A0_D00E, "the program finished: {:02X?} pc {:08X} {:?}", &m.board.ram[0x10_0000..0x10_0020], m.cpu.regs.pc, m.board.cdrom.activity);
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

// The RAM-recording rewrites of ps1-tests' TTY programs (Diamond_Records.md), each record as Diamond answers it. Seven
// carry the release's code from the corpus and pass unrun without it.

#[path = "support/records.rs"]
mod records;

/// A rewrite with the release's code carried in, or none without the corpus.
fn carried(name: &str) -> Option<String> {
    let Some(corpus) = records::corpus() else {
        eprintln!("{name}: not run, {} names no ps1-tests build 158", records::CORPUS_VARIABLE);
        return None;
    };
    Some(records::source(records::named(name), &corpus).expect("the release's code"))
}

fn fnv(words: &[u32]) -> u64 {
    words.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, &w| (h ^ w as u64).wrapping_mul(0x0000_0100_0000_01B3))
}

/// `timers` on a PAL console, as its log's: the system clock's delay rows sample for sample as the release prints them
/// on Diamond's open kernel, a frame of 263 hblanks, and the target and flag checks (Diamond_Records.md §2).
#[test]
fn the_timers_record_reads_as_the_release_prints() {
    let m = run_for(include_str!("../programs/timers.s"), 120, Market::Europe);
    let row = |k: u32| results(&m, 0x10_0000 + 40 * k, 10);
    for k in [0, 6, 12] {
        assert_eq!((row(k), row(k + 1)), (vec![1013; 10], vec![5013; 10]), "timer {}", k / 6);
    }
    assert_eq!(row(3), [198, 199, 198, 199, 198, 199, 198, 199, 199, 198], "the dot clock, 1000 cycles");
    assert_eq!(row(11), [263; 10], "a frame's hblanks");
    assert_eq!(row(16), [627, 626, 627, 627, 627, 626, 627, 627, 627, 626], "the system clock over 8, 5000 cycles");
    for k in [2, 8, 14] {
        assert!(row(k)[3..].iter().all(|&v| v.abs_diff(112_557) <= 1), "a frame on the system clock, settled: {:?}", row(k));
    }
    assert_eq!(results(&m, 0x10_0D00, 17), [1666, 833, 833, 833, 834, 835, 834, 833, 833, 833, 833, 0, 0xC08, 1, 0, 1, 1]);
    assert_eq!(results(&m, 0x10_0D80, 6), [0x400, 0x7FF, 0x1C08, 0x408, 0x408, 0x408]);
    assert_eq!(fnv(&results(&m, 0x10_0000, 870)), 0x6024_1295_3685_3C8B, "the whole record");
}

/// `cpu/access-time`: the 51 cells, whole cycles and hundredths, equal to what the release prints on Diamond (§3);
/// D-4's rule meets the console's log within 0.2 cycles on 49 of them.
#[test]
fn the_access_time_record_reads_as_the_release_prints() {
    let Some(source) = carried("access-time") else { return };
    let m = run_for(&source, 40, Market::Americas);
    let cells: Vec<(u32, u32)> = results(&m, 0x10_0000, 102).chunks(2).map(|c| (c[0], c[1])).collect();
    assert_eq!(cells[..3], [(5, 1), (5, 1), (5, 8)], "RAM");
    assert_eq!(cells[12..15], [(11, 7), (26, 0), (56, 0)], "EXPANSION2");
    assert_eq!(cells[45..48], [(18, 7), (18, 0), (36, 7)], "SPUCNT, its 32-bit load an address error's round trip");
    assert_eq!(fnv(&results(&m, 0x10_0000, 102)), 0xE027_8218_730C_657E, "every cell");
}

/// `dma/chopping`: 130 transfers of 2,048 words to VRAM, their CPU cycles, and the GPU busy exactly where a block size
/// leaves words unsent (§4); each 14 cycles short of the release's where its kernel's call leaves the window cold.
#[test]
fn the_chopping_record_times_each_transfer() {
    let Some(source) = carried("chopping") else { return };
    let m = run_for(&source, 200, Market::Americas);
    let rows: Vec<Vec<u32>> = results(&m, 0x10_0000, 780).chunks(6).map(|r| r.to_vec()).collect();
    assert_eq!(rows[0], [1, 8192, 1, 0, 24_607, 0], "blocks of one word");
    assert_eq!(rows[65], [0, 8192, 0, 0, 2201, 0], "the burst");
    assert_eq!(rows[129], [0, 8192, 128, 128, 4209, 0], "128-word windows, 128 cycles between");
    for (i, r) in rows[..65].iter().enumerate() {
        assert_eq!(r[5] == 1, 2048 % r[2] != 0, "block size {}", r[2]);
        assert!(i == 64 || r[2] == i as u32 + 1);
    }
    assert!(rows[65..].iter().all(|r| r[5] == 0));
    assert_eq!(fnv(&results(&m, 0x10_0000, 780)), 0xF809_2905_A397_7592, "every row");
}

/// `gpu/bandwidth`: the hblanks of 400 full-screen transfers or primitives a row, nine of the fourteen equal to the
/// release's on Diamond and the rest within two (§5), every row within 0.6% of the console's; since D-11's refit, each
/// row within one hblank of what it read before.
#[test]
fn the_bandwidth_record_counts_each_rows_hblanks() {
    let Some(source) = carried("bandwidth") else { return };
    let m = run_for(&source, 860, Market::Americas);
    let hblanks: Vec<u32> = results(&m, 0x10_0000, 56).chunks(4).map(|r| r[2]).collect();
    assert_eq!(hblanks, [15_773, 12_123, 19_127, 1216, 7603, 11_461, 15_134, 15_134, 7721, 11_580, 39_976, 39_975, 5844, 3968]);
    assert_eq!(fnv(&results(&m, 0x10_0000, 56)), 0xC54B_955B_DA4F_277C, "the milliseconds and MB/s as well");
}

/// `dma/chain-looping`: the RAM loop's ticks with no DMA, an empty chain and two never-ending ones, and their
/// booleans (§6); one ÷8 tick short of the release's on Diamond where the prescaler's phase differs, D-5's 6.3% under a chain.
#[test]
fn the_chain_looping_record_times_the_loop_beside_each_chain() {
    let Some(source) = carried("chain-looping") else { return };
    let m = run_for(&source, 40, Market::Americas);
    assert_eq!(results(&m, 0x10_0000, 12), [16_024, !0, !0, 16_008, 1, 1, 24_016, 0, 0, 24_016, 0, 0]);
}

/// A mode write that turns a timer's sync on resets the count (psx-spx "Timers"): each read a few cycles after it is
/// those cycles, or zero where the mode pauses or waits for its blank, as every referee reads it (Diamond_Disputes.md
/// D-6). Timer 0 had read up to 36,658, credited with the time since the last video edge processed (Diamond_Records.md §8.1).
#[test]
fn a_mode_write_with_sync_resets_the_count() {
    let m = run_for(include_str!("../programs/timer-sync.s"), 60, Market::Americas);
    let free = [3, 6, 3, 6, 3, 6, 3, 6];
    assert_eq!(results(&m, 0x10_0000, 32), [[0, 0, 3, 6, 3, 6, 3, 6], free, [3, 6, 0, 0, 0, 0, 0, 0], [0; 8]].concat(), "timer 0, hblank");
    assert_eq!(results(&m, 0x10_0080, 32), [[0, 0, 0, 0, 3, 6, 3, 6], free, [3, 6, 3, 6, 0, 0, 0, 0], [0; 8]].concat(), "timer 1, vblank");
}

/// The console log's largest first sample and largest rise in each of `timers`' fourteen sync tables, in its order.
const SYNC_LOG: [(i32, i32); 14] =
    [(260, 123), (245, 123), (242, 123), (242, 123), (242, 123), (38, 20), (48, 25), (54, 28), (76, 39), (95, 49), (248, 123), (0, 1), (248, 123), (247, 123)];

/// `timers`' sync tables in the console log's shape, the rule of WiseMan's TTY gate (Diamond_Native.md §11.12): each row
/// starts within 1.2 times the log's largest first sample and 2, rises within 1.1 times its largest rise and 2, and
/// modes 0 and 3 never fall. Before the fix 8 of the 14 tables failed it, mode 0 at 512 pixels starting at 1,629.
#[test]
fn the_timers_records_sync_tables_have_the_logs_shape() {
    let m = run_for(include_str!("../programs/timers.s"), 120, Market::Europe);
    for (t, &(first, rise)) in SYNC_LOG.iter().enumerate() {
        let at = if t < 10 { 0x10_0300 + 160 * t as u32 } else { 0x10_0A00 + 160 * (t as u32 - 10) };
        for s in 0..4 {
            let row: Vec<i32> = results(&m, at + 40 * s, 10).into_iter().map(|w| w as i32).collect();
            assert!(row[0] as f64 <= 1.2 * first as f64 + 2.0, "table {t}, mode {s} starts at {}: {row:?}", row[0]);
            assert!(row.windows(2).all(|w| (w[1] - w[0]) as f64 <= 1.1 * rise as f64 + 2.0), "table {t}, mode {s} rises too fast: {row:?}");
            assert!(s % 3 != 0 || row.windows(2).all(|w| w[1] >= w[0]), "table {t}, mode {s} falls: {row:?}");
        }
    }
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

// The printf records of ps1-tests' value programs (Diamond_Records.md §11–§14): each call's format address and arguments.

/// A printf record's calls, six words each, up to the first with no format.
fn printf_calls(m: &Machine) -> Vec<Vec<u32>> {
    results(m, 0x10_0000, 3072).chunks(6).take_while(|c| c[0] != 0).map(|c| c.to_vec()).collect()
}

/// `dma/otc-test`: its heading, "pass - %s" for each of the fifteen tests, and "Done.", as the release prints them on
/// Diamond's open kernel.
#[test]
fn the_otc_record_passes_every_test() {
    let Some(source) = carried("otc-test") else { return };
    let m = run_for(&source, 60, Market::Americas);
    let c = printf_calls(&m);
    let formats: Vec<u32> = c.iter().map(|c| c[0]).collect();
    assert_eq!(formats, [&[0x8001_39B0][..], &[0x8001_33E8; 15], &[0x8001_39A8]].concat());
    assert_eq!(fnv(&c.concat()), 0x45D2_B6B9_5431_9863, "the calls with their arguments");
}

/// `dma/dpcr`: the SPU's channel completes with DPCR's enable set and times out without it, whatever the priority;
/// DPCR read as ResetGraph and SpuInit leave it.
#[test]
fn the_dpcr_record_completes_only_with_the_channel_enabled() {
    let Some(source) = carried("dpcr") else { return };
    let m = run_for(&source, 60, Market::Americas);
    let c = printf_calls(&m);
    let dpcr: Vec<[u32; 2]> = c.iter().filter(|c| c[0] == 0x8001_1E0C).map(|c| [c[1], c[2]]).collect();
    assert_eq!(dpcr, [[0x3B3B_3B33, 0x3B3D_3B33], [0x3B3B_3B33, 0x3B35_3B33], [0x3B3B_3B33, 0x3B30_3B33]]);
    let ends: Vec<u32> = c.iter().map(|c| c[0]).filter(|&f| f == 0x8001_1EAC || f == 0x8001_1E90).collect();
    assert_eq!(ends, [0x8001_1EAC, 0x8001_1E90, 0x8001_1E90], "complete, then timed out twice");
    assert_eq!(c.iter().filter(|c| c[0] == 0x8001_1D64).count(), 2, "both tests pass");
    assert_eq!(fnv(&c.concat()), 0xCEB9_CCA0_7DCD_4D84, "the calls with their arguments");
}

/// `cpu/io-access-bitwidth`: 108 reads after 8-, 16- and 32-bit writes, the BIOS row's all the ROM's own word, a
/// 32-bit store faulting at JOY_CTRL, SIO_CTRL and SPUCNT; 85 cells as the console's log has them, DICR's bit 6 and
/// GPUSTAT's bit 31 among those that are not, its bit 26 set since a reset ends a fill (Diamond_Records.md §16.1).
#[test]
fn the_io_bitwidth_record_reads_each_register_after_each_width() {
    let Some(source) = carried("io-access-bitwidth") else { return };
    let m = run_for(&source, 60, Market::Americas);
    let c = printf_calls(&m);
    let cells: Vec<&Vec<u32>> = c.iter().filter(|c| matches!(c[0], 1 | 2 | 0x8001_56D8 | 0x8001_56E4)).collect();
    let marked = |f: u32| cells.iter().enumerate().filter(|(_, c)| c[0] == f).map(|(i, _)| i).collect::<Vec<_>>();
    assert_eq!(cells.len(), 108);
    assert_eq!(marked(1), [3, 4, 5, 56, 57, 91], "the BIOS row");
    assert_eq!(marked(0x8001_56D8), [29, 35, 53], "--CRASH--");
    assert_eq!(cells[21..24].iter().map(|c| c[1]).collect::<Vec<_>>(), [0x34_0078; 3], "DICR, the console's 340038h");
    assert_eq!(cells[45..48].iter().map(|c| c[1]).collect::<Vec<_>>(), [0x9480_2000; 3], "GPUSTAT after GP1(00h), ready as the release reads it");
    assert_eq!(fnv(&c.concat()), 0x1E15_044A_5E79_7C9B, "the calls with their arguments");
}

// The GPU's step of amidog's FIFO, INTERLACE and cache groups (Diamond_Disputes.md D-23).

/// GP1(00h) during a long fill ends it: GPUSTAT reads psx-spx's 14802000h at once and at the first poll, as
/// DuckStation, SwanStation, Beetle PSX and PCSX-Redux read it (bit 13 aside), where the GPU stayed busy 5,855 polls.
#[test]
fn a_reset_during_a_fill_leaves_the_gpu_ready() {
    let m = run(include_str!("../programs/gp1-reset.s"));
    let r = results(&m, 0x10_0000, 5);
    assert_eq!(r[0] >> 26 & 1, 0, "drawing before the reset");
    assert_eq!(r[1..], [0x1480_2000, 0x1480_2000, 1, 0x1480_2000]);
}

/// D-23: an 8-bit page cached 64 texels by 32 lines, and cleared by a page set at another X or between 4 bits and the
/// others but not between 8 and 15 bits, as amidog's CACHE BLOCK and CACHE TAG expect (`programs/texture-cache.s`).
#[test]
fn the_texture_cache_keeps_a_64_texel_row_and_clears_on_a_page_change() {
    let m = run_for(include_str!("../programs/texture-cache.s"), 30, Market::Americas);
    let ends = |line: u32| {
        let w = |k: u32| m.board.ram_word(0x10_0000 + ((line - 10) * 32 + k) * 4);
        (w(0) & 0xFF, w(31) >> 16 & 0xFF)
    };
    let rows: Vec<(u32, u32)> = (10..17).map(ends).collect();
    assert_eq!(rows, [(1, 1), (1, 1), (2, 2), (2, 2), (2, 2), (4, 4), (5, 5)]);
}

/// D-23: in 480-line interlace with drawing to the displayed field prohibited, a fill and a rectangle drawn in vblank
/// skip the lines of GPUSTAT bit 31's parity counted from the display's first line, as amidog's INTERLACE expects;
/// PCSX-ReARMed skips the same lines with the display at line 0, Beetle PSX the odd lines in both (`programs/interlace.s`).
#[test]
fn interlaced_drawing_in_vblank_skips_the_lines_from_the_display_start() {
    let m = run_for(include_str!("../programs/interlace.s"), 60, Market::Americas);
    let px = |line: u32, x: u32| (m.board.ram_word(0x10_0000 + (line * 40 + x / 2) * 4) >> (16 * (x & 1))) & 0xFFFF != 0;
    let drawn: Vec<[bool; 3]> = (0..12).map(|l| [px(l, 0), px(l, 32), px(l, 64)]).collect();
    let (t, f) = (true, false);
    assert_eq!(drawn[..4], [[f, f, t], [t, t, t], [f, f, t], [t, t, t]], "the display from line 0: even lines left");
    assert_eq!(drawn[8..], [[t, t, f], [f, f, f], [t, t, f], [f, f, f]], "from line 1: odd lines left");
}

/// D-23: GPUREAD after a transfer's last word repeats the transfer's first, as amidog's FIFO read test expects and
/// PCSX-ReARMed reads it; after a one-word transfer, the word it repeated before, on amidog's expectations alone
/// (`programs/gpuread-repeat.s`).
#[test]
fn gpuread_repeats_the_first_word_of_the_last_longer_transfer() {
    let m = run(include_str!("../programs/gpuread-repeat.s"));
    assert_eq!(results(&m, 0x10_0000, 10), [0x0202_0101, 0x0404_0303, 0x0606_0505, 0x0202_0101, 0x0202_0101, 0x0303_0202, 0x0202_0101, 0x0202_0101, 0x0202_0101, 0x0202_0101]);
}

// --- Stage 7: the CD-ROM controller (Diamond_CDROM.md §9) ---

/// A disc of 40 mode 2 form 1 sectors from 00:02:00 whose user data names each one, sector 16 holding "CD001".
fn cd_disc() -> emusen_disc::Disc {
    let mut bin = Vec::new();
    for i in 0..40u32 {
        let mut user: Vec<u8> = (0..2048u32).map(|k| (k * 7 + i) as u8).collect();
        if i == 16 {
            user[..6].copy_from_slice(b"\x01CD001");
        }
        let mut s = [0u8; 2352];
        emusen_disc::ecma130::mode2_form1(150 + i, [0, 0, emusen_disc::ecma130::submode::DATA, 0], &user, &mut s);
        bin.extend_from_slice(&s);
    }
    emusen_disc::Disc::from_image(bin, emusen_disc::IsoForm::Mode2Form1).unwrap()
}

/// The kernel's read sequence of psx-spx's ReadN chapter, polled: each response in order, GetID's licence for the
/// console's region, IRQ2 in I_STAT, and the sector moved by DMA channel 3 whole.
#[test]
fn a_sector_reads_by_the_kernels_sequence() {
    let m = run_set_up(include_str!("../programs/cd-read.s"), 120, Market::Americas, |m| m.insert_disc(cd_disc()));
    let r: Vec<u8> = (0..27).map(|i| m.board.ram[0x10_0000 + i]).collect();
    assert_eq!(&r[..3], &[3, 0x02, 1], "Nop's INT3 and IRQ2");
    assert_eq!(&r[3..5], &[3, 0x02], "GetID's first");
    assert_eq!(&r[5..14], &[2, 0x02, 0x00, 0x20, 0x00, b'S', b'C', b'E', b'A'], "GetID's second");
    assert_eq!(&r[14..20], &[3, 0x02, 3, 0x02, 3, 0x02], "Setloc, Setmode, ReadN");
    assert_eq!(&r[20..22], &[1, 0x22], "INT1 while reading");
    assert_eq!(r[22], 0x00, "the DMA finished, start and trigger clear");
    assert_eq!(&r[23..27], &[3, 0x22, 2, 0x02], "Pause's two");
    assert_eq!(&m.board.ram[0x11_0000..0x11_0006], b"\x01CD001");
    let want: Vec<u8> = (6..2048u32).map(|k| (k * 7 + 16) as u8).collect();
    assert_eq!(&m.board.ram[0x11_0006..0x11_0800], &want[..]);
}

/// The opt-in disc tests' inputs: ps1-tests' release, a folder of the player's own BIOS and one of the tester's discs
/// copied to the cache (a cue sheet with its tracks beside it, or one image), or a message naming what is missing.
fn disc_inputs() -> Option<(std::path::PathBuf, Vec<u8>, emusen_disc::Disc)> {
    let (Some(corpus), Some(firmware), Some(disc)) = (std::env::var_os("EMUSEN_DIAMOND_PS1TESTS"), std::env::var_os("EMUSEN_DIAMOND_FIRMWARE"), std::env::var_os("EMUSEN_DIAMOND_DISC")) else {
        eprintln!("skipped: EMUSEN_DIAMOND_PS1TESTS, EMUSEN_DIAMOND_FIRMWARE and EMUSEN_DIAMOND_DISC name ps1-tests' release, the player's BIOS folder and a disc");
        return None;
    };
    let bios = std::fs::read(std::path::Path::new(&firmware).join("scph5501.bin")).expect("the player's scph5501.bin");
    let path = std::path::PathBuf::from(&disc);
    let image = std::fs::read(&path).expect("the disc");
    let mut files = Vec::new();
    if diamond::medium::is_sheet(&image) {
        for name in emusen_disc::set::companions(&String::from_utf8_lossy(&image)).expect("a sheet") {
            files.push(std::fs::read(path.parent().unwrap().join(name)).expect("a track file"));
        }
    }
    let refs: Vec<&[u8]> = files.iter().map(|f| f.as_slice()).collect();
    let Ok(diamond::medium::Opened::Disc(d, _)) = diamond::medium::open(&image, &refs) else { panic!("the disc opens") };
    Some((std::path::PathBuf::from(corpus), bios, *d))
}

/// A program run on the player's BIOS with the disc in the drive, the lid worked by `lid` each frame; the TTY from
/// `heading` on, the BIOS's own messages before the program left out.
fn run_on_disc(program: &[u8], bios: &[u8], disc: emusen_disc::Disc, frames: usize, heading: &str, mut lid: impl FnMut(&mut Machine, &str)) -> String {
    let media = Media::read(program).unwrap();
    let mut m = Machine::new(media, Config { market: Market::Americas, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None], program, Some((bios, Kernel::Player)));
    m.insert_disc(disc);
    let text = |m: &Machine| {
        let t = m.space(diamond::machine::TTY_ID).unwrap();
        let s = String::from_utf8_lossy(&t[..t.iter().position(|&b| b == 0).unwrap_or(t.len())]).into_owned();
        s.find(heading).map_or(String::new(), |at| s[at..].to_owned())
    };
    for _ in 0..frames {
        m.advance();
        let t = text(&m);
        lid(&mut m, &t);
    }
    text(&m)
}

/// ps1-tests' `cdrom/getloc` on a console log's terms: its own verdict, every GetlocL and GetlocP succeeding or failing
/// where the console's did (Diamond_CDROM.md §9).
#[test]
fn getloc_passes_on_the_players_bios() {
    let Some((corpus, bios, disc)) = disc_inputs() else { return };
    let exe = std::fs::read(corpus.join("cdrom/getloc/getloc.exe")).unwrap();
    let tty = run_on_disc(&exe, &bios, disc, 1200, "cdrom/", |_, _| {});
    let log = std::fs::read_to_string(corpus.join("cdrom/getloc/psx.log")).unwrap();
    let verdicts = |t: &str| t.lines().filter(|l| l.contains("succeeded") || l.contains("failed") || l.starts_with("> GetStat")).map(|l| l.split(" -").next().unwrap_or(l).split(',').next().unwrap_or(l).to_owned()).collect::<Vec<_>>();
    assert!(tty.contains("Test passed"), "{tty}");
    assert_eq!(verdicts(&tty), verdicts(&log), "{tty}");
}

/// `cdrom/disc-swap`: the lid opened a second after the program asks and closed a second after it asks again, each
/// line its console log shows reproduced but the last, which the release prints differently from the log's build.
#[test]
fn disc_swap_matches_the_consoles_log() {
    let Some((corpus, bios, disc)) = disc_inputs() else { return };
    let exe = std::fs::read(corpus.join("cdrom/disc-swap/disc-swap.exe")).unwrap();
    let (mut asked, mut at) = ((0, 0), 0);
    let tty = run_on_disc(&exe, &bios, disc, 700, "cdrom/", |m, t| {
        let now = (t.matches("Open the shell now").count(), t.matches("Close the shell now").count());
        if now != asked {
            (asked, at) = (now, m.frames);
        }
        if m.frames >= at + 60 && now.0 > now.1 && !m.board.cdrom.lid_open {
            m.set_lid(true, None);
        } else if m.frames >= at + 60 && now.0 == now.1 && now.1 > 0 && m.board.cdrom.lid_open {
            m.set_lid(false, None);
        }
    });
    let log = std::fs::read_to_string(corpus.join("cdrom/disc-swap/psx.log")).unwrap();
    let want: Vec<&str> = log.lines().take(log.lines().count() - 1).collect();
    assert_eq!(tty.lines().take(want.len()).collect::<Vec<_>>(), want, "{tty}");
}

/// `cdrom/timing`: every figure Diamond prints lies within the console log's minimum and maximum for it.
#[test]
fn the_timing_programs_figures_lie_within_the_consoles() {
    let Some((corpus, bios, disc)) = disc_inputs() else { return };
    let exe = std::fs::read(corpus.join("cdrom/timing/timing.exe")).unwrap();
    let tty = run_on_disc(&exe, &bios, disc, 6000, "cdrom/timing", |_, _| {});
    let log = std::fs::read_to_string(corpus.join("cdrom/timing/psx.log")).unwrap();
    let numbers = |t: &str| -> Vec<Vec<u64>> { t.lines().filter(|l| l.contains(':')).map(|l| l.split(|c: char| !c.is_ascii_digit()).filter_map(|w| w.parse().ok()).collect()).collect() };
    let (ours, console) = (numbers(&tty), numbers(&log));
    assert_eq!(ours.iter().map(Vec::len).collect::<Vec<_>>(), console.iter().map(Vec::len).collect::<Vec<_>>(), "{tty}");
    for (o, c) in ours.iter().zip(&console).filter(|(o, _)| o.len() == 3 || o.len() == 4) {
        assert!((c[0]..=c[1]).contains(&o[2]), "an average of {} outside the console's {}..{}", o[2], c[0], c[1]);
    }
    let reads = |t: &[Vec<u64>]| t.iter().filter(|r| r.len() == 8).cloned().collect::<Vec<_>>();
    let (ours, console) = (reads(&ours), reads(&console));
    for speed in 0..2 {
        let group = &console[5 * speed..5 * speed + 5];
        for r in &ours[5 * speed..5 * speed + 5] {
            for i in 0..8 {
                let (lo, hi) = (group.iter().map(|c| c[i]).min().unwrap(), group.iter().map(|c| c[i]).max().unwrap());
                assert!(r[i] * 10 >= lo * 9 && r[i] * 10 <= hi * 11, "figure {i} of {r:?} outside {lo}..{hi}");
            }
        }
    }
}

/// The kernel's own CD functions, called by `cd-kernel.s` once the player's BIOS has handed over: _96_init, the
/// drive's status, sector 16 read whole, and SYSTEM.CNF found, each as the disc holds it.
#[test]
fn the_kernels_cd_functions_read_the_disc() {
    let Some((_, bios, mut disc)) = disc_inputs() else { return };
    let words = emusen_mips::assemble(include_str!("../programs/cd-kernel.s"), 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).unwrap();
    let image = exe::build(&words);
    let mut pvd = [0u8; 2352];
    disc.read(150 + 16, &mut pvd);
    let media = Media::read(&image).unwrap();
    let mut m = Machine::new(media, Config { market: Market::Americas, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None], &image, Some((&bios, Kernel::Player)));
    m.insert_disc(disc);
    for _ in 0..1200 {
        m.advance();
    }
    let r: Vec<u32> = (0..5).map(|i| m.board.ram_word(0x10_0000 + 4 * i)).collect();
    assert_eq!((r[0], r[1], r[3]), (5, 1, 1), "every call returned, _96_init and CdReadSector with success: {r:08X?}");
    assert_eq!(r[2] & 0x02, 0x02, "the motor on");
    assert!(r[4] > 16 && r[4] < 0x10000, "SYSTEM.CNF's sector: {}", r[4]);
    assert_eq!(&m.board.ram[0x11_0000..0x11_0800], &pvd[24..2072]);
}
