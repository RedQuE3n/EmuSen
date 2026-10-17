//! The C layer over emusen-systems: the table as JSON, for the C# runtime's `SystemTable` facade (EmuSen_Backend.md
//! §5.4, §12). Nothing is decided here.

use crate::{fail, put, status};

/// The table as `emusen_systems::to_json` writes it, by the length-query idiom.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_systems_table(out: *mut u8, len: usize) -> i64 {
    match emusen_systems::table_json() {
        Ok(text) => unsafe { put(text.as_bytes(), out, len) },
        Err(e) => fail(status::PARSE, e.to_string()) as i64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_comes_through_the_c_layer_whole() {
        let n = unsafe { emusen_systems_table(std::ptr::null_mut(), 0) };
        assert!(n > 0);
        let mut small = vec![0u8; n as usize - 1];
        assert_eq!(unsafe { emusen_systems_table(small.as_mut_ptr(), small.len()) }, n);
        assert!(small.iter().all(|&b| b == 0), "a buffer too small is not written");
        let mut out = vec![0u8; n as usize];
        assert_eq!(unsafe { emusen_systems_table(out.as_mut_ptr(), out.len()) }, n);
        assert_eq!(std::str::from_utf8(&out).unwrap(), emusen_systems::table_json().unwrap());
    }
}
