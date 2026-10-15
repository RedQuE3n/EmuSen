//! Serenity's C layer for the Vulkan device and the chain.
//!
//! A device handle locks itself and may be used from any thread; a chain is one thread's at a time, and its handle
//! holds a lock so that a mistake about that is a wait and not a race. What the validation layer says and what a
//! probe saw wait in the device's handle to be taken: the library never calls its host. A chain that cannot be built
//! leaves a document of why on the calling thread, since the exception the C# throws for it has a type as well as
//! words. A readback lent to a picture is the one pointer into the library's memory that crosses, given back by its
//! number from any thread. See EmuSen_RustPlatform.md §3.3 and §17.

use crate::serenity::{CacheHandle, bytes, document, failure, fault_json, leave, list, number, object, out_json, parameter_json, shaderc};
use crate::{fail, put, required, status, text};
use emusen_galaxia::json::Value;
use emusen_serenity::cache;
use emusen_serenity::chain::{self, Chain, Failure, Unfed};
use emusen_serenity::compiler::Stage;
use emusen_serenity::format;
use emusen_serenity::lend::Lend;
use emusen_serenity::preset::{PassSpec, Preset, ScaleType, TextureSpec, Wrap};
use emusen_serenity::vulkan::{Device, Findings, Mark, Shown};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A device, what its layer has said, and what of either is waiting for a buffer large enough.
pub struct DeviceHandle {
    device: Mutex<Option<Device>>,
    findings: Arc<Findings>,
    heard: Mutex<Vec<u8>>,
    marks: Mutex<Vec<Mark>>,
}

/// # Safety
/// `h` from `emusen_serenity_device_new`, not freed, or null.
unsafe fn device(h: *const DeviceHandle) -> Result<Device, i32> {
    let handle = unsafe { h.as_ref() }.ok_or_else(|| fail(status::NULL, "no handle"))?;
    locked(&handle.device).clone().ok_or_else(|| fail(status::DEVICE, "The device has been closed."))
}

/// The device whose name holds `name` whatever its case (null or empty for any), a discrete one before an integrated
/// one before a virtual one. With `validate`, the Khronos layer is loaded with synchronisation validation and its
/// findings wait for `emusen_serenity_device_findings`. Null when there is none, with the C#'s report of why as the
/// words.
///
/// # Safety
/// `name` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_device_new(name: *const u8, len: usize, validate: i32) -> *mut DeviceHandle {
    let Ok(name) = (unsafe { text(name, len) }) else { return std::ptr::null_mut() };
    match Device::create(name, validate != 0) {
        Ok(device) => {
            let findings = device.findings();
            Box::into_raw(Box::new(DeviceHandle { device: Mutex::new(Some(device)), findings, heard: Mutex::default(), marks: Mutex::default() }))
        }
        Err(report) => {
            fail(status::DEVICE, report);
            std::ptr::null_mut()
        }
    }
}

/// Lets go of the device, which is destroyed once no chain and no lent readback made on it is left; what its layer
/// said, to the end, stays to be taken until the handle is freed.
///
/// # Safety
/// `h` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_device_close(h: *const DeviceHandle) -> i32 {
    let Some(handle) = (unsafe { h.as_ref() }) else { return fail(status::NULL, "no handle") };
    let device = locked(&handle.device).take();
    drop(device);
    0
}

/// # Safety
/// `h` from `emusen_serenity_device_new`, not freed, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_device_free(h: *mut DeviceHandle) -> i32 {
    if !h.is_null() {
        drop(unsafe { Box::from_raw(h) });
    }
    0
}

/// The device's name, as Vulkan gives it.
///
/// # Safety
/// `h` live; `out` valid for `cap` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_device_name(h: *const DeviceHandle, out: *mut u8, cap: usize) -> i64 {
    match unsafe { device(h) } {
        Ok(device) => unsafe { put(device.name().as_bytes(), out, cap) },
        Err(status) => status as i64,
    }
}

