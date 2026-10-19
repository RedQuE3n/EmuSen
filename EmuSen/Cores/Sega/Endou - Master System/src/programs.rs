//! The stage 2 test programs (Endou_Native.md §9.8): Z80 programs assembled with Beryl's assembler, each run on the
//! machine to pin one rule of the bus, the boards, the I/O chip or the interrupts by what the program reads back, and
//! ZEXALL's Master System port where the corpus is present.

#[path = "../programs/images.rs"]
pub mod images;

use images::image;

use crate::board::{Board, Mapper, Why};
use crate::io::{BUTTON_1, DOWN, PAUSE, RESET, RIGHT, UP};
use crate::machine::Machine;
use crate::media::{Console, Market, Media};
use crate::model::Model;

fn machine_on(rom: Vec<u8>, console: Console, model: Model, board: Board) -> Machine {
    let media = Media { console, skipped: 0, header: None };
    Machine::with_board(media, if model == Model::MarkIII { Market::Japan } else { Market::Americas }, model, board, rom)
}

fn machine(rom: Vec<u8>, model: Model) -> Machine {
    let console = if model == Model::Sg1000 { Console::Sg1000 } else { Console::Sms };
    let board = crate::board::identify(&rom, console);
    machine_on(rom, console, model, board)
}

fn frames(m: &mut Machine, n: usize) {
    for _ in 0..n {
        m.advance();
    }
}

#[test]
fn every_program_assembles_into_its_image() {
    for p in images::PROGRAMS {
        let rom = image(p);
        assert_eq!(rom.len(), p.banks * 0x4000, "{}", p.name);
        let media = Media::read(&rom).unwrap();
        assert_eq!(media.console == Console::Sms, p.header.is_some(), "{}: a host takes it as the system it is for", p.name);
    }
}

#[test]
fn the_pads_reset_button_and_th_lines_read_through_the_joystick_ports() {
    for model in [Model::Sms1, Model::Sms2, Model::MarkIII, Model::Sg1000] {
        let mut m = machine(image(&images::PADS), model);
        m.pads = [(UP | BUTTON_1 | RESET) as u32, (DOWN | RIGHT) as u32];
        frames(&mut m, 1);
        let reset = if model == Model::Sms1 { 0 } else { 0x10 };
        let high = if model == Model::Sg1000 { 0xF0 } else { 0xC0 | 0x20 | reset };
        assert_eq!((m.bus.ram[0], m.bus.ram[1]), (0b0110_1110, high | 0b1101), "{model:?}");
    }
}

#[test]
fn a_pause_press_is_one_nmi_and_a_held_button_no_more() {
    for model in [Model::Sms2, Model::Sg1000] {
        let mut m = machine(image(&images::PAUSE), model);
        frames(&mut m, 2);
        m.pads[0] = PAUSE as u32;
        frames(&mut m, 3);
        assert_eq!(m.bus.ram[0], 1, "{model:?}: held over three frames");
        m.pads[0] = 0;
        frames(&mut m, 1);
        m.pads[0] = PAUSE as u32;
        frames(&mut m, 1);
        assert_eq!(m.bus.ram[0], 2, "{model:?}");
        assert!(!m.cpu.regs.iff1 && !m.cpu.regs.iff2, "RETN restores IFF2, which DI cleared");
    }
}

#[test]
fn the_region_reads_back_as_each_console_s() {
    for (model, japanese) in [(Model::Sms1, 0), (Model::Sms2, 0), (Model::MarkIII, 1)] {
        let mut m = machine(image(&images::REGION), model);
        frames(&mut m, 1);
        assert_eq!(m.bus.ram[0], japanese, "{model:?}");
    }
}

#[test]
fn sega_s_mapper_banks_slots_under_a_program_and_keeps_its_first_kilobyte() {
    let mut m = machine(image(&images::SEGA), Model::Sms2);
    assert_eq!(m.bus.cart.board.mapper, Mapper::Sega);
    frames(&mut m, 1);
    assert_eq!(&m.bus.ram[..9], [5, 6, 7, 0xF3, 1, 33, 0x77, 0x00, 1], "banks, the fixed first kilobyte, the wrap, RAM's copy, cartridge RAM in both banks");
    assert_eq!(m.bus.cart.ram[0], 0x77);
}

