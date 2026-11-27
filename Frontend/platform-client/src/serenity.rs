//! Serenity's device and chain, as far as the game picture needs them (EmuSen_Qt.md §1.8): a device made with the
//! extensions a host's renderer adopts it with, its raw handles, a built-in filter written as a preset and built as a
//! chain, the frame advanced and the picture drawn in place; a preset file read and its parameters merged.

use crate::error::Result;
use crate::platform::{Lent, Platform};
use crate::raw;
use std::path::PathBuf;
use std::ptr::NonNull;
use std::sync::{Arc, OnceLock};

const SPIRV_CACHE_LIMIT: i64 = 64 << 20;
const SPIRV_CACHE_FILE: &str = "spirv-cache.db";
#[cfg(target_os = "macos")]
const SQLITE_LIB: &str = "libsqlite3.dylib";
#[cfg(not(target_os = "macos"))]
const SQLITE_LIB: &str = "libsqlite3.so.0";

fn take_out(platform: &Platform, call: impl FnOnce(*mut u8, usize) -> i64) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; 4096];
    let n = platform.status(call(buf.as_mut_ptr(), buf.len()))? as usize;
    if n <= buf.len() {
        buf.truncate(n);
        return Ok(buf);
    }
    let mut exact = vec![0u8; n];
    let m = platform.status(unsafe { platform.lib.emusen_serenity_take(exact.as_mut_ptr(), exact.len()) })? as usize;
    exact.truncate(m);
    Ok(exact)
}

/// Lends SQLite to Serenity for the process's life, used by the SPIR-V shader cache.
pub fn lend_sqlite(platform: &Platform, sqlite: &Lent) -> Result<()> {
    platform.status(unsafe { platform.lib.emusen_serenity_sqlite_lend(sqlite.handle()) }.into()).map(drop)
}

fn ensure_sqlite(platform: &Platform) -> bool {
    static SQLITE: OnceLock<Option<Lent>> = OnceLock::new();
    let Some(lent) = SQLITE.get_or_init(|| Lent::open(SQLITE_LIB).ok()).as_ref() else {
        return false;
    };
    lend_sqlite(platform, lent).is_ok()
}

fn spirv_cache_path(platform: &Platform) -> Option<PathBuf> {
    platform.directory(crate::galaxia::consts::DIR_SHADERS).ok().flatten().map(|dir| dir.join(SPIRV_CACHE_FILE))
}

struct CacheGuard<'a> {
    platform: &'a Platform,
    handle: *mut raw::emusen_serenity_cache,
}

impl<'a> CacheGuard<'a> {
    fn open(platform: &'a Platform) -> Self {
        let handle = if ensure_sqlite(platform) {
            spirv_cache_path(platform).map_or(std::ptr::null_mut(), |path| {
                let p = path.to_string_lossy();
                unsafe { platform.lib.emusen_serenity_cache_new(p.as_ptr(), p.len(), SPIRV_CACHE_LIMIT, std::ptr::null(), 0) }
            })
        } else {
            std::ptr::null_mut()
        };
        Self { platform, handle }
    }

    fn as_ptr(&self) -> *const raw::emusen_serenity_cache {
        self.handle.cast_const()
    }
}

impl Drop for CacheGuard<'_> {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe { self.platform.lib.emusen_serenity_cache_free(self.handle) };
        }
    }
}

/// Vulkan's handles a host adopts the device by (`emusen_serenity_device_raw`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Raw {
    pub instance: u64,
    pub physical: u64,
    pub device: u64,
    pub queue: u64,
    pub queue_family: u32,
}

/// Serenity's Vulkan device.
pub struct Device {
    platform: Arc<Platform>,
    handle: NonNull<raw::emusen_serenity_device>,
}

// The header: the handle locks itself and may be used from any thread.
unsafe impl Send for Device {}
unsafe impl Sync for Device {}

impl Device {
    /// The device whose name holds `name` (any for none), with `instance` and `device` extensions enabled for a host
    /// to adopt it; with `validate`, the validation layer with synchronisation validation.
    pub fn new_for(platform: &Arc<Platform>, name: Option<&str>, validate: bool, instance: &[&str], device: &[&str]) -> Result<Device> {
        let (n, nl) = crate::out::text(name);
        let (i, d) = (instance.join(" "), device.join(" "));
        let made = unsafe { platform.lib.emusen_serenity_device_new_for(n, nl, validate as i32, i.as_ptr(), i.len(), d.as_ptr(), d.len()) };
        let handle = NonNull::new(made).ok_or_else(|| crate::error::status(raw::EMUSEN_PLATFORM_DEVICE, platform.last_error()))?;
        Ok(Device { platform: platform.clone(), handle })
    }

