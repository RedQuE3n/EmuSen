//! Diamond's open kernel: the PlayStation's BIOS kernel written from psx-spx's descriptions of its interface, run by
//! the emulated R3000A from the BIOS's address range: the reset path, the exception and interrupt dispatch, the
//! A0h/B0h/C0h tables, and since phase 2 the CD and its files, the boot from a disc, the memory cards in their on-card
//! format, and the screen with no disc. Diamond_OpenKernel.md and Diamond_OpenKernel_Phase2.md.

#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

mod blocks;
mod boot;
mod bu;
mod card;
mod cdfs;
mod cdrom;
mod convert;
mod entry;
mod event;
mod exception;
mod exec;
mod file;
mod font;
mod gpu;
mod heap;
mod layout;
mod manager;
mod mem;
mod memory;
mod misc;
mod names;
mod pad;
mod phase2;
mod printf;
mod screen;
mod shell;
mod sio;
mod string;
mod table;
mod thread;
mod timer;
mod tty;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { entry::hang() }
}
