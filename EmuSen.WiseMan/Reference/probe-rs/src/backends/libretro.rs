// The libretro backend: any core that speaks the ABI, driven through dlopen.
// See EmuSen_Debugging_Tools_Reference_v5.md §3.50.
#![allow(non_upper_case_globals, non_camel_case_types)]

use std::ffi::{CStr, CString, c_char, c_uint, c_void};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicPtr, Ordering};

use libloading::os::unix::Library;

use crate::backend::{
    InputSchedule, MemorySpace, ProbeBackend, ProbeButton, ProbeOptions, ScreenFormat, ScreenView,
};

// The generated module is the whole ABI, not just the part this backend uses;
// binding all of it is what keeps it honest against the real header.
pub mod sys {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/libretro_sys.rs"));
}

// libretro's callbacks are free functions with no user pointer, so the core is
// process-global by design. One core per probe run, which is all this tool ever
// wants. The C++ pointed a global at the backend object itself; this points at
// only the state the callbacks touch, so the rest of the backend stays behind
// an ordinary &mut and the aliasing is confined to one struct.
static SHARED: AtomicPtr<Shared> = AtomicPtr::new(null_mut());

struct Shared {
    pixel_format: sys::retro_pixel_format,
    home_folder: CString,
    width: u32,
    height: u32,
    format: ScreenFormat,
    screen: Vec<u8>,
    live: u32,
    schedule: InputSchedule,
    frame: u32,
    // Interleaved stereo, accumulated across the whole run - see Mercury_Gameplan.md §3.1.
    audio: Vec<i16>,
    capture_audio: bool,
}

// Safety: the pointer is installed by load() before the core can call anything
// back, cleared by shutdown(), and every callback runs on the thread that is
// inside run() or load_game() at the time. No emulation thread exists here -
// that is the whole reason this backend needs no source patch.
fn shared() -> Option<&'static mut Shared> {
    unsafe { SHARED.load(Ordering::Relaxed).as_mut() }
}

fn joypad_id(button: ProbeButton) -> c_uint {
    match button {
        ProbeButton::A => sys::RETRO_DEVICE_ID_JOYPAD_A,
        ProbeButton::B => sys::RETRO_DEVICE_ID_JOYPAD_B,
        ProbeButton::X => sys::RETRO_DEVICE_ID_JOYPAD_X,
        ProbeButton::Y => sys::RETRO_DEVICE_ID_JOYPAD_Y,
        ProbeButton::L => sys::RETRO_DEVICE_ID_JOYPAD_L,
        ProbeButton::R => sys::RETRO_DEVICE_ID_JOYPAD_R,
        ProbeButton::Up => sys::RETRO_DEVICE_ID_JOYPAD_UP,
        ProbeButton::Down => sys::RETRO_DEVICE_ID_JOYPAD_DOWN,
        ProbeButton::Left => sys::RETRO_DEVICE_ID_JOYPAD_LEFT,
        ProbeButton::Right => sys::RETRO_DEVICE_ID_JOYPAD_RIGHT,
        ProbeButton::Start => sys::RETRO_DEVICE_ID_JOYPAD_START,
        ProbeButton::Select => sys::RETRO_DEVICE_ID_JOYPAD_SELECT,
    }
}

// Which machine a core is emulating is not something libretro reports, and the
// space names have to line up with the native backends' or a cross-emulator
// diff compares "wram" against "ram". The ROM extension is the one signal
// available before the core is even loaded.
fn system_from_rom(rom_path: &str) -> &'static str {
    let extension = rom_path.rsplit_once('.').map_or(String::new(), |(_, e)| e.to_lowercase());
    match extension.as_str() {
        "nes" | "fds" | "unf" => "nes",
        "smc" | "sfc" | "swc" | "fig" => "snes",
        "gb" | "gbc" => "gb",
        "gba" => "gba",
        "sms" | "gg" => "sms",
        "md" | "gen" | "smd" => "megadrive",
        _ => "unknown",
    }
}

