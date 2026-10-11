//! The calls whose devices arrive at stage 10 (the CD and its files, the memory cards, the ROM's character sets):
//! each returns the failure psx-spx documents for it. Diamond_OpenKernel.md §5.

/// The calls that return a count or a handle: -1.
pub extern "C" fn failed() -> u32 {
    u32::MAX
}

/// The calls that return 0 when they fail, or nothing.
pub extern "C" fn nothing() -> u32 {
    0
}

/// B(5Ch) and B(5Dh): 11h, failed or timed out, as with no card inserted.
pub extern "C" fn card_status() -> u32 {
    0x11
}

/// A(A0h): a warm boot needs the CD; with none it stops as SystemError("B") does.
pub extern "C" fn boot() -> u32 {
    unsafe { crate::entry::system_error() }
}

static mut FIND_MODE: u32 = 0;

/// C(1Ah).
pub extern "C" fn set_card_find_mode(mode: u32) -> u32 {
    unsafe { FIND_MODE = mode };
    0
}

/// C(1Dh).
pub extern "C" fn get_card_find_mode() -> u32 {
    unsafe { FIND_MODE }
}
