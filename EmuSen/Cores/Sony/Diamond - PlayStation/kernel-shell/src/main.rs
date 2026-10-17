//! The open kernel's shell: what the PlayStation shows when no disc boots, a program of its own started by the kernel
//! under hand-over v1 and reaching it only through its A, B and C call tables. Diamond_Shell.md §1.3.

#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

mod bios;
mod cards;
mod clock;
mod drive;
mod manager;
mod pads;
mod tty;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

/// Exec's entry: param1 is 1, `reason` what the drive held (0 empty, 1 an audio disc, 2 a refused disc). Returns 0 once
/// the drive holds a disc to boot, its pads and cards stopped and vblank masked, as hand-over v1 requires.
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.shell_start")]
pub extern "C" fn shell_start(_one: u32, reason: u32) -> u32 {
    let _ = drive::Held::from_reason(reason);
    manager::run();
    0
}