unsafe extern "C" fn environment(cmd: c_uint, data: *mut c_void) -> bool {
    let Some(state) = shared() else { return false };

    match cmd {
        sys::RETRO_ENVIRONMENT_SET_PIXEL_FORMAT => {
            state.pixel_format = unsafe { *data.cast::<sys::retro_pixel_format>() };
            true
        }

        sys::RETRO_ENVIRONMENT_GET_SYSTEM_DIRECTORY
        | sys::RETRO_ENVIRONMENT_GET_SAVE_DIRECTORY
        | sys::RETRO_ENVIRONMENT_GET_CORE_ASSETS_DIRECTORY => {
            unsafe { *data.cast::<*const c_char>() = state.home_folder.as_ptr() };
            true
        }

        sys::RETRO_ENVIRONMENT_GET_CAN_DUPE => {
            unsafe { *data.cast::<bool>() = true };
            true
        }

        // Every core option is left at its default, deliberately: a probe that
        // silently ran with different settings than the one it is compared
        // against is the same class of error as a muted reference.
        sys::RETRO_ENVIRONMENT_GET_VARIABLE => {
            unsafe { (*data.cast::<sys::retro_variable>()).value = std::ptr::null() };
            false
        }

        sys::RETRO_ENVIRONMENT_GET_VARIABLE_UPDATE => {
            unsafe { *data.cast::<bool>() = false };
            true
        }

        sys::RETRO_ENVIRONMENT_SET_VARIABLES
        | sys::RETRO_ENVIRONMENT_SET_CORE_OPTIONS
        | sys::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_INTL
        | sys::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2
        | sys::RETRO_ENVIRONMENT_SET_CORE_OPTIONS_V2_INTL
        | sys::RETRO_ENVIRONMENT_SET_PERFORMANCE_LEVEL
        | sys::RETRO_ENVIRONMENT_SET_INPUT_DESCRIPTORS
        | sys::RETRO_ENVIRONMENT_SET_CONTROLLER_INFO
        | sys::RETRO_ENVIRONMENT_SET_SUPPORT_NO_GAME
        | sys::RETRO_ENVIRONMENT_SET_MEMORY_MAPS
        | sys::RETRO_ENVIRONMENT_SET_GEOMETRY => true,

        sys::RETRO_ENVIRONMENT_GET_LOG_INTERFACE => {
            unsafe { (*data.cast::<sys::retro_log_callback>()).log = Some(log_shim()) };
            true
        }

        _ => false,
    }
}

// Below WARN is dropped, exactly as the C++ did. Declining the interface instead
// is not equivalent and was measured: gambatte then falls back to its own
// logger, which writes five INFO lines to *stdout* and lands them in the middle
// of a dump report. See §3.50.
//
// Arguments are not interpolated. retro_log_printf_t is a printf-style variadic
// and defining one is nightly-only in Rust, so this prints the format string as
// vprintf would have with no arguments to substitute - identical for the messages
// that carry none, and visibly literal ("%u") for the rest rather than wrong.
unsafe extern "C" fn log_callback(level: sys::retro_log_level, format: *const c_char) {
    if level < sys::retro_log_level_RETRO_LOG_WARN || format.is_null() {
        return;
    }
    print!("{}", unsafe { CStr::from_ptr(format) }.to_string_lossy());
}

// Installed into a variadic slot from a non-variadic definition. The callee
// reads only the two leading arguments and never the vector-register count, so
// every ABI this probe builds for passes the extra ones harmlessly.
fn log_shim() -> unsafe extern "C" fn(sys::retro_log_level, *const c_char, ...) {
    unsafe { std::mem::transmute(log_callback as unsafe extern "C" fn(sys::retro_log_level, *const c_char)) }
}

unsafe extern "C" fn video_refresh(data: *const c_void, width: c_uint, height: c_uint, pitch: usize) {
    // A null frame is a deliberate duplicate; keeping the previous buffer is
    // what every frontend does and what makes a dump at that frame meaningful.
    if data.is_null() {
        return;
    }
    let Some(state) = shared() else { return };

    state.width = width;
    state.height = height;

    let bytes_per_pixel = if state.pixel_format == sys::retro_pixel_format_RETRO_PIXEL_FORMAT_XRGB8888 { 4 } else { 2 };
    state.format = match state.pixel_format {
        sys::retro_pixel_format_RETRO_PIXEL_FORMAT_XRGB8888 => ScreenFormat::Xrgb8888,
        sys::retro_pixel_format_RETRO_PIXEL_FORMAT_RGB565 => ScreenFormat::Rgb565,
        _ => ScreenFormat::Bgr555,
    };

    // Repacked to tight rows. The core's pitch is padding, not data, and a
    // consumer diffing two emulators' screens should not have to model it.
    let row_bytes = width as usize * bytes_per_pixel;
    state.screen.resize(row_bytes * height as usize, 0);
    for y in 0..height as usize {
        let source = unsafe { data.cast::<u8>().add(y * pitch) };
        let destination = state.screen[y * row_bytes..][..row_bytes].as_mut_ptr();
        unsafe { std::ptr::copy_nonoverlapping(source, destination, row_bytes) };
    }
}

