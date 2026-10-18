//! The project's own test programs in `programs/`, assembled with `emusen-mips` and run with no kernel: each one's
//! results as Diamond reads the documents, and as the witnesses of Diamond_Native.md §11.13 and the referees of
//! Diamond_Referees.md were seen to answer, or as a console's log shows.

use diamond::exe;
use diamond::kernel::{self, Kernel};
use diamond::machine::{Config, Gpu, Machine};
use diamond::media::{Market, Media};

// The goldens' runs, shared with the renderer's gate (tests/renderer.rs).
#[path = "support/goldens.rs"]
mod goldens;
use goldens::*;

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
    assert_eq!(results(&m, 0x10_0000, 30), [0; 30], "every read zero, and no exception");
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

/// The GTE's interlock as BlackMoon_GTE.md D-13 has it since amidog's timing tests: a command's cycles from its fetch,
/// and an access that must wait resumes one cycle after the command ends, so an access at once costs the command's
/// cycles, RTPS's MFC2 after N NOPs 15 - N up to 13 NOPs and nothing from 14; every access waits, writes included; the
/// unlisted numbers in the cycles amidog's table gives them (D-16).
#[test]
fn the_gte_holds_every_access_until_its_command_is_done() {
    let m = run(include_str!("../programs/gte-timing.s"));
    let commands = [0x01, 0x06, 0x0C, 0x10, 0x11, 0x12, 0x13, 0x14, 0x16, 0x1B, 0x1C, 0x1E, 0x20, 0x28, 0x29, 0x2A, 0x2D, 0x2E, 0x30, 0x3D, 0x3E, 0x3F];
    assert_eq!(results(&m, 0x10_0000, 22), commands.map(|c| blackmoon_gte::CYCLES[c] as u32));
    assert_eq!(results(&m, 0x10_0100, 21), (0..21).map(|n| if n < 14 { 15 - n } else { 0 }).collect::<Vec<_>>());
    assert_eq!(results(&m, 0x10_0200, 7), [15; 7]);
    assert_eq!(results(&m, 0x10_0280, 6), [16, 13, 8, 8, 18, 18]);
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
            assert_eq!([hblanks[1].wrapping_sub(hblanks[0]) & 0xFFFF, hblanks[2].wrapping_sub(hblanks[1]) & 0xFFFF], lines, "{market:?} mode {k}");
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

/// The open kernel's interrupt path, timed (Diamond_OpenKernel_Phase2.md §25.3): with a pad on port 1, port 2 empty and
/// timer 2 at Crash Bandicoot's period of 32,776 cycles, a vblank returns to the program in under 16,000 cycles and
/// timer 2's IRQ is acknowledged within 2,000 cycles of its exception; the kernel of §23 took 23,351 and 4,039.
#[test]
fn the_open_kernels_interrupt_path_fits_within_timer_2s_period() {
    let words = emusen_mips::assemble(include_str!("../programs/kernel-irq.s"), 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).expect("assembles");
    let image = exe::build(&words);
    let media = Media::read(&image).unwrap();
    let rom = kernel::open_kernel().map(|k| (k, Kernel::Open));
    let mut m = Machine::new(media, Config { market: Market::Americas, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None], &image, rom);
    for _ in 0..10 {
        m.advance();
    }
    assert_eq!(m.board.ram_word(0xFFFF0), 0xD1A0_D00E, "the program reached its loop");
    let (mut vblank, mut timer, mut entry) = (Vec::new(), Vec::new(), None);
    while m.frames < 20 {
        let pc = m.cpu.regs.pc;
        if pc == 0x8000_0080 {
            entry = Some((m.board.now, m.board.irq.stat & m.board.irq.mask, None));
        }
        m.step();
        if let Some((start, pending, ack)) = &mut entry {
            if ack.is_none() && m.board.irq.stat & 0x40 == 0 && *pending & 0x40 != 0 {
                *ack = Some(m.board.now - *start);
            }
            if (0x8001_0000..0x8020_0000).contains(&m.cpu.regs.pc) {
                match (*pending, *ack) {
                    (0x01, _) => vblank.push(m.board.now - *start),
                    (0x40, Some(a)) => timer.push(a),
                    _ => {}
                }
                entry = None;
            }
        }
        if m.board.field_done {
            m.board.field_done = false;
            m.frames += 1;
        }
    }
    let (v, t) = (vblank.iter().max().copied().unwrap_or(0), timer.iter().max().copied().unwrap_or(0));
    assert!(vblank.len() >= 8 && timer.len() >= 100, "{} vblanks and {} timer IRQs seen", vblank.len(), timer.len());
    assert!(v < 16_000 && t < 2_000, "the longest vblank {v} cycles to the return, timer 2 acknowledged after {t} at most");
    assert_eq!(bytes(&m, 0x10_0000, 2), [0x00, 0x41], "port 1's pad read");
    assert_eq!(bytes(&m, 0x10_0040, 1), [0xFF], "port 2 empty");
}

/// The stretch behind B(5Bh)'s table entry (Diamond_OpenKernel_Phase2.md §27) on Diamond's bus: a program stores at the
/// offsets Mega Man Legends' pad library stores at and calls where it calls, each call coming back with v0 and S2-S7 as
/// they were, within the cycles §27.3 gives for a cold and a warm I-cache, and the kernel reads the pads afterwards.
#[test]
fn calls_and_stores_past_b_5bh_s_entry_come_back_and_leave_the_pads_read() {
    let words = emusen_mips::assemble(include_str!("../programs/kernel-pad-entry.s"), 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).expect("assembles");
    let image = exe::build(&words);
    let media = Media::read(&image).unwrap();
    let rom = kernel::open_kernel().map(|k| (k, Kernel::Open));
    let mut m = Machine::new(media, Config { market: Market::Americas, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None], &image, rom);
    let user = |pc: u32| (0x8001_0000..0x8020_0000).contains(&pc);
    let (mut slides, mut inside) = (Vec::new(), None);
    while m.frames < 30 && m.board.ram_word(0xFFFF0) != 0xD1A0_D00E {
        let pc = m.cpu.regs.pc;
        m.step();
        let next = m.cpu.regs.pc;
        if user(pc) && next >> 20 == 0x9FC {
            inside = Some((next, m.board.now, 0u32));
        } else if let Some((at, since, steps)) = &mut inside {
            *steps += 1;
            if user(next) {
                slides.push((*at, m.board.now - *since, *steps));
                inside = None;
            }
        }
        if m.board.field_done {
            m.board.field_done = false;
            m.frames += 1;
        }
    }
    assert_eq!(m.board.ram_word(0xFFFF0), 0xD1A0_D00E, "the program finished, port 1's pad read: pc {:08X}", m.cpu.regs.pc);
    let r = results(&m, 0x10_0100, 15);
    let e = r[0];
    assert_eq!(e >> 20, 0x9FC, "B(5Bh)'s entry names the ROM");
    assert_eq!((r[13], r[14]), (1, 0), "ChangeClearPAD's old flags");
    assert_eq!(r[1..7], [0x1234_5678; 6], "v0 after each call");
    assert_eq!(r[8..13], [0x8010_011C, 0x5454_0000, 0x5555_0000, 0x5656_0000, 0x5757_0000], "S3-S7 after the calls");
    assert!(user(r[7]), "S2, the return address the program kept");
    let offsets: Vec<u32> = slides.iter().map(|s| s.0 - e).collect();
    assert_eq!(offsets, [0x884, 0x884, 0x7A0, 0x894, 0xFC, 0xFC], "the calls seen entering the stretch");
    let steps: Vec<u32> = slides.iter().map(|s| s.2).collect();
    assert_eq!(steps, [31, 31, 24, 27, 65, 65], "instructions to the return's delay slot");
    // Measured 2026-10-12: 798, 31, 618, 695, 1,673 and 65 cycles; a cold line of the ROM costs 100 at the reset's timing.
    let cycles: Vec<u64> = slides.iter().map(|s| s.1).collect();
    assert!(cycles[0] < 1_000 && cycles[2] < 1_000 && cycles[3] < 1_000, "the game's three calls, cold: {cycles:?}");
    assert!(cycles[4] < 2_000, "the longest slide, cold: {cycles:?}");
    assert!(cycles[1] < 100 && cycles[5] < 100, "the same calls with the cache warm: {cycles:?}");
    assert_eq!(bytes(&m, 0x10_0000, 2), [0x00, 0x41], "port 1's pad read");
    assert_eq!(bytes(&m, 0x10_0040, 1), [0xFF], "port 2 empty");
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
/// row within one hblank of what it read before; since D-41, the textured quads 39,916 where they were 39,976 and 39,975;
/// since D-46, each triangle charged by the rows it spans, those clipped at the sides included: the clipped quads
/// 5,844 and 3,967 (the console 5,845 and 3,968).
#[test]
fn the_bandwidth_record_counts_each_rows_hblanks() {
    let Some(source) = carried("bandwidth") else { return };
    let m = run_for(&source, 860, Market::Americas);
    let hblanks: Vec<u32> = results(&m, 0x10_0000, 56).chunks(4).map(|r| r[2]).collect();
    assert_eq!(hblanks, [15_773, 12_123, 19_127, 1216, 7603, 11_461, 15_134, 15_134, 7720, 11_579, 39_915, 39_915, 5844, 3967]);
    assert_eq!(fnv(&results(&m, 0x10_0000, 56)), 0x5191_BCBD_B27F_FC42, "the milliseconds and MB/s as well");
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
    assert_eq!(busy[26..30], [7, 43, 358, 4651], "triangles charged by their rows (D-46)");
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
/// 32-bit store faulting at JOY_CTRL, SIO_CTRL and SPUCNT; 91 cells as the console's log has them, since DICR's bit 6
/// reads zero (Diamond_Records.md §16.2), GPUSTAT's bit 31 among those that are not, its bit 26 set since a reset ends
/// a fill (Diamond_Records.md §16.1).
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
    assert_eq!(cells[21..24].iter().map(|c| c[1]).collect::<Vec<_>>(), [0x34_0038; 3], "DICR, as the console reads it");
    assert_eq!(cells[45..48].iter().map(|c| c[1]).collect::<Vec<_>>(), [0x9480_2000; 3], "GPUSTAT after GP1(00h), ready as the release reads it");
    assert_eq!(fnv(&c.concat()), 0x1914_461B_7A92_CDDB, "the calls with their arguments");
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

/// The cycles of a pass of each of amidog's psxtest_gte 1.4 timing loops by its own rule, as read from its RAM: the
/// loop's instructions and the command's cycles c, with c taken as the NOPs' count when it is one more or fewer.
fn amidog_pass(c: u32, variant: usize, number: usize) -> u32 {
    let at_least = |nops: u32| if c == nops + 1 || c < nops { nops } else { c };
    match variant {
        0 => (if c == 1 { 0 } else { c }) + 6 + (number == 0x0A || number == 0x0E) as u32,
        1 => (if c == 0 || c == 2 { 1 } else { c }) + 6,
        2 => at_least(4) + 6,
        3 => at_least(7) + 6,
        4 => (if c == 1 { 0 } else { c }) + 8,
        5 => (if c == 0 || c == 2 { 1 } else { c }) + 8,
        6 => at_least(4) + 8,
        7 => at_least(7) + 8,
        8 => 2 * at_least(4) + 7,
        _ => 2 * at_least(7) + 9,
    }
}

/// D-13 and D-16 against amidog's psxtest_gte 1.4 (Diamond_Pad.md §5.3): its ten timing loops for every number, each
/// count of 501 passes within 50 cycles of its rule's, the cycles c those its table gives (the 22 commands psx-spx's).
/// Its rule adds a cycle to the at-once CFC2 after 0Ah and 0Eh that no wait accounts for; those two stay 501 short.
#[test]
fn amidogs_gte_timing_loops_take_the_cycles_its_test_expects() {
    let m = run_for(include_str!("../programs/gte-loops.s"), 40, Market::Americas);
    for number in 0..64 {
        let counts = results(&m, 0x10_0000 + 0x40 * number as u32, 10);
        for (variant, &count) in counts.iter().enumerate() {
            let wanted = amidog_pass(blackmoon_gte::CYCLES[number] as u32, variant, number) * 501 - 1;
            let short = if variant == 0 && (number == 0x0A || number == 0x0E) { 501 } else { 0 };
            assert!(count.abs_diff(wanted - short) <= 50, "number {number:02X}h, loop {variant}: {count} against {wanted}");
        }
    }
}

/// Diamond_Pad.md §11, the round trip: card-read.s reads a card through the protocol, the card card.s writes here or the
/// one `DIAMOND_CARD_IN` names (a reference's, which the five referees wrote byte for byte alike): FLAG 08h on insertion,
/// sector 0100h the bytes 3n with their checksum, 0101h zeros, the ID, and IRQ7 after every byte but each command's last.
#[test]
fn a_card_reads_back_through_the_protocol_as_written() {
    let written = run_set_up(include_str!("../programs/card.s"), 30, Market::Americas, |m| m.insert_card(0, None));
    let card = match std::env::var("DIAMOND_CARD_IN") {
        Ok(path) => std::fs::read(path).unwrap(),
        Err(_) => written.card_in(0).unwrap().data.clone(),
    };
    let m = run_set_up(include_str!("../programs/card-read.s"), 30, Market::Americas, |m| m.insert_card(0, Some(&card)));
    let data: Vec<u8> = (0..128u32).map(|i| (3 * i) as u8).collect();
    let checksum = data.iter().fold(1u8, |c, &d| c ^ d);
    let read = bytes(&m, 0x10_0000, 140);
    assert_eq!(read[..10], [0xFF, 0x08, 0x5A, 0x5D, 0x00, 0x01, 0x5C, 0x5D, 0x01, 0x00], "FLAG's bit 3 on insertion");
    assert_eq!((read[10..138].to_vec(), read[138], read[139]), (data, checksum, 0x47));
    let next = bytes(&m, 0x10_0100, 140);
    assert_eq!((next[..10].to_vec(), next[138..].to_vec()), (vec![0xFF, 0x08, 0x5A, 0x5D, 0x00, 0x01, 0x5C, 0x5D, 0x01, 0x01], vec![0x00, 0x47]));
    assert!(next[10..138].iter().all(|&b| b == 0));
    assert_eq!(bytes(&m, 0x10_0200, 10), [0xFF, 0x08, 0x5A, 0x5D, 0x5C, 0x5D, 0x04, 0x00, 0x00, 0x80]);
    let acks = |at: usize, n: usize| bytes(&m, at + 0x400, n);
    for (at, n) in [(0x10_0000, 140), (0x10_0100, 140), (0x10_0200, 10)] {
        assert!(acks(at, n - 1).iter().all(|&a| a == 1) && acks(at + n - 1, 1) == [0], "IRQ7 at {at:X}");
    }
}

// DICR's bit 6 (Diamond_Records.md §16.2), SWCz's store (BlackMoon_R3000.md D-21) and the garbage registers (§6.1).

/// DICR keeps bits 0-5 of a write and reads bit 6 zero after a byte, halfword or word store, as ps1-tests' console log
/// and the referees of three lineages read it; bits 7-14 read zero.
#[test]
fn dicr_reads_bit_6_zero_after_any_store() {
    let m = run(include_str!("../programs/dicr.s"));
    assert_eq!(results(&m, 0x10_0000, 10), [0x34_0038, 0x34_0038, 0x34_0038, 0x3F, 0, 0x3F, 0xFF_003F, 0x3F, 0x3F, 0]);
}

/// SWC0, SWC1 and SWC3 store a word of zero, and take DBE where no device answers, as the one referee that models the
/// store reads them (D-21); SWC2 stores the GTE's register.
#[test]
fn swcz_stores_zero_on_the_bus() {
    let m = run(include_str!("../programs/swcz.s"));
    assert_eq!(results(&m, 0x10_0000, 14), [0, 0, 0, 0, 0, 0, 0, 0x1234_5678, 0, 0, 0, 0x1C, 0x1C, 0x1C]);
}

// --- Stage 7: the CD-ROM controller (Diamond_CDROM.md §9) ---

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
fn disc_inputs() -> Option<(std::path::PathBuf, Vec<u8>, diamond::medium::Disc)> {
    let (Some(corpus), Some(firmware), Some(disc)) = (std::env::var_os("EMUSEN_DIAMOND_PS1TESTS"), std::env::var_os("EMUSEN_DIAMOND_FIRMWARE"), std::env::var_os("EMUSEN_DIAMOND_DISC")) else {
        eprintln!("skipped: EMUSEN_DIAMOND_PS1TESTS, EMUSEN_DIAMOND_FIRMWARE and EMUSEN_DIAMOND_DISC name ps1-tests' release, the player's BIOS folder and a disc");
        return None;
    };
    let name = diamond::v1::bios(Market::Americas).name;
    let bios = std::fs::read(std::path::Path::new(&firmware).join(&name)).expect("the American BIOS the core declares");
    let path = std::path::PathBuf::from(&disc);
    let image = std::fs::read(&path).expect("the disc");
    let mut files = Vec::new();
    if diamond::medium::is_sheet(&image) {
        for name in emusen_disc::set::companions(&String::from_utf8_lossy(&image)).expect("a sheet") {
            files.push(std::fs::read(path.parent().unwrap().join(name)).expect("a track file"));
        }
    }
    let kept = files.into_iter().map(emusen_native::core::Kept::from).collect();
    let Ok(diamond::medium::Opened::Disc(d, _)) = diamond::medium::open(emusen_native::core::Kept::from(image), kept) else { panic!("the disc opens") };
    Some((std::path::PathBuf::from(corpus), bios, *d))
}

/// A program run on the player's BIOS with the disc in the drive, the lid worked by `lid` each frame; the TTY from
/// `heading` on, the BIOS's own messages before the program left out.
fn run_on_disc(program: &[u8], bios: &[u8], disc: diamond::medium::Disc, frames: usize, heading: &str, mut lid: impl FnMut(&mut Machine, &str)) -> String {
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

// --- Stage 9: the MDEC (Diamond_MDEC.md) ---

/// `mdec-timing.s`: five streams decoded by DMA, 96 blocks each, DMA1 done 512 cycles a block after DMA0's start
/// whatever the depth and the blocks' content, within 0.3% of Beetle PSX's ticks (D-31).
#[test]
fn the_mdec_decodes_a_block_in_512_cycles_at_every_depth() {
    let m = run(include_str!("../programs/mdec-timing.s"));
    assert_eq!(results(&m, 0x10_0000, 10), [0xFCE, 0x1809, 0x178D, 0x1809, 0xFCE, 0x1808, 0xFCE, 0x180D, 0x178D, 0x1806]);
}

/// A ps1-tests MDEC program of the release on the open kernel, its machine after `frames`.
fn mdec_program(corpus: &std::path::Path, path: &str, frames: usize) -> Machine {
    let image = std::fs::read(corpus.join(path)).expect("the release's program");
    let media = Media::read(&image).unwrap();
    let mut m = Machine::new(media, Config { market: Market::Americas, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None], &image, kernel::open_kernel().map(|k| (k, Kernel::Open)));
    for _ in 0..frames {
        m.advance();
    }
    m
}

/// The bytes of a console log's hexdump.
fn hexdump(log: &str) -> Vec<u8> {
    log.lines().filter_map(|l| l.trim_start().split_once(": ")).filter(|(at, _)| u32::from_str_radix(at, 16).is_ok()).flat_map(|(_, b)| b.split_whitespace().map(|x| u8::from_str_radix(x, 16).unwrap()).collect::<Vec<_>>()).collect()
}

/// `mdec/4bit` and `mdec/8bit`: the block the program copies to VRAM at 32,32 against the console log's hexdump as
/// the program converts it (four bits doubled, eight bits halved twice and thrice, as its `vram.png` shows): 4-bit
/// exact, 8-bit one pixel off at (3,3), 239 on the console and 240 here (D-30).
#[test]
fn the_monochrome_programs_copy_the_consoles_block_to_vram() {
    let Some(corpus) = records::corpus() else { return };
    for (name, at4) in [("4bit", true), ("8bit", false)] {
        let m = mdec_program(&corpus, &format!("mdec/{name}/{name}.exe"), 300);
        let console = hexdump(&std::fs::read_to_string(corpus.join(format!("mdec/{name}/psx.log"))).unwrap());
        let mut off = Vec::new();
        for y in 0..8 {
            for x in 0..8 {
                let p = if at4 { (console[y * 4 + x / 2] >> (4 * (x % 2)) & 15) << 1 } else { console[y * 8 + x] >> 3 };
                let a = ((32 + y) * 1024 + 32 + x) * 2;
                let v = u16::from_le_bytes([m.board.gpu.vram.bytes[a], m.board.gpu.vram.bytes[a + 1]]);
                if v != p as u16 * 0x421 {
                    off.push((x, y));
                }
            }
        }
        assert_eq!(off, if at4 { vec![] } else { vec![(3, 3)] }, "{name}");
    }
}

/// `mdec/frame`'s CPU builds read each word after waiting for the output to be non-empty, the last word's wait
/// included, so they never return from their twentieth stripe on a console that empties its output as the
/// step-by-step log shows; the DMA builds finish and reset the MDEC.
#[test]
fn the_frame_programs_cpu_builds_wait_after_their_last_word() {
    let Some(corpus) = records::corpus() else { return };
    for (path, done, status) in [("mdec/frame/frame-15bit.exe", false, 0x8604_FFFF), ("mdec/frame/frame-15bit-dma.exe", true, 0x8004_0000)] {
        let m = mdec_program(&corpus, path, 400);
        let t = m.space(diamond::machine::TTY_ID).unwrap();
        let tty = String::from_utf8_lossy(&t[..t.iter().position(|&b| b == 0).unwrap_or(t.len())]).into_owned();
        assert_eq!((tty.matches("mdec_readDecoded").count(), tty.contains("Done")), (20, done), "{path}: {tty}");
        assert_eq!(m.board.mdec.status(), status, "{path}: every word decoded and read, or the MDEC reset after them");
    }
}

/// `mdec/step-by-step-log`'s status after each word the program writes and reads, against the console's log, as far
/// as the TTY space holds it: 414 of its 422 status lines, the eight others Cr's later turn in its third phase (D-32).
#[test]
fn the_step_by_step_status_follows_the_consoles_log() {
    let Some(corpus) = records::corpus() else { return };
    let m = mdec_program(&corpus, "mdec/step-by-step-log/step-by-step-log.exe", 9_500);
    let t = m.space(diamond::machine::TTY_ID).unwrap();
    let tty = String::from_utf8_lossy(t).into_owned();
    let log = std::fs::read_to_string(corpus.join("mdec/step-by-step-log/psx.log")).unwrap();
    let statuses = |s: &str| s.lines().filter(|l| l.starts_with("==== MDEC_STATUS: 0x") && l.len() > 30).map(|l| l[19..29].to_owned()).collect::<Vec<_>>();
    let (ours, console) = (statuses(&tty), statuses(&log));
    let compared = ours.len();
    let same = ours.iter().zip(&console).take(compared).filter(|(a, b)| a == b).count();
    assert_eq!((compared, same), (422, 414));
}

// --- Stage 8: the SPU (Diamond_SPU.md) ---

/// `spu-capture.s` (Diamond_SPU.md §10.2): its records pinned whole. T1, T2, T3, T5, T8 and T11 equal DuckStation's
/// sample for sample; the registers of T1 and T7 equal Beetle PSX's, SwanStation's, DuckStation's and PCSX-Redux's:
/// LSAX untouched by a key on, ENVX zero after an end and mute; T11's first samples after a key on show the envelope's
/// step after the sample it scales (D-36).
#[test]
fn the_capture_program_records_the_voices_as_the_referees_do() {
    let m = run_for(include_str!("../programs/spu-capture.s"), 450, Market::Americas);
    assert_eq!(results(&m, 0x10_0800, 5), [0x3FFF, 0x3FFF, 0x208, 0x208, 0xFFFF], "T1: ENVX and LSAX of voices 1 and 3, and ENDX");
    assert_eq!(results(&m, 0x10_1D00, 6), [0x1234, 0xFFFE, 0, 0xFFFE, 0x1234, 0x208], "T7: LSAX kept by a key on, ENVX zero after an end and mute");
    let half = |a: u32| m.board.ram_word(a & !3) >> (8 * (a & 2)) & 0xFFFF;
    assert_eq!((2..6).map(|i| half(0x10_6F80 + 2 * i) as i16).collect::<Vec<_>>(), [1864, 21308, 28559, 14279], "T11");
    assert_eq!(fnv(&results(&m, 0x10_0000, 0x7380 / 4)), 0xA09B_0287_3DEF_0CAD, "every record");
}

/// `spu-voices.s` (Diamond_SPU.md §10.1): its nine scenes run to their end, the output's samples pinned.
#[test]
fn the_voice_scenes_play_to_their_end() {
    let words = emusen_mips::assemble(include_str!("../programs/spu-voices.s"), 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).expect("assembles");
    let image = exe::build(&words);
    let mut m = Machine::new(Media::read(&image).unwrap(), Config { market: Market::Americas, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None], &image, None);
    let mut chunk = vec![0i16; 4096];
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    let mut loud = 0;
    for _ in 0..800 {
        m.advance();
        let n = m.board.spu.out.drain(&mut chunk, usize::MAX);
        for &s in &chunk[..n] {
            h = (h ^ s as u16 as u64).wrapping_mul(0x0000_0100_0000_01B3);
            loud += (s != 0) as u32;
        }
    }
    assert_eq!(m.board.ram_word(0xFFFF0), 0xD1A0_D00E);
    assert_eq!(results(&m, 0x10_0000, 9), [400, 443, 477, 510, 571, 614, 645, 660, 683], "each scene's key on, by frame");
    assert_eq!((loud, h), (335_782, 0x006B_FB6B_A2C3_9F1B), "the output: its samples not zero, and all of them");
}

/// ps1-tests' `spu/memory-transfer` on the open kernel passes every test, its DMA timing ones among them, and
/// `spu/test` reads back every bit of MVOLL and MVOLR written at either width.
#[test]
fn the_spu_programs_pass_on_the_open_kernel() {
    let Some(corpus) = records::corpus() else { return };
    let tty = |path: &str, frames: usize| {
        let m = mdec_program(&corpus, path, frames);
        let t = m.space(diamond::machine::TTY_ID).unwrap();
        String::from_utf8_lossy(&t[..t.iter().position(|&b| b == 0).unwrap_or(t.len())]).into_owned()
    };
    let transfer = tty("spu/memory-transfer/memory-transfer.exe", 200);
    assert_eq!((transfer.matches("pass - ").count(), transfer.matches("fail").count(), transfer.contains("Done.")), (9, 0, true), "{transfer}");
    let test = tty("spu/test/test.exe", 400);
    let lines: Vec<&str> = test[test.find("16 bit access").unwrap_or(0)..].lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(lines, ["16 bit access", "Done", "32 bit access", "Done, crashing now..."]);
}

/// `spu-dma.s` (Diamond_SPU.md §6): DMA4 moves 1 KiB in blocks of 16 words in 15,419 cycles, a halfword every 32 cycles
/// as the FIFO empties, within `spu/memory-transfer`'s bounds on a console (1,638 to 18,022) where Beetle PSX takes
/// 12,282 and DuckStation 7,640 (D-38); a mode written to ATTR shows in STATX at the next sample, 582 cycles, as on Beetle PSX.
#[test]
fn the_spus_transfers_take_a_halfword_each_32_cycles() {
    let m = run_for(include_str!("../programs/spu-dma.s"), 30, Market::Americas);
    assert_eq!(results(&m, 0x10_0000, 7), [15_419, 16_534, 1068, 16_402, 582, 993, 2054]);
}

// --- Stage 10: the open kernel's second phase (Diamond_OpenKernel_Phase2.md) ---

#[path = "support/iso.rs"]
mod iso;

/// DATA.BIN's byte k.
fn data_byte(k: usize) -> u8 {
    (k * 7 + 3) as u8
}

/// The boot disc: SYSTEM.CNF naming `kernel-cd.s` in a subdirectory with an argument and its own TCB, EVENT and STACK,
/// and a 5,000-byte DATA.BIN; the program's header gives GP 80123450h.
fn boot_disc() -> diamond::medium::Disc {
    let words = emusen_mips::assemble(include_str!("../programs/kernel-cd.s"), 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).expect("assembles");
    let mut exe = exe::build(&words);
    exe[0x14..0x18].copy_from_slice(&0x8012_3450u32.to_le_bytes());
    let data: Vec<u8> = (0..5000).map(data_byte).collect();
    let cnf = b"BOOT = cdrom:\\DIR\\PROG.EXE;1 hello\r\nTCB = 6\r\nEVENT = 14\r\nSTACK = 801FF800\r\n";
    iso::disc(&[iso::Entry::File("SYSTEM.CNF", cnf), iso::Entry::File("DATA.BIN", &data), iso::Entry::Dir("DIR"), iso::Entry::File("DIR\\PROG.EXE", &exe)])
}

/// A machine on the open kernel with no program given, as a disc is booted: reset at BFC00000h.
fn open_machine(disc: Option<diamond::medium::Disc>) -> Machine {
    let media = Media { kind: diamond::media::Kind::Disc { raw: true }, market: Some(Market::Americas) };
    let mut m = Machine::new(media, Config { market: Market::Americas, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None], &[], kernel::open_kernel().map(|k| (k, Kernel::Open)));
    if let Some(d) = disc {
        m.insert_disc(d);
    }
    m
}

/// The CD boot of Diamond_OpenKernel_Phase2.md §15 with no firmware: SYSTEM.CNF's program loaded from a subdirectory and
/// entered with a0 = 1, a1 = 0, SYSTEM.CNF's stack and the header's GP, the argument at 180h, SetConf with TCB and EVENT;
/// then the kernel's CD and file calls on the same disc, none of them changing S0-S7, GP, SP or FP (§12).
#[test]
fn the_open_kernel_boots_a_disc_by_its_system_cnf_and_reads_its_files() {
    let mut m = open_machine(Some(boot_disc()));
    for _ in 0..900 {
        m.advance();
        if m.board.ram_word(0xFFFF0) == 0xD1A0_D00E {
            break;
        }
    }
    assert_eq!(m.board.ram_word(0xFFFF0), 0xD1A0_D00E, "the program finished at frame {}, pc {:08X}", m.frames, m.cpu.regs.pc);
    let r = results(&m, 0x10_0000, 27);
    assert_eq!(&r[..4], [1, 0, 0x801F_F800, 0x8012_3450], "a0, a1, SYSTEM.CNF's STACK and the header's GP at the entry");
    assert_eq!(r[4], u32::from_le_bytes(*b"hell"), "the argument at 180h");
    assert_eq!(&r[5..8], [0x14, 6, 0x801F_F800], "GetConf: EVENT and TCB, hexadecimal, and STACK");
    assert_eq!(r[8], 0, "no call changed S0-S7, GP, SP or FP");
    let fd = r[9];
    assert!(fd >= 2 && fd < 16, "open: {fd:X}");
    assert_eq!(&r[10..15], [100, 4000, 900, u32::MAX, fd], "lseek, two reads to the file's end and one past it, close");
    let want: Vec<u8> = (100..5000).map(data_byte).collect();
    assert_eq!(&m.board.ram[0x11_0000..0x11_0000 + 4000], &want[..4000]);
    assert_eq!(&m.board.ram[0x11_2000..0x11_2000 + 900], &want[4000..]);
    let mut pvd = [0u8; 2352];
    boot_disc().read(150 + 16, &mut pvd);
    assert!(r[15] > 20 && r[15] < 40, "CdGetLbn: {}", r[15]);
    assert_eq!(r[16], 1, "CdReadSector");
    assert_eq!(&m.board.ram[0x11_8000..0x11_8800], &pvd[24..2072]);
    assert_eq!((r[17], r[18], r[19], r[20]), (0x8010_0200, 0x8010_0228, 0x8010_0250, 0), "firstfile2 and nextfile over the root");
    let name = |at: usize| String::from_utf8_lossy(&m.board.ram[at..at + 20]).trim_end_matches('\0').to_owned();
    assert_eq!([name(0x10_0200), name(0x10_0228), name(0x10_0250)], ["DATA.BIN;1", "DIR", "SYSTEM.CNF;1"]);
    assert_eq!(m.board.ram_word(0x10_0200 + 0x18), 5000, "DATA.BIN's size in its direntry");
    assert_eq!(r[21], 1, "cd");
    assert!(r[22] < 16 && r[23] == r[22], "a name relative to cd's directory: {:X} {:X}", r[22], r[23]);
    assert_eq!((r[24], r[25]), (u32::MAX, 2), "a missing file: -1, error 02h");
    assert_eq!(r[26] & 0x02, 0x02, "CdGetStatus: the motor on");
}

/// Mega Man Legends' hand-over (Diamond_OpenKernel_Phase2.md §26) on a synthetic disc: a program with its own exit
/// structure and vblank enabled makes the kernel's synchronous CD calls, then LoadExec of a 128 KiB program over
/// itself. No handler of the program is shown IRQ2 while the kernel's own operation runs, its hook never runs once
/// LoadExec has begun, and the new program finds I_MASK and I_STAT clear and the default exit, not the old one.
#[test]
fn loadexec_starts_its_program_free_of_the_callers_interrupt_handlers() {
    let assemble = |src: &str| emusen_mips::assemble(src, 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).expect("assembles");
    let first = exe::build(&assemble(include_str!("../programs/kernel-loadexec.s")));
    let mut second = exe::build(&assemble(include_str!("../programs/kernel-loadexec-second.s")));
    second.resize(0x800 + 0x2_0000, 0);
    second[0x1C..0x20].copy_from_slice(&0x2_0000u32.to_le_bytes());
    let cnf = b"BOOT = cdrom:\\FIRST.EXE;1\r\n";
    let disc = || iso::disc(&[iso::Entry::File("SYSTEM.CNF", cnf), iso::Entry::File("FIRST.EXE", &first), iso::Entry::File("SECOND.EXE", &second)]);
    let mut m = open_machine(Some(disc()));
    for _ in 0..1200 {
        m.advance();
        if m.board.ram_word(0xFFFF0) == 0xD1A0_D00E {
            break;
        }
    }
    assert_eq!(m.board.ram_word(0xFFFF0), 0xD1A0_D00E, "the second program finished at frame {}, pc {:08X}", m.frames, m.cpu.regs.pc);
    let r = results(&m, 0x18_0100, 14);
    assert_eq!((r[13], r[12]), (1, 4), "_96_init and CdReadSector");
    let mut pvd = [0u8; 2352];
    disc().read(150 + 16, &mut pvd);
    assert_eq!(&m.board.ram[0x19_0000..0x19_0800], &pvd[24..2072]);
    assert_eq!(r[1] & 1, 1, "the hook ran while the CD calls did, with vblank enabled");
    assert_eq!(r[1] & 4, 0, "and was never shown IRQ2 enabled: I_MASK ORed {:X}", r[1]);
    assert_eq!(r[10], r[2], "no hook ran from LoadExec's start to the new program's entry");
    assert_eq!((r[8], r[9]), (0, 0), "I_MASK and I_STAT at the new program's entry");
    assert_eq!(r[11], 0, "the old exit structure was not taken once vblank was enabled again");
}

/// The card calls of Diamond_OpenKernel_Phase2.md §13-§14 on a blank card: format, files made, written and read, listed,
/// erased, undeleted and renamed, asynchronous and low-level reads ended by their events; and the card left in the
/// on-card format of psx-spx's "Memory Card Data Format", every directory frame's checksum and chain as it gives them.
#[test]
fn the_open_kernels_card_files_keep_the_on_card_format() {
    let words = emusen_mips::assemble(include_str!("../programs/kernel-card.s"), 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).expect("assembles");
    let image = exe::build(&words);
    let media = Media::read(&image).unwrap();
    let mut m = Machine::new(media, Config { market: Market::Americas, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None], &image, kernel::open_kernel().map(|k| (k, Kernel::Open)));
    m.insert_card(0, None);
    for _ in 0..3000 {
        m.advance();
        if m.board.ram_word(0xFFFF0) == 0xD1A0_D00E {
            break;
        }
    }
    assert_eq!(m.board.ram_word(0xFFFF0), 0xD1A0_D00E, "the program finished at frame {}, pc {:08X}", m.frames, m.cpu.regs.pc);
    let r = results(&m, 0x10_0000, 42);
    assert_eq!(r[3], 1, "format");
    assert!(r[4] < 16, "a new file opened: {:X}", r[4]);
    assert_eq!((r[5], r[6], r[8], r[9]), (0x4000, r[4], 0x4000, r[7]), "16 KiB written, closed, read back, closed");
    let pattern: Vec<u8> = (0..0x1000u32).flat_map(|k| (k ^ 0xA500_0000).to_le_bytes()).collect();
    assert_eq!(&m.board.ram[0x13_0000..0x13_4000], &pattern[..]);
    assert_eq!((r[12], r[13], r[14]), (0x8010_0200, 0x8010_0228, 0), "firstfile2 and nextfile");
    assert_eq!((r[15], r[16], r[17]), (1, u32::MAX, 2), "erase, then the erased file not found");
    assert_eq!(r[19], 0x8010_0278, "the deleted file found in find mode 1");
    assert_eq!(m.board.ram_word(0x10_0278 + 0x14), 0xA0, "its attribute");
    assert_eq!((r[21], r[22], r[23], r[24]), (1, 1, 1, u32::MAX), "undelete, erase, rename, the old name gone");
    assert_eq!((r[28], r[29]), (0, 1), "the asynchronous read accepted with 0 (§25.2) and its event comes");
    assert_eq!(&m.board.ram[0x14_0000..0x14_0080], &pattern[..0x80]);
    assert_eq!((r[33], r[34], r[35]), (1, 1, 1), "_card_read queued, its event, _card_status ready");
    assert_eq!((r[38], r[39], r[40], r[41]), (1, 1, 0x11, u32::MAX), "the empty slot: _card_info's timeout, status 11h, open fails");
    let card = &m.card_in(0).unwrap().data;
    let frame = |i: usize| &card[i * 128..i * 128 + 128];
    let xor = |f: &[u8]| f[..127].iter().fold(0u8, |a, b| a ^ b);
    assert_eq!(&m.board.ram[0x15_0000..0x15_0080], frame(0), "_card_read's sector 0");
    assert_eq!(&frame(0)[..2], b"MC");
    for i in 0..36 {
        assert_eq!(xor(frame(i)), frame(i)[127], "frame {i}'s checksum");
    }
    let dir = |b: usize| (u32::from_le_bytes(frame(b)[..4].try_into().unwrap()), u32::from_le_bytes(frame(b)[4..8].try_into().unwrap()), u16::from_le_bytes([frame(b)[8], frame(b)[9]]), String::from_utf8_lossy(&frame(b)[10..31]).trim_end_matches('\0').to_owned());
    assert_eq!(dir(1), (0x51, 0x4000, 1, "BASLUS-00000RENAMED".into()), "the first block of two, linked to block 2");
    assert_eq!(dir(2), (0x53, 0, 0xFFFF, String::new()), "its last block");
    assert_eq!(dir(3), (0xA1, 0x2000, 0xFFFF, "BASLUS-00000SECOND".into()), "the erased file");
    for b in 4..16 {
        assert_eq!(dir(b), (0xA0, 0, 0xFFFF, String::new()), "block {b} free");
    }
    for i in 16..36 {
        assert_eq!(&frame(i)[..4], [0xFF; 4], "no broken sector in frame {i}");
    }
    assert!((36..63).all(|i| frame(i).iter().all(|&b| b == 0xFF)) && frame(63) == frame(0), "frames 36-62 FFh, 63 the header");
    assert_eq!(&card[0x2000..0x6000], &pattern[..], "the file's 16 KiB in blocks 1 and 2");
}

/// Stage 7's `cd-kernel.s`, side-loaded on the open kernel instead of the player's BIOS, with the synthetic disc:
/// _96_init, the drive's status, sector 16 read whole and SYSTEM.CNF found, as the program expects of either kernel.
#[test]
fn the_kernels_cd_functions_read_a_disc_on_the_open_kernel() {
    let words = emusen_mips::assemble(include_str!("../programs/cd-kernel.s"), 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).unwrap();
    let image = exe::build(&words);
    let media = Media::read(&image).unwrap();
    let mut m = Machine::new(media, Config { market: Market::Americas, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None], &image, kernel::open_kernel().map(|k| (k, Kernel::Open)));
    m.insert_disc(boot_disc());
    for _ in 0..300 {
        m.advance();
    }
    let r: Vec<u32> = (0..5).map(|i| m.board.ram_word(0x10_0000 + 4 * i)).collect();
    assert_eq!((r[0], r[1], r[3]), (5, 1, 1), "every call returned, _96_init and CdReadSector with success: {r:08X?}");
    assert_eq!(r[2] & 0x02, 0x02, "the motor on");
    assert_eq!(r[4], 21, "SYSTEM.CNF's sector, after the root and DIR");
    let mut pvd = [0u8; 2352];
    boot_disc().read(150 + 16, &mut pvd);
    assert_eq!(&m.board.ram[0x11_0000..0x11_0800], &pvd[24..2072]);
}

/// The screen with no disc (§16): an empty drive and a blank card, which the shell formats from Memory Cards' card
/// actions and their confirmation (Diamond_Shell.md §3.2, since S4); then a disc put in, which the shell finds within its
/// two-second poll and the kernel boots by its SYSTEM.CNF.
#[test]
fn the_screen_with_no_disc_formats_a_card_and_boots_a_disc_put_in() {
    let mut m = open_machine(None);
    m.insert_card(0, None);
    format_from_the_shell(&mut m);
    let card = &m.card_in(0).unwrap().data;
    assert_eq!(&card[..2], b"MC", "formatted from the manager");
    assert_eq!(&card[128..132], [0xA0, 0, 0, 0], "block 1 free");
    assert_eq!(background(&m), SHELL_BACKGROUND, "the manager's background");
    m.set_lid(true, None);
    for _ in 0..30 {
        m.advance();
    }
    m.set_lid(false, Some(boot_disc()));
    for _ in 0..1200 {
        m.advance();
        if m.board.ram_word(0xFFFF0) == 0xD1A0_D00E {
            break;
        }
    }
    assert_eq!(m.board.ram_word(0xFFFF0), 0xD1A0_D00E, "the disc put in booted, by frame {}", m.frames);
    assert_eq!(m.board.ram_word(0x10_0000 + 4 * 8), 0, "and its calls kept the saved registers");
}

/// Diamond_Shell.md §4.2 case 5: the shell side-loaded as a PS-EXE on the open kernel, which then never runs its own
/// shell code, formats a card and draws its screen all the same: it reaches the kernel through the call tables alone.
#[test]
fn the_shell_runs_as_a_program_side_loaded_on_the_open_kernel() {
    let image = include_bytes!("../kernel-shell/diamond-shell.exe");
    let media = Media::read(image).unwrap();
    let mut m = Machine::new(media, Config { market: Market::Americas, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None], image, kernel::open_kernel().map(|k| (k, Kernel::Open)));
    m.insert_card(0, None);
    format_from_the_shell(&mut m);
    assert_eq!(&m.card_in(0).unwrap().data[..2], b"MC", "formatted by the side-loaded shell");
    assert_eq!(background(&m), SHELL_BACKGROUND, "the shell's background");
}

/// The shell's background, LunaP's menu void at 15 bits (Diamond_Shell.md §3.2).
const SHELL_BACKGROUND: u16 = 1 | 1 << 5 | 2 << 10;

fn background(m: &Machine) -> u16 {
    let (x, y) = (2, 120);
    u16::from_le_bytes([m.board.gpu.vram.bytes[2 * (y * 1024 + x)], m.board.gpu.vram.bytes[2 * (y * 1024 + x) + 1]])
}

/// The card in slot 1 formatted from the shell once it has read the cards: Memory Cards, Triangle, Format, then the
/// confirmation's Format, the cursor starting on Cancel.
fn format_from_the_shell(m: &mut Machine) {
    while !String::from_utf8_lossy(&m.board.tty.text).contains("shell: card 2") {
        m.advance();
    }
    for bits in [1u32 << 14, 1 << 12, 1 << 14, 1 << 4, 1 << 14] {
        m.board.sio0.set_buttons(0, bits);
        for _ in 0..4 {
            m.advance();
        }
        m.board.sio0.set_buttons(0, 0);
        for _ in 0..4 {
            m.advance();
        }
    }
    while !String::from_utf8_lossy(&m.board.tty.text).contains("shell: format 1") {
        m.advance();
    }
    for _ in 0..60 {
        m.advance();
    }
}

/// `card-write-race.s` on a machine with blank cards in both slots, the port it writes poked first, run to the B0h entry
/// of its `_card_write` call; the machine and the instructions stepped so far.
fn card_write_call(port: u32) -> Machine {
    let words = emusen_mips::assemble(include_str!("../programs/card-write-race.s"), 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).unwrap();
    let image = exe::build(&words);
    let media = Media::read(&image).unwrap();
    let mut m = Machine::new(media, Config { market: Market::Americas, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None], &image, kernel::open_kernel().map(|k| (k, Kernel::Open)));
    m.insert_card(0, None);
    m.insert_card(1, None);
    m.board.ram[0x10_0100..0x10_0104].copy_from_slice(&port.to_le_bytes());
    while !(m.cpu.regs.pc & 0x1FFF_FFFF == 0xB0 && m.cpu.regs.r[9] == 0x4E) {
        m.step();
    }
    m
}

/// §23: a vblank taken at any instruction of `_card_write` leaves the request whole, so that frame 3Fh holds the bytes
/// the call names. The first build stored the request a field at a time with interrupts on, and a vblank between the
/// stores carried out a write of sector 3Fh from address 0, the kernel's copy of its exception vector, as the
/// cross-kernel step saw on frame 63 (§22.2). The window swept is where the call stores into the kernel's RAM.
#[test]
fn a_vblank_inside_card_write_never_sends_a_half_stored_request() {
    let pattern: Vec<u8> = (0..128u32).map(|n| (3 * n + 1) as u8).collect();
    for port in [0x00, 0x10] {
        let mut m = card_write_call(port);
        let before = m.board.ram[..0xE000].to_vec();
        let mut stores = Vec::new();
        for k in 0..20_000usize {
            m.step();
            if m.cpu.regs.pc & 0x1FFF_FFFF < 0x1_0000 || m.cpu.regs.pc >= 0x9FC0_0000 {
                let now = &m.board.ram[0x500..0xDF80];
                if now != &before[0x500..0xDF80] && stores.last() != Some(&k) {
                    stores.push(k);
                }
            }
            if m.cpu.regs.pc & 0x1FFF_0000 == 0x1_0000 {
                break;
            }
        }
        let (first, last) = (stores.first().copied().unwrap_or(0), stores.last().copied().unwrap_or(0));
        for k in first.saturating_sub(40)..last + 40 {
            let mut m = card_write_call(port);
            for _ in 0..k {
                m.step();
            }
            m.board.irq.raise(diamond::irq::VBLANK);
            for _ in 0..60 {
                m.advance();
                if m.board.ram_word(0xFFFF0) == 0xD1A0_D00E {
                    break;
                }
            }
            let card = &m.card_in((port >> 4) as usize).unwrap().data;
            assert_eq!(&card[0x3F * 128..0x40 * 128], &pattern[..], "port {port:02X}: vblank raised {k} instructions into the call");
        }
    }
}

// ---- Stage 12, the debugger (Diamond_Debugger.md) ----------------------------------------------------------------

/// How a table is set on a machine.
type Arm = fn(&mut Machine);

/// The debugger's tables, each armed alone and then all at once: what each is set to, and how it is set.
fn tables() -> Vec<(&'static str, Arm)> {
    use diamond::machine::{RAM_ID, SCRATCH_ID};
    use emusen_native::debug::{flag, range};
    vec![
        ("calls", |m| m.debug.hooks.configure(flag::CALLS, i32::MIN, -1)),
        ("writes watched over RAM and the scratchpad", |m| {
            m.debug.hooks.configure(flag::WRITES, i32::MIN, -1);
            m.debug.hooks.set_ranges(0, &[RAM_ID, 0, 0x1F_FFFF, SCRATCH_ID, 0, 0x3FF]);
        }),
        ("data breakpoints over RAM", |m| {
            m.debug.hooks.configure(flag::WRITES, i32::MIN, -1);
            m.debug.hooks.set_ranges(1, &[RAM_ID, 0, 0x1F_FFFF]);
        }),
        ("reads watched over RAM and the scratchpad", |m| {
            m.debug.hooks.configure(flag::READS, i32::MIN, -1);
            m.debug.hooks.set_ranges(range::READ_WATCH, &[RAM_ID, 0, 0x1F_FFFF, SCRATCH_ID, 0, 0x3FF]);
        }),
        ("read data breakpoints over RAM", |m| {
            m.debug.hooks.configure(flag::READS, i32::MIN, -1);
            m.debug.hooks.set_ranges(range::READ_BREAK, &[RAM_ID, 0, 0x1F_FFFF]);
        }),
        ("profile", |m| m.debug.hooks.configure(flag::CALLS | flag::PROFILING, i32::MIN, -1)),
        ("coverage", |m| m.debug.hooks.configure(1 << flag::COVERAGE, i32::MIN, -1)),
        ("interrupts", |m| m.debug.hooks.configure(flag::CALLS | flag::INTERRUPTS, i32::MIN, -1)),
        ("breakpoints over KSEG0's RAM", |m| m.debug.hooks.set_breakpoints(&[0x8000_0000u32 as i32, 0x801F_FFFFu32 as i32])),
        ("each instruction", |m| m.debug.hooks.configure(flag::EACH | flag::CALLS, i32::MIN, -1)),
        ("the kit's C15, all with nothing to hit", |m| {
            m.debug.hooks.configure(flag::CALLS | flag::WRITES | flag::READS | flag::PROFILING | 1 << flag::COVERAGE, i32::MIN, -1);
            m.debug.hooks.set_breakpoints(&[-2, -2]);
            for kind in [range::WATCH, range::BREAK, range::READ_WATCH, range::READ_BREAK] {
                m.debug.hooks.set_ranges(kind, &[u32::MAX - 1, 0, u32::MAX]);
            }
        }),
    ]
}

/// The digests of `frames` frames, plain or through the debugger's frame with every stop continued and every log
/// drained as a host does: the pictures', the sound's and the final state's, and the stops.
fn debug_digests(mut m: Machine, frames: usize, observed: bool) -> ([u64; 3], u64) {
    use emusen_native::ffi::StateMachine;
    let bytes = |h: u64, b: &[u8]| b.iter().fold(h, |h, &x| (h ^ x as u64).wrapping_mul(0x0000_0100_0000_01B3));
    let (mut picture, mut sound, mut stops) = (0xcbf2_9ce4_8422_2325u64, 0xcbf2_9ce4_8422_2325u64, 0u64);
    let mut chunk = vec![0i16; 1 << 16];
    for _ in 0..frames {
        if observed {
            let mut flags = 0;
            while m.run_frame_debug(flags) != 0 {
                stops += 1;
                m.debug.hooks.calls_log.clear();
                m.debug.hooks.writes_log.clear();
                m.debug.hooks.reads_log.clear();
                flags = emusen_native::debug::run::UNCHECKED;
            }
        } else {
            m.advance();
        }
        picture = bytes(picture, m.board.gpu.display.frame());
        let n = m.board.spu.out.drain(&mut chunk, usize::MAX);
        sound = chunk[..n].iter().fold(sound, |h, &s| bytes(h, &s.to_le_bytes()));
    }
    let mut state = vec![0; m.state_size()];
    let n = m.save_state(&mut state).unwrap();
    ([picture, sound, bytes(0xcbf2_9ce4_8422_2325, &state[..n])], stops)
}

/// G7's oracle on one program: each table armed gives the plain run's digests; the digests and the stops printed.
fn every_table_gives_the_plain_run(name: &str, frames: usize, make: impl Fn() -> Machine) {
    let (plain, _) = debug_digests(make(), frames, false);
    eprintln!("{name}, {frames} frames: plain {:016X} {:016X} {:016X}", plain[0], plain[1], plain[2]);
    for (table, arm) in tables() {
        let mut m = make();
        arm(&mut m);
        let (got, stops) = debug_digests(m, frames, true);
        eprintln!("  {table}: {stops} stops, {}", if got == plain { "the plain run's digests" } else { "DIFFERENT" });
        assert_eq!(got, plain, "{name} with {table} armed");
    }
}

/// The project's programs: exceptions of every kind, the GTE, the SPU's voices, the GPU's raster order, the MDEC and a
/// sector read from a disc.
#[test]
fn every_table_armed_gives_the_plain_runs_digests_on_the_projects_programs() {
    every_table_gives_the_plain_run("exceptions.s", 10, || bare(include_str!("../programs/exceptions.s")));
    every_table_gives_the_plain_run("gte-fuzz.s", 10, || bare(include_str!("../programs/gte-fuzz.s")));
    every_table_gives_the_plain_run("spu-voices.s", 20, || bare(include_str!("../programs/spu-voices.s")));
    every_table_gives_the_plain_run("raster-order.s", 10, || bare(include_str!("../programs/raster-order.s")));
    every_table_gives_the_plain_run("mdec-timing.s", 10, || bare(include_str!("../programs/mdec-timing.s")));
    every_table_gives_the_plain_run("cd-read.s", 60, || {
        let mut m = bare(include_str!("../programs/cd-read.s"));
        m.insert_disc(cd_disc());
        m
    });
}

/// ps1-tests' programs of the release on the open kernel, the kernel's own start-up among what is observed.
#[test]
fn every_table_armed_gives_the_plain_runs_digests_on_ps1_tests() {
    let Some(corpus) = records::corpus() else { return };
    for (path, frames) in [("timers/timers.exe", 30), ("gpu/triangle/triangle.exe", 30), ("gte/test-all/test-all.exe", 30), ("spu/test/test.exe", 30), ("mdec/movie/movie-15bit.exe", 30), ("dma/otc-test/otc-test.exe", 30)] {
        every_table_gives_the_plain_run(path, frames, || mdec_program(&corpus, path, 0));
    }
}

/// The disc `EMUSEN_DIAMOND_DISC` names, one of the tester's copied to the cache: its file's name, and a maker of
/// machines on the open kernel with it in the drive and no firmware. None, and a line saying so, where it names none.
fn named_disc() -> Option<(String, impl Fn() -> Machine)> {
    let Some(path) = std::env::var_os("EMUSEN_DIAMOND_DISC") else {
        eprintln!("skipped: EMUSEN_DIAMOND_DISC names no disc");
        return None;
    };
    let path = std::path::PathBuf::from(path);
    Some((path.file_name().unwrap().to_string_lossy().into_owned(), disc_maker(path)))
}

/// One of the tester's discs, copied to the cache and named by `EMUSEN_DIAMOND_DISC`, on the open kernel with no
/// firmware: what the open kernel does with it at this stage, observed.
#[test]
fn every_table_armed_gives_the_plain_runs_digests_on_a_disc() {
    let Some((name, make)) = named_disc() else { return };
    every_table_gives_the_plain_run(&name, 120, make);
}

/// D-42: GP1(01h) written while a fill draws leaves the whole fill, after no delay and after 100 to 1,600 loop turns,
/// as all five referees draw it, where amidog's FIFO overflow test expects one stopped some five lines in
/// (`programs/fill-reset.s`).
#[test]
fn a_fill_cut_by_gp1_01h_is_drawn_whole_as_the_referees_draw_it() {
    let m = run_for(include_str!("../programs/fill-reset.s"), 30, Market::Americas);
    let lines = |band: u32| (0..511).filter(|&l| m.board.ram_word(0x10_0000 + (band * 511 + l) * 4) & 0xFFFF == 0x7FFF).count();
    assert_eq!([0, 1, 2, 3].map(lines), [511; 4]);
}

/// The lines on which `programs/triangle-rows.s`'s rectangle found its cache entry kept, first to last, by shape and
/// column.
fn kept_rows(m: &Machine) -> Vec<[(u32, u32); 4]> {
    let kept = |s: u32, c: u32| {
        let lines: Vec<u32> = (0..64).filter(|&l| m.board.ram_word(0x10_0000 + (s * 256 + c * 64 + l) * 4) & 0xFFFF == 0x8001).collect();
        (lines[0], lines[lines.len() - 1])
    };
    (0..7).map(|s| [0, 1, 2, 3].map(|c| kept(s, c))).collect()
}

/// D-41: a triangle's rows from its middle vertex's row down, then from above it up, as Beetle PSX draws all seven
/// shapes of `programs/triangle-rows.s` and amidog's CACHE BLOCK expects of its flat triangles and quads.
#[test]
fn a_triangles_rows_go_down_from_its_middle_vertex_then_up_as_the_referee_draws_them() {
    let m = run_for(include_str!("../programs/triangle-rows.s"), 30, Market::Americas);
    let flat_bottom = [(56, 63), (38, 63), (20, 63), (2, 63)];
    let flat_top = [(8, 63), (0, 53), (0, 35), (0, 17)];
    let low_left = [(39, 63), (3, 63), (1, 63), (41, 63)];
    let low_right = [(43, 63), (3, 63), (1, 63), (37, 63)];
    let high_left = [(5, 24), (1, 42), (1, 60), (41, 63)];
    let quad = [(8, 63), (8, 63), (8, 63), (2, 63)];
    assert_eq!(kept_rows(&m), [flat_bottom, flat_top, low_left, low_right, high_left, low_left, quad]);
}

// --- D-29: DICR's master flag and I_STAT's bit 3 (Diamond_Disputes.md D-29, Diamond_Records.md §16.3) ---

/// I_STAT's bit 3 is set by bit 31's 0-to-1 transition alone, a completion's or a write's, as psx-spx has it and the
/// referees of three lineages read all ten cases: a write that leaves the flag up sets nothing (cases 1 and 7).
#[test]
fn dicr_sets_i_stat_only_when_its_master_flag_rises() {
    let m = run(include_str!("../programs/dicr-edge.s"));
    assert_eq!(
        results(&m, 0x10_0000, 20),
        [8, 0xC0C0_0000, 0, 0xC0C0_0000, 0, 0x4040_0000, 8, 0xC0C0_0000, 0, 0xC0C0_0000, 0, 0x00C0_0000, 8, 0x80C0_8000, 0, 0x80C0_8000, 0, 0x00C0_0000, 8, 0xC0C0_0000]
    );
}

/// A handler that leaves a channel's flag up and writes DICR runs once, as on all five referees; before the rule it
/// ran without end, which is how the player's BIOS stopped at its first screen.
#[test]
fn a_dicr_write_with_a_flag_left_up_does_not_ask_again() {
    let m = run(include_str!("../programs/dicr-storm.s"));
    assert_eq!(results(&m, 0x10_0000, 3), [1, 0xC0C8_0000, 0], "the handler's runs, DICR and I_STAT's bit 3");
}

// --- D-43: a channel started beside channel 2's list (Diamond_Disputes.md D-43) ---

/// Channel 6, started while channel 2's list waits for the GPU or is between two of its nodes, runs at once whichever
/// DPCR ranks first, as the referees of three lineages have it: its table is written and its CHCR idle with the list
/// still moving.
#[test]
fn channel_6_runs_beside_a_list_that_waits_or_walks() {
    let m = run_for(include_str!("../programs/dma-beside.s"), 60, Market::Americas);
    for (case, what) in ["rectangles, channel 2 first", "rectangles, channel 6 first", "empty nodes, channel 2 first", "empty nodes, channel 6 first"].iter().enumerate() {
        let r = results(&m, 0x10_0000 + 32 * case as u32, 8);
        assert_eq!([r[0], r[2], r[3], r[4]], [0x0100_0401, 2, 0x00FF_FFFF, 0x0100_0401], "{what}: the list moving, channel 6 done and its table written");
        assert_eq!(r[5..], [2, 0x00FF_FFFF, 0x0000_0401], "{what}: once the list has ended");
    }
}

// --- A narrow store to a 32-bit register (Diamond_Native.md §11.2) ---

/// A byte or halfword store to a 32-bit register at an address not word-aligned writes the whole register, the store's
/// word shifted to its byte (psx-spx "(w32)"), so that `sb` to DICR's third byte sets the channels' interrupt enables.
#[test]
fn a_narrow_store_writes_its_register_shifted_to_its_byte() {
    let m = run(include_str!("../programs/narrow-stores.s"));
    assert_eq!(results(&m, 0x10_0000, 7), [0x0088_0000, 0x0044_0000, 0x0000_0B00, 0, 0x1234, 0x0500, 0x0012_0000]);
}

// ---- Stage 13, speed (Diamond_Speed.md) ---------------------------------------------------------------------------

/// The digests of `frames` plain frames, the pictures', the sound's and the final state's, and the machine.
fn plain_digests(mut m: Machine, frames: usize) -> ([u64; 3], Machine) {
    use emusen_native::ffi::StateMachine;
    let bytes = |h: u64, b: &[u8]| b.iter().fold(h, |h, &x| (h ^ x as u64).wrapping_mul(0x0000_0100_0000_01B3));
    let (mut picture, mut sound) = (0xcbf2_9ce4_8422_2325u64, 0xcbf2_9ce4_8422_2325u64);
    let mut chunk = vec![0i16; 1 << 16];
    for _ in 0..frames {
        m.advance();
        picture = bytes(picture, m.board.gpu.display.frame());
        let n = m.board.spu.out.drain(&mut chunk, usize::MAX);
        sound = chunk[..n].iter().fold(sound, |h, &s| bytes(h, &s.to_le_bytes()));
    }
    let mut state = vec![0; m.state_size()];
    let n = m.save_state(&mut state).unwrap();
    ([picture, sound, bytes(0xcbf2_9ce4_8422_2325, &state[..n])], m)
}

/// A machine's digests over `frames` plain frames with the exact levers on, which must equal the interpreter's with
/// them off, every `verify`th skip checked against the interpreter as it is made; the machine run with them.
fn levers_give_the_interpreters_digests(name: &str, frames: usize, verify: u64, make: impl Fn() -> Machine) -> Machine {
    let mut off = make();
    off.idle.on = false;
    let (plain, _) = plain_digests(off, frames);
    let mut on = make();
    on.idle.verify = verify;
    let (fast, m) = plain_digests(on, frames);
    eprintln!("{name}, {frames} frames: {:016X} {:016X} {:016X}, {} skips of {} turns", fast[0], fast[1], fast[2], m.idle.skips, m.idle.turns);
    assert_eq!(fast, plain, "{name}: the levers' run against the interpreter's");
    m
}

/// Each loop form of `idle.s` is skipped, and the run with the skipper equals the run without it word for word, in
/// state, picture and sound: the vblank waits, the timeouts that expire in a register and on the stack at turns either
/// side of the period, the DMA waits, and a wait on timer 2, which is never skipped.
#[test]
fn the_idle_skipper_leaves_the_machine_as_the_interpreter_does() {
    let on = levers_give_the_interpreters_digests("idle.s", 40, 1, || bare(include_str!("../programs/idle.s")));
    assert_eq!(on.board.ram_word(0xFFFF0), 0xD1A0_D00E, "the program finished");
    assert!(on.idle.skips > 500 && on.idle.turns > 1_000_000, "the loops were skipped: {} skips, {} turns", on.idle.skips, on.idle.turns);
}

/// The project's timing programs, whose figures are the console's to the cycle, give the interpreter's digests with
/// the levers on: delay loops, the timers' waits, the GTE's and the MDEC's.
#[test]
fn the_levers_leave_the_projects_timing_programs_as_the_interpreter_does() {
    for (name, source, frames) in [
        ("timers.s", include_str!("../programs/timers.s"), 60),
        ("timer-sync.s", include_str!("../programs/timer-sync.s"), 30),
        ("gte-loops.s", include_str!("../programs/gte-loops.s"), 10),
        ("draw-time.s", include_str!("../programs/draw-time.s"), 10),
        ("spu-voices.s", include_str!("../programs/spu-voices.s"), 20),
        ("exceptions.s", include_str!("../programs/exceptions.s"), 10),
    ] {
        levers_give_the_interpreters_digests(name, frames, 1, || bare(source));
    }
    levers_give_the_interpreters_digests("cd-read.s", 60, 1, || {
        let mut m = bare(include_str!("../programs/cd-read.s"));
        m.insert_disc(cd_disc());
        m
    });
}

/// The debugger's frame never skips, so each table armed must still give the plain run's digests, now the plain run's
/// with the skipper (Diamond_Debugger.md §6.1).
#[test]
fn every_table_armed_gives_the_plain_runs_digests_on_the_idle_program() {
    every_table_gives_the_plain_run("idle.s", 40, || bare(include_str!("../programs/idle.s")));
}

/// ps1-tests' release on the open kernel, its waits in PSn00bSDK's own loops.
#[test]
fn the_levers_leave_ps1_tests_as_the_interpreter_does() {
    let Some(corpus) = records::corpus() else { return };
    for (path, frames) in [("timers/timers.exe", 120), ("gpu/triangle/triangle.exe", 60), ("gpu/bandwidth/bandwidth.exe", 120), ("spu/test/test.exe", 60), ("mdec/movie/movie-15bit.exe", 60), ("dma/otc-test/otc-test.exe", 60), ("cpu/access-time/access-time.exe", 60), ("gte/test-all/test-all.exe", 60)] {
        levers_give_the_interpreters_digests(path, frames, 1, || mdec_program(&corpus, path, 0));
    }
}

/// One of the tester's discs on the open kernel, named by `EMUSEN_DIAMOND_DISC`: 600 frames with the levers and
/// without.
#[test]
fn the_levers_leave_a_disc_as_the_interpreter_does() {
    let Some((name, make)) = named_disc() else { return };
    levers_give_the_interpreters_digests(&name, 600, 1, make);
}

// --- D-46: a triangle's drawing time by its rows (Diamond_Disputes.md D-46) ---

/// A triangle is charged by the rows it draws, with no fixed setup: one of one pixel is ready within the cycles a 1x1
/// rectangle is, as Beetle PSX and SwanStation read it (10 and 25), and one of 128 rows within a fifth of theirs
/// (4,006 and 4,129); a setup of 876 GPU cycles had made each 556 cycles, and Tekken 3's fighters vanish.
#[test]
fn a_triangle_is_charged_by_its_rows() {
    let m = run(include_str!("../programs/draw-time.s"));
    let busy: Vec<u32> = results(&m, 0x10_0000, 38).iter().map(|w| (w >> 16) - (w & 0xFFFF)).collect();
    assert!(busy[26] <= 30 && busy[27] <= 50, "one pixel and legs of 8: {:?}", &busy[26..30]);
    assert!(busy[29] < 4129 * 6 / 5, "legs of 128: {}", busy[29]);
}

// ---- Speed, the second pass: runs (Diamond_Speed.md §13) -----------------------------------------------------------

/// A machine's digests over `frames` plain frames stepped in runs, which must equal those of the same frames stepped one
/// at a time, with the skipper off and with it on; every `verify`th run checked against its steps as it is made, and
/// every skip against the interpreter. The machine run with both levers.
fn runs_give_the_steps_digests(name: &str, frames: usize, verify: u64, make: impl Fn() -> Machine) -> Machine {
    let mut stepped = make();
    (stepped.idle.on, stepped.runs.on) = (false, false);
    let (plain, _) = plain_digests(stepped, frames);
    let mut run = make();
    (run.idle.on, run.runs.verify) = (false, verify);
    let (alone, alone_machine) = plain_digests(run, frames);
    assert_eq!(alone, plain, "{name}: stepped in runs against stepped one at a time");
    let mut both = make();
    (both.idle.verify, both.runs.verify) = (1, verify);
    let (fast, m) = plain_digests(both, frames);
    eprintln!("{name}, {frames} frames: {:016X} {:016X} {:016X}; {} runs of {} steps alone, {} of {} beside {} skips", fast[0], fast[1], fast[2], alone_machine.runs.made, alone_machine.runs.steps, m.runs.made, m.runs.steps, m.idle.skips);
    assert_eq!(fast, plain, "{name}: runs and skips against stepped one at a time");
    assert!(alone_machine.runs.steps > 2 * alone_machine.runs.made.max(1), "{name}: the frames were stepped in runs");
    m
}

/// The project's programs give the digests of their steps when stepped in runs: the loops games wait in, the timers'
/// figures, the GTE's and the GPU's timing, the SPU's voices, exceptions of every kind and a sector read from a disc.
#[test]
fn runs_leave_the_projects_programs_as_their_steps_do() {
    for (name, source, frames) in [
        ("idle.s", include_str!("../programs/idle.s"), 40),
        ("timers.s", include_str!("../programs/timers.s"), 60),
        ("timer-sync.s", include_str!("../programs/timer-sync.s"), 30),
        ("gte-loops.s", include_str!("../programs/gte-loops.s"), 10),
        ("draw-time.s", include_str!("../programs/draw-time.s"), 10),
        ("spu-voices.s", include_str!("../programs/spu-voices.s"), 20),
        ("exceptions.s", include_str!("../programs/exceptions.s"), 10),
        ("gte-fuzz.s", include_str!("../programs/gte-fuzz.s"), 10),
        ("mdec-timing.s", include_str!("../programs/mdec-timing.s"), 10),
    ] {
        runs_give_the_steps_digests(name, frames, 7, || bare(source));
    }
    runs_give_the_steps_digests("cd-read.s", 60, 7, || {
        let mut m = bare(include_str!("../programs/cd-read.s"));
        m.insert_disc(cd_disc());
        m
    });
}

/// A kernel whose putchar calls are tapped is stepped one instruction at a time, so that a call at the A0h or B0h
/// entry is met: a kernel image of this test's own, which calls both with a character, gives them to the TTY.
#[test]
fn a_tapped_kernels_putchar_calls_are_met_with_runs_on() {
    let source = "lui t1, 0x03E0\nori t1, t1, 0x0008\nsw t1, 0xA0(zero)\nsw zero, 0xA4(zero)\nsw t1, 0xB0(zero)\nsw zero, 0xB4(zero)\n\
        addiu t1, zero, 0x3C\naddiu a0, zero, 0x48\naddiu t2, zero, 0xA0\njalr t2\nnop\n\
        addiu t1, zero, 0x3D\naddiu a0, zero, 0x69\naddiu t2, zero, 0xB0\njalr t2\nnop\nloop: j loop\nnop";
    let words = emusen_mips::assemble(source, 0xBFC0_0000, &blackmoon_r3000::dialect(None)).expect("assembles");
    let mut rom = vec![0u8; 0x8_0000];
    for (i, w) in words.iter().enumerate() {
        rom[4 * i..4 * i + 4].copy_from_slice(&w.to_le_bytes());
    }
    for skipper in [false, true] {
        let media = Media { kind: diamond::media::Kind::Disc { raw: true }, market: None };
        let mut m = Machine::new(media, Config { market: Market::Americas, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None], &[], Some((&rom, Kernel::Player)));
        m.idle.on = skipper;
        m.advance();
        assert_eq!(&m.space(diamond::machine::TTY_ID).unwrap()[..3], b"Hi\0", "the skipper {skipper}");
        assert_eq!(m.runs.made, 0, "no run was made");
    }
}

/// ps1-tests' release on the open kernel, each side-loaded at the shell's entry, stepped in runs and one at a time.
#[test]
fn runs_leave_ps1_tests_as_their_steps_do() {
    let Some(corpus) = records::corpus() else { return };
    for (path, frames) in [("timers/timers.exe", 120), ("gpu/triangle/triangle.exe", 60), ("gpu/bandwidth/bandwidth.exe", 120), ("spu/test/test.exe", 60), ("mdec/movie/movie-15bit.exe", 60), ("dma/otc-test/otc-test.exe", 60), ("cpu/access-time/access-time.exe", 60), ("gte/test-all/test-all.exe", 60)] {
        runs_give_the_steps_digests(path, frames, 251, || mdec_program(&corpus, path, 0));
    }
}

/// One of the tester's discs on the open kernel, named by `EMUSEN_DIAMOND_DISC`: 600 frames stepped in runs and one at
/// a time.
#[test]
fn runs_leave_a_disc_as_its_steps_do() {
    let Some((name, make)) = named_disc() else { return };
    runs_give_the_steps_digests(&name, 600, 61, make);
}

/// One of the tester's discs on the open kernel, named by `EMUSEN_DIAMOND_DISC`: at every 50th of its first 1,500
/// frames a machine just made loads the state saved there, shows the picture saved, and continues as the first for
/// three frames, to the state (Diamond_Native.md §23). A single save point can fall where nothing is lost: the kit's,
/// at frame 300 of 600, met no woven picture on any of the discs.
#[test]
fn a_state_loads_as_it_was_saved_all_along_a_disc() {
    use emusen_native::ffi::StateMachine;
    let Some((name, make)) = named_disc() else { return };
    let whole = |m: &Machine| {
        let mut state = vec![0; m.state_size()];
        let n = m.save_state(&mut state).unwrap();
        state.truncate(n);
        state
    };
    let picture = |m: &Machine| m.board.gpu.display.frame().to_vec();
    let mut a = make();
    let (mut points, mut woven) = (0, 0);
    while a.frames < 1500 {
        a.advance();
        if a.frames % 50 != 0 {
            continue;
        }
        let at = a.frames;
        points += 1;
        woven += (a.board.gpu.display.height > 300) as u32;
        let mut b = make();
        b.load_state(&whole(&a)).unwrap();
        assert!(picture(&b) == picture(&a), "{name}: the picture a load shows at frame {at} is not the one saved");
        for _ in 0..3 {
            a.advance();
            b.advance();
            assert!(picture(&b) == picture(&a), "{name}: loaded at frame {at}, frame {}'s picture is not the first machine's", a.frames);
        }
        assert!(whole(&b) == whole(&a), "{name}: loaded at frame {at}, the state three frames on is not the first machine's");
    }
    eprintln!("{name}: {points} save points, {woven} of them in a woven picture");
}

// ---- The tester's games in play, the second step (Diamond_Players.md §14) ----------------------------------------

/// D-47: a primitive whose first vertex and the offset pass 1,023 together is drawn where its vertices fall modulo
/// 2,048, as Crash Bandicoot's ground at the right of its right buffer needs and DuckStation, SwanStation, Beetle PSX
/// and PCSX-Redux draw it; the triangle from the screen and the rectangle's wrap of D-23 are unchanged.
#[test]
fn a_primitive_past_the_offsets_eleven_bits_is_drawn_modulo_2048() {
    let m = run(include_str!("../programs/offset-fold.s"));
    assert_eq!(m.board.ram_word(0xFFFF0), 0xD1A0_D00E);
    let px = |x: u32, y: u32| (m.board.ram_word(0x10_0000 + (y * 128 + x - 896) / 2 * 4) >> (16 * (x & 1))) & 0xFFFF;
    assert_eq!((px(992, 16), px(1023, 16), px(1000, 30), px(991, 20)), (0x001F, 0x001F, 0x001F, 0), "the triangle from 286");
    assert_eq!((px(992, 64), px(1023, 64), px(1000, 78)), (0x001F, 0x001F, 0x001F), "the triangle from the screen");
    assert_eq!((px(968, 176), px(1023, 176), px(967, 176)), (0x03E0, 0x03E0, 0), "the line from 300");
    assert_ne!(px(1000, 115), 0, "the shaded triangle from 300");
}

// ---- E1, the accurate goldens (Diamond_Enhancements.md §6) --------------------------------------------------------

/// Whether this run records the goldens rather than checking them: `EMUSEN_DIAMOND_GOLDENS=record`, by the rule of
/// Diamond_Enhancements.md §6.3.
fn recording_goldens() -> bool {
    std::env::var("EMUSEN_DIAMOND_GOLDENS").is_ok_and(|v| v == "record")
}

/// The test observer attached, when `observed`, for the exactness harness (Diamond_Enhancements.md §7).
fn observe(m: &mut Machine, observed: bool) {
    if observed {
        m.attach(Box::new(diamond::layer::Tally::new()));
    }
}

/// What the test observer was told over the run, detached; none when it was not attached.
fn told(m: &mut Machine) -> Option<diamond::layer::Tally> {
    m.detach().map(|l| *(l as Box<dyn std::any::Any>).downcast::<diamond::layer::Tally>().unwrap())
}

/// A program's golden digests after `frames` frames: the pictures, the sound, the state, and VRAM and the TTY apart;
/// with the test observer attached when `observed`, detached before the state is taken, and what it was told.
fn program_digests(mut m: Machine, frames: usize, observed: bool) -> (String, Option<diamond::layer::Tally>) {
    observe(&mut m, observed);
    let mut fold = Fold::new();
    for _ in 0..frames {
        m.advance();
        fold.frame(&mut m);
    }
    let tally = told(&mut m);
    (end_digests(&m, &fold), tally)
}

/// The exactness harness's two runs against one golden file: plain, and with the test observer attached, each line
/// the recorded one; recording takes the plain run's. What the observer was told is summed and printed.
fn check_both(file: &str, header: &str, run: impl Fn(bool) -> Vec<(String, Option<diamond::layer::Tally>)>) {
    let plain: Vec<String> = run(false).into_iter().map(|l| l.0).collect();
    check_goldens(file, header, &plain);
    if recording_goldens() {
        return;
    }
    let observed = run(true);
    let told = observed.iter().filter_map(|l| l.1).fold(diamond::layer::Tally::new(), |a, t| diamond::layer::Tally { loads: a.loads + t.loads, stores: a.stores + t.stores, cop2: a.cop2 + t.cop2, copies: a.copies + t.copies, projections: a.projections + t.projections, from_cpu: a.from_cpu + t.from_cpu, from_dma: a.from_dma + t.from_dma, ..a });
    eprintln!("{file} with the test observer: {} loads, {} stores, {} coprocessor 2 moves, {} copies, {} projections, {} words to GP0 from the processor and {} by DMA", told.loads, told.stores, told.cop2, told.copies, told.projections, told.from_cpu, told.from_dma);
    assert!(told.loads > 0 && told.stores > 0, "{file}: the observer was told nothing");
    check_goldens(file, header, &observed.into_iter().map(|l| l.0).collect::<Vec<_>>());
}

/// The goldens' text against the file's, or the file written when recording: each line that differs named.
fn check_goldens(file: &str, header: &str, lines: &[String]) {
    let text = format!("{header}{}\n", lines.join("\n"));
    let path = golden_file(file);
    if recording_goldens() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &text).unwrap();
        eprintln!("recorded {} lines in {}", lines.len(), path.display());
        return;
    }
    let kept = std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("{} holds no goldens; record them by Diamond_Enhancements.md §6.3", path.display()));
    let kept: Vec<&str> = kept.lines().filter(|l| !l.starts_with('#')).collect();
    let made: Vec<&str> = text.lines().filter(|l| !l.starts_with('#')).collect();
    let differ: Vec<String> = made.iter().zip(&kept).filter(|(a, b)| a != b).map(|(a, b)| format!("  made {a}\n  kept {b}")).collect();
    assert!(differ.is_empty() && made.len() == kept.len(), "{file}: {} of {} lines differ ({} made, {} kept):\n{}", differ.len(), kept.len(), made.len(), kept.len(), differ.join("\n"));
}

/// E1's goldens of the project's programs: each of `programs/` for 60 frames, its digests those it gave on the accurate
/// path when the goldens were recorded (Diamond_Enhancements.md §6).
#[test]
fn the_projects_programs_give_their_goldens() {
    let programs = own_programs();
    check_both("programs.txt", "# The accurate goldens of programs/ (Diamond_Enhancements.md §6): name, frames, digests.\n", |observed| {
        programs
            .iter()
            .map(|(name, source)| {
                let (digests, told) = program_digests(golden_program(name, source), GOLDEN_PROGRAM_FRAMES, observed);
                (format!("{name}\t{GOLDEN_PROGRAM_FRAMES}\t{digests}"), told)
            })
            .collect()
    });
}

/// E1's goldens of ps1-tests' release on the open kernel: every program, its digests those it gave on the accurate path
/// when the goldens were recorded. WiseMan's gates grade the same runs' verdicts and pictures against the console's.
#[test]
fn ps1_tests_give_their_goldens() {
    let Some(corpus) = records::corpus() else { return };
    let programs = ps1_tests_programs(&corpus);
    check_both("ps1-tests.txt", "# The accurate goldens of ps1-tests' release build 158 on the open kernel (Diamond_Enhancements.md §6): path, frames, digests.\n", |observed| {
        programs
            .iter()
            .map(|path| {
                let (frames, market) = ps1_tests_run(path);
                let (digests, told) = program_digests(ps1_tests_machine(&corpus, path, market), frames, observed);
                (format!("{path}\t{frames}\t{digests}"), told)
            })
            .collect()
    });
}

/// A disc's golden digests: 3,000 frames played by its script with a blank card in slot 1, and at every 300th frame
/// the pictures and the sound folded from the start and the state then, the test observer detached for the state's
/// digest when `observed` and attached again; and what it was told.
fn disc_digests(mut m: Machine, script: &str, observed: bool) -> (Vec<String>, Option<diamond::layer::Tally>) {
    let (_, presses, sticks) = press_script(script);
    m.insert_card(0, None);
    observe(&mut m, observed);
    let mut fold = Fold::new();
    let mut out = Vec::new();
    while m.frames < GOLDEN_DISC_FRAMES {
        press(&mut m, &presses, &sticks);
        m.advance();
        fold.frame(&mut m);
        if m.frames % GOLDEN_DISC_ANCHOR == 0 {
            let layer = m.detach();
            out.push(format!("\t{}\tpicture {:016X}\tsound {:016X}\tstate {:016X}", m.frames, fold.picture, fold.sound, state_digest(&m)));
            if let Some(l) = layer {
                m.attach(l);
            }
        }
    }
    (out, told(&mut m))
}

/// E1's goldens of the eight bench discs, opt-in by `EMUSEN_DIAMOND_DISCS`, the folder holding the tester's copies:
/// each disc's digests at its anchors those it gave on the accurate path when the goldens were recorded.
#[test]
fn the_bench_discs_give_their_goldens() {
    let Some(folder) = std::env::var_os("EMUSEN_DIAMOND_DISCS") else {
        eprintln!("skipped: EMUSEN_DIAMOND_DISCS names no folder of the bench discs");
        return;
    };
    let folder = std::path::PathBuf::from(folder);
    check_both("discs.txt", "# The accurate goldens of the bench discs on the open kernel (Diamond_Enhancements.md §6): each disc, its press script and anchor, then\n# at every 300th frame the pictures and the sound folded from the start and the state then.\n", |observed| {
        let mut lines = Vec::new();
        for (file, script) in GOLDEN_DISCS {
            let (anchor, ..) = press_script(script);
            let make = disc_maker(folder.join(file));
            lines.push((format!("{file}\t{script}\tanchor {anchor}"), None));
            let (digests, told) = disc_digests(make(), script, observed);
            if let Some(t) = told {
                eprintln!("{file}: {} projections, {} words to GP0 by DMA, {} of {} moves told", t.projections, t.from_dma, t.loads + t.stores + t.cop2 + t.copies, t.total());
            }
            lines.extend(digests.into_iter().map(|d| (d, None)));
            lines.last_mut().unwrap().1 = told;
        }
        lines
    });
}