/// What the validation layer has said of the device's calls, each finding ended by a NUL, removed once a buffer has
/// taken them all.
///
/// # Safety
/// `h` live; `out` valid for `cap` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_device_findings(h: *const DeviceHandle, out: *mut u8, cap: usize) -> i64 {
    let Some(handle) = (unsafe { h.as_ref() }) else { return fail(status::NULL, "no handle") as i64 };
    let mut heard = locked(&handle.heard);
    heard.extend(crate::join_nul(&handle.findings.take()));
    let length = unsafe { put(&heard, out, cap) };
    if !out.is_null() && heard.len() <= cap {
        heard.clear();
    }
    length
}

pub const RAW_INSTANCE: u32 = 0;
pub const RAW_DEVICE: u32 = 1;

/// The `VkInstance` or the `VkDevice` itself, for a caller that makes a Vulkan call of its own on it; zero for a
/// closed device or a kind there is none of. The device's queue is the library's.
///
/// # Safety
/// `h` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_device_raw(h: *const DeviceHandle, which: u32) -> u64 {
    match (unsafe { device(h) }, which) {
        (Ok(device), RAW_INSTANCE) => device.raw_instance(),
        (Ok(device), RAW_DEVICE) => device.raw_device(),
        _ => 0,
    }
}

/// The device's UUID into `uuid` (16 bytes), and 1 when it can export images and semaphores as opaque file
/// descriptors, 0 when it cannot (EmuSen_DevicePicture.md §2, §4.4).
///
/// # Safety
/// `h` live; `uuid` valid for 16 bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_device_identity(h: *const DeviceHandle, uuid: *mut u8) -> i32 {
    let device = match unsafe { device(h) } {
        Ok(device) => device,
        Err(status) => return status,
    };
    if uuid.is_null() {
        return fail(status::NULL, "nowhere to put the UUID");
    }
    unsafe { std::ptr::copy_nonoverlapping(device.uuid().as_ptr(), uuid, 16) };
    device.exports() as i32
}

/// `emusen_serenity_shown_info`: a target's size and its three descriptors, each the caller's.
#[repr(C)]
pub struct ShownInfo {
    pub size: u32,
    pub width: i32,
    pub height: i32,
    pub memory_fd: i32,
    pub memory_bytes: u64,
    pub ready_fd: i32,
    pub free_fd: i32,
}

/// The target's memory laid out row after row rather than in the device's own tiling.
pub const SHOWN_LINEAR: u32 = 1;

pub struct ShownHandle {
    shown: Shown,
}

/// An image a chain's picture is presented in, laid out linearly with `SHOWN_LINEAR`, exported with its two
/// semaphores; `out` receives their descriptors,
/// which are the caller's to hand on or close. Null when the device cannot export, with the words why.
///
/// # Safety
/// `d` live; `out` valid, with its size set.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_shown_new(d: *const DeviceHandle, width: i32, height: i32, flags: u32, out: *mut ShownInfo) -> *mut ShownHandle {
    let made = (|| {
        let device = unsafe { device(d) }?;
        let out = unsafe { out.as_mut() }.ok_or_else(|| fail(status::NULL, "nowhere to put the target"))?;
        if (out.size as usize) < std::mem::size_of::<ShownInfo>() {
            return Err(fail(status::BAD_ARGUMENT, "the target's struct is too small"));
        }
        if width < 1 || height < 1 {
            return Err(fail(status::BAD_ARGUMENT, "a target is at least one by one"));
        }
        let (shown, fds) = device.create_target(width as u32, height as u32, flags & SHOWN_LINEAR != 0).map_err(|words| fail(status::DEVICE, words))?;
        (out.width, out.height, out.memory_fd, out.memory_bytes, out.ready_fd, out.free_fd) = (width, height, fds.memory, shown.memory_bytes, fds.ready, fds.free);
        Ok(ShownHandle { shown })
    })();
    match made {
        Ok(handle) => Box::into_raw(Box::new(handle)),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Frees a target, waiting for the device to finish with it first.
///
/// # Safety
/// `h` from `emusen_serenity_shown_new`, not freed, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_shown_free(h: *mut ShownHandle) -> i32 {
    if !h.is_null() {
        drop(unsafe { Box::from_raw(h) });
    }
    0
}

