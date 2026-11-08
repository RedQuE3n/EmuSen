//! The words of a thread's last failing call, one slot for each header with its own `*_last_error`, copied out by
//! the length query. See EmuSen_RustPlatform.md §47.2.

use std::cell::RefCell;

/// A header whose last-error export keeps its own words; the shell's shares the platform's.
#[derive(Clone, Copy)]
pub(crate) enum Slot {
    Platform,
    Crystal,
    Session,
    Library,
    Scrape,
}

thread_local! {
    static WORDS: RefCell<[String; 5]> = const { RefCell::new([const { String::new() }; 5]) };
}

/// Records why a call failed and returns its status.
pub(crate) fn fail(slot: Slot, status: i32, words: String) -> i32 {
    WORDS.with_borrow_mut(|all| all[slot as usize] = words);
    status
}

/// A slot's words by the length-query idiom.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
pub(crate) unsafe fn copy_out(slot: Slot, out: *mut u8, len: usize) -> i64 {
    WORDS.with_borrow(|all| unsafe { crate::put(all[slot as usize].as_bytes(), out, len) })
}
