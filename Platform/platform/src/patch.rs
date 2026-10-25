//! Soft patching's C layer (EmuSen_GamePackages.md §5.4, §15): a packaged game's image with its package's patches
//! applied, for the C# runtime, which applies them through this library as Crystal in Rust does through `emusen-patch`
//! itself. The result is a Galaxia mapping that holds the bytes, so the host reads, lends and frees it as it does a
//! mapped file.

use crate::{bytes, fail, required, status};
use emusen_galaxia::mapped::Mapped;

/// The image `image` with the patches of the package at `folder` applied: 0 with the mapping that holds them in
/// `mapped`, freed by `emusen_galaxia_unmap`; 0 with null where the package lists no patch; REFUSED with words naming
/// the patch and both values where one does not apply, and IO where the manifest does not read.
///
/// # Safety
/// `folder` and `image` must be valid for their lengths (`image` may be null for none); `mapped` must be valid to write.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_patch_package_image(folder: *const u8, folder_len: usize, image: *const u8, image_len: usize, mapped: *mut *mut Mapped) -> i32 {
    if mapped.is_null() {
        return fail(status::NULL, "no place for the patched image's handle");
    }
    unsafe { *mapped = std::ptr::null_mut() };
    let folder = match unsafe { required(folder, folder_len) } {
        Ok(f) => f,
        Err(code) => return code,
    };
    let source = match unsafe { bytes(image, image_len) } {
        Ok(b) => b,
        Err(code) => return code,
    };
    let manifest = emusen_galaxia::dotnet_files::join(folder, emusen_galaxia::package::FILE_NAME);
    let doc = match emusen_galaxia::package::read(&manifest) {
        Ok(Ok(doc)) => doc,
        Ok(Err(refusal)) => return fail(status::IO, refusal.words()),
        Err(e) => return fail(status::IO, format!("{manifest}: {e}")),
    };
    match emusen_patch::package::image(folder, &doc, source) {
        Ok(Some(patched)) => {
            unsafe { *mapped = Box::into_raw(Box::new(Mapped::held(patched))) };
            0
        }
        Ok(None) => 0,
        Err(words) => fail(status::REFUSED, words),
    }
}