/// `emusen_serenity_lend_info`: what a core is lent, as the core ABI's `emusen_device_lend` holds it.
#[repr(C)]
pub struct LendInfo {
    pub size: u32,
    pub api_version: u32,
    pub get_instance_proc_addr: u64,
    pub instance: u64,
    pub physical_device: u64,
    pub device: u64,
    pub queue_family: u32,
    pub queue_index: u32,
    pub host_queue_family: u32,
    pub reserved: u32,
    pub done: u64,
}

pub struct LendHandle {
    lend: Lend,
}

/// The device lent to one core machine, `out` filled with what the core is given; null when the device cannot lend,
/// with the words why.
///
/// # Safety
/// `d` live; `out` valid, with its size set.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_lend_new(d: *const DeviceHandle, out: *mut LendInfo) -> *mut LendHandle {
    let made = (|| {
        let device = unsafe { device(d) }?;
        let out = unsafe { out.as_mut() }.ok_or_else(|| fail(status::NULL, "nowhere to put the lend"))?;
        if (out.size as usize) < std::mem::size_of::<LendInfo>() {
            return Err(fail(status::BAD_ARGUMENT, "the lend's struct is too small"));
        }
        let lend = Lend::new(&device).map_err(|words| fail(status::DEVICE, words))?;
        let l = lend.lent();
        (out.api_version, out.get_instance_proc_addr, out.instance, out.physical_device, out.device) = (l.api, l.get_instance_proc_addr as u64, l.instance, l.physical, l.device);
        (out.queue_family, out.queue_index, out.host_queue_family, out.reserved, out.done) = (l.family, l.queue, l.host_family, 0, l.done);
        Ok(LendHandle { lend })
    })();
    match made {
        Ok(handle) => Box::into_raw(Box::new(handle)),
        Err(_) => std::ptr::null_mut(),
    }
}

/// The core's picture taken: 1, 0 for one not newer than the last, or a status.
///
/// # Safety
/// `h` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_lend_take(h: *const LendHandle, buffer: u64, offset: u64, width: i32, height: i32, stride: i32, ready: u64, value: u64) -> i32 {
    let Some(handle) = (unsafe { h.as_ref() }) else { return fail(status::NULL, "no lend") };
    if width < 1 || height < 1 || stride < 1 {
        return fail(status::BAD_ARGUMENT, "a picture is at least one by one");
    }
    match handle.lend.take(buffer, offset, width as u32, height as u32, stride as u32, ready, value) {
        Ok(taken) => taken as i32,
        Err(words) => fail(status::DEVICE, words),
    }
}

/// Waits for every copy taken.
///
/// # Safety
/// `h` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_lend_settle(h: *const LendHandle) -> i32 {
    let Some(handle) = (unsafe { h.as_ref() }) else { return fail(status::NULL, "no lend") };
    match handle.lend.settle() {
        Ok(()) => 0,
        Err(words) => fail(status::DEVICE, words),
    }
}

/// # Safety
/// `h` from `emusen_serenity_lend_new`, not freed, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_lend_free(h: *mut LendHandle) -> i32 {
    if !h.is_null() {
        drop(unsafe { Box::from_raw(h) });
    }
    0
}

/// Turns the device's probe on, its clock started as the call returns, or off.
///
/// # Safety
/// `h` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_device_probe(h: *const DeviceHandle, on: i32) -> i32 {
    let made = (|| unsafe { device(h) }?.probe(on != 0).map_err(|words| fail(status::DEVICE, words)))();
    if let Some(handle) = unsafe { h.as_ref() } {
        locked(&handle.marks).clear();
    }
    made.err().unwrap_or(0)
}

/// What the probe saw since the last taking, in order: their count, copied and removed when `out` holds them all.
///
/// # Safety
/// `h` live; `out` valid for `cap` marks, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_device_marks(h: *const DeviceHandle, out: *mut Mark, cap: usize) -> i64 {
    let Some(handle) = (unsafe { h.as_ref() }) else { return fail(status::NULL, "no handle") as i64 };
    let mut marks = locked(&handle.marks);
    if let Some(device) = locked(&handle.device).as_ref() {
        marks.extend(device.take_marks());
    }
    let count = marks.len();
    if !out.is_null() && count <= cap {
        unsafe { std::ptr::copy_nonoverlapping(marks.as_ptr(), out, count) };
        marks.clear();
    }
    count as i64
}

