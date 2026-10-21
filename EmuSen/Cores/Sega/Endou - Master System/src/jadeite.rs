//! Stage J2's tests (Jadeite_Native.md §9, Jadeite_OpenBoot.md §7): the Game Gear's system ports and stereo on the
//! machine, the EEPROM board's protocol through its register, the TH latch, port $3E's bits, and the open boot program
//! with the player's own image as a stand-in.

use beryl_z80::asm::assemble;
use emusen_native::core::{Core, Create, FirmwareSource, Settings, status, sys};

use crate::board::{Board, Mapper, Why};
use crate::boot::{Boot, GG_BOOT_SIZE, OPEN_BOOT_GG};
use crate::eeprom;
use crate::io::PAUSE;
use crate::machine::{BIOS_ID, Machine};
use crate::media::{Console, Market, Media};
use crate::model::Model;
use crate::programs::images::{self, Program, image};
use crate::v1::{BIOS_GG, BIOS_PLAYER};

fn on(p: &Program, model: Model, board: Board) -> Machine {
    let rom = image(p);
    let media = Media::read_as(&rom, Some(if p.gg { Console::Gg } else { Console::Sms })).unwrap();
    Machine::with_board(media, Market::Americas, model, board, rom)
}

fn sega(ram: usize) -> Board {
    Board { mapper: Mapper::Sega, ram, battery: false, why: Why::Size }
}

fn frames(m: &mut Machine, n: usize) {
    for _ in 0..n {
        m.advance();
    }
}

fn request<'a>(image: &'a [u8], settings: &[(&str, &str)], files: Vec<emusen_native::abi::File<'a>>) -> Create<'a> {
    let settings = Settings::from_pairs(settings.iter().map(|&(k, v)| (k.to_owned(), v.to_owned())).collect());
    Create { image, settings, files, pixel_formats: 1, host_abi_version: sys::ABI_VERSION, shared_removable: false, shared_console: false, device: None }
}

fn game_gear(image: &[u8], settings: &[(&str, &str)]) -> Machine {
    Machine::create_as("gg", &request(image, settings, Vec::new())).unwrap()
}

// Jadeite_Native.md §9.2, measured against the five references 2026-10-20: MacDonald's power-on reads, the pins, the
// serial status's byte time, the even ports not reaching $3E nor the odd $3F, and Start and the market at port $00.
#[test]
fn the_system_ports_answer_as_an_unlinked_unit_s() {
    for (model, market) in [(Model::GameGear, 0x40), (Model::JapaneseGameGear, 0x00)] {
        let mut m = on(&images::GG_IO, model, sega(0));
        assert!(m.bus.vdp.gg_mode);
        frames(&mut m, 5);
        let r = &m.bus.ram;
        assert_eq!(r[..7], [0x80 | market, 0x7F, 0xFF, 0x00, 0xFF, 0x00, 0xFF], "{model:?}: 'C0 7F FF 00 FF 00 FF' [R smstech §3]");
        assert_eq!((r[8], r[9], r[0xA], r[0xB], r[0xC]), (0x5F, 0x0F, 0xA5, 0x00, 0xD1), "{model:?}");
        let loops = u16::from_le_bytes([r[0xD], r[0xE]]) as u64;
        let byte = 10 * 3_579_545 / 300;
        assert!(loops.abs_diff(byte / 33) <= 2, "{model:?}: {loops} loops of 33 cycles for {byte} clocks");
        assert_eq!((r[0xF], r[0x11], m.bus.io.control, m.bus.memory_control), (0xFF, 0x5A, 0xFF, 0xA8), "{model:?}: no write reached $3E or $3F");
        assert_eq!(r[0x12], 0x80 | market, "{model:?}: Start released");
        m.pads[0] = PAUSE as u32;
        frames(&mut m, 1);
        assert_eq!((m.bus.ram[0x12], m.cpu.regs.pc < 0x66 || m.cpu.regs.pc > 0x70), (market, true), "{model:?}: Start is bit 7, no NMI");
    }
}

// SMS Power!'s "Game Gear Stereo", Sega's notes "I/O port 06H": port $06 sends each channel to its sides; the program puts
// every channel on the right and sounds tone 0, so the left is silent and the right a square wave.
#[test]
fn port_6_sends_the_psg_to_the_right_alone() {
    let mut m = on(&images::GG_IO, Model::GameGear, sega(0));
    frames(&mut m, 30);
    let mut out = vec![0i16; 1 << 16];
    let n = m.audio.drain(&mut out, usize::MAX);
    let tail = &out[n / 2..n];
    assert_eq!(m.bus.sound.psg.stereo, 0x0F);
    assert!(tail.chunks(2).all(|p| p[0] == 0), "the left is silent");
    let (lo, hi) = tail.chunks(2).fold((i16::MAX, i16::MIN), |(lo, hi), p| (lo.min(p[1]), hi.max(p[1])));
    assert!(lo <= 0 && hi >= 4000, "the right sounds: {lo} to {hi}");
}

