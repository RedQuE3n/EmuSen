//! What libretro's callbacks reach: the library's declarations (options, controllers, content rules), kept for the
//! process, and the running machine's picture, sound, input and requests, reached through one pointer set for the
//! machine's life. libretro's callbacks carry no user pointer, which is why both are process-wide
//! (EmuSen_LibretroHost.md §1.1, §4).

use std::ffi::{CString, c_char, c_int, c_uint, c_void};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU16, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use emusen_libretro_sys::{self as sys, env};
use emusen_native::core::Level;

use crate::audio::Sound;
use crate::input::Input;
use crate::memory;
use crate::options::{Options, text};
use crate::video::{self, Picture};

/// What the core declares of itself, for the life of the process.
#[derive(Default)]
pub struct Declared {
    pub options: Options,
    /// The devices each port can hold, as `SET_CONTROLLER_INFO` lists them: (description, device id).
    pub controllers: Vec<Vec<(String, c_uint)>>,
    pub support_no_game: bool,
    pub performance_level: u32,
    /// Options declared after the library's first initialisation, which the schema of this process lacks.
    pub declared_late: bool,
    /// Every environment command refused, so each is logged once.
    pub refused: Vec<c_uint>,
    /// The hardware contexts the core asked for and was refused, for the words of a failed load.
    pub contexts_asked: Vec<(c_int, c_uint, c_uint)>,
    pub initialised: bool,
    /// Whether a machine has been made, after which a late declaration is noted.
    pub loaded_once: bool,
    /// Whether the core took the rumble interface, so that each port declares its two motors (1.17).
    pub rumble: bool,
}

/// Each port's two motors as the core last set them, strong then weak: atomics, since a core may set them from threads
/// of its own between frames (EmuSen_LibretroHost.md §4.4).
pub static RUMBLE: [[AtomicU16; 2]; crate::input::PORTS] = [const { [const { AtomicU16::new(0) }; 2] }; crate::input::PORTS];

unsafe extern "C" fn set_rumble_state(port: c_uint, effect: c_int, strength: u16) -> bool {
    match RUMBLE.get(port as usize).and_then(|p| p.get(effect as usize)) {
        Some(motor) => {
            motor.store(strength, Ordering::Relaxed);
            true
        }
        None => false,
    }
}

/// The private folders the core is given, as the C strings it is handed.
pub struct Folders {
    pub system: CString,
    pub save: CString,
    pub content: Option<CString>,
}

/// Video buffer and geometry state serialised by `Shared::video`.
pub struct VideoShared {
    pub retro_format: c_int,
    pub picture: Picture,
    pub av: sys::retro_system_av_info,
    /// A frame refused because it is larger than the core declared, logged once.
    pub oversized: bool,
    pub drew_in_place: bool,
}

/// One machine's state as the callbacks reach it, synchronised across callbacks and host calls.
pub struct Shared {
    pub accepted: u64,
    pub video: Mutex<VideoShared>,
    pub sound: Mutex<Sound>,
    pub input: Mutex<Input>,
    /// Events for the host, drained by the machine after each call: (kind, a, b).
    pub events: Mutex<Vec<(u32, i64, i64)>>,
    pub quirks: AtomicU64,
    pub frame_time: Mutex<Option<(unsafe extern "C" fn(i64), i64)>>,
    pub disk: Mutex<Option<sys::retro_disk_control_ext_callback>>,
    pub maps: Mutex<Option<Vec<memory::Space>>>,
    pub descriptors: Mutex<Vec<(c_uint, c_uint, c_uint, c_uint, String)>>,
    pub skip_video: AtomicBool,
    pub shutdown: AtomicBool,
    pub savestate_context: AtomicI32,
    pub game_info_ext: Mutex<Option<GameInfoExt>>,
    pub folders: Mutex<Option<Folders>>,
    /// Since stage 3: the Vulkan context the core asked for, its negotiation interface, and its device once made.
    pub hw: Mutex<Option<crate::vulkan::Ask>>,
    pub negotiation: Mutex<Option<sys::vulkan::retro_hw_render_context_negotiation_interface_vulkan>>,
    pub gpu: Mutex<Option<Arc<crate::vulkan::Gpu>>>,
}

/// `retro_game_info_ext` and the strings it points into.
pub struct GameInfoExt {
    pub strings: Vec<CString>,
    pub ext: sys::retro_game_info_ext,
}

// SAFETY: strings and backing game bytes remain pinned for the lifetime of GameInfoExt behind Mutex.
unsafe impl Send for GameInfoExt {}
unsafe impl Sync for GameInfoExt {}

fn lock_unpoison<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