#[test]
fn codemasters_mapper_banks_by_its_slots_addresses_and_maps_its_ram() {
    let rom = image(&images::CODEMASTERS);
    let mut m = machine(rom, Model::Sms2);
    assert_eq!(m.bus.cart.board.mapper, Mapper::Codemasters);
    frames(&mut m, 1);
    assert_eq!(&m.bus.ram[..6], [5, 3, 0x5A, 0x00, 0x00, 4], "slot 2 then 1 banked; RAM over $A000-$BFFF with $8000 still ROM, slot 1 bank 4");
}

#[test]
fn port_3e_takes_the_cartridge_and_the_ram_off_the_bus() {
    for (model, off, port) in [(Model::Sms1, 0x01, 0x3E), (Model::Sms2, 0xFF, 0xFF), (Model::MarkIII, 0x01, 0x3E)] {
        let rom = image(&images::PORT_3E);
        let mut m = machine(rom, model);
        frames(&mut m, 1);
        assert_eq!(&m.bus.ram[..4], [off, 0x42, 0x00, port], "{model:?}: the cartridge off, then on; a write to RAM switched off is lost; port $3E's read");
    }
}

#[test]
fn an_sg_1000_ram_board_mirrors_its_ram_through_the_slot() {
    let rom = image(&images::SG_RAM);
    let board = Board { mapper: Mapper::SgRam, ram: 0x800, battery: false, why: Why::Table };
    let mut m = machine_on(rom, Console::Sg1000, Model::Sg1000, board);
    frames(&mut m, 1);
    assert_eq!(&m.bus.ram[..2], [0x5A, 0x5A], "2 KiB at $8000 mirrored; 1 KiB of work RAM mirrored");
}

#[test]
fn the_psg_is_written_through_its_ports_and_the_vdp_s_counter_and_status_read() {
    let mut m = machine(image(&images::PSG), Model::Sms2);
    let fresh = m.bus.psg.clone();
    frames(&mut m, 1);
    assert_ne!(m.bus.psg, fresh);
    assert_eq!(&m.bus.ram[..2], [0xFF, 0x1F], "the frame's first line reads $FF; no flag, and the undriven bits high");
}

/// ZEXDOC and ZEXALL for the Master System (`Endou_References.md` §2), found through `EMUSEN_ENDOU_CORPUS`
/// (`~/.cache/emusen/probe/endou`), report in the cartridge's RAM: every one of the 79 tests "OK" and the run complete.
/// Without the corpus the test passes unrun. Run it in release: about twenty seconds each (Endou_Native.md §9.9).
#[test]
fn zexdoc_and_zexall_pass_on_the_machine() {
    let Some(root) = std::env::var_os("EMUSEN_ENDOU_CORPUS") else { return };
    for name in ["zexdoc.sms", "zexall.sms"] {
        let path = std::path::Path::new(&root).join("roms/ZEXALL-SMS/ZEXALL-SMS-0.20").join(name);
        let image = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let mut m = machine(image, Model::Sms2);
        let report = |m: &Machine| m.bus.cart.ram.iter().map(|&b| if b == 13 { '\n' } else { b as char }).collect::<String>();
        let mut n = 0;
        while !report(&m).contains("Tests complete") {
            frames(&mut m, 600);
            n += 600;
            assert!(n < 400_000, "{name} did not finish:\n{}", report(&m));
        }
        let text = report(&m);
        assert_eq!(text.lines().filter(|l| l.trim_end().ends_with(" OK")).count(), 79, "{name}:\n{text}");
        assert!(!text.contains("expected"), "{name}:\n{text}");
    }
}

/// The frame's picture as FNV-1a over its RGBA bytes.
fn digest(m: &Machine) -> u64 {
    m.frame_bytes().iter().fold(0xCBF2_9CE4_8422_2325, |h, &b| (h ^ b as u64).wrapping_mul(0x100_0000_01B3))
}

fn pixel(m: &Machine, x: usize, y: usize) -> [u8; 3] {
    let i = (y * 256 + x) * 4;
    [m.picture[i], m.picture[i + 1], m.picture[i + 2]]
}