// In a Master System cartridge's mode the Game Gear is a Master System II: ports $00-$06 read $FF and their writes are
// $3E's and $3F's (SMS Power!'s "I/O Port Map - Game Gear"); Start is the Pause button's NMI.
#[test]
fn a_master_system_cartridge_s_mode_has_none_of_the_system_ports() {
    let p = Program { header: Some(4), gg: false, ..images::GG_IO };
    let mut m = on(&p, Model::GameGear, sega(0));
    assert!(!m.bus.vdp.gg_mode);
    frames(&mut m, 1);
    assert_eq!(m.bus.ram[..7], [0xFF; 7]);
    assert_ne!(m.bus.io.control, 0xFF, "the odd ports reach $3F");
}

// Endou_Disputes.md D-25, decided 2026-10-20: the Game Gear's TH lines latch the H counter as the Master System II's do,
// and its port $3E's bits 7, 6, 5 and 2 do nothing [R `smstech` §4].
#[test]
fn th_latches_the_h_counter_and_port_3e_keeps_the_cartridge_and_the_io_on() {
    let mut m = on(&images::GG_IO, Model::GameGear, sega(0));
    m.bus.vdp.gg_mode = false;
    use beryl_z80::Bus as _;
    m.bus.clock = 1000;
    m.bus.output(0x3F, 0b1101_0101);
    let latched = m.bus.vdp.read_h();
    m.bus.clock = 1100;
    m.bus.output(0x3F, 0b1111_0101);
    assert_ne!(m.bus.vdp.read_h(), latched, "TH rising latched the counter again");
    m.bus.output(0x3E, 0xE4 | 0x08);
    assert_eq!((m.bus.peek(0x0000), m.bus.input(0xDC)), (image(&images::GG_IO)[0], 0xFF), "the cartridge and the pad still answer");
    m.bus.output(0x3E, 0x10 | 0x08);
    assert_eq!(m.bus.peek(0xC000), 0xFF, "bit 4 still switches the work RAM off");
}

/// The EEPROM board for the protocol program.
fn eeprom_board() -> Board {
    Board { mapper: Mapper::SegaEeprom, ram: eeprom::BYTES, battery: true, why: Why::Table }
}

// The 93C46B's data sheet through the board's register (D-26): EWEN's protection, WRITE, ERASE, WRAL and their status
// polled for the self-timed cycle's length, READ of one word and two in sequence, and EWDS.
#[test]
fn the_eeprom_s_protocol_runs_through_the_board_s_register() {
    let mut m = on(&images::GG_EEPROM, Model::GameGear, eeprom_board());
    frames(&mut m, 10);
    let w = |at: usize| u16::from_le_bytes([m.bus.ram[at], m.bus.ram[at + 1]]);
    assert_eq!(m.bus.ram[0x10], 1, "the program finished");
    let polls = |clocks: u64| clocks / 35;
    for (at, clocks) in [(0, eeprom::WRITE_CLOCKS), (4, eeprom::WRITE_CLOCKS), (8, eeprom::WRAL_CLOCKS)] {
        let got = w(at) as u64;
        assert!(got.abs_diff(polls(clocks)) <= 4, "status at ${at:02X}: {got} loops for {clocks} clocks");
    }
    assert_eq!((w(2), w(6), w(0xA), w(0xC), w(0xE)), (0x1234, 0xFFFF, 0x5AA5, 0x5AA5, 0x5AA5));
    assert!(m.bus.cart.ram.chunks(2).all(|c| c == [0xA5, 0x5A]), "every word $5AA5, low byte first");
    assert!(m.bus.cart.dirty && !m.bus.cart.eeprom.enabled);
}