impl Shared {
    pub fn new(accepted: u64) -> Shared {
        Shared {
            accepted,
            video: Mutex::new(VideoShared {
                retro_format: sys::PIXEL_FORMAT_0RGB1555,
                picture: Picture::new(0, 0, video::RGBA8888),
                av: sys::retro_system_av_info::default(),
                oversized: false,
                drew_in_place: false,
            }),
            sound: Mutex::new(Sound::new((0, 1))),
            input: Mutex::new(Input::default()),
            events: Mutex::new(Vec::new()),
            quirks: AtomicU64::new(0),
            frame_time: Mutex::new(None),
            disk: Mutex::new(None),
            maps: Mutex::new(None),
            descriptors: Mutex::new(Vec::new()),
            skip_video: AtomicBool::new(false),
            shutdown: AtomicBool::new(false),
            savestate_context: AtomicI32::new(sys::SAVESTATE_CONTEXT_NORMAL),
            game_info_ext: Mutex::new(None),
            folders: Mutex::new(None),
            hw: Mutex::new(None),
            negotiation: Mutex::new(None),
            gpu: Mutex::new(None),
        }
    }

    pub fn lock_video(&self) -> MutexGuard<'_, VideoShared> {
        lock_unpoison(&self.video)
    }

    pub fn lock_sound(&self) -> MutexGuard<'_, Sound> {
        lock_unpoison(&self.sound)
    }

    pub fn lock_input(&self) -> MutexGuard<'_, Input> {
        lock_unpoison(&self.input)
    }

    pub fn push_event(&self, kind: u32, a: i64, b: i64) {
        lock_unpoison(&self.events).push((kind, a, b));
    }

    pub fn drain_events(&self) -> Vec<(u32, i64, i64)> {
        std::mem::take(&mut *lock_unpoison(&self.events))
    }

    pub fn gpu(&self) -> Option<Arc<crate::vulkan::Gpu>> {
        lock_unpoison(&self.gpu).clone()
    }
}

static DECLARED: Mutex<Option<Declared>> = Mutex::new(None);
static SHARED: Mutex<Option<Arc<Shared>>> = Mutex::new(None);
/// Serialises the environment's changes to the machine's state when a core calls it from threads of its own.
static ENV: Mutex<()> = Mutex::new(());
/// Log records from any thread, moved into the machine's or the library's log after each call.
static LOGS: Mutex<Vec<(Level, String)>> = Mutex::new(Vec::new());
/// The library-level folders, answered while no machine is made.
static FOLDERS: Mutex<Option<(CString, CString)>> = Mutex::new(None);

pub fn declared() -> MutexGuard<'static, Option<Declared>> {
    DECLARED.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn with_declared<R>(f: impl FnOnce(&mut Declared) -> R) -> R {
    f(declared().get_or_insert_with(Declared::default))
}

pub fn set_library_folders(system: CString, save: CString) {
    *FOLDERS.lock().unwrap_or_else(|e| e.into_inner()) = Some((system, save));
}

/// The machine's state, set by `bind` for its life.
pub fn bind(shared: Option<Arc<Shared>>) {
    *lock_unpoison(&SHARED) = shared;
}

/// The machine's state, cloned atomically as an `Arc` so no `&mut Shared` aliasing can occur.
pub fn shared() -> Option<Arc<Shared>> {
    lock_unpoison(&SHARED).clone()
}

pub fn log(level: Level, text: String) {
    LOGS.lock().unwrap_or_else(|e| e.into_inner()).push((level, text));
}

/// The records waiting, into the call in flight's log.
pub fn flush_logs() {
    flush_logs_with(|_, _| {});
}

/// The records waiting, each shown to `read` and then put into the call in flight's log.
pub fn flush_logs_with(mut read: impl FnMut(Level, &str)) {
    let records = std::mem::take(&mut *LOGS.lock().unwrap_or_else(|e| e.into_inner()));
    for (level, text) in records {
        read(level, &text);
        emusen_native::core::log(level, "libretro", &text);
    }
}

fn level_of(l: c_int) -> Level {
    match l {
        sys::LOG_DEBUG => Level::Debug,
        sys::LOG_INFO => Level::Info,
        sys::LOG_WARN => Level::Warn,
        _ => Level::Error,
    }
}

/// The C shim's finished line.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_libretro_log_line(level: c_int, line: *const c_char) {
    log(level_of(level), text(line).trim_end().to_owned());
}

unsafe extern "C" {
    fn emusen_libretro_log(level: c_int, fmt: *const c_char, ...);
}

// ---- The picture, sound and input callbacks ------------------------------------------------------------------