fn integer(value: Option<&Value>) -> Option<i32> {
    match value? {
        Value::Number(token) => token.parse().ok(),
        _ => None,
    }
}

fn string(value: Option<&Value>) -> Option<String> {
    value?.as_str().map(str::to_string)
}

fn wrap_of(value: Option<&Value>) -> Option<Wrap> {
    match number(value)? {
        0 => Some(Wrap::ClampToBorder),
        1 => Some(Wrap::ClampToEdge),
        2 => Some(Wrap::Repeat),
        3 => Some(Wrap::MirroredRepeat),
        _ => None,
    }
}

fn scale_type_of(value: Option<&Value>) -> Option<ScaleType> {
    match number(value)? {
        0 => Some(ScaleType::Source),
        1 => Some(ScaleType::Viewport),
        2 => Some(ScaleType::Absolute),
        _ => None,
    }
}

/// A preset from the document `emusen_serenity_preset_load` answers with.
fn preset_of(value: &Value) -> Option<Preset> {
    let float = |value: Option<&Value>| number(value).map(f32::from_bits);
    let passes = list(value.get("passes"))?
        .iter()
        .map(|pass| {
            let alias = pass.get("alias")?;
            Some(PassSpec {
                shader_path: string(pass.get("shader"))?,
                alias: if alias.is_null() { None } else { Some(alias.as_str()?.to_string()) },
                filter_linear: pass.get("linear")?.as_bool()?,
                wrap: wrap_of(pass.get("wrap"))?,
                scale_type_x: scale_type_of(pass.get("type_x"))?,
                scale_type_y: scale_type_of(pass.get("type_y"))?,
                scale_x: float(pass.get("scale_x"))?,
                scale_y: float(pass.get("scale_y"))?,
                float_framebuffer: pass.get("float")?.as_bool()?,
                srgb_framebuffer: pass.get("srgb")?.as_bool()?,
                mipmap_input: pass.get("mipmap")?.as_bool()?,
                frame_count_mod: integer(pass.get("mod"))?,
            })
        })
        .collect::<Option<Vec<PassSpec>>>()?;
    let textures = list(value.get("textures"))?
        .iter()
        .map(|texture| {
            Some(TextureSpec {
                name: string(texture.get("name"))?,
                path: string(texture.get("path"))?,
                linear: texture.get("linear")?.as_bool()?,
                wrap: wrap_of(texture.get("wrap"))?,
                mipmap: texture.get("mipmap")?.as_bool()?,
            })
        })
        .collect::<Option<Vec<TextureSpec>>>()?;
    let parameters = list(value.get("parameters"))?
        .iter()
        .map(|pair| {
            let pair = list(Some(pair))?;
            Some((pair.first()?.as_str()?.to_string(), float(pair.get(1))?))
        })
        .collect::<Option<Vec<(String, f32)>>>()?;
    (!passes.is_empty()).then_some(Preset { path: string(value.get("path"))?, passes, textures, parameters })
}

/// Why a chain was not built, as a document of the exception the C# throws for it, left for the caller to take.
fn refuse(why: &Failure) -> i32 {
    let said = |kind: &str, message: &str| object(vec![("fail", Value::string(kind)), ("message", Value::string(message))]);
    let (code, document) = match why {
        Failure::Source(error) => (status::IO, failure(error)),
        Failure::Compile(words) => (status::PARSE, said("compile", words)),
        Failure::Reflection(fault) => (status::PARSE, fault_json(fault)),
        Failure::Device(words) => (status::DEVICE, said("device", words)),
        Failure::Overflow => (status::BAD_ARGUMENT, said("overflow", "A pass samples a frame further back than can be counted.")),
    };
    leave(Some(&document));
    let words = document.get("message").and_then(Value::as_str).unwrap_or_default().to_string();
    fail(code, words)
}

pub struct ChainHandle {
    chain: Mutex<Chain>,
}

