//! psx-spx's "BIOS File Execute and Flush Cache" over the file layer: LoadTest, Load and LoadExec. Exec and
//! FlushCache are assembly (entry.rs). No device holds a file before phase 2, so a load fails there and LoadExec
//! ends in its documented lock-up. Diamond_OpenKernel.md §4.11.

use crate::entry;
use crate::exception;
use crate::file;
use crate::layout::io;
use crate::mem::{rd, rd8, wr, wr8};

static mut SECTOR: [u32; 0x200] = [0; 0x200];

/// The 800h-byte header read to the sector buffer and its bytes 10h-4Bh copied to `header`; with `body`, the program
/// read to its destination and the cache flushed. 1, or 0 when it fails.
fn load_file(name: u32, header: u32, body: bool) -> u32 {
    let fd = file::open(name, 1);
    if fd == u32::MAX {
        return 0;
    }
    let sector = &raw mut SECTOR as u32;
    let mut ok = file::read(fd, sector, 0x800) == 0x800;
    if ok {
        for i in 0..0x3C {
            wr8(header + i, rd8(sector + 0x10 + i));
        }
        if body {
            let (address, size) = (rd(header + 0x08), rd(header + 0x0C));
            ok = size == 0 || file::read(fd, address, size) == size;
        }
    }
    file::close(fd);
    if ok && body {
        unsafe { entry::flush_cache() };
    }
    ok as u32
}

/// A(41h).
pub extern "C" fn load_test(name: u32, header: u32) -> u32 {
    load_file(name, header, false)
}

/// A(42h).
pub extern "C" fn load(name: u32, header: u32) -> u32 {
    load_file(name, header, true)
}

static mut NAME: [u8; 32] = [0; 32];
static mut HEADER: [u32; 15] = [0; 15];

/// A(51h): the name's path upper-cased and ";1" added; the caller's interrupt sources masked and the default exit
/// restored, since the load replaces the memory the caller's handlers lie in; interrupts enabled, the program loaded
/// and run with the given stack by Exec(headerbuf,1,0) with I_MASK and I_STAT clear, as the boot starts its program
/// (Diamond_OpenKernel_Phase2.md §26.2); then the lock-up that follows it (the boot program it would reload is not).
pub extern "C" fn load_exec(name: u32, stackbase: u32, stackoffset: u32) -> u32 {
    let copy = &raw mut NAME as u32;
    let mut i = 0;
    let mut device = true;
    let mut version = false;
    while i < 28 && rd8(name + i) != 0 {
        let mut c = rd8(name + i);
        if !device {
            c = crate::convert::toupper(c);
        }
        device &= c != b':' as u32;
        version |= c == b';' as u32;
        wr8(copy + i, c);
        i += 1;
    }
    if !version && i + 2 < 32 {
        wr8(copy + i, b';' as u32);
        wr8(copy + i + 1, b'1' as u32);
        i += 2;
    }
    wr8(copy + i, 0);
    exception::reset_entry_int();
    wr(io::I_MASK, 0);
    unsafe { entry::kernel_syscall(2, 0) };
    let header = &raw mut HEADER as u32;
    if load(copy, header) != 0 {
        wr(header + 0x20, stackbase);
        wr(header + 0x24, stackoffset);
        wr(io::I_MASK, 0);
        wr(io::I_STAT, 0);
        unsafe { entry::exec(header, 1, 0) };
    }
    unsafe { entry::hang() }
}