pub unsafe extern "C" fn video_refresh(data: *const c_void, width: c_uint, height: c_uint, pitch: usize) {
    let _env = ENV.lock().unwrap_or_else(|e| e.into_inner());
    let Some(s) = shared() else { return };
    if s.skip_video.load(Ordering::Acquire) || data.is_null() {
        return;
    }
    if data == sys::HW_FRAME_BUFFER_VALID {
        hardware_frame(&s, width, height);
        return;
    }
    let mut v = s.lock_video();
    if data as *const u8 == v.picture.buffer.as_ptr() {
        if !v.picture.fits(width, height) {
            if !v.oversized {
                v.oversized = true;
                log(Level::Error, format!("a {width}x{height} frame is larger than the {}x{} the core declared; such frames are not shown", v.av.geometry.max_width, v.av.geometry.max_height));
            }
            return;
        }
        if v.retro_format == sys::PIXEL_FORMAT_XRGB8888 && (width as usize).checked_mul(4).is_some_and(|w4| w4 <= v.picture.stride) {
            let stride = v.picture.stride;
            let row_bytes = width as usize * 4;
            for y in 0..height as usize {
                let row = &mut v.picture.buffer[y * stride..y * stride + row_bytes];
                row.chunks_exact_mut(4).for_each(|p| p[3] = 0xFF);
            }
        }
        let _ = v.picture.drawn_in_place(width, height);
        v.drew_in_place = true;
        return;
    }
    let bpp = video::retro_bytes_per_pixel(v.retro_format);
    let Some(len) = (width as usize).checked_mul(bpp).and_then(|row| pitch.checked_mul((height as usize).saturating_sub(1)).and_then(|rows| rows.checked_add(row))) else {
        if !v.oversized {
            v.oversized = true;
            log(Level::Error, format!("a {width}x{height} frame overflows size calculations; such frames are not shown"));
        }
        return;
    };
    let bytes = unsafe { std::slice::from_raw_parts(data as *const u8, len) };
    let retro_format = v.retro_format;
    if !v.picture.take(retro_format, bytes, width, height, pitch) && !v.oversized {
        v.oversized = true;
        log(Level::Error, format!("a {width}x{height} frame is larger than the {}x{} the core declared; such frames are not shown", v.av.geometry.max_width, v.av.geometry.max_height));
    }
}

/// A frame on the core's device: in the ring, or read back into the picture; one larger than the ring's images is held
/// and the larger maxima raised as `MACHINE_INFO` (EmuSen_LibretroHost.md §5.4).
fn hardware_frame(s: &Shared, width: c_uint, height: c_uint) {
    let Some(g) = s.gpu() else {
        log(Level::Error, "the core handed a hardware frame with no device made for it".into());
        return;
    };
    let shown = g.frame(width, height);
    for line in g.drain_logs() {
        log(Level::Warn, line);
    }
    let (w, h, bgra, larger) = match shown {
        crate::vulkan::Shown::Bytes(w, h, bgra) => (w, h, bgra, false),
        crate::vulkan::Shown::Larger(w, h, bgra) => (w, h, bgra, true),
        // Its shape and number now, its bytes only when frame_copy asks (EmuSen_LibretroHost.md §10h).
        crate::vulkan::Shown::Ring => return s.lock_video().picture.drawn_unread(width, height),
        crate::vulkan::Shown::Nothing => return,
    };
    let raised = {
        let mut v = s.lock_video();
        let fits = w <= v.av.geometry.max_width && h <= v.av.geometry.max_height;
        if larger || !fits {
            v.av.geometry.max_width = v.av.geometry.max_width.max(w);
            v.av.geometry.max_height = v.av.geometry.max_height.max(h);
            if !fits {
                v.picture = Picture::new(v.av.geometry.max_width, v.av.geometry.max_height, v.picture.format);
            }
            true
        } else {
            false
        }
    };
    if raised {
        s.push_event(emusen_native::core::event::MACHINE_INFO, 0, 0);
    }
    g.read_with(w, h, bgra, |bytes| {
        let _ = s.lock_video().picture.take(sys::PIXEL_FORMAT_XRGB8888, bytes, w, h, w as usize * 4);
    });
}

pub unsafe extern "C" fn audio_sample(left: i16, right: i16) {
    if let Some(s) = shared() {
        s.lock_sound().push(&[left, right]);
    }
}

pub unsafe extern "C" fn audio_sample_batch(data: *const i16, frames: usize) -> usize {
    if let Some(s) = shared()
        && !data.is_null()
        && let Some(samples) = frames.checked_mul(2)
    {
        s.lock_sound().push(unsafe { std::slice::from_raw_parts(data, samples) });
    }
    frames
}

