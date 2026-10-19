//! Stage 6's tests (Endou_OpenBoot.md §7): the BIOS's slot on the bus, the open boot program's hand-over and export
//! check, the player's own image with a stand-in boot ROM, and the console started empty.

use beryl_z80::asm::assemble;
use emusen_native::core::{Core, Create, FirmwareSource, MachineInfo, Settings, status, sys};

use crate::boot::{BIOS_SIZE, Boot, OPEN_BOOT, RAM_LEN, WAIT};
use crate::machine::{BIOS_ID, Machine, ROM_ID};
use crate::media::{SIGNATURE, checksum};
use crate::model::Model;
use crate::programs::images::{self, image, with_codemasters_header};
use crate::v1::{BIOS_PLAYER, BIOS_SMS1, BIOS_SMS2, make_empty};

fn request<'a>(image: &'a [u8], settings: &[(&str, &str)], files: Vec<emusen_native::abi::File<'a>>) -> Create<'a> {
    let settings = Settings::from_pairs(settings.iter().map(|&(k, v)| (k.to_owned(), v.to_owned())).collect());
    Create { image, settings, files, pixel_formats: 1, host_abi_version: sys::ABI_VERSION, shared_removable: false, shared_console: false, device: None }
}

fn create(image: &[u8], settings: &[(&str, &str)]) -> Machine {
    Machine::create(&request(image, settings, Vec::new())).unwrap()
}

/// A cartridge of `size` bytes: `program` at $0000, each bank's number at $3F00, and Sega's header at $7FF0 with
/// `region`, `code` and the sum the console reads (`sum`).
fn cartridge(size: usize, program: &str, region: u8, code: u8, sum: fn(&[u8]) -> u16) -> Vec<u8> {
    let mut rom = vec![0u8; size];
    for b in 0..size / 0x4000 {
        rom[b * 0x4000 + 0x3F00] = b as u8;
    }
    let code_bytes = assemble(program, 0).unwrap().bytes;
    rom[..code_bytes.len()].copy_from_slice(&code_bytes);
    rom[0x7FF0..0x7FF8].copy_from_slice(SIGNATURE);
    rom[0x7FFF] = region << 4 | code;
    let s = sum(&rom);
    rom[0x7FFA..0x7FFC].copy_from_slice(&s.to_le_bytes());
    rom
}

/// The sum over the size code's range less the header, as Sega's cartridges state it (Endou_OpenBoot.md §1.6).
fn linear(rom: &[u8]) -> u16 {
    checksum(rom, 0x7FF0, rom[0x7FFF] & 0xF)
}

/// A program that keeps what the console handed it: $C000 at $C010, the BIOS's mark at $C100 at $C011, then loops.
const KEEPER: &str = "        di\n        ld a,($C000)\n        ld ($C010),a\n        ld a,($C100)\n        ld ($C011),a\nloop:   jr loop\n";

/// Steps until the game is handed over (port $3E names the cartridge and the PC is $0000), at most `frames` frames.
fn until_handed_over(m: &mut Machine, frames: u64) -> bool {
    let limit = m.frame_clocks() * frames;
    while m.bus.clock < limit {
        m.cpu.step(&mut m.bus);
        if m.cpu.regs.pc == 0 && m.bus.memory_control == 0xAB {
            return true;
        }
    }
    false
}

fn frames(m: &mut Machine, n: usize) {
    for _ in 0..n {
        m.advance();
    }
}

/// The name table's row as text, the font's tile n being the character 32 + n.
fn row(m: &Machine, r: usize) -> String {
    m.bus.vdp.vram[0x3800 + r * 64..0x3800 + r * 64 + 64].chunks(2).map(|e| (e[0].wrapping_add(32)) as char).collect::<String>().trim().to_owned()
}

fn screen(m: &Machine) -> Vec<String> {
    (9..17).map(|r| row(m, r)).filter(|l| !l.is_empty()).collect()
}