/// A preset built on a device: `preset` is the document `emusen_serenity_preset_load` answers with, each stage is
/// compiled through `cache` (null for none) with `now` for a row's last use, `builders` passes at a time. Its lookup
/// images are handed over afterwards, one call each. Null when it cannot be built: then a document of why waits for
/// `emusen_serenity_take`, as a reader's failure does, with "compile" for a stage that does not compile, "device" for
/// a Vulkan call that failed and "overflow" for a history too deep to count; nothing waits for a failure that is the
/// call's own.
///
/// # Safety
/// `d` live; `preset` valid for `len` bytes; `cache` live or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_chain_new(d: *const DeviceHandle, preset: *const u8, len: usize, cache: *const CacheHandle, now: i64, builders: i32) -> *mut ChainHandle {
    leave(None);
    let made = (|| {
        let device = unsafe { device(d) }?;
        let preset = preset_of(&unsafe { document(preset, len) }?).ok_or_else(|| fail(status::PARSE, "not a preset with a pass"))?;
        let shaderc = shaderc()?;
        let cache = unsafe { cache.as_ref() };
        let compile = |text: &str, stage: Stage, name: &str| match cache {
            Some(handle) => cache::compile(&handle.cache, &shaderc, text, stage, name, now),
            None => shaderc.compile(text, stage, name),
        };
        Chain::new(&device, &preset, &compile, usize::try_from(builders).unwrap_or(0)).map_err(|why| refuse(&why))
    })();
    match made {
        Ok(chain) => Box::into_raw(Box::new(ChainHandle { chain: Mutex::new(chain) })),
        Err(_) => std::ptr::null_mut(),
    }
}

/// # Safety
/// `h` from `emusen_serenity_chain_new`, not freed, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_chain_free(h: *mut ChainHandle) -> i32 {
    if !h.is_null() {
        drop(unsafe { Box::from_raw(h) });
    }
    0
}

macro_rules! chain {
    ($h:expr) => {
        match unsafe { $h.as_ref() } {
            Some(handle) => locked(&handle.chain),
            None => return fail(status::NULL, "no handle").into(),
        }
    };
}

/// Every parameter the passes declare, with the preset's value as its default where it gives one: a JSON array, as
/// `emusen_serenity_parameters_merge` answers with.
///
/// # Safety
/// `h` live; `out` valid for `cap` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_chain_parameters(h: *const ChainHandle, out: *mut u8, cap: usize) -> i64 {
    let chain = chain!(h);
    unsafe { out_json(&Value::Array(chain.parameters().iter().map(parameter_json).collect()), out, cap) }
}

/// The preset's lookup image at `index`, decoded by the host: `width` by `height` pixels of RGBA8, a row after a row.
///
/// # Safety
/// `h` live; `rgba` valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_chain_lut(h: *const ChainHandle, index: u32, width: i32, height: i32, rgba: *const u8, len: usize) -> i32 {
    let mut chain = chain!(h);
    let needed = width.wrapping_mul(height).wrapping_mul(4);
    let Some(pixels) = usize::try_from(needed).ok().and_then(|needed| unsafe { bytes(rgba, len) }.get(..needed)) else {
        return fail(status::BAD_ARGUMENT, "fewer bytes than the image's size says");
    };
    match chain.load_lut(index as usize, width, height, pixels) {
        Ok(()) => 0,
        Err(words) => fail(status::DEVICE, words),
    }
}

/// A value for each parameter, in the order `emusen_serenity_chain_parameters` gives them, read by the next render.
/// BAD_ARGUMENT unless there is one for each.
///
/// # Safety
/// `h` live; `values` valid for `count` floats.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_chain_set_parameters(h: *const ChainHandle, values: *const f32, count: usize) -> i32 {
    let mut chain = chain!(h);
    let values = if values.is_null() || count == 0 { &[][..] } else { unsafe { std::slice::from_raw_parts(values, count) } };
    if chain.set_values(values) { 0 } else { fail(status::BAD_ARGUMENT, "not one value for each parameter") }
}