pub unsafe extern "C" fn input_poll() {}

pub unsafe extern "C" fn input_state(port: c_uint, device: c_uint, index: c_uint, id: c_uint) -> i16 {
    shared().map_or(0, |s| s.lock_input().state(port, device, index, id))
}

// ---- The performance interface -------------------------------------------------------------------------------

unsafe extern "C" fn time_usec() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_micros() as i64)
}

unsafe extern "C" fn perf_counter() -> u64 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    START.get_or_init(std::time::Instant::now).elapsed().as_nanos() as u64
}

/// libretro's `RETRO_SIMD_*` bits for what this processor has.
unsafe extern "C" fn cpu_features() -> u64 {
    #[cfg(not(target_arch = "x86_64"))]
    let bits = 0u64;
    #[cfg(target_arch = "x86_64")]
    let bits = {
        let has = [
            (is_x86_feature_detected!("sse"), 1 << 3),
            (is_x86_feature_detected!("sse2"), 1 << 4),
            (is_x86_feature_detected!("sse3"), 1 << 8),
            (is_x86_feature_detected!("ssse3"), 1 << 9),
            (is_x86_feature_detected!("sse4.1"), 1 << 10),
            (is_x86_feature_detected!("sse4.2"), 1 << 11),
            (is_x86_feature_detected!("avx"), 1 << 0),
            (is_x86_feature_detected!("avx2"), 1 << 1),
            (is_x86_feature_detected!("popcnt"), 1 << 13),
            (is_x86_feature_detected!("cmpxchg16b"), 1 << 18),
        ];
        has.iter().filter(|h| h.0).fold(0, |b, h| b | h.1)
    };
    bits
}

unsafe extern "C" fn perf_register(counter: *mut sys::retro_perf_counter) {
    if let Some(c) = unsafe { counter.as_mut() } {
        c.registered = true;
    }
}

unsafe extern "C" fn perf_start(counter: *mut sys::retro_perf_counter) {
    if let Some(c) = unsafe { counter.as_mut() } {
        c.call_cnt += 1;
        c.start = unsafe { perf_counter() };
    }
}

unsafe extern "C" fn perf_stop(counter: *mut sys::retro_perf_counter) {
    if let Some(c) = unsafe { counter.as_mut() } {
        c.total += unsafe { perf_counter() }.saturating_sub(c.start);
    }
}

unsafe extern "C" fn perf_log() {}

// ---- The environment -----------------------------------------------------------------------------------------

/// The frame's own geometry and timing, as the events the host reads.
fn av_changed(s: &Shared, av: sys::retro_system_av_info) {
    let (max_changed, base_changed, fps_changed) = {
        let mut v = s.lock_video();
        let old = v.av;
        v.av = av;
        let max_changed = (av.geometry.max_width, av.geometry.max_height) != (old.geometry.max_width, old.geometry.max_height);
        if max_changed {
            v.picture = Picture::new(av.geometry.max_width, av.geometry.max_height, v.picture.format);
        }
        (max_changed, (av.geometry.base_width, av.geometry.base_height) != (old.geometry.base_width, old.geometry.base_height), av.timing.fps != old.timing.fps)
    };
    if max_changed {
        s.push_event(emusen_native::core::event::MACHINE_INFO, 0, 0);
    }
    if base_changed {
        s.push_event(emusen_native::core::event::GEOMETRY, av.geometry.base_width as i64, av.geometry.base_height as i64);
    }
    let rate = video::ratio(av.timing.sample_rate, 1_000_000);
    let rate_changed = {
        let mut sound = s.lock_sound();
        if rate != sound.rate() {
            sound.set_rate(rate);
            true
        } else {
            false
        }
    };
    if rate_changed {
        s.push_event(emusen_native::core::event::AUDIO_RATE, crate::audio::rounded(rate) as i64, 0);
        s.push_event(emusen_native::core::event::MACHINE_INFO, 0, 0);
    }
    if fps_changed {
        s.push_event(emusen_native::core::event::MACHINE_INFO, 0, 0);
    }
}

/// A command refused, logged the first time.
fn refuse(cmd: c_uint) -> bool {
    let first = with_declared(|d| {
        let first = !d.refused.contains(&cmd);
        if first {
            d.refused.push(cmd);
        }
        first
    });
    if first {
        log(Level::Info, format!("refused environment command {} ({}): this host does not offer it", cmd & !env::EXPERIMENTAL, name_of(cmd)));
    }
    false
}

