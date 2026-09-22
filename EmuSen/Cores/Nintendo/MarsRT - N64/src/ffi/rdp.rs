//! A test-only C ABI over MarsRT's display processor alone, for the differential against the C# `Rdp`. See Mars_Native.md §5.3.

use std::ptr;

use crate::rdp::{Rdp, RdpMemory};
use crate::state::{State, StateReader, StateWriter};

/// A null handle, a state of the wrong length, or a state the reader refused.
pub const RDP_STATUS_NULL: i32 = -1;
pub const RDP_STATUS_LENGTH: i32 = -9;

/// # Safety
/// `data` must be valid for `len` bytes, or null with `len` zero.
unsafe fn bytes<'a>(data: *const u8, len: usize) -> &'a [u8] {
    if data.is_null() { &[] } else { unsafe { std::slice::from_raw_parts(data, len) } }
}

/// # Safety
/// `data` must be valid for `len` bytes and not aliased, or null with `len` zero.
unsafe fn bytes_mut<'a>(data: *mut u8, len: usize) -> &'a mut [u8] {
    if data.is_null() { &mut [] } else { unsafe { std::slice::from_raw_parts_mut(data, len) } }
}

/// A processor at power-on, its modes decoded as C#'s constructor decodes them.
#[unsafe(no_mangle)]
pub extern "C" fn mars_rdp_new() -> *mut Rdp {
    Box::into_raw(Box::new(Rdp::default()))
}

/// # Safety
/// `rdp` must come from `mars_rdp_new` and not be used again, or be null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_rdp_free(rdp: *mut Rdp) {
    if !rdp.is_null() {
        drop(unsafe { Box::from_raw(rdp) });
    }
}

/// The C# serializer's bytes for one `Rdp`, whole: zero, or a negative status, leaving the processor as it was.
///
/// # Safety
/// `rdp` must be live or null; `data` valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_rdp_load_state(rdp: *mut Rdp, data: *const u8, len: usize) -> i32 {
    let Some(rdp) = (unsafe { rdp.as_mut() }) else { return RDP_STATUS_NULL };
    let input = unsafe { bytes(data, len) };
    let mut fresh = Rdp::default();
    let mut reader = StateReader::new(input);
    match fresh.read_state(&mut reader) {
        Ok(()) if reader.position() == input.len() => {
            *rdp = fresh;
            0
        }
        Ok(()) => RDP_STATUS_LENGTH,
        Err(e) => e.status(),
    }
}

/// The processor's state in the C# serializer's bytes; returns the length, and writes only when `len` holds it.
///
/// # Safety
/// `rdp` must be live or null; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_rdp_save_state(rdp: *const Rdp, out: *mut u8, len: usize) -> i64 {
    let Some(rdp) = (unsafe { rdp.as_ref() }) else { return RDP_STATUS_NULL as i64 };
    let mut counter = StateWriter::counter();
    rdp.write_state(&mut counter);
    let size = counter.len();
    if !out.is_null() && len >= size {
        let mut writer = StateWriter::new(unsafe { bytes_mut(out, len) });
        rdp.write_state(&mut writer);
    }
    size as i64
}

/// The processor's four kilobytes of texture memory.
///
/// # Safety
/// `rdp` must be live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mars_rdp_texture_memory(rdp: *const Rdp) -> *const u8 {
    unsafe { rdp.as_ref() }.map_or(ptr::null(), |r| r.texture_memory.as_ptr())
}

/// Runs words until one completes a full sync or none are left: returns how many it took, and whether the last synced.
///
/// # Safety
/// `rdp` must be live or null; `words` valid for `count`; `rdram` and `hidden` valid for their lengths and not aliased.
#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn mars_rdp_accept(
    rdp: *mut Rdp,
    words: *const u64,
    count: usize,
    rdram: *mut u8,
    rdram_len: usize,
    hidden: *mut u8,
    hidden_len: usize,
    synced: *mut u32,
) -> i64 {
    let Some(rdp) = (unsafe { rdp.as_mut() }) else { return RDP_STATUS_NULL as i64 };
    let words = if words.is_null() { &[][..] } else { unsafe { std::slice::from_raw_parts(words, count) } };
    let mut memory = RdpMemory { rdram: unsafe { bytes_mut(rdram, rdram_len) }, hidden: unsafe { bytes_mut(hidden, hidden_len) } };

    let mut taken = 0;
    let mut sync = false;
    for &word in words {
        taken += 1;
        if rdp.accept(word, &mut memory) {
            sync = true;
            break;
        }
    }
    if let Some(flag) = unsafe { synced.as_mut() } {
        *flag = sync as u32;
    }
    taken as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_state_goes_out_and_back_through_the_abi() {
        let rdp = mars_rdp_new();
        let size = unsafe { mars_rdp_save_state(rdp, ptr::null_mut(), 0) };
        assert!(size > 80_000, "{size}");
        let mut state = vec![0u8; size as usize];
        state[0] = 0x5A;
        assert_eq!(unsafe { mars_rdp_load_state(rdp, state.as_ptr(), state.len()) }, 0);
        assert_eq!(unsafe { *mars_rdp_texture_memory(rdp) }, 0x5A);
        assert_eq!(unsafe { mars_rdp_load_state(rdp, state.as_ptr(), state.len() - 1) }, -2);
        state.push(0);
        assert_eq!(unsafe { mars_rdp_load_state(rdp, state.as_ptr(), state.len()) }, RDP_STATUS_LENGTH);
        let mut back = vec![0u8; size as usize];
        assert_eq!(unsafe { mars_rdp_save_state(rdp, back.as_mut_ptr(), back.len()) }, size);
        assert_eq!(&back[..], &state[..size as usize]);
        unsafe { mars_rdp_free(rdp) };
    }
}
