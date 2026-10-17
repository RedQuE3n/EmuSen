//! Volatile access to fixed addresses: the kernel's structures are shared with games and with the exception path, and
//! the hardware's registers are read for their side effects.

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
