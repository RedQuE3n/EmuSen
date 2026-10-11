//! The two heaps of one format: InitHeap's for malloc and its kin, and the kernel's control-block memory of
//! SysInitMemory and B(00h). Each block's word before it holds its size, bit 0 set when free, as free's documented
//! [buf-4] OR 1 requires; a zero word ends the heap. Diamond_OpenKernel.md §4.5.

use crate::mem::{rd, wr};
use crate::memory;

static mut USER: u32 = 0;
static mut KERNEL: u32 = 0;

fn init(heap: *mut u32, address: u32, size: u32) {
    let start = (address + 3) & !3;
    let end = (address.wrapping_add(size)) & !3;
    unsafe { *heap = 0 };
    if end <= start || end - start < 12 {
        return;
    }
    wr(start, (end - start - 8) | 1);
    wr(end - 4, 0);
    unsafe { *heap = start };
}

fn allocate(heap: u32, size: u32) -> u32 {
    if heap == 0 {
        return 0;
    }
    let n = (size.wrapping_add(3) & !3).max(4);
    let mut h = heap;
    loop {
        let head = rd(h);
        if head == 0 {
            return 0;
        }
        if head & 1 != 0 {
            let mut length = head & !3;
            loop {
                let next = rd(h + 4 + length);
                if next == 0 || next & 1 == 0 {
                    break;
                }
                length += 4 + (next & !3);
            }
            if length >= n {
                if length - n >= 8 {
                    wr(h, n);
                    wr(h + 4 + n, (length - n - 4) | 1);
                } else {
                    wr(h, length);
                }
                return h + 4;
            }
            wr(h, length | 1);
        }
        h += 4 + (rd(h) & !3);
    }
}

/// A(39h).
pub extern "C" fn init_heap(address: u32, size: u32) -> u32 {
    init(&raw mut USER, address, size);
    0
}

/// A(33h).
pub extern "C" fn malloc(size: u32) -> u32 {
    allocate(unsafe { USER }, size)
}

/// A(34h) and B(01h): no checks, as documented.
pub extern "C" fn free(buffer: u32) -> u32 {
    wr(buffer.wrapping_sub(4), rd(buffer.wrapping_sub(4)) | 1);
    0
}

/// A(37h).
pub extern "C" fn calloc(x: u32, y: u32) -> u32 {
    let n = x.wrapping_mul(y);
    let a = malloc(n);
    if a != 0 {
        memory::bzero(a, n);
    }
    a
}

/// A(38h): copies new_size bytes from the old block, as documented.
pub extern "C" fn realloc(old: u32, size: u32) -> u32 {
    if old == 0 {
        return malloc(size);
    }
    if size == 0 {
        free(old);
        return 0;
    }
    let new = malloc(size);
    if new != 0 {
        memory::bcopy(old, new, size);
        free(old);
    }
    new
}

/// C(08h).
pub extern "C" fn sys_init_memory(address: u32, size: u32) -> u32 {
    init(&raw mut KERNEL, address, size);
    0
}

/// B(00h).
pub extern "C" fn alloc_kernel_memory(size: u32) -> u32 {
    allocate(unsafe { KERNEL }, size)
}