    /// Vulkan's own handles.
    pub fn raw(&self) -> Raw {
        let get = |which| unsafe { self.platform.lib.emusen_serenity_device_raw(self.handle.as_ptr(), which) };
        Raw {
            instance: get(raw::EMUSEN_SERENITY_RAW_INSTANCE),
            physical: get(raw::EMUSEN_SERENITY_RAW_PHYSICAL),
            device: get(raw::EMUSEN_SERENITY_RAW_DEVICE),
            queue: get(raw::EMUSEN_SERENITY_RAW_QUEUE),
            queue_family: get(raw::EMUSEN_SERENITY_RAW_QUEUE_FAMILY) as u32,
        }
    }

    /// The device's name, as Vulkan gives it.
    pub fn name(&self) -> Result<String> {
        let bytes =
            self.platform.read(|out, cap| unsafe { self.platform.lib.emusen_serenity_device_name(self.handle.as_ptr(), out, cap) })?;
        Ok(String::from_utf8_lossy(&bytes.unwrap_or_default()).into_owned())
    }

    /// What the validation layer has said since the last call, each finding whole.
    pub fn findings(&self) -> Result<Vec<String>> {
        let bytes =
            self.platform.read(|out, cap| unsafe { self.platform.lib.emusen_serenity_device_findings(self.handle.as_ptr(), out, cap) })?;
        Ok(bytes
            .unwrap_or_default()
            .split(|b| *b == 0)
            .filter(|f| !f.is_empty())
            .map(|f| String::from_utf8_lossy(f).into_owned())
            .collect())
    }

    /// The handle the frontend API's open takes as its `device`.
    pub fn as_ptr(&self) -> *const std::ffi::c_void {
        self.handle.as_ptr().cast_const().cast()
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.platform.lib.emusen_serenity_device_close(self.handle.as_ptr());
            self.platform.lib.emusen_serenity_device_free(self.handle.as_ptr());
        }
    }
}

/// The image a draw left its picture in, valid until the chain's next call.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Drawn {
    pub image: u64,
    pub view: u64,
    pub width: u32,
    pub height: u32,
    pub format: u32,
    pub layout: u32,
}

/// A built-in filter's preset written under `root` (by its catalog key, such as `effect.None.nearest`) and read as
/// the document a chain is built from.
pub fn builtin_preset(platform: &Platform, key: &str, console: Option<&str>, root: &str) -> Result<Vec<u8>> {
    filter_preset(platform, key, console, None, root)
}

/// A built-in filter's preset on `console` with the player's `chosen` values (`{id: an f32's bits}`, none for its
/// defaults there), written under `root` and read as the document a chain is built from (`emusen_serenity_filter_write`).
pub fn filter_preset(platform: &Platform, key: &str, console: Option<&str>, chosen: Option<&serde_json::Value>, root: &str) -> Result<Vec<u8>> {
    let filter = serde_json::json!({ "key": key }).to_string();
    let (c, cl) = crate::out::text(console);
    let chosen = chosen.map(serde_json::Value::to_string);
    let (v, vl) = chosen.as_deref().map_or((std::ptr::null(), 0), |v| (v.as_ptr(), v.len()));
    let path = platform.read(|out, cap| unsafe {
        platform.lib.emusen_serenity_filter_write(filter.as_ptr(), filter.len(), c, cl, v, vl, 1, 0.0, 0.0, root.as_ptr(), root.len(), out, cap)
    })?;
    preset_text(platform, &String::from_utf8_lossy(&path.unwrap_or_default()))
}

/// A `.slangp` preset read through its `#reference` chain, as the document a chain is built from.
pub fn preset_document(platform: &Platform, path: &str) -> Result<Vec<u8>> {
    preset_text(platform, path)
}

fn preset_text(platform: &Platform, path: &str) -> Result<Vec<u8>> {
    take_out(platform, |out, cap| unsafe { platform.lib.emusen_serenity_preset_load(path.as_ptr(), path.len(), out, cap) })
}

/// Serenity's catalogue (`emusen_serenity_filters`): the filters with their settings, the screens with their kinds, and
/// the crop's limits, as the header describes the document.
pub fn catalogue(platform: &Platform) -> Result<serde_json::Value> {
    let bytes = take_out(platform, |out, cap| unsafe { platform.lib.emusen_serenity_filters(out, cap) })?;
    crate::out::json("serenity_filters", &bytes)
}

