//! Volatile access to fixed addresses: the kernel's structures are shared with games and with the exception path.

pub use diamond_guest::mem::{rd, rd8, rd16, wr, wr8, wr16};

/// A byte, sign-extended as the kernel's string functions compare them.
#[inline(always)]
pub fn rd8s(address: u32) -> i32 {
    unsafe { (address as *const i8).read_volatile() as i32 }
}

/// A function at a 32-bit address, called with up to four arguments.
#[inline(always)]
pub fn call(address: u32, a0: u32, a1: u32, a2: u32, a3: u32) -> u32 {
    let f: extern "C" fn(u32, u32, u32, u32) -> u32 = unsafe { core::mem::transmute(address as usize) };
    f(a0, a1, a2, a3)
}

/// A function across the ROM and the kernel's RAM, which lie in different 256 MiB regions that a `jal` cannot cross:
/// its address hidden from the optimiser so that the call goes through a register (Diamond_OpenKernel_Phase2.md §25.3).
#[inline(always)]
pub fn call_far(address: u32, a0: u32, a1: u32, a2: u32, a3: u32) -> u32 {
    call(core::hint::black_box(address), a0, a1, a2, a3)
}