unsafe extern "C" fn input_poll() {}

unsafe extern "C" fn input_state(port: c_uint, device: c_uint, _index: c_uint, id: c_uint) -> i16 {
    if port != 0 || device != sys::RETRO_DEVICE_JOYPAD {
        return 0;
    }
    let Some(state) = shared() else { return 0 };

    for button in ProbeButton::ALL {
        if joypad_id(button) != id {
            continue;
        }
        let live = state.live & (1u32 << button.index()) != 0;
        return i16::from(live || state.schedule.held_at(state.frame, button));
    }
    0
}

// A core pushes audio here; discarding it is what made --wav look impossible.
unsafe extern "C" fn audio_sample(left: i16, right: i16) {
    let Some(state) = shared() else { return };
    if !state.capture_audio {
        return;
    }

    state.audio.push(left);
    state.audio.push(right);
}

unsafe extern "C" fn audio_batch(data: *const i16, frames: usize) -> usize {
    let Some(state) = shared() else { return frames };
    if !state.capture_audio || data.is_null() {
        return frames;
    }

    // libretro counts a "frame" as one sample per channel, always two channels here.
    let batch = unsafe { std::slice::from_raw_parts(data, frames * 2) };
    state.audio.extend_from_slice(batch);
    frames
}

struct CoreApi {
    init: unsafe extern "C" fn(),
    deinit: unsafe extern "C" fn(),
    run: unsafe extern "C" fn(),
    unload_game: unsafe extern "C" fn(),
    load_game: unsafe extern "C" fn(*const sys::retro_game_info) -> bool,
    get_system_info: unsafe extern "C" fn(*mut sys::retro_system_info),
    get_system_av_info: unsafe extern "C" fn(*mut sys::retro_system_av_info),
    set_controller_port_device: unsafe extern "C" fn(c_uint, c_uint),
    get_memory_data: unsafe extern "C" fn(c_uint) -> *mut c_void,
    get_memory_size: unsafe extern "C" fn(c_uint) -> usize,
}

struct SpaceRef {
    name: String,
    data: *const u8,
    size: usize,
}

pub struct LibretroBackend {
    core_path: String,
    name: String,
    system: &'static str,
    library: Option<Library>,
    api: Option<CoreApi>,
    shared: Box<Shared>,
    rom_data: Vec<u8>,
    spaces: Vec<SpaceRef>,
    loaded: bool,
    wav_path: String,
    sample_rate: u32,
}

impl LibretroBackend {
    // Runs from shutdown(), which is before main's audio::finish re-encodes it.
    fn write_capture(&mut self) {
        if self.wav_path.is_empty() || !self.shared.capture_audio {
            return;
        }

        let samples = std::mem::take(&mut self.shared.audio);
        if samples.is_empty() {
            println!("[WARN] core produced no audio; not writing {}", self.wav_path);
            return;
        }

        match crate::audio::write_wav(std::path::Path::new(&self.wav_path), &samples, 2, self.sample_rate) {
            Ok(()) => println!("[INFO] captured {} stereo frames", samples.len() / 2),
            Err(reason) => println!("[ERROR] {reason}"),
        }
    }

    pub fn new(core_path: &str) -> LibretroBackend {
        LibretroBackend {
            core_path: core_path.to_string(),
            // load() replaces this with the core's own library_name, but the
            // probe needs a name before then to site the profile directory.
            name: core_stem(core_path),
            system: "unknown",
            library: None,
            api: None,
            shared: Box::new(Shared {
                pixel_format: sys::retro_pixel_format_RETRO_PIXEL_FORMAT_0RGB1555,
                home_folder: CString::default(),
                width: 0,
                height: 0,
                format: ScreenFormat::None,
                screen: Vec::new(),
                live: 0,
                schedule: InputSchedule::default(),
                frame: 0,
                audio: Vec::new(),
                capture_audio: false,
            }),
            rom_data: Vec::new(),
            spaces: Vec::new(),
            loaded: false,
            wav_path: String::new(),
            sample_rate: 0,
        }
    }

    fn add_space(&mut self, name: &str, id: c_uint) {
        let Some(api) = self.api.as_ref() else { return };
        let data = unsafe { (api.get_memory_data)(id) };
        let size = unsafe { (api.get_memory_size)(id) };
        if !data.is_null() && size > 0 {
            self.spaces.push(SpaceRef { name: name.to_string(), data: data.cast(), size });
        }
    }