/// A parameter's value by its id: 1 with it in `out`, or 0 for an id no pass declares.
///
/// # Safety
/// `h` live; `id` valid for `len` bytes; `out` valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_chain_value(h: *const ChainHandle, id: *const u8, len: usize, out: *mut f32) -> i32 {
    let chain = chain!(h);
    let id = match unsafe { required(id, len) } {
        Ok(id) => id,
        Err(status) => return status,
    };
    match (chain.value_of(id), unsafe { out.as_mut() }) {
        (Some(value), Some(out)) => {
            *out = value;
            1
        }
        (Some(_), None) => fail(status::NULL, "nowhere to put the value"),
        (None, _) => 0,
    }
}

/// A new frame, `width` by `height` pixels of RGBA8, each row shown `row_repeat` times. BAD_ARGUMENT for fewer bytes
/// than the size says; DEVICE for a Vulkan call that failed.
///
/// # Safety
/// `h` live; `rgba` valid for `len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_chain_advance(h: *const ChainHandle, rgba: *const u8, len: usize, width: i32, height: i32, row_repeat: i32) -> i32 {
    let mut chain = chain!(h);
    match chain.advance(unsafe { bytes(rgba, len) }, width, height, row_repeat) {
        Ok(()) => 0,
        Err(Unfed::Short) => fail(status::BAD_ARGUMENT, "fewer bytes than the frame's size says"),
        Err(Unfed::Device(words)) => fail(status::DEVICE, words),
    }
}

/// Runs every pass for the viewport's size and copies the last one's pixels, RGBA8, a row after a row: their length,
/// four bytes for each pixel of a viewport at least one by one. BAD_ARGUMENT, with nothing run, unless `out` holds
/// them; DEVICE when no frame has been given yet or a Vulkan call failed.
///
/// # Safety
/// `h` live; `out` valid for `cap` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_chain_render(h: *const ChainHandle, width: i32, height: i32, out: *mut u8, cap: usize) -> i64 {
    let mut chain = chain!(h);
    let needed = width.max(1) as u64 * height.max(1) as u64 * 4;
    if out.is_null() || (cap as u64) < needed {
        return fail(status::BAD_ARGUMENT, format!("the picture needs {needed} bytes")) as i64;
    }
    match chain.render(width, height) {
        Ok(pixels) => {
            unsafe { std::ptr::copy_nonoverlapping(pixels.as_ptr(), out, pixels.len()) };
            pixels.len() as i64
        }
        Err(words) => fail(status::DEVICE, words) as i64,
    }
}

/// `emusen_serenity_lent`: a picture over a readback.
#[repr(C)]
pub struct LentOut {
    pub size: u32,
    pub pixels: *const u8,
    pub bytes: usize,
    pub width: i32,
    pub height: i32,
    pub row_bytes: i32,
    pub slot: u64,
}

/// Runs every pass and lends the readback the picture is in, with no copy: `pixels` is valid, and not written, until
/// `emusen_serenity_lent_release` is given `slot`. When every readback the chain may lend is held, `slot` is zero and
/// the picture is in the chain's own readback, valid only until the chain's next call, for the caller to copy.
/// `out->size` is the caller's size of the struct.
///
/// # Safety
/// `h` live; `out` valid, with its size set.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_chain_render_lent(h: *const ChainHandle, width: i32, height: i32, out: *mut LentOut) -> i32 {
    let mut chain = chain!(h);
    let Some(out) = (unsafe { out.as_mut() }) else { return fail(status::NULL, "nowhere to put the picture") };
    if (out.size as usize) < std::mem::size_of::<LentOut>() {
        return fail(status::BAD_ARGUMENT, "the picture's struct is too small");
    }
    match chain.render_lent(width, height) {
        Ok(lent) => {
            (out.pixels, out.bytes, out.width, out.height, out.row_bytes, out.slot) = (lent.pixels, lent.bytes, lent.width, lent.height, lent.row_bytes, lent.slot);
            0
        }
        Err(words) => fail(status::DEVICE, words),
    }
}