/// A command's name, for the log.
pub fn name_of(cmd: c_uint) -> &'static str {
    match cmd {
        env::SET_ROTATION => "SET_ROTATION",
        env::GET_OVERSCAN => "GET_OVERSCAN",
        env::SET_KEYBOARD_CALLBACK => "SET_KEYBOARD_CALLBACK",
        env::SET_HW_RENDER => "SET_HW_RENDER",
        env::SET_AUDIO_CALLBACK => "SET_AUDIO_CALLBACK",
        env::GET_RUMBLE_INTERFACE => "GET_RUMBLE_INTERFACE",
        env::GET_USERNAME => "GET_USERNAME",
        env::GET_VFS_INTERFACE => "GET_VFS_INTERFACE",
        env::SET_SUBSYSTEM_INFO => "SET_SUBSYSTEM_INFO",
        env::SET_AUDIO_BUFFER_STATUS_CALLBACK => "SET_AUDIO_BUFFER_STATUS_CALLBACK",
        env::SET_FASTFORWARDING_OVERRIDE => "SET_FASTFORWARDING_OVERRIDE",
        env::EXEC_MEM_ALLOC => "EXEC_MEM_ALLOC",
        env::EXEC_MEM_FREE => "EXEC_MEM_FREE",
        env::SET_NETPACKET_INTERFACE => "SET_NETPACKET_INTERFACE",
        env::GET_HW_RENDER_INTERFACE => "GET_HW_RENDER_INTERFACE",
        env::SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE => "SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE",
        env::GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT => "GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT",
        env::GET_PREFERRED_HW_RENDER => "GET_PREFERRED_HW_RENDER",
        env::SET_HW_SHARED_CONTEXT => "SET_HW_SHARED_CONTEXT",
        env::SET_CONTENT_INFO_OVERRIDE => "SET_CONTENT_INFO_OVERRIDE",
        _ => "not offered",
    }
}

unsafe fn put<T>(data: *mut c_void, value: T) -> bool {
    if data.is_null() {
        return false;
    }
    unsafe { (data as *mut T).write_unaligned(value) };
    true
}

/// The ports a machine offers: those the core describes, at least two, at most the input's.
pub fn ports() -> usize {
    with_declared(|d| d.controllers.len()).clamp(2, crate::input::PORTS)
}

static PERF: sys::retro_perf_callback = sys::retro_perf_callback {
    get_time_usec: Some(time_usec),
    get_cpu_features: Some(cpu_features),
    get_perf_counter: Some(perf_counter),
    perf_register: Some(perf_register),
    perf_start: Some(perf_start),
    perf_stop: Some(perf_stop),
    perf_log: Some(perf_log),
};

/// libretro's environment call, as EmuSen_LibretroHost.md §4.10 answers each command.
pub unsafe extern "C" fn environment(cmd: c_uint, data: *mut c_void) -> bool {
    let _env = ENV.lock().unwrap_or_else(|e| e.into_inner());
    unsafe { answer(cmd, data) }
}