    fn cache_spaces(&mut self) {
        self.spaces.clear();
        // SYSTEM_RAM is the console's main work RAM, which the native backends
        // call "ram" on an NES and "wram" on a SNES or Game Boy.
        let anchor = self.anchor_space().to_string();
        self.add_space(&anchor, sys::RETRO_MEMORY_SYSTEM_RAM);
        self.add_space("sram", sys::RETRO_MEMORY_SAVE_RAM);
        self.add_space("vram", sys::RETRO_MEMORY_VIDEO_RAM);
        self.add_space("rtc", sys::RETRO_MEMORY_RTC);
    }
}

// "…/gambatte_libretro.so" -> "gambatte": the pre-load name, taken from the path
// because retro_get_system_info needs a library this backend has not opened yet.
fn core_stem(core_path: &str) -> String {
    let file = core_path.rsplit('/').next().unwrap_or(core_path);
    let stem = file.strip_suffix(".so").unwrap_or(file);
    let stem = stem.strip_suffix("_libretro").unwrap_or(stem);
    if stem.is_empty() { "libretro".to_string() } else { stem.to_ascii_lowercase() }
}

// Resolves in the order the C++ did and stops at the first missing symbol, so a
// core that is not a core reports one line rather than fifteen.
fn resolve<T: Copy>(library: &Library, symbol: &str) -> Option<T> {
    let mut name = symbol.as_bytes().to_vec();
    name.push(0);
    match unsafe { library.get::<T>(&*name) } {
        Ok(found) => Some(*found),
        Err(_) => {
            println!("[ERROR] core has no {symbol}");
            None
        }
    }
}

impl ProbeBackend for LibretroBackend {
    fn name(&self) -> &str {
        &self.name
    }

    fn system(&self) -> &str {
        self.system
    }

    fn anchor_space(&self) -> &str {
        if self.system == "snes" || self.system == "gb" { "wram" } else { "ram" }
    }

