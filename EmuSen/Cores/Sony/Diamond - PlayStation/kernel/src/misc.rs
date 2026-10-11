//! psx-spx's "BIOS Misc Functions": exit, the random generator, sorting and searching, GetSystemInfo, SetMem and the
//! I/O abort. Diamond_OpenKernel.md §4.12.

use crate::heap;
use crate::layout::{self, io};
use crate::mem::{call, rd, rd8, wr, wr8};
use crate::{entry, tty};

/// A(06h) and B(38h): to A(3Ah) _exit through the table.
pub extern "C" fn exit(code: u32) -> u32 {
    call(rd(layout::A_TABLE + 4 * 0x3A), code, 0, 0, 0)
}

static mut SEED: u32 = 0;

/// A(2Fh): x = x*41C64E6Dh + 3039h, bits 16-30.
pub extern "C" fn rand() -> u32 {
    unsafe {
        SEED = SEED.wrapping_mul(0x41C6_4E6D).wrapping_add(0x3039);
        SEED >> 16 & 0x7FFF
    }
}

/// A(30h).
pub extern "C" fn srand(seed: u32) -> u32 {
    unsafe { SEED = seed };
    0
}

fn swap(a: u32, b: u32, width: u32, temp: u32) {
    for i in 0..width {
        wr8(temp + i, rd8(a + i));
        wr8(a + i, rd8(b + i));
        wr8(b + i, rd8(temp + i));
    }
}

fn sort(base: u32, low: u32, high: u32, width: u32, compare: u32, temp: u32) {
    if low >= high {
        return;
    }
    let at = |i: u32| base + i * width;
    let mut store = low;
    for i in low..high {
        if (call(compare, at(i), at(high), 0, 0) as i32) < 0 {
            swap(at(i), at(store), width, temp);
            store += 1;
        }
    }
    swap(at(store), at(high), width, temp);
    if store > low {
        sort(base, low, store - 1, width, compare, temp);
    }
    sort(base, store + 1, high, width, compare, temp);
}

/// A(31h): an element's copy held on the heap while swapping, so the heap must have `width` bytes free.
pub extern "C" fn qsort(base: u32, count: u32, width: u32, compare: u32) -> u32 {
    if count < 2 || width == 0 {
        return 0;
    }
    let temp = heap::malloc(width);
    if temp == 0 {
        return 0;
    }
    sort(base, 0, count - 1, width, compare, temp);
    heap::free(temp);
    0
}

/// A(35h).
pub extern "C" fn lsearch(key: u32, base: u32, count: u32, width: u32, compare: u32) -> u32 {
    (0..count).map(|i| base + i * width).find(|e| call(compare, key, *e, 0, 0) == 0).unwrap_or(0)
}

/// A(36h).
pub extern "C" fn bsearch(key: u32, base: u32, count: u32, width: u32, compare: u32) -> u32 {
    let (mut low, mut high) = (0, count);
    while low < high {
        let middle = low + (high - low) / 2;
        let e = base + middle * width;
        match call(compare, key, e, 0, 0) as i32 {
            0 => return e,
            r if r < 0 => high = middle,
            _ => low = middle + 1,
        }
    }
    0
}

/// A(B4h): the kernel's date, flags, version string and RAM size, and the halfwords psx-spx lists for 06h-0Eh.
pub extern "C" fn get_system_info(index: u32) -> u32 {
    const HALFWORDS: [u32; 9] = [0, 0x400, 0, 0x200, 0, 0, 1, 1, 1];
    match index {
        0 => layout::KERNEL_DATE,
        1 => 3,
        2 => crate::boot::version(),
        5 => rd(layout::RAM_SIZE_MB) << 10,
        6..=0x0E => HALFWORDS[index as usize - 6],
        _ => 0,
    }
}

/// A(9Fh): 2 or 8 MiB, through bits 8-10 of the RAM's size register with bit 11 assumed set.
pub extern "C" fn set_mem(megabytes: u32) -> u32 {
    let banks = match megabytes {
        2 => 0x000,
        8 => 0x300,
        _ => return 0,
    };
    wr(io::RAM_SIZE, rd(io::RAM_SIZE) & !0x700 | banks);
    wr(layout::RAM_SIZE_MB, megabytes);
    0
}

/// The buffer _ioabort returns through; until something records one, a lock-up.
static mut IOABORT: [u32; 12] = [0; 12];

pub fn reset_ioabort() {
    let b = &raw mut IOABORT as u32;
    wr(b, entry::system_error as *const () as u32);
    wr(b + 4, layout::DEFAULT_STACKTOP);
}

/// A(B2h).
pub extern "C" fn ioabort_raw(param: u32) -> u32 {
    unsafe { entry::longjmp(&raw const IOABORT as u32, param) }
}

/// C(19h).
pub extern "C" fn ioabort(text1: u32, text2: u32) -> u32 {
    tty::puts(text1);
    tty::puts(text2);
    ioabort_raw(1)
}
