//! The project's own test programs in `programs/`, assembled with `emusen-mips` and run with no kernel: each one's
//! results as Diamond reads the documents, and as the witnesses of Diamond_Native.md §11.13 were seen to answer, or as
//! a console's log shows.

use diamond::exe;
use diamond::machine::{Config, Gpu, Machine};
use diamond::media::{Market, Media};

fn run(source: &str) -> Machine {
    let words = emusen_mips::assemble(source, 0x8001_0000, &blackmoon_r3000::dialect(Some(&blackmoon_gte::COPROCESSOR))).expect("assembles");
    let image = exe::build(&words);
    let media = Media::read(&image).unwrap();
    let mut m = Machine::new(media, Config { market: Market::Americas, gpu: Gpu::Pin208 }, [Some("psx.dualshock"), None], &image, None);
    for _ in 0..10 {
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