unsafe fn answer(cmd: c_uint, data: *mut c_void) -> bool {
    let s = shared();
    match cmd {
        env::GET_CAN_DUPE => unsafe { put(data, true) },
        env::SET_PIXEL_FORMAT => {
            let Some(f) = (unsafe { (data as *const c_int).as_ref() }) else { return false };
            let Some(s) = s else { return false };
            if !matches!(*f, sys::PIXEL_FORMAT_0RGB1555 | sys::PIXEL_FORMAT_XRGB8888 | sys::PIXEL_FORMAT_RGB565) {
                return false;
            }
            let mut v = s.lock_video();
            v.retro_format = *f;
            let handed = video::handed_format(*f, s.accepted);
            if handed != v.picture.format {
                v.picture = Picture::new(v.av.geometry.max_width, v.av.geometry.max_height, handed);
            }
            true
        }
        env::GET_SYSTEM_DIRECTORY | env::GET_SAVE_DIRECTORY | env::GET_CORE_ASSETS_DIRECTORY => {
            let folders = FOLDERS.lock().unwrap_or_else(|e| e.into_inner());
            let s_folders = s.as_ref().and_then(|x| lock_unpoison(&x.folders).as_ref().map(|f| (f.save.as_ptr(), f.system.as_ptr())));
            let p = match (cmd, s_folders, folders.as_ref()) {
                (env::GET_SAVE_DIRECTORY, Some(f), _) => f.0,
                (env::GET_SAVE_DIRECTORY, None, Some(f)) => f.1.as_ptr(),
                (_, Some(f), _) => f.1,
                (_, None, Some(f)) => f.0.as_ptr(),
                _ => std::ptr::null(),
            };
            !p.is_null() && unsafe { put(data, p) }
        }
        env::GET_LIBRETRO_PATH => crate::library::get().is_ok_and(|l| unsafe { put(data, l.core_path_c.as_ptr()) }),
        env::SET_INPUT_DESCRIPTORS => {
            let Some(s) = s else { return false };
            let mut d = data as *const sys::retro_input_descriptor;
            let mut descriptors = lock_unpoison(&s.descriptors);
            descriptors.clear();
            while !d.is_null() && !unsafe { (*d).description }.is_null() {
                let x = unsafe { &*d };
                descriptors.push((x.port, x.device, x.index, x.id, text(x.description)));
                d = unsafe { d.add(1) };
            }
            true
        }
        env::SET_CONTROLLER_INFO => {
            let mut c = data as *const sys::retro_controller_info;
            let mut ports = Vec::new();
            while !c.is_null() && !unsafe { (*c).types }.is_null() {
                let info = unsafe { &*c };
                ports.push((0..info.num_types as usize).map(|i| unsafe { &*info.types.add(i) }).map(|t| (text(t.desc), t.id)).collect());
                c = unsafe { c.add(1) };
            }
            with_declared(|d| d.controllers = ports);
            true
        }
        env::GET_INPUT_BITMASKS => true,
        env::GET_INPUT_MAX_USERS => unsafe { put(data, ports() as c_uint) },
        env::GET_INPUT_DEVICE_CAPABILITIES => unsafe { put(data, (1u64 << sys::DEVICE_JOYPAD) | (1u64 << sys::DEVICE_ANALOG)) },
        env::GET_VARIABLE => {
            let Some(var) = (unsafe { (data as *mut sys::retro_variable).as_mut() }) else { return false };
            let value = with_declared(|d| d.options.get(&text(var.key)));
            var.value = value.unwrap_or(std::ptr::null());
            value.is_some()
        }
        env::GET_VARIABLE_UPDATE => {
            let updated = with_declared(|d| std::mem::take(&mut d.options.updated));
            unsafe { put(data, updated) }
        }
        env::SET_VARIABLES | env::SET_CORE_OPTIONS | env::SET_CORE_OPTIONS_V2 | env::SET_CORE_OPTIONS_INTL | env::SET_CORE_OPTIONS_V2_INTL => {
            if data.is_null() {
                return false;
            }
            with_declared(|d| {
                let before: Vec<String> = d.options.declared.iter().map(|o| o.key.clone()).collect();
                unsafe {
                    match cmd {
                        env::SET_VARIABLES => d.options.declare_legacy(data as *const sys::retro_variable),
                        env::SET_CORE_OPTIONS => d.options.declare_v1(data as *const sys::retro_core_option_definition),
                        env::SET_CORE_OPTIONS_INTL => d.options.declare_v1((*(data as *const sys::retro_core_options_intl)).us),
                        env::SET_CORE_OPTIONS_V2 => d.options.declare_v2(data as *const sys::retro_core_options_v2),
                        _ => d.options.declare_v2((*(data as *const sys::retro_core_options_v2_intl)).us),
                    }
                }
                if d.loaded_once && d.options.declared.iter().any(|o| !before.contains(&o.key)) {
                    d.declared_late = true;
                }
            });
            true
        }
        env::GET_CORE_OPTIONS_VERSION => unsafe { put(data, 2 as c_uint) },
        env::SET_CORE_OPTIONS_DISPLAY => {
            let Some(o) = (unsafe { (data as *const sys::retro_core_option_display).as_ref() }) else { return false };
            with_declared(|d| d.options.hidden.insert(text(o.key), !o.visible));
            true
        }
        env::SET_CORE_OPTIONS_UPDATE_DISPLAY_CALLBACK => {
            let cb = unsafe { (data as *const sys::retro_core_options_update_display_callback).as_ref() }.and_then(|c| c.callback);
            with_declared(|d| d.options.update_display = cb);
            true
        }
        env::SET_VARIABLE => match unsafe { (data as *const sys::retro_variable).as_ref() } {
            None => true,
            Some(v) => with_declared(|d| d.options.set_by_core(&text(v.key), &text(v.value))),
        },
        env::SET_GEOMETRY => {
            let (Some(s), Some(g)) = (s, unsafe { (data as *const sys::retro_game_geometry).as_ref() }) else { return false };
            let mut av = s.lock_video().av;
            av.geometry.base_width = g.base_width.min(av.geometry.max_width);
            av.geometry.base_height = g.base_height.min(av.geometry.max_height);
            av.geometry.aspect_ratio = g.aspect_ratio;
            av_changed(&s, av);
            true
        }
        env::SET_SYSTEM_AV_INFO => {
            let (Some(s), Some(av)) = (s, unsafe { (data as *const sys::retro_system_av_info).as_ref() }) else { return false };
            av_changed(&s, *av);
            true
        }
        env::GET_LOG_INTERFACE => unsafe { put(data, sys::retro_log_callback { log: Some(emusen_libretro_log) }) },
        env::SET_MESSAGE => {
            let Some(m) = (unsafe { (data as *const sys::retro_message).as_ref() }) else { return false };
            log(Level::Info, format!("message: {}", text(m.msg)));
            true
        }
        env::SET_MESSAGE_EXT => {
            let Some(m) = (unsafe { (data as *const sys::retro_message_ext).as_ref() }) else { return false };
            log(level_of(m.level), format!("message: {}", text(m.msg)));
            true
        }
        env::GET_MESSAGE_INTERFACE_VERSION => unsafe { put(data, 1 as c_uint) },
        env::GET_PERF_INTERFACE => unsafe { put(data, std::ptr::read(&PERF)) },
        env::SET_FRAME_TIME_CALLBACK => {
            let (Some(s), Some(f)) = (s, unsafe { (data as *const sys::retro_frame_time_callback).as_ref() }) else { return false };
            *lock_unpoison(&s.frame_time) = f.callback.map(|c| (c, f.reference));
            true
        }
        env::GET_AUDIO_VIDEO_ENABLE => {
            let video = s.is_none_or(|s| !s.skip_video.load(Ordering::Acquire));
            unsafe { put(data, if video { sys::AV_ENABLE_VIDEO } else { 0 } | sys::AV_ENABLE_AUDIO) }
        }
        env::GET_FASTFORWARDING => unsafe { put(data, false) },
        env::GET_THROTTLE_STATE => {
            let rate = s.map_or(0.0, |s| s.lock_video().av.timing.fps as f32);
            unsafe { put(data, sys::retro_throttle_state { mode: sys::THROTTLE_NONE, rate }) }
        }
        env::GET_TARGET_REFRESH_RATE => unsafe { put(data, s.map_or(0.0, |s| s.lock_video().av.timing.fps as f32)) },
        env::GET_LANGUAGE => unsafe { put(data, sys::LANGUAGE_ENGLISH) },
        env::SET_SUPPORT_NO_GAME => {
            let v = unsafe { (data as *const bool).as_ref() }.copied().unwrap_or(false);
            with_declared(|d| d.support_no_game = v);
            true
        }
        env::SET_MEMORY_MAPS => {
            let Some(s) = s else { return false };
            *lock_unpoison(&s.maps) = Some(unsafe { memory::from_maps(data as *const sys::retro_memory_map) });
            true
        }
        env::SET_SERIALIZATION_QUIRKS => {
            let Some(q) = (unsafe { (data as *mut u64).as_mut() }) else { return false };
            if let Some(s) = s {
                s.quirks.store(*q, Ordering::Release);
            }
            *q |= sys::QUIRK_FRONT_VARIABLE_SIZE;
            true
        }
        env::GET_SAVESTATE_CONTEXT => unsafe { put(data, s.map_or(sys::SAVESTATE_CONTEXT_NORMAL, |s| s.savestate_context.load(Ordering::Acquire))) },
        env::GET_GAME_INFO_EXT => {
            let ptr = s.as_ref().and_then(|s| lock_unpoison(&s.game_info_ext).as_ref().map(|g| &g.ext as *const sys::retro_game_info_ext));
            match ptr {
                Some(p) => unsafe { put(data, p) },
                None => false,
            }
        }
        env::GET_JIT_CAPABLE => unsafe { put(data, true) },
        env::SET_PERFORMANCE_LEVEL => {
            let v = unsafe { (data as *const c_uint).as_ref() }.copied().unwrap_or(0);
            with_declared(|d| d.performance_level = v);
            true
        }
        env::SET_SUPPORT_ACHIEVEMENTS | env::SET_PROC_ADDRESS_CALLBACK | env::SET_MINIMUM_AUDIO_LATENCY => true,
        env::SHUTDOWN => {
            if let Some(s) = s {
                s.shutdown.store(true, Ordering::Release);
            }
            true
        }
        env::GET_CURRENT_SOFTWARE_FRAMEBUFFER => {
            let (Some(s), Some(fb)) = (s, unsafe { (data as *mut sys::retro_framebuffer).as_mut() }) else { return false };
            if s.skip_video.load(Ordering::Acquire) {
                return false;
            }
            let mut v = s.lock_video();
            let same = matches!((v.retro_format, v.picture.format), (sys::PIXEL_FORMAT_XRGB8888, video::BGRA8888) | (sys::PIXEL_FORMAT_RGB565, video::RGB565));
            if !same || !v.picture.fits(fb.width, fb.height) {
                return false;
            }
            fb.data = v.picture.buffer.as_mut_ptr() as *mut c_void;
            fb.pitch = v.picture.stride;
            fb.format = v.retro_format;
            fb.access_flags = sys::MEMORY_ACCESS_WRITE | sys::MEMORY_ACCESS_READ;
            fb.memory_flags = sys::MEMORY_TYPE_CACHED;
            true
        }
        env::SET_DISK_CONTROL_INTERFACE | env::SET_DISK_CONTROL_EXT_INTERFACE => {
            let Some(s) = s else { return false };
            if data.is_null() {
                return false;
            }
            *lock_unpoison(&s.disk) = Some(if cmd == env::SET_DISK_CONTROL_EXT_INTERFACE {
                unsafe { std::ptr::read(data as *const sys::retro_disk_control_ext_callback) }
            } else {
                let base = unsafe { std::ptr::read(data as *const sys::retro_disk_control_callback) };
                sys::retro_disk_control_ext_callback { base, set_initial_image: None, get_image_path: None, get_image_label: None }
            });
            true
        }
        env::GET_DISK_CONTROL_INTERFACE_VERSION => unsafe { put(data, 1 as c_uint) },
        env::GET_RUMBLE_INTERFACE => {
            if !unsafe { put(data, sys::retro_rumble_interface { set_rumble_state: Some(set_rumble_state) }) } {
                return false;
            }
            if with_declared(|d| !std::mem::replace(&mut d.rumble, true))
                && let Some(s) = s
            {
                s.push_event(emusen_native::core::event::MACHINE_INFO, 0, 0);
            }
            true
        }
        env::SET_HW_RENDER => {
            let Some(h) = (unsafe { (data as *const sys::retro_hw_render_callback).as_ref() }) else { return false };
            if h.context_type != sys::HW_CONTEXT_VULKAN || s.is_none() {
                with_declared(|d| d.contexts_asked.push((h.context_type, h.version_major, h.version_minor)));
                log(Level::Warn, format!("the core asked for hardware context {}, and this host gives Vulkan only", h.context_type));
                return false;
            }
            *lock_unpoison(&s.unwrap().hw) = Some(crate::vulkan::Ask { reset: h.context_reset, destroy: h.context_destroy, version: (h.version_major, h.version_minor) });
            true
        }
        env::SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE => {
            let Some(n) = (unsafe { (data as *const sys::vulkan::retro_hw_render_context_negotiation_interface_vulkan).as_ref() }) else { return false };
            let Some(s) = s else { return false };
            if n.interface_type != sys::vulkan::INTERFACE_VULKAN {
                return false;
            }
            // The v2 members exist only from version 2; an older interface is read up to destroy_device.
            let mut copy = sys::vulkan::retro_hw_render_context_negotiation_interface_vulkan { create_instance: None, create_device2: None, ..*n };
            if n.interface_version >= 2 {
                copy = *n;
            }
            *lock_unpoison(&s.negotiation) = Some(copy);
            true
        }
        env::GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT => {
            let Some(q) = (unsafe { (data as *mut [c_uint; 2]).as_mut() }) else { return false };
            if q[0] != sys::vulkan::INTERFACE_VULKAN {
                q[1] = 0;
                return false;
            }
            q[1] = sys::vulkan::HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_VULKAN_VERSION;
            true
        }
        env::GET_PREFERRED_HW_RENDER => unsafe { put(data, sys::HW_CONTEXT_VULKAN as c_uint) },
        env::GET_HW_RENDER_INTERFACE => match s.and_then(|s| s.gpu()) {
            Some(g) => unsafe { put(data, g.interface() as *const c_void) },
            None => false,
        },
        _ => refuse(cmd),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_binding_keeps_arc_alive_and_video_refresh_rejects_oversized_in_place() {
        let s = Arc::new(Shared::new(0b111));
        {
            let mut v = s.lock_video();
            v.retro_format = sys::PIXEL_FORMAT_XRGB8888;
            v.av.geometry.max_width = 4;
            v.av.geometry.max_height = 2;
            v.picture = Picture::new(4, 2, video::BGRA8888);
        }
        bind(Some(Arc::clone(&s)));
        let ptr = s.lock_video().picture.buffer.as_ptr() as *const c_void;
        unsafe { video_refresh(ptr, 8, 2, 16) };
        let v = s.lock_video();
        assert_eq!((v.picture.width, v.picture.height, v.picture.serial), (0, 0, 0));
        assert!(v.oversized);
        drop(v);
        bind(None);
    }
}