/// Runs every pass for a view of `width` by `height`, no larger than the target, and copies the last one's picture into
/// the target's corner, without waiting: the submission
/// waits on the device for the target's `free` semaphore when `wait_free` is not zero, and signals its `ready`.
/// DEVICE when no frame has been given yet or a Vulkan call failed.
///
/// # Safety
/// `h` and `target` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_chain_present(h: *const ChainHandle, target: *const ShownHandle, width: i32, height: i32, wait_free: i32) -> i32 {
    let mut chain = chain!(h);
    let Some(target) = (unsafe { target.as_ref() }) else { return fail(status::NULL, "no target") };
    match chain.present(&target.shown, width, height, wait_free != 0) {
        Ok(()) => 0,
        Err(words) => fail(status::DEVICE, words),
    }
}

/// The newest picture taken through `lend` as the chain's new frame: 1, 0 when there is none newer, or a status.
///
/// # Safety
/// `h` and `lend` live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_chain_advance_lent(h: *const ChainHandle, lend: *const LendHandle) -> i32 {
    let mut chain = chain!(h);
    let Some(lend) = (unsafe { lend.as_ref() }) else { return fail(status::NULL, "no lend") };
    match chain.advance_lent(&lend.lend) {
        Ok(advanced) => advanced as i32,
        Err(words) => fail(status::DEVICE, words),
    }
}

/// Gives a lent readback back, from any thread, once: its chain may write it again, or, if the chain was freed
/// meanwhile, it is freed. ABSENT for a number that names no lent readback.
#[unsafe(no_mangle)]
pub extern "C" fn emusen_serenity_lent_release(slot: u64) -> i32 {
    if chain::release(slot) { 0 } else { status::ABSENT }
}

pub const CHAIN_COPIED_IMAGES: u32 = 0;
pub const CHAIN_READBACKS: u32 = 1;
pub const CHAIN_MAX_LENT: u32 = 2;

/// How many pictures were copies because every lendable readback was held, how many readbacks the chain has to lend,
/// or how many it may have.
///
/// # Safety
/// `h` live, or null for the last.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_chain_get(h: *const ChainHandle, which: u32) -> i64 {
    if which == CHAIN_MAX_LENT {
        return chain::MAX_LENT as i64;
    }
    let chain = chain!(h);
    match which {
        CHAIN_COPIED_IMAGES => chain.copied_images() as i64,
        CHAIN_READBACKS => chain.readback_count() as i64,
        _ => fail(status::BAD_ARGUMENT, "no such number") as i64,
    }
}

