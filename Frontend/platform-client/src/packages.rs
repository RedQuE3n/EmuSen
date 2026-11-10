//! Galaxia's game packages (emusen_platform.h, `emusen_galaxia_package_*`; EmuSen_GamePackages.md §3): a manifest read,
//! written and laid out, the claim index over the packages folder, and what a path opens as. A manifest crosses as its
//! text, so that every member stays in its place, those this build does not know included; the index and a resolve's
//! answer cross as `serde_json::Value`.

use crate::error::Result;
use crate::out::{json, string, text};
use crate::platform::Platform;
use serde_json::Value;

/// Where a package's paths are read from: the packages folder, the library (`AppSettings.RomDirectory`, none for
/// none) and the compressed discs' folder, each a full path.
#[derive(Clone, Copy, Debug)]
pub struct Roots<'a> {
    pub packages: &'a str,
    pub library: Option<&'a str>,
    pub compressed: &'a str,
}

impl Platform {
    /// The manifest at `file`, checked, as compact JSON.
    pub fn package_read(&self, file: &str) -> Result<String> {
        let bytes = self.read(|o, l| unsafe { self.lib.emusen_galaxia_package_read(file.as_ptr(), file.len(), o, l) })?.unwrap_or_default();
        string("package_read", bytes)
    }

    /// A manifest checked and written at `file`, atomically, laid out as `package_format` lays it; a refused one is not written.
    pub fn package_write(&self, file: &str, manifest: &str) -> Result<()> {
        let n = unsafe { self.lib.emusen_galaxia_package_write(file.as_ptr(), file.len(), manifest.as_ptr(), manifest.len()) };
        self.status(n.into()).map(drop)
    }

    /// A manifest checked and laid out as it is written.
    pub fn package_format(&self, manifest: &str) -> Result<String> {
        let bytes = self
            .read(|o, l| unsafe { self.lib.emusen_galaxia_package_format(manifest.as_ptr(), manifest.len(), o, l) })?
            .unwrap_or_default();
        string("package_format", bytes)
    }

    /// The claim index: `{"folders", "packages": [{"folder", "system", "game", "claims"}]}`.
    pub fn package_index(&self, roots: Roots) -> Result<Value> {
        let (l, ll) = text(roots.library);
        let (p, c) = (roots.packages, roots.compressed);
        let bytes = self
            .read(|o, len| unsafe { self.lib.emusen_galaxia_package_index(p.as_ptr(), p.len(), l, ll, c.as_ptr(), c.len(), o, len) })?
            .unwrap_or_default();
        json("package_index", &bytes)
    }

    /// What `at` opens as: `{"game", "package", "moved"}`; a moved game's manifest is rewritten, once, the library
    /// answering the fill from the work its length query did.
    pub fn package_resolve(&self, at: &str, roots: Roots) -> Result<Value> {
        let (l, ll) = text(roots.library);
        let (p, c) = (roots.packages, roots.compressed);
        let bytes = self
            .read(|o, len| unsafe {
                self.lib.emusen_galaxia_package_resolve(at.as_ptr(), at.len(), p.as_ptr(), p.len(), l, ll, c.as_ptr(), c.len(), o, len)
            })?
            .unwrap_or_default();
        json("package_resolve", &bytes)
    }
}