// The EEPROM's 128 bytes are the battery file: given at create, kept by the board, and given back (EmuSen_CoreAPI.md's
// battery files), and its state continues mid-instruction.
#[test]
fn the_eeprom_is_a_battery_file_and_its_state_continues() {
    use emusen_native::ffi::StateMachine;
    let rom = image(&images::GG_EEPROM);
    let table = [crate::board::Row { crc: crate::board::crc32(&rom), mapper: Mapper::SegaEeprom, ram: eeprom::BYTES, battery: true }];
    let save: Vec<u8> = (0..128).map(|i| i as u8 ^ 0x5C).collect();
    let file = emusen_native::abi::File { which: 0, data: &save };
    let req = request(&rom, &[], vec![file]);
    let m = crate::v1::make_as(&req, &table, Some(Console::Gg)).unwrap();
    assert_eq!((m.battery(0).unwrap().0, m.machine_info().battery.len()), (&save[..], 1));
    let mut a = on(&images::GG_EEPROM, Model::GameGear, eeprom_board());
    while a.bus.cart.eeprom.phase != eeprom::Phase::Data {
        a.cpu.step(&mut a.bus);
    }
    let mut s = vec![0; a.state_size()];
    a.save_state(&mut s).unwrap();
    let mut b = on(&images::GG_EEPROM, Model::GameGear, eeprom_board());
    b.load_state(&s).unwrap();
    frames(&mut a, 10);
    frames(&mut b, 10);
    assert_eq!((&b.bus.ram[..0x12], &b.bus.cart.ram), (&a.bus.ram[..0x12], &a.bus.cart.ram));
}

/// A program that keeps what the console handed it: $C000 at $C010 and A at $C011, then reads Start forever.
const KEEPER: &str = "        ld ($C011),a\n        ld a,($C000)\n        ld ($C010),a\nloop:   jr loop\n";

fn keeper(region: u8, header: bool) -> Vec<u8> {
    let mut rom = vec![0u8; 0x8000];
    let code = assemble(KEEPER, 0).unwrap().bytes;
    rom[..code.len()].copy_from_slice(&code);
    if header {
        rom[0x7FF0..0x7FF8].copy_from_slice(crate::media::SIGNATURE);
        rom[0x7FFF] = region << 4 | 0xC;
    }
    rom
}

fn until_handed_over(m: &mut Machine, frames: u64) -> bool {
    let limit = m.frame_clocks() * frames;
    while m.bus.clock < limit {
        m.cpu.step(&mut m.bus);
        if m.cpu.regs.pc == 0 && m.bus.memory_control == 0xA8 {
            return true;
        }
    }
    false
}

/// The name table's row as text: tile n is the letter $40 + n, tile 0 a space, tile 27 the full stop.
fn row(m: &Machine, r: usize) -> String {
    let ch = |t: u8| match t {
        0 => ' ',
        27 => '.',
        t => (0x40 + t) as char,
    };
    m.bus.vdp.vram[0x3800 + r * 64..0x3800 + r * 64 + 64].chunks(2).map(|e| ch(e[0])).collect::<String>().trim().to_owned()
}

/// Sega's power-on VDP registers (Sega's Mark III manual, "VDP register description").
const POWER_ON: [u8; 11] = [0x36, 0xA0, 0xFF, 0xFF, 0xFF, 0xFF, 0xFB, 0x00, 0x00, 0x00, 0xFF];

// Its SHA-256 is in Jadeite_Native.md §9.4; the bytes are pinned so that a change to the program is a change seen.
#[test]
fn the_game_gear_s_open_boot_program_is_1_kib_and_its_bytes_are_pinned() {
    assert_eq!(OPEN_BOOT_GG.len(), GG_BOOT_SIZE);
    let fnv = OPEN_BOOT_GG.iter().fold(0xCBF2_9CE4_8422_2325u64, |h, &b| (h ^ b as u64).wrapping_mul(0x100_0000_01B3));
    assert_eq!(fnv, 0x65A3_DFBE_CB93_8D9C, "{fnv:#018X}");
    assert_eq!(&OPEN_BOOT_GG[..4], [0xF3, 0xED, 0x56, 0x31], "DI, IM 1, the stack");
    assert_eq!((OPEN_BOOT_GG[0x38], OPEN_BOOT_GG[0x39], OPEN_BOOT_GG[0x66], OPEN_BOOT_GG[0x67]), (0xED, 0x4D, 0xED, 0x45), "RETI and RETN");
}

