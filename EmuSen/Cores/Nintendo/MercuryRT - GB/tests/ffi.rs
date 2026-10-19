//! MercuryRT's create through the core ABI v1: the Model setting, the battery file, the input mask and the patch table,
//! each where a host always passes the same value and so cannot tell. See Mercury_Native.md §8.7.

use emusen_native::core::{Core, Create, File, Settings, status};
use mercuryrt::ffi::{STATUS_UNKNOWN_MODEL, build_rom_patches};
use mercuryrt::machine::Machine;
use mercuryrt::memory::cartridge::{HEADER_CHECKSUM_ADDRESS, header_checksum};

/// A 32K image of cartridge type `kind` with 8K of RAM (`$02`), the program a jump to itself.
fn rom(kind: u8) -> Vec<u8> {
    let mut rom = vec![0u8; 0x8000];
    rom[0x100..0x104].copy_from_slice(&[0x00, 0xC3, 0x50, 0x01]);
    rom[0x150..0x152].copy_from_slice(&[0x18, 0xFE]);
    rom[0x147] = kind;
    rom[0x149] = 0x02;
    rom[HEADER_CHECKSUM_ADDRESS] = header_checksum(&rom);
    rom
}

fn create(kind: u8, settings: &str, files: &[File<'_>]) -> Result<Machine, i32> {
    let image = rom(kind);
    let request = Create {
        image: &image,
        settings: Settings::parse(settings.as_bytes()).unwrap(),
        files: files.to_vec(),
        pixel_formats: 1,
        host_abi_version: 0,
        shared_removable: false,
        shared_console: false,
        device: None,
    };
    <Machine as Core>::create(&request)
}

#[test]
fn the_model_must_be_one_of_three() {
    assert!(!create(0x00, "", &[]).unwrap().cgb_hardware());
    assert!(!create(0x00, "Model=Game Boy", &[]).unwrap().cgb_hardware());
    assert!(create(0x00, "Model=Game Boy Color", &[]).unwrap().cgb_hardware());
    assert_eq!(create(0x00, "Model=2", &[]).err(), Some(STATUS_UNKNOWN_MODEL));
}

#[test]
fn the_battery_file_fills_cartridge_ram_clipped_only_with_a_battery() {
    let save: Vec<u8> = (0..9000u32).map(|i| (i * 7) as u8).collect();
    let m = create(0x03, "", &[File { which: 0, data: &save }]).unwrap();
    let (data, _) = Core::battery(&m, 0).unwrap();
    assert_eq!(data, &save[..8192]);
    assert_eq!(create(0x03, "", &[File { which: 1, data: &save }]).err(), Some(status::BAD_FILE));

    let m = create(0x02, "", &[File { which: 0, data: &save }]).unwrap();
    assert!(m.bus.cart.ram.iter().all(|&b| b == 0), "a cartridge without a battery keeps no save");
    assert!(Core::battery(&m, 0).unwrap().0.is_empty(), "a cartridge without a battery has RAM and no save");
}

#[test]
fn set_buttons_moves_only_the_changed_bits_and_only_on_the_first_port() {
    let mut m = create(0x00, "", &[]).unwrap();
    let (a, b) = (1u32 << 4, 1u32 << 5);
    Core::set_buttons(&mut m, 0, a, a).unwrap();
    Core::set_buttons(&mut m, 0, b, b).unwrap();
    assert!(m.bus.joypad.a && m.bus.joypad.b);
    Core::set_buttons(&mut m, 1, 0, 0xFF).unwrap();
    assert!(m.bus.joypad.a && m.bus.joypad.b, "port 1 is not a Game Boy's");
    Core::set_buttons(&mut m, 0, 0, a).unwrap();
    assert!(!m.bus.joypad.a && m.bus.joypad.b);
}

#[test]
fn the_patch_table_keeps_the_first_entry_for_each_original_byte() {
    const ANY: u32 = u32::MAX;
    let map = build_rom_patches(&[0x1123, 0x42, 0x07, 0x1123, 0x55, ANY, 0x1123, 0x66, 0x07, 0x1124, 0x10, ANY, 0x1124, 0x20, ANY, 0x10000, 0x01, ANY])
        .expect("a table");
    assert_eq!(map[&0x1123][0x07], 0x142);
    assert_eq!(map[&0x1123][0x08], 0x155);
    assert!(map[&0x1124].iter().all(|&e| e == 0x110));
    assert_eq!(map.len(), 2, "an address past 16 bits is dropped, not wrapped");
    assert!(build_rom_patches(&[]).is_none());
}

#[test]
fn a_space_the_machine_does_not_have_is_refused_through_the_core_abi() {
    use emusen_native::core::{Core, status as core_status};
    let mut m = create(0x00, "", &[]).unwrap();
    assert_eq!(<Machine as Core>::space_size(&m, 9).err(), Some(core_status::NO_SUCH_SPACE));
    assert_eq!(<Machine as Core>::space_write(&mut m, 9, 0, &[1]).err(), Some(core_status::NO_SUCH_SPACE));
    assert_eq!(<Machine as Core>::space_write(&mut m, 7, 0, &[1]).err(), Some(core_status::READ_ONLY), "palette RAM is read only");
    <Machine as Core>::space_write(&mut m, 3, 0, &[0x5A]).unwrap();
    assert_eq!(m.bus.wram[0], 0x5A, "WRAM, space 3, takes the write");
}