/// Sega's power-on VDP registers (Sega's Mark III manual, "VDP register description").
const POWER_ON: [u8; 11] = [0x36, 0xA0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFB, 0x00, 0x00, 0x00, 0xFF];

// The image's bytes are pinned, so that a change to the program is a change seen; its SHA-256 is in Endou_Native.md §13.
#[test]
fn the_open_boot_program_is_8_kib_and_its_bytes_are_pinned() {
    assert_eq!(OPEN_BOOT.len(), BIOS_SIZE);
    let fnv = OPEN_BOOT.iter().fold(0xCBF2_9CE4_8422_2325u64, |h, &b| (h ^ b as u64).wrapping_mul(0x100_0000_01B3));
    assert_eq!(fnv, 0xC16C_9C33_CA12_3EA4, "{fnv:#018X}");
    assert_eq!(&OPEN_BOOT[..4], [0xF3, 0xED, 0x56, 0x31], "DI, IM 1, the stack");
    assert_eq!((OPEN_BOOT[0x66], OPEN_BOOT[0x67]), (0xED, 0x45), "RETN at the NMI's vector");
}

// Endou_OpenBoot.md §1.3-§1.4, §2: the state a game finds, on each console with a BIOS, NTSC and PAL.
#[test]
fn the_open_boot_hands_a_cartridge_over_in_the_state_sega_s_manual_gives() {
    let rom = cartridge(0x8000, KEEPER, 4, 0xC, linear);
    for (settings, model, lines) in [(&[][..], Model::Sms2, 262u64), (&[("model", "sms1")][..], Model::Sms1, 262), (&[("region", "sms.eu")][..], Model::Sms2, 313)] {
        let mut m = create(&rom, settings);
        assert_eq!((m.model(), m.boot, m.bus.memory_control), (model, Boot::Open, 0xE3), "{model:?}: the BIOS alone at reset");
        assert!(until_handed_over(&mut m, 60), "{model:?}");
        let frame = m.frame_clocks();
        assert!(m.bus.clock % frame < 120, "{model:?}: handed over on the last line, {} clocks into it", m.bus.clock % frame);
        assert_eq!(m.bus.vdp.line as u64, lines - 1);
        assert_eq!((m.bus.ram[0], m.bus.io.control, m.bus.vdp.regs), (0xAB, 0xA5, POWER_ON), "{model:?}");
        assert_eq!((m.cpu.regs.im, m.cpu.regs.iff1, m.cpu.regs.sp, m.cpu.regs.af >> 8), (1, false, 0xDFF0, 0xAB));
        assert!(!m.bus.vdp.frame_flag && !m.bus.vdp.line_flag, "no interrupt pending");
        assert!(m.bus.vdp.vram.iter().all(|&b| b == 0) && m.bus.vdp.cram[..32].iter().all(|&b| b == 0), "VRAM and CRAM cleared");
        let program = 0x700..0x700 + RAM_LEN;
        assert!(m.bus.ram.iter().enumerate().all(|(i, &b)| i == 0 || program.contains(&i) || b == 0), "RAM clear but $C000 and the program at $C700");
        assert_eq!(m.bus.ram[0x700..0x703], OPEN_BOOT[crate::boot::RAM_AT..crate::boot::RAM_AT + 3], "the program left at $C700");
        for _ in 0..20 {
            m.cpu.step(&mut m.bus);
        }
        assert_eq!(m.bus.ram[0x10], 0xAB, "{model:?}: the game reads $C000");
    }
}

