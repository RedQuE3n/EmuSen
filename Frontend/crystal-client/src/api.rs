//! The library that serves the frontend API, loaded by its path: its version read first, and a major other than this
//! binding's, or a minor older than the header it was generated from, refused with words before anything else is called.

use crate::error::{Error, Result};
use crate::out::{copy_out, string};
use crate::raw;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// The frontend API as one library serves it.
pub struct Api {
    pub(crate) lib: raw::CrystalLibrary,
    path: PathBuf,
    version: u32,
}

impl Api {
    /// Loads the library at `path` (the platform library, libemusen_platform.so) and checks its version.
    pub fn load(path: impl AsRef<Path>) -> Result<Arc<Api>> {
        let path = path.as_ref().to_path_buf();
        let (lib, version) = emusen_c_calls::open(&path, b"emusen_crystal_abi_version\0")?;
        check_version(&path.display().to_string(), version)?;
        let lib = unsafe { raw::CrystalLibrary::from_library(lib) }
            .map_err(|e| Error::Load { path: path.display().to_string(), words: e.to_string() })?;
        Ok(Arc::new(Api { lib, path, version }))
    }

    /// The library's path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The major and minor the library was built with.
    pub fn version(&self) -> (u32, u32) {
        (self.version >> 16, self.version & 0xffff)
    }

    /// The implementation's words for itself.
    pub fn version_text(&self) -> Result<String> {
        let bytes = copy_out(|o, l| unsafe { self.lib.emusen_crystal_version_text(o, l) }, || self.last_error(std::ptr::null_mut()))?;
        string("version_text", bytes)
    }

    /// The words for the last failing call on this thread.
    pub(crate) fn last_error(&self, rt: *mut raw::emusen_crystal_runtime) -> String {
        emusen_c_calls::words(|o, l| unsafe { self.lib.emusen_crystal_last_error(rt, o, l) })
    }
}

/// A library of this binding's major and at least the header's minor; any other refused with words.
#[allow(clippy::absurd_extreme_comparisons)] // the header's minor is 0 today and only grows
fn check_version(path: &str, version: u32) -> Result<()> {
    let (major, minor) = (version >> 16, version & 0xffff);
    if major != raw::EMUSEN_CRYSTAL_MAJOR || minor < raw::EMUSEN_CRYSTAL_MINOR {
        let (want_major, want_minor) = (raw::EMUSEN_CRYSTAL_MAJOR, raw::EMUSEN_CRYSTAL_MINOR);
        let words = format!(
            "The frontend API's library at {path} is version {major}.{minor}; this program needs {want_major}.{want_minor} or a later minor of {want_major}."
        );
        return Err(Error::Refused { path: path.into(), words });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn another_major_is_refused_with_words_and_a_later_minor_taken() {
        let here = raw::EMUSEN_CRYSTAL_VERSION;
        assert!(check_version("lib", here).is_ok());
        assert!(check_version("lib", here + 1).is_ok(), "a later minor only adds");
        let older = check_version("lib", (raw::EMUSEN_CRYSTAL_MAJOR - 1) << 16 | 13).unwrap_err();
        let (major, minor) = (raw::EMUSEN_CRYSTAL_MAJOR, raw::EMUSEN_CRYSTAL_MINOR);
        assert_eq!(older.to_string(), format!("The frontend API's library at lib is version 1.13; this program needs {major}.{minor} or a later minor of {major}."));
        assert!(check_version("lib", (raw::EMUSEN_CRYSTAL_MAJOR + 1) << 16).is_err());
    }
}