/// A `.slangp` preset read through its `#reference` chain: `{"path", "passes", "textures", "parameters"}`, a float as its
/// 32 bits; a read that fails is a document too, `{"fail", "message", ...}`, as the header gives it.
pub fn preset_load(platform: &Platform, path: &str) -> Result<serde_json::Value> {
    crate::out::json("serenity_preset_load", &preset_text(platform, path)?)
}

/// A `.slang` source with its includes in place: `{"path", "vertex", "fragment", "name", "format", "parameters"}`, a
/// float as its 32 bits; a read that fails is a document too, `{"fail", "message", ...}`, as the header gives it.
pub fn source_load(platform: &Platform, path: &str) -> Result<serde_json::Value> {
    let bytes = take_out(platform, |out, cap| unsafe { platform.lib.emusen_serenity_source_load(path.as_ptr(), path.len(), out, cap) })?;
    crate::out::json("serenity_source_load", &bytes)
}

/// The parameters the passes declare, merged, the first declaration winning and the overrides' values as defaults:
/// `{"sources": [[parameter, ...], ...], "overrides": [[id, bits], ...]}` in.
pub fn parameters_merge(platform: &Platform, input: &serde_json::Value) -> Result<serde_json::Value> {
    let input = crate::out::to_json(input);
    let bytes = take_out(platform, |out, cap| unsafe { platform.lib.emusen_serenity_parameters_merge(input.as_ptr(), input.len(), out, cap) })?;
    crate::out::json("serenity_parameters_merge", &bytes)
}

/// A preset built on a device. One thread at a time; it holds its device.
pub struct Chain {
    platform: Arc<Platform>,
    handle: NonNull<raw::emusen_serenity_chain>,
    _device: Arc<Device>,
}

// The header: a chain is used by one thread at a time.
unsafe impl Send for Chain {}

impl Chain {
    /// The chain for `preset`, compiled through `spirv-cache.db` with parallel pass builders.
    pub fn new(device: &Arc<Device>, preset: &[u8]) -> Result<Chain> {
        let platform = &device.platform;
        let cache = CacheGuard::open(platform);
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64);
        let builders = std::thread::available_parallelism().map_or(1, |n| n.get()).min(i32::MAX as usize) as i32;
        let made = unsafe {
            platform.lib.emusen_serenity_chain_new(device.handle.as_ptr(), preset.as_ptr(), preset.len(), cache.as_ptr(), now, builders)
        };
        let handle = NonNull::new(made).ok_or_else(|| {
            let why = take_out(platform, |out, cap| unsafe { platform.lib.emusen_serenity_take(out, cap) }).unwrap_or_default();
            let mut words = platform.last_error();
            if !why.is_empty() {
                words = String::from_utf8_lossy(&why).into_owned();
            }
            crate::error::status(raw::EMUSEN_PLATFORM_DEVICE, words)
        })?;
        Ok(Chain { platform: platform.clone(), handle, _device: device.clone() })
    }

    /// Every parameter the passes declare, in the chain's order, with its value as the preset gives it.
    pub fn parameters(&self) -> Result<Vec<(String, f32)>> {
        let bytes = take_out(&self.platform, |out, cap| unsafe { self.platform.lib.emusen_serenity_chain_parameters(self.handle.as_ptr(), out, cap) })?;
        let all: Vec<serde_json::Value> = crate::out::json("serenity_chain_parameters", &bytes)?;
        Ok(all.iter().map(|p| (p["id"].as_str().unwrap_or_default().to_owned(), f32::from_bits(p["initial"].as_u64().unwrap_or_default() as u32))).collect())
    }

    /// A value for each parameter, in the order `parameters` gives them, read by the next draw.
    pub fn set_parameters(&mut self, values: &[f32]) -> Result<()> {
        let n = unsafe { self.platform.lib.emusen_serenity_chain_set_parameters(self.handle.as_ptr(), values.as_ptr(), values.len()) };
        self.platform.status(n.into()).map(drop)
    }

    /// A new frame, RGBA8, each row shown `row_repeat` times.
    pub fn advance(&mut self, rgba: &[u8], width: i32, height: i32, row_repeat: i32) -> Result<()> {
        let n = unsafe {
            self.platform.lib.emusen_serenity_chain_advance(self.handle.as_ptr(), rgba.as_ptr(), rgba.len(), width, height, row_repeat)
        };
        self.platform.status(n.into()).map(drop)
    }

    /// The newest picture a machine took through `lend` (a runtime's `picture_lend`) as the new frame, read on the device
    /// by the next draw: true when it was newer than the last.
    ///
    /// # Safety
    /// `lend` is a live `emusen_serenity_lend` made on this chain's device, alive until the next draw.
    pub unsafe fn advance_lent(&mut self, lend: NonNull<std::ffi::c_void>) -> Result<bool> {
        let n = unsafe { self.platform.lib.emusen_serenity_chain_advance_lent(self.handle.as_ptr(), lend.as_ptr().cast()) };
        self.platform.status(n.into()).map(|n| n == 1)
    }

    /// The picture in `ring` (a runtime's `picture_ring`) at `slot` once its ready reaches `value`, as the new frame; the
    /// next draw copies it and gives the slot back.
    ///
    /// # Safety
    /// `ring` is a live `emusen_serenity_ring` made on this chain's device, alive until the next draw.
    pub unsafe fn advance_ring(&mut self, ring: NonNull<std::ffi::c_void>, slot: u32, value: u64, width: i32, height: i32) -> Result<()> {
        let n = unsafe {
            self.platform.lib.emusen_serenity_chain_advance_ring(self.handle.as_ptr(), ring.as_ptr().cast(), slot, value, width, height)
        };
        self.platform.status(n.into()).map(drop)
    }

    /// Every pass run for a view of `width` by `height`, the picture left in place for the adopting host to sample.
    pub fn draw(&mut self, width: i32, height: i32) -> Result<Drawn> {
        let mut out = raw::emusen_serenity_drawn { size: size_of::<raw::emusen_serenity_drawn>() as u32, ..Default::default() };
        let n = unsafe { self.platform.lib.emusen_serenity_chain_draw(self.handle.as_ptr(), width, height, &mut out) };
        self.platform.status(n.into())?;
        Ok(Drawn { image: out.image, view: out.view, width: out.width, height: out.height, format: out.format, layout: out.layout })
    }
}