// The open boot program takes a quarter second over a 32 KiB check; a 256 KiB one pages slot 2 and puts it back.
#[test]
fn a_cartridge_past_32_kib_is_summed_through_slot_2_and_slot_2_is_put_back() {
    let rom = cartridge(0x40000, KEEPER, 4, 0x0, linear);
    let mut m = create(&rom, &[]);
    assert!(until_handed_over(&mut m, 300));
    assert_eq!((m.bus.cart.regs, m.bus.ram[0x1FFF]), ([0, 0, 1, 2], 2), "slot 2 at bank 2 again, RAM keeping the write");
    let small = cartridge(0x8000, KEEPER, 4, 0xC, linear);
    let mut s = create(&small, &[]);
    assert!(until_handed_over(&mut s, 60));
    assert_eq!(s.bus.ram[0x1FFF], 0, "a 32 KiB check pages nothing");
    let (big, small) = (m.bus.clock / m.frame_clocks(), s.bus.clock / s.frame_clocks());
    assert!(small <= 30 && big <= 130, "{small} and {big} frames");
}

// A seeded fault that summed the header where it lies inside the range passed every cartridge with its header at $7FF0,
// the range's end (Endou_Native.md §13.9): a header at $3FF0 or $1FF0 splits the sum in two.
#[test]
fn a_header_inside_the_range_is_left_out_of_the_sum() {
    for (at, code, size) in [(0x3FF0usize, 0xCu8, 0x8000usize), (0x1FF0, 0xB, 0x4000), (0x3FF0, 0xF, 0x20000)] {
        let mut rom = vec![0u8; size];
        let code_bytes = assemble(KEEPER, 0).unwrap().bytes;
        rom[..code_bytes.len()].copy_from_slice(&code_bytes);
        for (i, b) in rom.iter_mut().enumerate().skip(0x100) {
            *b = ((i * 7) >> 3) as u8;
        }
        rom[at..at + 8].copy_from_slice(SIGNATURE);
        rom[at + 15] = 0x40 | code;
        let sum = checksum(&rom, at, code);
        rom[at + 10..at + 12].copy_from_slice(&sum.to_le_bytes());
        let mut m = Machine::create_as("sms", &request(&rom, &[], Vec::new())).unwrap();
        assert!(until_handed_over(&mut m, 200), "header at {at:#06X}, code {code:X}: {:?}", screen(&m));
    }
}

// Micro Machines states the sum read through its own board, slot 2 showing bank 0 to every write of $FFFF (Endou_OpenBoot.md §1.6).
#[test]
fn a_codemasters_board_is_summed_as_the_console_reads_it() {
    let through = |rom: &[u8]| {
        let first: u16 = rom[..0x7FF0].iter().fold(0u16, |s, &b| s.wrapping_add(b as u16));
        let bank0: u16 = rom[..0x4000].iter().fold(0u16, |s, &b| s.wrapping_add(b as u16));
        (2..rom.len() / 0x4000).fold(first, |s, _| s.wrapping_add(bank0))
    };
    let build = |sum: fn(&[u8]) -> u16| {
        let mut rom = with_codemasters_header(cartridge(0x20000, KEEPER, 4, 0xF, linear));
        let s = sum(&rom);
        rom[0x7FFA..0x7FFC].copy_from_slice(&s.to_le_bytes());
        rom
    };
    let mut m = create(&build(through), &[]);
    assert_eq!(m.bus.cart.board.mapper, crate::board::Mapper::Codemasters);
    assert!(until_handed_over(&mut m, 200), "{:?}", screen(&m));
    let mut wrong = create(&build(linear), &[]);
    frames(&mut wrong, 200);
    assert_eq!(screen(&wrong)[2..], ["ITS CHECKSUM DOES NOT MATCH", "ITS DATA."], "the image's own banks are not what the console reads");
}