/// The format a pass's `#pragma format` names, as the C# resolves it: `out` is the format's value, or `fallback`
/// for a word that names none.
///
/// # Safety
/// `name` valid for `len` bytes; `out` valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_serenity_parse_format(name: *const u8, len: usize, fallback: i32, out: *mut i32) -> i32 {
    let name = match unsafe { required(name, len) } {
        Ok(name) => name,
        Err(status) => return status,
    };
    match unsafe { out.as_mut() } {
        Some(out) => {
            *out = format::parse(name).unwrap_or(fallback);
            0
        }
        None => fail(status::NULL, "nowhere to put the format"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use emusen_galaxia::json;

    #[test]
    fn a_preset_crosses_back_from_the_document_its_reader_answers_with() {
        let text = r#"{"path":"/p/a.slangp","passes":[{"shader":"/p/a.slang","alias":null,"linear":true,"wrap":2,"type_x":0,"type_y":2,"scale_x":1073741824,"scale_y":4290772992,"float":false,"srgb":true,"mipmap":false,"mod":-3},{"shader":"/p/b.slang","alias":"B","linear":false,"wrap":0,"type_x":1,"type_y":1,"scale_x":1065353216,"scale_y":1065353216,"float":true,"srgb":false,"mipmap":true,"mod":0}],"textures":[{"name":"Lut","path":"/p/lut.png","linear":true,"wrap":3,"mipmap":true}],"parameters":[["gain",1056964608]]}"#;
        let preset = preset_of(&json::parse(text).unwrap()).unwrap();
        assert_eq!(preset.path, "/p/a.slangp");
        assert_eq!((preset.passes.len(), preset.passes[0].alias.as_deref(), preset.passes[1].alias.as_deref()), (2, None, Some("B")));
        assert_eq!((preset.passes[0].wrap, preset.passes[0].scale_type_x, preset.passes[0].scale_type_y), (Wrap::Repeat, ScaleType::Source, ScaleType::Absolute));
        assert_eq!((preset.passes[0].scale_x, preset.passes[0].scale_y.to_bits(), preset.passes[0].frame_count_mod), (2.0, 0xFFC0_0000, -3));
        assert!(preset.passes[0].srgb_framebuffer && preset.passes[1].float_framebuffer && preset.passes[1].mipmap_input && !preset.passes[1].filter_linear);
        assert_eq!(preset.textures, vec![TextureSpec { name: "Lut".into(), path: "/p/lut.png".into(), linear: true, wrap: Wrap::MirroredRepeat, mipmap: true }]);
        assert_eq!(preset.parameters, vec![("gain".to_string(), 0.5)]);
        // A preset with no pass, or with a member of the wrong kind, is not one.
        assert!(preset_of(&json::parse(r#"{"path":"/p","passes":[],"textures":[],"parameters":[]}"#).unwrap()).is_none());
        assert!(preset_of(&json::parse(&text.replace("\"wrap\":2", "\"wrap\":9")).unwrap()).is_none());
        assert!(preset_of(&json::parse(&text.replace("\"linear\":true,\"wrap\":2", "\"wrap\":2")).unwrap()).is_none());
    }

    #[test]
    fn a_failure_to_build_waits_as_a_document_and_a_null_handle_is_a_status() {
        assert_eq!(refuse(&Failure::Compile("a.slang (Vertex) does not compile:\nwords".into())), status::PARSE);
        let mut out = [0u8; 256];
        let length = unsafe { crate::serenity::emusen_serenity_take(out.as_mut_ptr(), out.len()) } as usize;
        let waiting = json::parse(std::str::from_utf8(&out[..length]).unwrap()).unwrap();
        assert_eq!(waiting.get("fail").and_then(Value::as_str), Some("compile"));
        assert_eq!(waiting.get("message").and_then(Value::as_str), Some("a.slang (Vertex) does not compile:\nwords"));
        assert_eq!(refuse(&Failure::Device("vkCreateImage returned ErrorOutOfDeviceMemory".into())), status::DEVICE);
        assert_eq!(refuse(&Failure::Overflow), status::BAD_ARGUMENT);
        leave(None);
        assert_eq!(unsafe { crate::serenity::emusen_serenity_take(out.as_mut_ptr(), out.len()) }, 0);

        assert!(unsafe { emusen_serenity_chain_new(std::ptr::null(), b"{}".as_ptr(), 2, std::ptr::null(), 0, 1) }.is_null());
        assert_eq!(unsafe { emusen_serenity_chain_advance(std::ptr::null(), std::ptr::null(), 0, 1, 1, 1) }, status::NULL);
        assert_eq!(unsafe { emusen_serenity_chain_render(std::ptr::null(), 1, 1, std::ptr::null_mut(), 0) }, status::NULL as i64);
        assert_eq!(unsafe { emusen_serenity_device_name(std::ptr::null(), std::ptr::null_mut(), 0) }, status::NULL as i64);
        assert_eq!(unsafe { emusen_serenity_device_raw(std::ptr::null(), RAW_DEVICE) }, 0);
        assert_eq!(unsafe { emusen_serenity_chain_get(std::ptr::null(), CHAIN_MAX_LENT) }, 3);
        assert_eq!(emusen_serenity_lent_release(0), status::ABSENT);
        assert_eq!(std::mem::size_of::<LentOut>(), 48);
    }

    #[test]
    fn a_format_is_parsed_or_the_fallback_given() {
        let mut format = 0;
        assert_eq!(unsafe { emusen_serenity_parse_format(b"R16G16B16A16_SFLOAT".as_ptr(), 19, 37, &mut format) }, 0);
        assert_eq!(format, 97);
        assert_eq!(unsafe { emusen_serenity_parse_format(b"nothing".as_ptr(), 7, -7, &mut format) }, 0);
        assert_eq!(format, -7);
        assert_eq!(unsafe { emusen_serenity_parse_format(b"".as_ptr(), 0, 43, &mut format) }, 0);
        assert_eq!(format, 43);
    }
}