impl Drop for Chain {
    fn drop(&mut self) {
        unsafe { self.platform.lib.emusen_serenity_chain_free(self.handle.as_ptr()) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use emusen_client_fixture::{World, library};

    #[test]
    fn large_preset_and_catalogue_use_serenity_take_and_cache_opens_in_shaders_dir() {
        let world = World::new("serenity-cache-and-take");
        let p = Platform::load(library()).unwrap();
        let cat = catalogue(&p).unwrap();
        assert!(cat["filters"].as_array().is_some_and(|f| !f.is_empty()));

        let dir = world.dir("slang");
        let preset_path = dir.join("big.slangp");
        let mut slangp = String::from("shaders = 1\nshader0 = p0.slang\nparameters = \"");
        let mut names = Vec::new();
        let mut values = String::new();
        for i in 0..250 {
            let name = format!("param_{i:03}");
            names.push(name.clone());
            values.push_str(&format!("{name} = 1.0\n"));
        }
        slangp.push_str(&names.join(";"));
        slangp.push_str("\"\n");
        slangp.push_str(&values);
        std::fs::write(&preset_path, &slangp).unwrap();
        let loaded = preset_load(&p, preset_path.to_str().unwrap()).unwrap();
        assert_eq!(loaded["parameters"].as_array().map(Vec::len), Some(250));

        let path_str = preset_path.to_str().unwrap();
        let n = unsafe { p.lib.emusen_serenity_preset_load(path_str.as_ptr(), path_str.len(), std::ptr::null_mut(), 0) };
        assert!(n > 4096, "preset JSON must exceed 4 KiB: {n}");
        std::fs::remove_file(&preset_path).unwrap();
        let mut taken = vec![0u8; n as usize];
        let m = unsafe { p.lib.emusen_serenity_take(taken.as_mut_ptr(), taken.len()) };
        assert_eq!(m, n);

        let cache = CacheGuard::open(&p);
        assert!(!cache.as_ptr().is_null(), "spirv-cache.db must open once SQLite is lent");
        assert!(spirv_cache_path(&p).is_some_and(|path| path.is_file()));
        let src = "#version 450\nvoid main() {}\n";
        let name = "a.slang";
        let mut out = vec![0u8; 4096];
        let s1 = unsafe { p.lib.emusen_serenity_cache_compile(cache.as_ptr(), src.as_ptr(), src.len(), 1, name.as_ptr(), name.len(), 100, out.as_mut_ptr(), out.len()) };
        assert!(s1 > 0);
        let s2 = unsafe { p.lib.emusen_serenity_cache_compile(cache.as_ptr(), src.as_ptr(), src.len(), 1, name.as_ptr(), name.len(), 100, out.as_mut_ptr(), out.len()) };
        assert_eq!(s1, s2);
        assert_eq!(unsafe { p.lib.emusen_serenity_cache_get(cache.as_ptr(), raw::EMUSEN_SERENITY_CACHE_HITS) }, 1);
    }
}
