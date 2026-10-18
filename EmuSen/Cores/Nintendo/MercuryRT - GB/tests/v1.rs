//! MercuryRT on the core ABI v1 alone: what B4 moved into the core from its C# shim, the battery rule, the CPU's
//! registers, the disassembly and palette RAM's spaces. See EmuSen_Backend.md §17.2 (L4, D1, D5, D10).

use emusen_native::core::{Core, Create, File, Reference, Settings, status};
use mercuryrt::machine::Machine;
use mercuryrt::memory::cartridge::{HEADER_CHECKSUM_ADDRESS, header_checksum};

/// A 32K image of cartridge type `kind` with 8K of RAM (`$02`): at `$100` a jump to `$150`, there a jump to itself.
fn rom(kind: u8) -> Vec<u8> {
    let mut rom = vec![0u8; 0x8000];
    rom[0x100..0x104].copy_from_slice(&[0x00, 0xC3, 0x50, 0x01]);
    rom[0x150..0x152].copy_from_slice(&[0x18, 0xFE]);
    rom[0x147] = kind;
    rom[0x149] = 0x02;
    rom[HEADER_CHECKSUM_ADDRESS] = header_checksum(&rom);
    rom
}

fn create<'a>(image: &'a [u8], files: Vec<File<'a>>) -> Result<Machine, i32> {
    let request = Create {
        image,
        settings: Settings::parse(b"Model=Auto").unwrap(),
        files,
        pixel_formats: 1,
        host_abi_version: 0,
        shared_removable: false,
        shared_console: false,
        device: None,
    };
    <Machine as Core>::create(&request)
}

fn cartridge_ram(m: &mut Machine) -> Vec<u8> {
    let mut ram = vec![0u8; 8192];
    <Machine as Core>::space_read(m, 2, 0, &mut ram).unwrap();
    ram
}

#[test]
fn file_0_is_listed_for_both_systems_and_kept_only_on_a_board_with_a_battery() {
    for system in ["gb", "gbc"] {
        let files = <Machine as Core>::battery_files(system).expect("a list");
        assert_eq!(files.iter().map(|b| (b.which, b.suffix.as_str())).collect::<Vec<_>>(), [(0, ".srm")]);
    }
    let save = vec![0xA5u8; 8192];
    let (with_battery, without) = (rom(0x03), rom(0x02));
    let mut kept = create(&with_battery, vec![File { which: 0, data: &save }]).unwrap();
    assert_eq!(cartridge_ram(&mut kept), save, "MBC1+RAM+BATTERY takes the save");
    let mut plain = create(&without, vec![File { which: 0, data: &save }]).unwrap();
    let mut fresh = create(&without, Vec::new()).unwrap();
    assert_eq!(cartridge_ram(&mut plain), cartridge_ram(&mut fresh), "MBC1+RAM keeps no save, so a stray one changes nothing");
    assert!(<Machine as Core>::machine_info(&plain).battery.is_empty());
}

#[test]
fn the_cpu_s_sixteen_registers_are_machine_info_s_and_debug_registers_answers_them() {
    let image = rom(0x00);
    let m = create(&image, Vec::new()).unwrap();
    let info = <Machine as Core>::machine_info(&m);
    let names: Vec<&str> = info.processors[0].registers.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["A", "F", "BC", "DE", "HL", "SP", "PC", "Z", "N", "H", "C", "IME", "HALT", "IE", "IF", "Cycles"]);
    let values = <Machine as Core>::debug_registers(&m, 0).unwrap();
    assert_eq!(values.len(), 16);
    assert_eq!((values[0], values[1], values[4], values[6]), (m.cpu.a as i64, m.cpu.f as i64, m.cpu.hl() as i64, m.cpu.pc as i64));
    assert_eq!((values[7], values[10]), ((m.cpu.f as i64 >> 7) & 1, (m.cpu.f as i64 >> 4) & 1), "Z and C are F's bits 7 and 4");
    assert_eq!(<Machine as Core>::debug_registers(&m, 1).err(), Some(status::NOT_SUPPORTED));
}