// The export check refuses in words (Endou_OpenBoot.md §3.3), on both consoles, and waits with the ROM on.
#[test]
fn the_export_check_refuses_in_words() {
    let headerless = assemble(KEEPER, 0).unwrap().bytes.into_iter().chain(std::iter::repeat(0x5A)).take(0x8000).collect::<Vec<u8>>();
    let mut bad_sum = cartridge(0x8000, KEEPER, 4, 0xC, linear);
    bad_sum[0x7FFA] ^= 1;
    let cases: [(Vec<u8>, &[&str]); 5] = [
        (cartridge(0x8000, KEEPER, 3, 0xC, linear), &["THE GAME CANNOT START.", "IT IS FOR A JAPANESE CONSOLE."]),
        (headerless, &["THE GAME CANNOT START.", "IT HAS NO SEGA HEADER.", "IT MAY BE A JAPANESE GAME."]),
        (cartridge(0x8000, KEEPER, 6, 0xC, linear), &["THE GAME CANNOT START.", "IT IS NOT FOR THIS CONSOLE."]),
        (cartridge(0x8000, KEEPER, 4, 0x3, linear), &["THE GAME CANNOT START.", "ITS HEADER'S SIZE CODE IS", "NOT ONE SEGA LISTS."]),
        (bad_sum, &["THE GAME CANNOT START.", "ITS CHECKSUM DOES NOT MATCH", "ITS DATA."]),
    ];
    for model in ["sms1", "sms2"] {
        for (rom, words) in &cases {
            let mut m = Machine::create_as("sms", &request(rom, &[("model", model)], Vec::new())).unwrap();
            frames(&mut m, 60);
            let mut want = vec!["ENDOU OPEN BOOT".to_owned()];
            want.extend(words.iter().map(|w| w.to_string()));
            assert_eq!(screen(&m), want, "{model}");
            assert_eq!((m.bus.memory_control, m.cpu.regs.pc), (0xE3, WAIT + 1), "{model}: halted in the ROM's wait");
            assert!(m.bus.vdp.regs[1] & 0x40 != 0, "the display on");
        }
    }
}

// A Japanese cartridge runs on the Mark III by default, which has no BIOS; on an export console it is refused.
#[test]
fn a_japanese_cartridge_starts_on_the_mark_iii_and_is_refused_on_an_export_console() {
    let rom = cartridge(0x8000, KEEPER, 3, 0xC, linear);
    let mark3 = create(&rom, &[]);
    assert_eq!((mark3.model(), mark3.boot, mark3.bus.memory_control), (Model::MarkIII, Boot::None, 0xAB));
    assert!(mark3.machine_info().firmware.is_empty());
    let mut sms2 = create(&rom, &[("model", "sms2")]);
    frames(&mut sms2, 60);
    assert_eq!(row(&sms2, 15), "IT IS FOR A JAPANESE CONSOLE.");
}

/// A stand-in for a console's own BIOS, written for the tests: it marks $C100, copies a two-instruction hand-over to
/// RAM and runs it, as a boot ROM's entry and hand-over do (Endou_OpenBoot.md §5). It is no BIOS's code.
fn stand_in() -> Vec<u8> {
    let source = "        di\n        im 1\n        ld sp,$DFF0\n        ld a,$5C\n        ld ($C100),a\n        ld hl,stub\n        ld de,$C700\n        ld bc,stub_end-stub\n        ldir\n        jp $C700\nstub:   ld a,$AB\n        ld ($C000),a\n        out ($3E),a\n        jp $0000\nstub_end:\n";
    let mut rom = assemble(source, 0).unwrap().bytes;
    rom.resize(BIOS_SIZE, 0xFF);
    rom
}

fn firmware(info: &MachineInfo) -> Vec<(u32, FirmwareSource)> {
    info.firmware.clone()
}

