//! Serenity's device and chain, as far as the game picture needs them (EmuSen_Qt.md §1.8): a device made with the
//! extensions a host's renderer adopts it with, its raw handles, a built-in filter written as a preset and built as a
//! chain, the frame advanced and the picture drawn in place; a preset file read and its parameters merged.

use crate::error::Result;
use crate::platform::Platform;
use crate::raw;
use std::ptr::NonNull;
use std::sync::Arc;

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
    let filter = serde_json::json!({ "key": key }).to_string();
    let (c, cl) = crate::out::text(console);
    let path = platform.read(|out, cap| unsafe {
        platform.lib.emusen_serenity_filter_write(
            filter.as_ptr(),
            filter.len(),
            c,
            cl,
            std::ptr::null(),
            0,
            1,
            0.0,
            0.0,
            root.as_ptr(),
            root.len(),
            out,
            cap,
        )
    })?;
    preset_text(platform, &String::from_utf8_lossy(&path.unwrap_or_default()))
}

/// A `.slangp` preset read through its `#reference` chain, as the document a chain is built from.
fn preset_text(platform: &Platform, path: &str) -> Result<Vec<u8>> {
    Ok(platform.read(|out, cap| unsafe { platform.lib.emusen_serenity_preset_load(path.as_ptr(), path.len(), out, cap) })?.unwrap_or_default())
}

/// A `.slangp` preset read through its `#reference` chain: `{"path", "passes", "textures", "parameters"}`, a float as its
/// 32 bits; a read that fails is a document too, `{"fail", "message", ...}`, as the header gives it.
pub fn preset_load(platform: &Platform, path: &str) -> Result<serde_json::Value> {
    crate::out::json("serenity_preset_load", &preset_text(platform, path)?)
}

/// The parameters the passes declare, merged, the first declaration winning and the overrides' values as defaults:
/// `{"sources": [[parameter, ...], ...], "overrides": [[id, bits], ...]}` in.
pub fn parameters_merge(platform: &Platform, input: &serde_json::Value) -> Result<serde_json::Value> {
    let input = crate::out::to_json(input);
    let bytes = platform
        .read(|out, cap| unsafe { platform.lib.emusen_serenity_parameters_merge(input.as_ptr(), input.len(), out, cap) })?
        .unwrap_or_default();
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
    /// The chain for `preset`, a document `builtin_preset` or a preset's reader gave, compiled with no cache.
    pub fn new(device: &Arc<Device>, preset: &[u8]) -> Result<Chain> {
        let platform = &device.platform;
        let made = unsafe {
            platform.lib.emusen_serenity_chain_new(device.handle.as_ptr(), preset.as_ptr(), preset.len(), std::ptr::null(), 0, 1)
        };
        let handle = NonNull::new(made).ok_or_else(|| {
            let why = platform.read(|out, cap| unsafe { platform.lib.emusen_serenity_take(out, cap) }).ok().flatten().unwrap_or_default();
            let mut words = platform.last_error();
            if !why.is_empty() {
                words = String::from_utf8_lossy(&why).into_owned();
            }
            crate::error::status(raw::EMUSEN_PLATFORM_DEVICE, words)
        })?;
        Ok(Chain { platform: platform.clone(), handle, _device: device.clone() })
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
