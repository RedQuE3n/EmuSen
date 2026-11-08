//! The platform library, loaded by its path and matched exactly: its interface number is read first and any other than
//! this binding's refused with words before anything else is called (EmuSen_RustPlatform.md §3.2's rule B).

use crate::error::{Error, Result};
use crate::out::{check, copy_out, words};
use crate::raw;
use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// The platform library as one process has it loaded.
pub struct Platform {
    pub(crate) lib: raw::PlatformLibrary,
    path: PathBuf,
}

impl Platform {
    /// Loads the library at `path` (libemusen_platform.so) and checks its interface number.
    pub fn load(path: impl AsRef<Path>) -> Result<Arc<Platform>> {
        let path = path.as_ref().to_path_buf();
        let shown = path.display().to_string();
        let (lib, found) = emusen_c_calls::open(&path, b"emusen_platform_abi_version\0")?;
        check_interface(&shown, found)?;
        let lib = unsafe { raw::PlatformLibrary::from_library(lib) }.map_err(|e| Error::Load { path: shown, words: e.to_string() })?;
        Ok(Arc::new(Platform { lib, path }))
    }

    /// The library's path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The interface number, the same as this binding's once loaded.
    pub fn interface(&self) -> u32 {
        unsafe { self.lib.emusen_platform_abi_version() }
    }

    /// The words for the last failing call on this thread.
    pub fn last_error(&self) -> String {
        words(|o, l| unsafe { self.lib.emusen_platform_last_error(o, l) })
    }

    pub(crate) fn status(&self, n: i64) -> Result<i64> {
        check(n, || self.last_error())
    }

    pub(crate) fn read(&self, call: impl FnMut(*mut u8, usize) -> i64) -> Result<Option<Vec<u8>>> {
        copy_out(call, || self.last_error())
    }

    /// The calling thread's waiting diagnostics, taken.
    pub fn diagnostics(&self) -> Result<Vec<String>> {
        let bytes = self.read(|o, l| unsafe { self.lib.emusen_platform_diagnostics(o, l) })?.unwrap_or_default();
        Ok(bytes.split(|b| *b == 0).filter(|m| !m.is_empty()).map(|m| String::from_utf8_lossy(m).into_owned()).collect())
    }

    /// Lends SDL3, loaded once, for the process's life; nothing of Endymion's that needs SDL is made before.
    pub fn lend_sdl(&self, sdl: &Lent) -> Result<()> {
        self.status(unsafe { self.lib.emusen_endymion_sdl_lend(sdl.handle()) }.into()).map(drop)
    }
}

/// The interface number this binding was generated for, and no other.
fn check_interface(path: &str, found: u32) -> Result<()> {
    if found != raw::EMUSEN_PLATFORM_ABI_VERSION {
        let wanted = raw::EMUSEN_PLATFORM_ABI_VERSION;
        let words = format!(
            "The platform library at {path} has interface {found}; this program was built for interface {wanted}, and the two are built from the same commit."
        );
        return Err(Error::Refused { path: path.into(), words });
    }
    Ok(())
}

/// A host library lent to the platform library (SDL3, SQLite): loaded by its path or name and kept for the process's
/// life, since the platform library holds it and never closes it.
#[derive(Clone, Copy, Debug)]
pub struct Lent {
    handle: *mut c_void,
}

// A loader's handle, never closed, read by the platform library from any thread.
unsafe impl Send for Lent {}
unsafe impl Sync for Lent {}

impl Lent {
    /// The library at `name` (a path, or a name the loader searches for), loaded and never closed.
    pub fn open(name: &str) -> Result<Lent> {
        let lib =
            unsafe { libloading::os::unix::Library::new(name) }.map_err(|e| Error::Load { path: name.into(), words: e.to_string() })?;
        Ok(Lent { handle: lib.into_raw() })
    }

    pub(crate) fn handle(&self) -> *mut c_void {
        self.handle
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn any_interface_but_this_binding_s_is_refused_with_words() {
        assert!(check_interface("lib", raw::EMUSEN_PLATFORM_ABI_VERSION).is_ok());
        let older = check_interface("lib", raw::EMUSEN_PLATFORM_ABI_VERSION - 1).unwrap_err();
        let wanted = raw::EMUSEN_PLATFORM_ABI_VERSION;
        assert_eq!(
            older.to_string(),
            format!(
                "The platform library at lib has interface {}; this program was built for interface {wanted}, and the two are built from the same commit.",
                wanted - 1
            )
        );
        assert!(check_interface("lib", raw::EMUSEN_PLATFORM_ABI_VERSION + 1).is_err(), "a newer interface is not taken either");
    }
}