// Decided 2026-10-19: the player's image is an opt-in per console, the open boot program the default.
#[test]
fn the_player_s_bios_runs_when_chosen_and_given_and_the_open_program_otherwise() {
    let rom = cartridge(0x8000, KEEPER, 4, 0xC, linear);
    let bios = stand_in();
    for (model, which, key) in [("sms1", BIOS_SMS1, "bios_sms1"), ("sms2", BIOS_SMS2, "bios_sms2")] {
        let file = || vec![emusen_native::abi::File { which, data: &bios[..] }];
        let mut chosen = Machine::create(&request(&rom, &[("model", model), (key, BIOS_PLAYER)], file())).unwrap();
        assert_eq!((chosen.boot, firmware(&chosen.machine_info())), (Boot::Player, vec![(which, FirmwareSource::File)]), "{model}");
        assert_eq!(chosen.space(BIOS_ID).unwrap(), &bios[..]);
        frames(&mut chosen, 3);
        assert_eq!((chosen.bus.ram[0x10], chosen.bus.ram[0x11]), (0xAB, 0x5C), "{model}: the stand-in ran and handed over");
        let given = Machine::create(&request(&rom, &[("model", model)], file())).unwrap();
        assert_eq!((given.boot, firmware(&given.machine_info())), (Boot::Open, vec![(which, FirmwareSource::Replacement)]), "{model}: given but not chosen");
        let missing = create(&rom, &[("model", model), (key, BIOS_PLAYER)]);
        assert_eq!((missing.boot, missing.space(BIOS_ID).unwrap()), (Boot::Open, OPEN_BOOT), "{model}: chosen but not given");
    }
    let other = Machine::create(&request(&rom, &[("model", "sms2"), ("bios_sms1", BIOS_PLAYER)], vec![emusen_native::abi::File { which: BIOS_SMS1, data: &bios }])).unwrap();
    assert_eq!(other.boot, Boot::Open, "the first Master System's choice is not the second's");
    let short = vec![0u8; 0x1000];
    let refused = Machine::create(&request(&rom, &[], vec![emusen_native::abi::File { which: BIOS_SMS2, data: &short }]));
    assert_eq!(refused.err(), Some(status::BAD_FILE), "an image of another size");
}

// A state is held to its boot and its BIOS's image, as to its model.
#[test]
fn a_state_of_another_boot_is_foreign() {
    use emusen_native::ffi::{StateMachine, Status};
    let rom = cartridge(0x8000, KEEPER, 4, 0xC, linear);
    let bios = stand_in();
    let mut open = create(&rom, &[]);
    frames(&mut open, 3);
    let mut s = vec![0; open.state_size()];
    open.save_state(&mut s).unwrap();
    let mut player = Machine::create(&request(&rom, &[("bios_sms2", BIOS_PLAYER)], vec![emusen_native::abi::File { which: BIOS_SMS2, data: &bios }])).unwrap();
    assert_eq!(player.load_state(&s).unwrap_err().status(), status::FOREIGN);
    let mut again = create(&rom, &[]);
    again.load_state(&s).unwrap();
    assert_eq!((again.frames, again.cpu.regs.pc), (open.frames, open.cpu.regs.pc));
}

// Core ABI 1.11: the Master System starts empty into its BIOS; the open boot program says no game is in it.
#[test]
fn an_empty_master_system_shows_the_open_program_s_words() {
    assert!(Machine::starts_empty("sms") && !Machine::starts_empty("sg1000"));
    let mut m = make_empty(&request(&[], &[], Vec::new())).unwrap();
    assert_eq!((m.model(), m.boot, firmware(&m.machine_info())), (Model::Sms2, Boot::Open, vec![(BIOS_SMS2, FirmwareSource::Replacement)]));
    assert!(m.space(ROM_ID).is_none() && m.machine_info().spaces.iter().all(|s| s.name != "ROM"), "no cartridge, no ROM");
    frames(&mut m, 20);
    assert_eq!(screen(&m), ["ENDOU OPEN BOOT", "NO GAME IS IN THE CONSOLE."]);
    let mut sms1 = make_empty(&request(&[], &[("model", "sms1")], Vec::new())).unwrap();
    frames(&mut sms1, 20);
    assert_eq!(row(&sms1, 13), "NO GAME IS IN THE CONSOLE.", "the first Master System's floating bus is no game either");
    let japan = make_empty(&request(&[], &[("region", "sms.jp")], Vec::new())).unwrap();
    assert_eq!((japan.model(), japan.boot), (Model::MarkIII, Boot::None), "the Mark III has no BIOS to start into");
}

