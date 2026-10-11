//! Diamond's open kernel: the PlayStation's BIOS kernel written from psx-spx's descriptions of its interface, run by
//! the emulated R3000A from the BIOS's address range. Phase 1: the reset path, the exception and interrupt dispatch,
//! the A0h/B0h/C0h tables with the memory, string, TTY, event, thread, heap and program calls; the CD, the files on it,
//! the memory cards and the screen are phase 2. Diamond_OpenKernel.md.

#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]

mod blocks;
mod boot;
mod convert;
mod entry;
mod event;
mod exception;
mod exec;
mod file;
mod gpu;
mod heap;
mod layout;
mod mem;
mod memory;
mod misc;
mod pad;
mod phase2;
mod printf;
mod string;
mod table;
mod thread;
mod timer;
mod tty;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    unsafe { entry::hang() }
}
