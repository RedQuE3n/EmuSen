//! The reset path after the entry's first three instructions: the memory control and cache set up from KSEG1, then
//! the kernel's RAM, the vectors and tables, the control blocks, the devices and standard I/O, and the start of what
//! follows the kernel at 80030000h, the shell. Diamond_OpenKernel.md §3.

use crate::layout::{self, io, setting};
use crate::mem::{rd, wr};
use crate::{blocks, entry, exception, file, misc, table};

unsafe extern "C" {
    static __data_start: u32;
    static __data_end: u32;
    static __data_rom: u32;
}

/// Runs uncached: nothing may be fetched through the I-cache until its tags are cleared.
#[unsafe(no_mangle)]
extern "C" fn boot_uncached() {
    wr(io::BIOS_DELAY, setting::BIOS_DELAY);
    wr(io::DEV0_BASE, setting::DEV0_BASE);
    wr(io::DEV8_BASE, setting::DEV8_BASE);
    wr(io::DEV0_DELAY, setting::DEV0_DELAY);
    wr(io::DEV1_DELAY, setting::DEV1_DELAY);
    wr(io::SPU_DELAY, setting::SPU_DELAY);
    wr(io::CDROM_DELAY, setting::CDROM_DELAY);
    wr(io::DEV8_DELAY, setting::DEV8_DELAY);
    wr(io::COM_DELAY, setting::COM_DELAY);
    wr(io::BCC, setting::BCC);
    unsafe {
        entry::flush_cache_uncached();
        entry::cop0_set_cause(0);
        entry::cop0_set_dcic(0);
    }
}

#[unsafe(no_mangle)]
extern "C" fn boot_cached() -> ! {
    let mut a = layout::KERNEL_RAM;
    while a < layout::KERNEL_RAM + layout::KERNEL_RAM_SIZE {
        wr(a, 0);
        a += 4;
    }
    let (start, end, rom) =
        (&raw const __data_start as u32, &raw const __data_end as u32, &raw const __data_rom as u32);
    let mut i = 0;
    while start + i < end {
        wr(start + i, rd(rom + i));
        i += 4;
    }
    wr(layout::RAM_SIZE_MB, 2);
    wr(layout::RAM_WORD_64, 0);
    wr(layout::RAM_WORD_68, 0xFF);
    table::install();
    exception::reset_entry_int();
    misc::reset_ioabort();
    blocks::set_conf(layout::DEFAULT_EVENTS, layout::DEFAULT_THREADS, layout::DEFAULT_STACKTOP);
    wr(io::I_MASK, 0);
    wr(io::I_STAT, 0);
    file::install_standard_devices();
    unsafe { entry::cop0_set_sr(0) };
    enter_shell()
}

/// 80030000h holds a jump to the shell in the ROM, where a side-loader takes over instead (Diamond_OpenKernel.md §3.3).
fn enter_shell() -> ! {
    let to = entry::shell_entry as *const () as u32;
    wr(layout::SHELL_ENTRY, 0x3C08_0000 | to.wrapping_add(0x8000) >> 16);
    wr(layout::SHELL_ENTRY + 4, 0x2508_0000 | to & 0xFFFF);
    wr(layout::SHELL_ENTRY + 8, 0x0100_0008);
    wr(layout::SHELL_ENTRY + 12, 0);
    unsafe {
        entry::flush_cache();
        entry::enter_shell(layout::SHELL_ENTRY, blocks::stacktop(), 0x0000_0401)
    }
}

/// The ROM header at BFC00100h (psx-spx "BIOS ROM Header/Footer"): the BCD date, the console type of a retail
/// console, and the maker and version strings, GetSystemInfo(2) giving the version's address.
#[repr(C)]
pub struct Header {
    date: u32,
    console: u32,
    maker: [u8; 8],
    version: [u8; 40],
}

#[used]
#[unsafe(link_section = ".header")]
pub static HEADER: Header = Header {
    date: layout::KERNEL_DATE,
    console: layout::CONSOLE_TYPE,
    maker: *b"EmuSen\0\0",
    version: *b"Diamond open kernel, phase 2\0\0\0\0\0\0\0\0\0\0\0\0",
};

pub fn version() -> u32 {
    &raw const HEADER.version as u32
}
