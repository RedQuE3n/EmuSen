//! ROM patches beside the cartridge: `MarsCheatTests`' cartridge cases, the first-match order and the host's list (Mars_Native.md §6.6.1).

use std::sync::Arc;

use super::support::{SyntheticRom, bus_with_cart, rom_image};
use crate::ffi::{Core, NO_COMPARE, mars_machine_set_rom_patches};
use crate::machine::Machine;
use crate::memory::bus::MemoryBus;
use crate::memory::bus_access::map::{CART_DOMAIN1_ADDRESS2, PI_BASE};
use crate::memory::rom_patches::{RomPatch, RomPatches};

/// A recognisable cartridge byte past the boot code, at ROM offset 0x1040, as `MarsCheatTests` places it.
const ORIGINAL: u8 = 0xCE;
const ROM_OFFSET: u32 = 0x1040;
const PHYSICAL: u32 = CART_DOMAIN1_ADDRESS2 + ROM_OFFSET;

fn image() -> Vec<u8> {
    SyntheticRom { length: 0x2000, patches: vec![((ROM_OFFSET - 0x40) as usize, vec![ORIGINAL, 0x11, 0x22, 0x33])], ..SyntheticRom::default() }.build()
}

fn patch(address: u32, value: u8, compare: Option<u8>) -> RomPatch {
    RomPatch { address, value, compare }
}

fn patched(entries: Vec<RomPatch>) -> MemoryBus {
    let mut bus = bus_with_cart(&image());
    *bus.rom_patches = RomPatches::new(entries).map(Arc::new);
    bus
}

#[test]
fn a_rom_patch_reaches_a_processor_load_by_rom_offset() {
    let mut bus = patched(vec![]);
    assert_eq!(bus.read8(PHYSICAL), ORIGINAL);

    let mut bus = patched(vec![patch(ROM_OFFSET, 0xAD, None)]);
    assert_eq!(bus.read8(PHYSICAL), 0xAD);
    assert_eq!(bus.read32(PHYSICAL), 0xAD11_2233);
    assert_eq!(bus.load(PHYSICAL, 4), 0xAD11_2233);
    assert_eq!(bus.load(PHYSICAL, 1), 0xAD);
    assert_eq!(bus.cart.as_ref().unwrap().rom[ROM_OFFSET as usize], ORIGINAL);
}

#[test]
fn a_rom_patch_reaches_a_transfer_into_rdram() {
    let mut bus = patched(vec![patch(ROM_OFFSET + 1, 0xEE, None)]);
    bus.write32(PI_BASE, 0x1000);
    bus.write32(PI_BASE + 0x04, PHYSICAL);
    bus.write32(PI_BASE + 0x0C, 4 - 1);
    assert_eq!(&bus.rdram[0x1000..0x1004], &[ORIGINAL, 0xEE, 0x22, 0x33]);
}

#[test]
fn a_compare_applies_only_over_the_byte_it_names_and_the_first_that_applies_wins() {
    let mut bus = patched(vec![patch(ROM_OFFSET, 0x01, Some(ORIGINAL ^ 0xFF)), patch(ROM_OFFSET, 0x02, Some(ORIGINAL)), patch(ROM_OFFSET, 0x03, None)]);
    assert_eq!(bus.read8(PHYSICAL), 0x02);
    let mut bus = patched(vec![patch(ROM_OFFSET, 0x03, None), patch(ROM_OFFSET, 0x02, Some(ORIGINAL))]);
    assert_eq!(bus.read8(PHYSICAL), 0x03);
    let mut bus = patched(vec![patch(ROM_OFFSET, 0x01, Some(ORIGINAL ^ 0xFF))]);
    assert_eq!(bus.read8(PHYSICAL), ORIGINAL);
}

#[test]
fn only_the_bytes_named_change_and_the_order_given_is_kept_across_the_sort() {
    let entries = vec![patch(ROM_OFFSET + 3, 0x44, None), patch(ROM_OFFSET + 1, 0x55, Some(0x00)), patch(ROM_OFFSET + 1, 0x66, None), patch(ROM_OFFSET - 1, 0x77, None)];
    let mut bus = patched(entries);
    assert_eq!(bus.read32(PHYSICAL), 0xCE66_2244);
    assert_eq!(bus.read32(PHYSICAL - 4) & 0xFF, 0x77);
    assert_eq!(bus.read32(PHYSICAL + 4), bus_with_cart(&image()).read32(PHYSICAL + 4));
}

#[test]
fn past_the_cartridge_s_end_is_zero_whatever_is_patched() {
    let length = image().len() as u32;
    let mut bus = patched(vec![patch(length, 0xAD, None), patch(length + 1, 0xAD, None)]);
    assert_eq!(bus.read32(CART_DOMAIN1_ADDRESS2 + length), 0);
}

#[test]
fn the_host_s_list_replaces_the_last_one_and_survives_a_state_load() {
    let mut core = Core::new(Machine::load_rom(rom_image(&image()), false, None, None));
    let words = [ROM_OFFSET, 0xAD, NO_COMPARE, ROM_OFFSET + 2, 0x99, ORIGINAL as u32, ROM_OFFSET + 3, 0x98, 0x33];
    assert_eq!(unsafe { mars_machine_set_rom_patches(&mut core, words.as_ptr(), 3) }, 3);
    assert_eq!(core.machine.bus.read32(PHYSICAL), 0xAD11_2298);

    let state = core.machine.save_state_vec(false).unwrap();
    core.machine.restore_state(&state).unwrap();
    assert_eq!(core.machine.bus.read32(PHYSICAL), 0xAD11_2298, "a state load dropped the patches");

    assert_eq!(unsafe { mars_machine_set_rom_patches(&mut core, std::ptr::null(), 0) }, 0);
    assert!(core.machine.bus.rom_patches.is_none());
    assert_eq!(core.machine.bus.read32(PHYSICAL), 0xCE11_2233);
}