// The picture equals Genesis Plus GX's, PicoDrive's and SMS Plus GX's up to a colour map, measured 2026-10-19 (Endou_Native.md §10.8).
#[test]
fn a_program_s_picture_is_its_tiles_sprite_scroll_mask_and_mid_frame_colour() {
    for model in [Model::Sms1, Model::Sms2] {
        let mut m = machine(image(&images::PICTURE), model);
        frames(&mut m, 10);
        assert_eq!((m.height, digest(&m)), (192, 0xFDEA_985F_7BF4_6025), "{model:?}");
        let (grey, blue, red, green, white) = ([85, 85, 85], [0, 0, 255], [255, 0, 0], [0, 255, 0], [255, 255, 255]);
        assert_eq!((pixel(&m, 0, 0), pixel(&m, 7, 98), pixel(&m, 0, 100)), (grey, grey, blue), "the masked column, its colour changed by line 100");
        assert_eq!((pixel(&m, 8, 0), pixel(&m, 11, 0), pixel(&m, 15, 0)), (green, red, green), "tile 1 scrolled right by 3");
        assert_eq!((pixel(&m, 100, 80), pixel(&m, 107, 87), pixel(&m, 108, 80), pixel(&m, 100, 79)), (white, white, red, red), "the sprite at Y 79 + 1");
    }
}

// The latch's dot is the one FluBBa's VDP test measures (Beryl_SmsVdp.md §3.2); without the corpus this holds it.
#[test]
fn the_h_counter_is_latched_as_th_rises_through_port_3f_and_not_as_it_falls() {
    for model in [Model::Sms1, Model::Sms2, Model::MarkIII] {
        let mut m = machine(image(&images::H_LATCH), model);
        frames(&mut m, 2);
        assert_eq!(&m.bus.ram[..2], [0x2C, 0x2C], "{model:?}");
    }
}

// The rule of Endou_Disputes.md D-8: no access is lost, whatever its rate; the references agree.
#[test]
fn data_port_writes_at_every_rate_land() {
    let mut m = machine(image(&images::SLOTS), Model::Sms2);
    frames(&mut m, 5);
    assert_eq!(m.bus.ram[0x100], 0x5A);
    for region in 0..4 {
        assert!(m.bus.ram[region * 64..region * 64 + 64].iter().enumerate().all(|(i, &b)| b == i as u8 + 1), "region {region}");
    }
}

// P9 of Endou_Plan.md: a run that skips the picture on alternate frames ends in the state of one that draws every frame.
#[test]
fn skipping_rendering_leaves_the_state_as_drawing() {
    use emusen_native::ffi::StateMachine;
    let save = |m: &Machine| {
        let mut s = vec![0; m.state_size()];
        m.save_state(&mut s).unwrap();
        s
    };
    let mut a = machine(image(&images::PICTURE), Model::Sms2);
    let mut b = machine(image(&images::PICTURE), Model::Sms2);
    for f in 0..30 {
        b.skip = f % 2 == 0;
        a.advance();
        b.advance();
    }
    assert_eq!(save(&a), save(&b));
}

/// FluBBa's SMS VDP Test 1.31 (`Endou_References.md` §2), found through `EMUSEN_ENDOU_CORPUS`: its three pages of the
/// Master System's tests, run by pressing button 1, keep each test's verdict in RAM at $C900 on, 0 for "Ok" (and the V
/// counter's, $C914, 2 for NTSC or 1 for PAL). Every one passes on each model and in PAL. Passes unrun without the corpus.
#[test]
fn flubba_s_vdp_test_passes_on_every_model() {
    let Some(root) = std::env::var_os("EMUSEN_ENDOU_CORPUS") else { return };
    let path = std::path::Path::new(&root).join("roms/SMSVDPTest-SMS/SMSVDPTest-SMS-1.31/VDPTEST.sms");
    let rom = std::fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    for (model, market) in [(Model::Sms1, Market::Americas), (Model::Sms2, Market::Americas), (Model::Sms2, Market::Europe), (Model::MarkIII, Market::Japan)] {
        let board = crate::board::identify(&rom, Console::Sms);
        let media = Media { console: Console::Sms, skipped: 0, header: None };
        let mut m = Machine::with_board(media, market, model, board, rom.clone());
        for f in 0..1500 {
            m.pads[0] = if [120, 300, 700, 1100].iter().any(|&p| (p..p + 4).contains(&f)) { BUTTON_1 as u32 } else { 0 };
            m.advance();
        }
        let verdicts = &m.bus.ram[0x900..0x93C];
        let failed: Vec<usize> = (1..=60).filter(|&t| !matches!(t, 20 | 37..=40) && t != 21 && verdicts[t - 1] != 0).collect();
        assert!(failed.is_empty(), "{model:?} {market:?}: tests {failed:?} failed");
        assert_eq!(verdicts[20], if market == Market::Europe { 1 } else { 2 }, "{model:?} {market:?}: the V counter's sequence");
    }
}
