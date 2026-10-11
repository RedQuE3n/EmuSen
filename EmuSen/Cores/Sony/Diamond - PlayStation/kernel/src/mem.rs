//! Volatile access to fixed addresses: the kernel's structures are shared with games and with the exception path.

#[inline(always)]
pub fn rd(address: u32) -> u32 {
    unsafe { (address as *const u32).read_volatile() }
}

#[inline(always)]
pub fn wr(address: u32, value: u32) {
    unsafe { (address as *mut u32).write_volatile(value) }
}

#[inline(always)]
pub fn rd16(address: u32) -> u32 {
    unsafe { (address as *const u16).read_volatile() as u32 }
}

#[inline(always)]
pub fn wr16(address: u32, value: u32) {
    unsafe { (address as *mut u16).write_volatile(value as u16) }
}

#[inline(always)]
pub fn rd8(address: u32) -> u32 {
    unsafe { (address as *const u8).read_volatile() as u32 }
}

#[inline(always)]
pub fn wr8(address: u32, value: u32) {
    unsafe { (address as *mut u8).write_volatile(value as u8) }
}

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