// Endou_OpenBoot.md §2, U1-U3: the BIOS alone at reset; with the cartridge too, the cartridge's byte on the first
// Master System and the two ANDed on the second; an 8 KiB BIOS mirrored through the slot area.
#[test]
fn the_bios_slot_shares_the_slot_area_as_each_console_s_does() {
    let rom: Vec<u8> = (0..0x8000).map(|i| (i >> 8) as u8 | 0x0F).collect();
    let bios: Vec<u8> = (0..BIOS_SIZE).map(|i| (i & 0xF0) as u8 | 0x03).collect();
    for (model, both) in [(Model::Sms1, 0x1F), (Model::Sms2, 0x13)] {
        let board = crate::board::identify(&rom, crate::media::Console::Sms);
        let media = crate::media::Media { console: crate::media::Console::Sms, skipped: 0, header: None };
        let mut m = Machine::with_boot(media, crate::media::Market::Americas, model, board, rom.clone(), Boot::Player, bios.clone());
        assert_eq!((m.bus.memory_control, m.bus.peek(0x1010), m.bus.peek(0x3010)), (0xE3, 0x13, 0x13), "{model:?}: the BIOS mirrored every 8 KiB");
        m.bus.memory_control = 0xA3;
        assert_eq!(m.bus.peek(0x1010), both, "{model:?}: both on");
        m.bus.memory_control = 0xAB;
        assert_eq!(m.bus.peek(0x1010), 0x1F, "{model:?}: the cartridge alone");
    }
}

// The gate (Endou_OpenBoot.md §7): each of Endou's Master System programs, booted by the open program on both export
// consoles, reaches the point it reaches with no BIOS: the RAM it keeps its results in, VRAM, CRAM and the picture. Two
// differences are the hand-over's (Endou_Native.md §13): $C000 holds $AB where a program does not write it, and the
// H counter `hlatch.s` latches is one step later, its poll loop meeting a start 76 clocks into the line.
#[test]
fn every_program_boots_to_the_point_it_reaches_with_no_bios() {
    for model in [Model::Sms1, Model::Sms2] {
        let mut differ = Vec::new();
        for p in images::PROGRAMS.iter().filter(|p| p.header.is_some()) {
            let rom = image(p);
            let board = crate::board::identify(&rom, crate::media::Console::Sms);
            let media = crate::media::Media::read(&rom).unwrap();
            let market = crate::media::Market::Americas;
            let mut plain = Machine::with_board(media.clone(), market, model, board, rom.clone());
            let mut booted = Machine::with_boot(media, market, model, board, rom.clone(), Boot::Open, OPEN_BOOT.to_vec());
            let mut at = 0;
            while booted.bus.memory_control != 0xAB {
                booted.advance();
                at += 1;
                assert!(at < 60, "{}: not handed over", p.name);
            }
            frames(&mut plain, 120);
            frames(&mut booted, 119);
            let ram: Vec<(usize, u8, u8)> = (0..0x100).filter(|&i| plain.bus.ram[i] != booted.bus.ram[i]).map(|i| (i, plain.bus.ram[i], booted.bus.ram[i])).collect();
            assert!(plain.bus.vdp.vram == booted.bus.vdp.vram && plain.bus.vdp.cram == booted.bus.vdp.cram, "{model:?} {}", p.name);
            assert!(plain.frame_bytes() == booted.frame_bytes(), "{model:?} {}: the picture", p.name);
            if !ram.is_empty() {
                differ.push((p.name, ram));
            }
        }
        assert_eq!(differ, [("picture", vec![(0, 0x00, 0xAB)]), ("hlatch", vec![(0, 0x2C, 0x2D), (1, 0x2C, 0x2D)]), ("voices", vec![(0, 0x00, 0xAB)])], "{model:?}");
    }
}