#[test]
fn the_disassembly_is_emusen_sm83_s_over_the_space_named() {
    let image = rom(0x00);
    let mut m = create(&image, Vec::new()).unwrap();
    let listing = <Machine as Core>::debug_disassemble(&mut m, 0, 0, 0x100, 3).unwrap();
    let text: Vec<(u32, &str, &str)> = listing.iter().map(|i| (i.address, i.mnemonic.as_str(), i.operands.as_str())).collect();
    assert_eq!(text, [(0x100, "NOP", ""), (0x101, "JP", "$0150"), (0x104, "NOP", "")]);
    assert_eq!(listing[1].bytes, [0xC3, 0x50, 0x01]);
    assert_eq!(listing[1].reference, Some((Reference::Call, 0x150)));
    let branch = <Machine as Core>::debug_disassemble(&mut m, 0, 6, 0x150, 1).unwrap();
    assert_eq!((branch[0].mnemonic.as_str(), branch[0].operands.as_str(), branch[0].reference), ("JR", "$0150", Some((Reference::Call, 0x150))));
    assert_eq!(<Machine as Core>::debug_disassemble(&mut m, 1, 0, 0, 1).err(), Some(status::NOT_SUPPORTED));
    assert_eq!(<Machine as Core>::debug_disassemble(&mut m, 0, 9, 0, 1).err(), Some(status::NO_SUCH_SPACE));
}

#[test]
fn palette_ram_is_two_read_only_spaces_of_64_bytes() {
    let image = rom(0x00);
    let mut m = create(&image, Vec::new()).unwrap();
    let info = <Machine as Core>::machine_info(&m);
    for (id, name) in [(7u32, "BGPAL"), (8, "OBJPAL")] {
        let space = info.spaces.iter().find(|s| s.id == id).expect("listed");
        assert_eq!((space.name.as_str(), space.read_only), (name, true));
        assert_eq!(<Machine as Core>::space_size(&m, id), Ok(64));
    }
    assert!(info.spaces[0].read_only, "the ROM is read only");
    assert_eq!(<Machine as Core>::space_write(&mut m, 0, 0x150, &[0x12]).err(), Some(status::READ_ONLY));
    let mut obj = [0u8; 64];
    <Machine as Core>::space_read(&mut m, 8, 0, &mut obj).unwrap();
    assert_eq!(&obj[..], &m.bus.ppu.obj_palette_ram[..]);
}

fn colour_rom() -> Vec<u8> {
    let mut rom = rom(0x00);
    rom[0x143] = 0x80;
    rom[HEADER_CHECKSUM_ADDRESS] = header_checksum(&rom);
    rom
}

#[test]
fn the_units_are_listed_from_256_each_answered_with_its_count() {
    let (mono, colour) = (rom(0x00), colour_rom());
    for (image, hdma) in [(&mono, false), (&colour, true)] {
        let mut m = create(image, Vec::new()).unwrap();
        for _ in 0..2 {
            let mut detail = 0;
            <Machine as Core>::advance(&mut m, &mut detail).unwrap();
        }
        let units = <Machine as Core>::units(&m);
        let names: Vec<&str> = units.iter().map(|u| u.name.as_str()).collect();
        let expected: &[&str] = if hdma { &["PPU", "APU", "Timer", "Pulse 1", "Pulse 2", "Wave", "Noise", "HDMA"] } else { &["PPU", "APU", "Timer", "Pulse 1", "Pulse 2", "Wave", "Noise"] };
        assert_eq!(names, expected);
        for (i, u) in units.iter().enumerate() {
            assert_eq!(u.id, 256 + i as u32);
            assert_eq!(<Machine as Core>::debug_registers(&m, u.id).unwrap().len(), u.registers.len(), "{}", u.name);
        }
        assert_eq!(units[0].registers.len(), if hdma { 19 } else { 14 }, "the colour registers only in colour mode");
        let ppu = <Machine as Core>::debug_registers(&m, 256).unwrap();
        let mut stat = [0u8];
        <Machine as Core>::space_read(&mut m, 6, 0xFF41, &mut stat).unwrap();
        assert_eq!((ppu[0], ppu[1], ppu[4]), (m.bus.ppu.lcdc as i64, stat[0] as i64, m.bus.ppu.ly as i64), "LCDC, STAT as read, LY");
        let apu = <Machine as Core>::debug_registers(&m, 257).unwrap();
        let mut nr52 = [0u8];
        <Machine as Core>::space_read(&mut m, 6, 0xFF26, &mut nr52).unwrap();
        assert_eq!((apu.len(), apu[22]), (25, nr52[0] as i64), "the sound registers as read, then Power and SeqStep");
        assert_eq!(<Machine as Core>::debug_registers(&m, 256 + units.len() as u32).err(), Some(status::NOT_SUPPORTED));
    }
}