// Jadeite_OpenBoot.md §2, U6-U9: the state a game finds, in GG mode and in a Master System cartridge's, on both models.
#[test]
fn the_open_boot_hands_a_cartridge_over_from_ram_in_the_state_chosen() {
    for (region, settings, model) in [(6u8, &[][..], Model::GameGear), (5, &[][..], Model::JapaneseGameGear), (4, &[("region", "sms.us")][..], Model::GameGear)] {
        let rom = keeper(region, true);
        let mut m = game_gear(&rom, settings);
        assert_eq!((m.model(), m.boot, m.bus.memory_control, m.bus.vdp.gg_mode), (model, Boot::Open, 0xE3, region != 4), "{region}");
        assert_eq!(m.bus.peek(0x0000), 0xF3, "the boot ROM at $0000");
        assert_eq!(m.bus.peek(0x0400), rom[0x0400], "the cartridge from $0400");
        assert!(until_handed_over(&mut m, 60), "{region}");
        assert!(m.bus.clock % m.frame_clocks() < 120, "{region}: on the last line");
        assert_eq!((m.bus.vdp.line, m.bus.vdp.regs, m.bus.ram[0]), (261, POWER_ON, 0xA8), "{region}");
        assert_eq!((m.cpu.regs.im, m.cpu.regs.iff1, m.cpu.regs.sp, m.cpu.regs.af >> 8), (1, false, 0xDFF0, 0xA8));
        assert!(m.bus.vdp.vram.iter().all(|&b| b == 0) && m.bus.vdp.cram.iter().all(|&b| b == 0), "VRAM and CRAM cleared");
        assert_eq!(m.bus.ram[0x700..0x705], [0xD3, 0x3E, 0xC3, 0x00, 0x00], "OUT ($3E),A and JP $0000 at $C700");
        assert!(m.bus.ram.iter().enumerate().all(|(i, &b)| i == 0 || (0x700..0x705).contains(&i) || b == 0), "RAM clear but $C000 and the hand-over");
        assert_eq!((m.bus.peek(0x0000), m.bus.gg.direction, m.bus.sound.psg.stereo), (rom[0], 0xFF, 0xFF), "the boot ROM gone, the ports as at power-on");
        for _ in 0..10 {
            m.cpu.step(&mut m.bus);
        }
        assert_eq!((m.bus.ram[0x10], m.bus.ram[0x11]), (0xA8, 0xA8), "{region}: the game reads $C000 and A");
    }
}

// Jadeite_OpenBoot.md §1.3, U4: a cartridge without TMR SEGA is refused in words in either mode, and Start's NMI in a
// Master System cartridge's mode returns to the halt; the screen has no logo, its words inside the LCD's window.
#[test]
fn a_cartridge_without_sega_s_header_is_refused_in_words() {
    for region in [6u8, 4] {
        let rom = keeper(region, false);
        let routed = if region == 4 { Some(Console::Sms) } else { None };
        let mut m = match routed {
            Some(_) => {
                let media = Media { console: Console::Gg, skipped: 0, header: None };
                let mut m = Machine::with_boot(media, Market::Americas, Model::GameGear, Board { mapper: Mapper::Plain, ram: 0, battery: false, why: Why::Size }, rom.clone(), Boot::Open, OPEN_BOOT_GG.to_vec());
                m.bus.vdp.gg_mode = false;
                m
            }
            None => game_gear(&rom, &[]),
        };
        frames(&mut m, 30);
        let lines: Vec<String> = (3..21).map(|r| row(&m, r)).filter(|l| !l.is_empty()).collect();
        assert_eq!(lines, ["ENDOU OPEN BOOT", "THE GAME CANNOT", "START. IT HAS NO", "SEGA HEADER."], "{region}");
        assert!(m.cpu.regs.pc < 0x400 && m.cpu.halted, "{region}: halted in the boot ROM");
        let used = m.bus.vdp.vram[0x3800..0x3F00].chunks(64).enumerate().filter(|(_, r)| r.iter().any(|&b| b != 0)).map(|(i, r)| (i, r.chunks(2).position(|e| e[0] != 0).unwrap()));
        assert!(used.clone().all(|(row, col)| (3..=20).contains(&row) && (6..=25).contains(&col)), "inside the window: {:?}", used.collect::<Vec<_>>());
        let cram = if m.bus.vdp.gg_mode { &m.bus.vdp.cram[..4] } else { &m.bus.vdp.cram[..2] };
        assert_eq!(cram, if m.bus.vdp.gg_mode { &[0x00, 0x00, 0xFF, 0x0F][..] } else { &[0x00, 0x3F][..] }, "{region}: white letters in the mode's form");
        m.pads[0] = PAUSE as u32;
        frames(&mut m, 2);
        assert!(m.cpu.regs.pc < 0x400 && m.cpu.halted, "{region}: back to the halt");
    }
}

/// A stand-in boot ROM, 1 KiB, as a test can make one: it marks $C100, copies a hand-over to RAM and runs it.
fn stand_in() -> Vec<u8> {
    let src = "        di\n        ld sp,$DFF0\n        ld a,$5C\n        ld ($C100),a\n        ld hl,stub\n        ld de,$C200\n        ld bc,5\n        ldir\n        ld a,$A8\n        jp $C200\nstub:   out ($3E),a\n        jp $0000\n";
    let mut b = assemble(src, 0).unwrap().bytes;
    b.resize(GG_BOOT_SIZE, 0xFF);
    b
}

