//! The kernel's calls the shell makes, each through its documented table entry (psx-spx "BIOS Function Summary"): the
//! number in t1, a jump to A0h, B0h or C0h, the result in v0.

use core::arch::global_asm;

macro_rules! calls {
    ($($table:literal $number:literal $name:ident ($($arg:ident: $ty:ty),*) -> $ret:ty;)*) => {
        $(global_asm!(concat!(
            ".set noreorder\n.section .text.", stringify!($name), ",\"ax\"\n.globl ", stringify!($name), "\n",
            stringify!($name), ":\n    li $t2, ", $table, "\n    jr $t2\n    li $t1, ", $number, "\n.set reorder\n"
        ));)*
        unsafe extern "C" {
            $(pub fn $name($($arg: $ty),*) -> $ret;)*
        }
    };
}

calls! {
    "0xA0" "0x55" bu_init() -> u32;
    "0xA0" "0xAB" card_info(port: u32) -> u32;
    "0xA0" "0xAC" card_load(port: u32) -> u32;
    "0xB0" "0x08" open_event(class: u32, spec: u32, mode: u32, func: u32) -> u32;
    "0xB0" "0x09" close_event(event: u32) -> u32;
    "0xB0" "0x0B" test_event(event: u32) -> u32;
    "0xB0" "0x0C" enable_event(event: u32) -> u32;
    "0xB0" "0x12" init_pad2(buf1: u32, size1: u32, buf2: u32, size2: u32) -> u32;
    "0xB0" "0x13" start_pad2() -> u32;
    "0xB0" "0x14" stop_pad2() -> u32;
    "0xB0" "0x32" open(name: u32, mode: u32) -> u32;
    "0xB0" "0x34" read(fd: u32, dst: u32, length: u32) -> u32;
    "0xB0" "0x35" write(fd: u32, src: u32, length: u32) -> u32;
    "0xB0" "0x36" close(fd: u32) -> u32;
    "0xB0" "0x3D" putchar(c: u32) -> u32;
    "0xB0" "0x41" format(device: u32) -> u32;
    "0xB0" "0x42" firstfile2(name: u32, direntry: u32) -> u32;
    "0xB0" "0x43" nextfile(direntry: u32) -> u32;
    "0xB0" "0x45" erase(name: u32) -> u32;
    "0xB0" "0x4A" init_card2(pad_enable: u32) -> u32;
    "0xB0" "0x4B" start_card2() -> u32;
    "0xB0" "0x4C" stop_card2() -> u32;
}

/// psx-spx's "Event Modes": the event marked ready when delivered, with no callback.
pub const EV_READY: u32 = 0x2000;