    fn load(&mut self, rom_path: &str, options: &ProbeOptions) -> bool {
        self.system = system_from_rom(rom_path);
        self.shared.home_folder = CString::new(options.home_folder.as_str()).unwrap_or_default();
        SHARED.store(&raw mut *self.shared, Ordering::Relaxed);

        // RTLD_NOW | RTLD_LOCAL, matching the C++: a core with an unresolved
        // symbol should fail here rather than mid-frame.
        let library = match unsafe { Library::open(Some(&self.core_path), libc_rtld_now_local()) } {
            Ok(library) => library,
            Err(error) => {
                println!("[ERROR] dlopen {}: {error}", self.core_path);
                return false;
            }
        };

        let api_version: unsafe extern "C" fn() -> c_uint = match resolve(&library, "retro_api_version") {
            Some(f) => f,
            None => return false,
        };

        let api = CoreApi {
            init: match resolve(&library, "retro_init") { Some(f) => f, None => return false },
            deinit: match resolve(&library, "retro_deinit") { Some(f) => f, None => return false },
            run: match resolve(&library, "retro_run") { Some(f) => f, None => return false },
            load_game: match resolve(&library, "retro_load_game") { Some(f) => f, None => return false },
            unload_game: match resolve(&library, "retro_unload_game") { Some(f) => f, None => return false },
            get_system_info: match resolve(&library, "retro_get_system_info") { Some(f) => f, None => return false },
            get_system_av_info: match resolve(&library, "retro_get_system_av_info") { Some(f) => f, None => return false },
            set_controller_port_device: match resolve(&library, "retro_set_controller_port_device") { Some(f) => f, None => return false },
            get_memory_data: match resolve(&library, "retro_get_memory_data") { Some(f) => f, None => return false },
            get_memory_size: match resolve(&library, "retro_get_memory_size") { Some(f) => f, None => return false },
        };

        type SetEnv = unsafe extern "C" fn(sys::retro_environment_t);
        type SetVideo = unsafe extern "C" fn(sys::retro_video_refresh_t);
        type SetPoll = unsafe extern "C" fn(sys::retro_input_poll_t);
        type SetState = unsafe extern "C" fn(sys::retro_input_state_t);
        type SetSample = unsafe extern "C" fn(sys::retro_audio_sample_t);
        type SetBatch = unsafe extern "C" fn(sys::retro_audio_sample_batch_t);

        let set_environment: SetEnv = match resolve(&library, "retro_set_environment") { Some(f) => f, None => return false };
        let set_video_refresh: SetVideo = match resolve(&library, "retro_set_video_refresh") { Some(f) => f, None => return false };
        let set_input_poll: SetPoll = match resolve(&library, "retro_set_input_poll") { Some(f) => f, None => return false };
        let set_input_state: SetState = match resolve(&library, "retro_set_input_state") { Some(f) => f, None => return false };
        let set_audio_sample: SetSample = match resolve(&library, "retro_set_audio_sample") { Some(f) => f, None => return false };
        let set_audio_batch: SetBatch = match resolve(&library, "retro_set_audio_sample_batch") { Some(f) => f, None => return false };

        let version = unsafe { api_version() };
        if version != sys::RETRO_API_VERSION {
            println!("[ERROR] core speaks libretro {version}, this probe speaks {}", sys::RETRO_API_VERSION);
            return false;
        }

        let mut info = sys::retro_system_info::default();
        unsafe { (api.get_system_info)(&raw mut info) };
        self.name = cstr_or(info.library_name, "libretro");
        // The dump prefix becomes a filename, so it cannot carry spaces.
        self.name = self.name.chars().map(|c| if c == ' ' { '-' } else { c.to_ascii_lowercase() }).collect();
        println!("[INFO] core {} {}", self.name, cstr_or(info.library_version, ""));

        // Before retro_init, which is the one ordering rule the ABI actually has.
        unsafe {
            set_environment(Some(environment));
            (api.init)();
            set_video_refresh(Some(video_refresh));
            set_input_poll(Some(input_poll));
            set_input_state(Some(input_state));
            set_audio_sample(Some(audio_sample));
            set_audio_batch(Some(audio_batch));
        }

        let rom_cstring = CString::new(rom_path).unwrap_or_default();
        let mut game = sys::retro_game_info { path: rom_cstring.as_ptr(), ..Default::default() };
        if !info.need_fullpath {
            self.rom_data = match std::fs::read(rom_path) {
                Ok(bytes) => bytes,
                Err(_) => {
                    println!("[ERROR] cannot read {rom_path}");
                    return false;
                }
            };
            game.data = self.rom_data.as_ptr().cast();
            game.size = self.rom_data.len();
        }

        self.library = Some(library);
        self.api = Some(api);

        let api = self.api.as_ref().unwrap();
        if !unsafe { (api.load_game)(&raw const game) } {
            println!("[ERROR] core refused {rom_path}");
            return false;
        }
        self.loaded = true;

        unsafe { (api.set_controller_port_device)(0, sys::RETRO_DEVICE_JOYPAD) };

        // The core reports its own rate; assuming 44100 would resample nothing and
        // mislabel everything - see EmuSen_Debugging_Tools_Reference_v5.md §3.51.
        let get_av = api.get_system_av_info;
        self.cache_spaces();

        let mut av: sys::retro_system_av_info = unsafe { std::mem::zeroed() };
        unsafe { get_av(&raw mut av) };
        self.sample_rate = av.timing.sample_rate.round().max(0.0) as u32;

        if !options.wav_path.is_empty() {
            if self.sample_rate == 0 {
                println!("[WARN] core reported no sample rate; not recording audio");
            } else {
                self.wav_path = options.wav_path.clone();
                self.shared.capture_audio = true;
                println!("[INFO] recording audio at {} Hz", self.sample_rate);
            }
        }
        if options.ram_state != "zeros" {
            println!("[WARN] --ramstate is not settable through libretro");
        }
        true
    }

    fn shutdown(&mut self) {
        self.write_capture();

        if let Some(api) = self.api.as_ref() {
            if self.loaded {
                unsafe { (api.unload_game)() };
                self.loaded = false;
            }
            unsafe { (api.deinit)() };
        }
        self.api = None;
        self.spaces.clear();
        // Dropping the Library is the dlclose, and it has to happen after the
        // last call through a pointer that lives inside it.
        self.library = None;
        SHARED.store(null_mut(), Ordering::Relaxed);
    }

    // Exact by construction: retro_run() is one frame, called from this thread.
    // Nothing here can overshoot, which is the whole problem the Mesen backend
    // needed a source patch to solve.
    fn run_until(&mut self, frame: u32) {
        let Some(api) = self.api.as_ref() else { return };
        while self.shared.frame < frame {
            unsafe { (api.run)() };
            self.shared.frame += 1;
        }
    }