// Jadeite_OpenBoot.md §5: the player's boot ROM runs when chosen and given, the open program otherwise; machine info names
// the path; an image of another size is refused; a state of another boot is foreign.
#[test]
fn the_player_s_boot_rom_runs_when_chosen_and_given() {
    use emusen_native::ffi::{StateMachine, Status};
    let rom = keeper(6, true);
    let bios = stand_in();
    let file = || vec![emusen_native::abi::File { which: BIOS_GG, data: &bios[..] }];
    let firmware = |m: &Machine| m.machine_info().firmware;
    for settings in [&[][..], &[("region", "sms.jp")][..]] {
        let mut settings = settings.to_vec();
        settings.push(("bios_gg", BIOS_PLAYER));
        let mut chosen = Machine::create_as("gg", &request(&rom, &settings, file())).unwrap();
        assert_eq!((chosen.boot, firmware(&chosen)), (Boot::Player, vec![(BIOS_GG, FirmwareSource::File)]));
        assert_eq!(chosen.space(BIOS_ID).unwrap(), &bios[..]);
        frames(&mut chosen, 2);
        assert_eq!((chosen.bus.ram[0x10], chosen.bus.ram[0x100]), (0, 0x5C), "the stand-in ran; the keeper read the $C000 it left");
        assert_eq!(chosen.bus.ram[0x11], 0xA8, "and A");
    }
    let given = Machine::create_as("gg", &request(&rom, &[], file())).unwrap();
    assert_eq!((given.boot, firmware(&given)), (Boot::Open, vec![(BIOS_GG, FirmwareSource::Replacement)]), "given but not chosen");
    let missing = game_gear(&rom, &[("bios_gg", BIOS_PLAYER)]);
    assert_eq!((missing.boot, missing.space(BIOS_ID).unwrap()), (Boot::Open, OPEN_BOOT_GG));
    let short = vec![0u8; 0x2000];
    assert_eq!(Machine::create_as("gg", &request(&rom, &[], vec![emusen_native::abi::File { which: BIOS_GG, data: &short }])).err(), Some(status::BAD_FILE));
    let mut open = game_gear(&rom, &[]);
    frames(&mut open, 3);
    let mut s = vec![0; open.state_size()];
    open.save_state(&mut s).unwrap();
    let mut player = Machine::create_as("gg", &request(&rom, &[("bios_gg", BIOS_PLAYER)], file())).unwrap();
    assert_eq!(player.load_state(&s).unwrap_err().status(), status::FOREIGN);
}

// G9 on the programs: every Game Gear test program booted by the open program reaches, 119 frames after its hand-over's
// frame, the picture, VRAM, CRAM and RAM it reaches in 120 from reset with no boot ROM, but $C000 and the hand-over left at $C700 where the program writes neither.
#[test]
fn every_game_gear_program_boots_to_the_point_it_reaches_with_no_boot_rom() {
    for p in [&images::GG, &images::GG_TMS, &images::GG_IO] {
        let rom = image(p);
        let board = crate::board::identify(&rom, Console::Gg);
        let media = Media::read_as(&rom, Some(Console::Gg)).unwrap();
        let mut plain = Machine::with_board(media.clone(), Market::Americas, Model::GameGear, board, rom.clone());
        let mut booted = Machine::with_boot(media, Market::Americas, Model::GameGear, board, rom, Boot::Open, OPEN_BOOT_GG.to_vec());
        let mut at = 0;
        while booted.bus.memory_control != 0xA8 {
            booted.advance();
            at += 1;
            assert!(at < 60, "{}: not handed over", p.name);
        }
        frames(&mut plain, 120);
        frames(&mut booted, 119);
        let ram: Vec<usize> = (0..0x2000).filter(|&i| plain.bus.ram[i] != booted.bus.ram[i]).collect();
        assert!(plain.bus.vdp.vram == booted.bus.vdp.vram && plain.bus.vdp.cram == booted.bus.vdp.cram, "{}", p.name);
        assert!(plain.frame_bytes() == booted.frame_bytes(), "{}: the picture", p.name);
        assert!(ram.iter().all(|&i| i == 0 || (0x700..0x705).contains(&i)), "{}: RAM differs only in $C000 and the hand-over's bytes at $C700 (U6): {ram:?}", p.name);
    }
}
