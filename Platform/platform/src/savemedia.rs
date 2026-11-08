//! The C layer over Galaxia's medium lock, for the C# `MediumLock` behind Galaxia's own switch (EmuSen_Backend.md
//! §15.2). The card manager is the frontend API's `cards_op` since 2.0 (EmuSen_RustPlatform.md §47.24).

use crate::{fail, put, required, status, text};
use emusen_galaxia::medium_lock;

/// `MediumLock.TryTake`: a handle above zero, or zero where another program holds the file or this one holds it unshared.
///
/// # Safety
/// Each pointer must be valid for its length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_medium_lock_take(file: *const u8, file_len: usize, holder: *const u8, holder_len: usize, shared: i32) -> u64 {
    let (Ok(file), Ok(holder)) = (unsafe { required(file, file_len) }, unsafe { text(holder, holder_len) }) else { return 0 };
    match medium_lock::try_take(file, holder.unwrap_or(""), shared != 0) {
        Some(lock) => {
            let id = lock.id;
            std::mem::forget(lock);
            id
        }
        None => 0,
    }
}

/// Lets go of a handle; 1 when it was held, 0 when it was let go already.
#[unsafe(no_mangle)]
pub extern "C" fn emusen_galaxia_medium_lock_release(handle: u64) -> i32 {
    i32::from(medium_lock::release(handle))
}

/// `MediumLock.HolderOf`: the holder's words, by the length-query idiom; ABSENT when no one holds the file.
///
/// # Safety
/// `file` must be valid for `file_len` bytes; `out` for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_galaxia_medium_lock_holder(file: *const u8, file_len: usize, out: *mut u8, len: usize) -> i64 {
    let file = match unsafe { required(file, file_len) } {
        Ok(f) => f,
        Err(s) => return s as i64,
    };
    match medium_lock::holder_of(file) {
        Some(holder) => unsafe { put(holder.as_bytes(), out, len) },
        None => fail(status::ABSENT, "no one holds the file") as i64,
    }
}