    fn frame_count(&self) -> u32 {
        self.shared.frame
    }

    fn spaces(&self) -> Vec<MemorySpace<'_>> {
        self.spaces
            .iter()
            .map(|space| MemorySpace {
                name: space.name.clone(),
                // Safety: the core owns these and keeps them alive for as long
                // as the game is loaded; the borrow of self expresses exactly
                // that, which the C++ could not.
                data: unsafe { std::slice::from_raw_parts(space.data, space.size) },
            })
            .collect()
    }

    fn set_schedule(&mut self, schedule: InputSchedule) {
        self.shared.schedule = schedule;
    }

    fn set_button(&mut self, button: ProbeButton, held: bool) {
        let bit = 1u32 << button.index();
        if held {
            self.shared.live |= bit;
        } else {
            self.shared.live &= !bit;
        }
    }

    fn screen(&self) -> Option<ScreenView<'_>> {
        if self.shared.screen.is_empty() {
            return None;
        }
        Some(ScreenView {
            data: &self.shared.screen,
            width: self.shared.width,
            height: self.shared.height,
            format: self.shared.format,
        })
    }

    fn state_line(&self) -> String {
        format!(
            "\n  {} {}x{} {}\n",
            self.name,
            self.shared.width,
            self.shared.height,
            self.shared.format.name()
        )
    }
}

fn cstr_or(text: *const c_char, fallback: &str) -> String {
    if text.is_null() {
        return fallback.to_string();
    }
    unsafe { CStr::from_ptr(text) }.to_string_lossy().into_owned()
}

// libloading does not re-export the dlopen flags, and the defaults are
// RTLD_LAZY; the C++ asked for RTLD_NOW so a broken core fails at open.
fn libc_rtld_now_local() -> i32 {
    const RTLD_NOW: i32 = 0x2;
    const RTLD_LOCAL: i32 = 0;
    RTLD_NOW | RTLD_LOCAL
}

#[cfg(test)]
mod tests {
    use super::*;

    // The pre-load name has to distinguish two cores, or they share one profile
    // directory and one save file - see §3.53.
    #[test]
    fn the_core_stem_names_the_core_not_the_api() {
        assert_eq!(core_stem("/usr/lib64/libretro/gambatte_libretro.so"), "gambatte");
        assert_eq!(core_stem("nestopia_libretro.so"), "nestopia");
        assert_eq!(core_stem("/x/bsnes_mercury_performance_libretro.so"), "bsnes_mercury_performance");
        assert_eq!(core_stem("/x/Gambatte.so"), "gambatte");
        assert_eq!(core_stem(""), "libretro");
    }

    #[test]
    fn system_comes_from_the_rom_extension() {
        assert_eq!(system_from_rom("/roms/smb3.nes"), "nes");
        assert_eq!(system_from_rom("/roms/Yoshi.SFC"), "snes");
        assert_eq!(system_from_rom("/roms/tetris.gb"), "gb");
        assert_eq!(system_from_rom("/roms/x.gba"), "gba");
        assert_eq!(system_from_rom("/roms/sonic.md"), "megadrive");
        assert_eq!(system_from_rom("/roms/noextension"), "unknown");
        assert_eq!(system_from_rom("/roms/x.zip"), "unknown");
    }

    // "ram" on an NES, "wram" on the two machines whose native backends use
    // that name - a cross-emulator diff compares columns by name.
    #[test]
    fn anchor_space_follows_the_system() {
        let mut backend = LibretroBackend::new("/dev/null");
        backend.system = "nes";
        assert_eq!(backend.anchor_space(), "ram");
        backend.system = "snes";
        assert_eq!(backend.anchor_space(), "wram");
        backend.system = "gb";
        assert_eq!(backend.anchor_space(), "wram");
        backend.system = "megadrive";
        assert_eq!(backend.anchor_space(), "ram");
    }

    #[test]
    fn joypad_ids_are_distinct_and_cover_every_button() {
        let ids: Vec<c_uint> = ProbeButton::ALL.into_iter().map(joypad_id).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), ids.len());
    }

    #[test]
    fn live_buttons_are_independent_bits() {
        let mut backend = LibretroBackend::new("/dev/null");
        backend.set_button(ProbeButton::A, true);
        backend.set_button(ProbeButton::Start, true);
        assert_eq!(backend.shared.live, 1 << 0 | 1 << 10);
        backend.set_button(ProbeButton::A, false);
        assert_eq!(backend.shared.live, 1 << 10);
    }
}
