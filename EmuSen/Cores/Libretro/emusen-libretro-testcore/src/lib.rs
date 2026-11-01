//! A libretro core of the project's own: deterministic, an option for each behaviour a libretro host must meet. See
//! EmuSen_LibretroHost.md §9.
#![allow(clippy::missing_safety_doc)]

mod fault;
pub mod layout;
mod options;
pub mod probe;
mod vulkan;

/// How many options the core declares.
pub const OPTION_COUNT: usize = options::OPTIONS.len();

use std::ffi::{CStr, CString, c_char, c_int, c_uint, c_void};
use std::sync::Mutex;

use emusen_libretro_sys::{self as sys, env};
use layout::*;
use options::Options;

/// The callbacks the host gave, copied out of the lock before each call.
#[derive(Clone, Copy)]
struct Callbacks {
    environment: sys::retro_environment_t,
    video: sys::retro_video_refresh_t,
    sample: sys::retro_audio_sample_t,
    batch: sys::retro_audio_sample_batch_t,
    poll: sys::retro_input_poll_t,
    input: sys::retro_input_state_t,
}

static CALLBACKS: Mutex<Callbacks> = Mutex::new(Callbacks { environment: None, video: None, sample: None, batch: None, poll: None, input: None });
static CORE: Mutex<Option<Core>> = Mutex::new(None);
static LOG: Mutex<sys::retro_log_printf_t> = Mutex::new(None);
/// The frame times reported since the last frame, added into the state's own sum at each frame.
static FRAME_TIME_SUM: Mutex<i64> = Mutex::new(0);
static RUMBLE: Mutex<sys::retro_set_rumble_state_t> = Mutex::new(None);

/// The levels `tc_rumble` drives at a frame, (port, strong, weak); the test reads them back as `OUTPUT` events.
pub fn rumble_pattern(frame: u64) -> [(u32, u16, u16); 2] {
    let strong = (frame.wrapping_mul(4099) & 0xFFFF) as u16;
    [(0, strong, 0xFFFF - strong), (1, if frame >= 10 { 0x1234 } else { 0 }, 0)]
}

fn callbacks() -> Callbacks {
    *CALLBACKS.lock().unwrap()
}

/// The environment call, or false where the host gave none.
pub(crate) fn environment(cmd: c_uint, data: *mut c_void) -> bool {
    match callbacks().environment {
        Some(f) => unsafe { f(cmd, data) },
        None => false,
    }
}

pub(crate) fn log(level: c_int, text: &str) {
    if let Some(f) = *LOG.lock().unwrap() {
        let text = CString::new(text).unwrap_or_default();
        unsafe { f(level, c"%s\n".as_ptr(), text.as_ptr()) };
    }
}

/// The machine: memory, the frame counter and what the options chose at load.
struct Core {
    ram: Vec<u8>,
    vram: Vec<u8>,
    sram: Vec<u8>,
    rtc: Vec<u8>,
    frame: u64,
    samples_made: u64,
    options: Options,
    format: c_int,
    picture: Vec<u8>,
    /// The frame the picture shows, and the last one drawn, which `tc_lag` shows next; neither is in the state.
    drawing: u64,
    shown: u64,
    width: u32,
    height: u32,
    fps_num: u64,
    rate: (u64, u64),
    files_root: Option<(String, String)>,
    disc: DiscState,
    shutdown_sent: bool,
    failed_once: bool,
}

#[derive(Clone, Copy, Default)]
struct DiscState {
    index: u8,
    ejected: bool,
    images: u8,
}

impl Core {
    fn new(options: Options) -> Core {
        let mut ram = vec![0u8; RAM_SIZE];
        ram[..4].copy_from_slice(b"TCOR");
        ram[LIVES] = 3;
        ram[PROBE..PROBE + PROBE_LEN].fill(PROBE_NOT_RUN);
        let rate = options.rate();
        Core {
            ram,
            vram: vec![0u8; VRAM_SIZE],
            sram: vec![0u8; SRAM_SIZE],
            rtc: vec![0u8; RTC_SIZE],
            frame: 0,
            samples_made: 0,
            format: options.pixel_format(),
            options,
            picture: vec![0u8; MAX_WIDTH as usize * MAX_HEIGHT as usize * 4],
            drawing: 0,
            shown: 0,
            width: BASE_WIDTH,
            height: BASE_HEIGHT,
            fps_num: 60,
            rate,
            files_root: None,
            disc: DiscState { index: 0, ejected: false, images: 1 },
            shutdown_sent: false,
            failed_once: false,
        }
    }

    fn bytes_per_pixel(&self) -> usize {
        if self.format == sys::PIXEL_FORMAT_XRGB8888 { 4 } else { 2 }
    }

    /// One frame of the game: the counter, the lives that fall every 60 frames, the level that rises when they run out.
    fn step_game(&mut self) {
        self.frame += 1;
        self.ram[FRAME_COUNTER..FRAME_COUNTER + 4].copy_from_slice(&(self.frame as u32).to_le_bytes());
        if self.frame % 60 == 0 {
            if self.ram[LIVES] == 0 {
                self.ram[LEVEL] = self.ram[LEVEL].wrapping_add(1);
                self.ram[LIVES] = 3;
            } else {
                self.ram[LIVES] -= 1;
            }
        }
        self.sram[(self.frame % SRAM_SIZE as u64) as usize] = self.frame as u8;
        self.rtc[..8].copy_from_slice(&self.frame.to_le_bytes());
        self.vram[(self.frame % VRAM_SIZE as u64) as usize] ^= 0xFF;
    }

    /// The picture's size as drawn: the native size times the internal scale.
    fn drawn(&self) -> (u32, u32) {
        (self.width * self.options.scale, self.height * self.options.scale)
    }

    /// The colour of a pixel, from the frame, the place, the lives and the pad.
    fn colour(&self, x: u32, y: u32) -> (u8, u8, u8) {
        let pad = u16::from_le_bytes([self.ram[PAD], self.ram[PAD + 1]]);
        let r = (x as u64 + self.drawing) as u8;
        let g = (y * 2 + self.ram[LIVES] as u32 * 40) as u8;
        let b = (pad as u8) ^ (x >> 3) as u8;
        (r, g, b)
    }

    fn draw(&self, out: &mut [u8], pitch: usize) {
        let bpp = self.bytes_per_pixel();
        let (scale, (width, height)) = (self.options.scale, self.drawn());
        for y in 0..height {
            let row = &mut out[y as usize * pitch..y as usize * pitch + width as usize * bpp];
            let native_y = if self.options.blend { (y / scale) & !1 } else { y / scale };
            for x in 0..width {
                let (r, g, b) = self.colour(x / scale, native_y);
                let at = x as usize * bpp;
                match self.format {
                    sys::PIXEL_FORMAT_XRGB8888 => row[at..at + 4].copy_from_slice(&[b, g, r, 0x5A]),
                    sys::PIXEL_FORMAT_RGB565 => {
                        let v = ((r as u16 >> 3) << 11) | ((g as u16 >> 2) << 5) | (b as u16 >> 3);
                        row[at..at + 2].copy_from_slice(&v.to_le_bytes());
                    }
                    _ => {
                        let v = ((r as u16 >> 3) << 10) | ((g as u16 >> 3) << 5) | (b as u16 >> 3);
                        row[at..at + 2].copy_from_slice(&v.to_le_bytes());
                    }
                }
            }
        }
    }

    /// The stereo frames this frame owes so that frame n has made exactly floor(n * rate / fps) of them.
    fn samples_due(&self) -> u64 {
        let (num, den) = self.rate;
        (self.frame * num) / (den * self.fps_num) - self.samples_made
    }
}

/// The value of stereo frame n: a saw on the left and its inverse on the right.
pub fn sample_at(n: u64) -> (i16, i16) {
    let v = ((n % 200) as i32 * 300 - 30_000) as i16;
    (v, v.wrapping_neg())
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_api_version() -> c_uint {
    sys::RETRO_API_VERSION
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn retro_set_environment(cb: sys::retro_environment_t) {
    CALLBACKS.lock().unwrap().environment = cb;
    options::declare();
    let mut no_game = false;
    environment(env::SET_SUPPORT_NO_GAME, &mut no_game as *mut bool as *mut c_void);
    let info = [
        sys::retro_controller_description { desc: c"RetroPad".as_ptr(), id: sys::DEVICE_JOYPAD },
        sys::retro_controller_description { desc: c"Test pad with sticks".as_ptr(), id: TEST_PAD_DEVICE },
    ];
    let ports = [
        sys::retro_controller_info { types: info.as_ptr(), num_types: 2 },
        sys::retro_controller_info { types: info.as_ptr(), num_types: 2 },
        sys::retro_controller_info { types: std::ptr::null(), num_types: 0 },
    ];
    environment(env::SET_CONTROLLER_INFO, ports.as_ptr() as *mut c_void);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn retro_set_video_refresh(cb: sys::retro_video_refresh_t) {
    CALLBACKS.lock().unwrap().video = cb;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn retro_set_audio_sample(cb: sys::retro_audio_sample_t) {
    CALLBACKS.lock().unwrap().sample = cb;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn retro_set_audio_sample_batch(cb: sys::retro_audio_sample_batch_t) {
    CALLBACKS.lock().unwrap().batch = cb;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn retro_set_input_poll(cb: sys::retro_input_poll_t) {
    CALLBACKS.lock().unwrap().poll = cb;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn retro_set_input_state(cb: sys::retro_input_state_t) {
    CALLBACKS.lock().unwrap().input = cb;
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_init() {
    let mut log_cb = sys::retro_log_callback { log: None };
    if environment(env::GET_LOG_INTERFACE, &mut log_cb as *mut _ as *mut c_void) {
        *LOG.lock().unwrap() = log_cb.log;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_deinit() {
    *CORE.lock().unwrap() = None;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn retro_get_system_info(info: *mut sys::retro_system_info) {
    let Some(info) = (unsafe { info.as_mut() }) else { return };
    info.library_name = c"EmuSen Test Core".as_ptr();
    info.library_version = c"1.0".as_ptr();
    info.valid_extensions = c"tcg|iso".as_ptr();
    info.need_fullpath = false;
    info.block_extract = false;
}

fn av_info(core: &Core) -> sys::retro_system_av_info {
    let (num, den) = core.rate;
    sys::retro_system_av_info {
        geometry: sys::retro_game_geometry {
            base_width: core.drawn().0,
            base_height: core.drawn().1,
            max_width: if core.options.grow { core.drawn().0 } else { MAX_WIDTH },
            max_height: if core.options.grow { core.drawn().1 } else { MAX_HEIGHT },
            aspect_ratio: 4.0 / 3.0,
        },
        timing: sys::retro_system_timing { fps: core.fps_num as f64, sample_rate: num as f64 / den as f64 },
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn retro_get_system_av_info(info: *mut sys::retro_system_av_info) {
    if let (Some(info), Some(core)) = (unsafe { info.as_mut() }, CORE.lock().unwrap().as_ref()) {
        *info = av_info(core);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_set_controller_port_device(port: c_uint, device: c_uint) {
    if let Some(core) = CORE.lock().unwrap().as_mut()
        && (port as usize) < 2
    {
        core.ram[PORT_DEVICE + port as usize] = device as u8;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_reset() {
    if let Some(core) = CORE.lock().unwrap().as_mut() {
        let options = core.options.clone();
        let (format, rate) = (core.format, core.rate);
        *core = Core::new(options);
        core.format = format;
        core.rate = rate;
    }
}

unsafe extern "C" fn frame_time(usec: i64) {
    *FRAME_TIME_SUM.lock().unwrap() += usec;
}

unsafe extern "C" fn set_eject(ejected: bool) -> bool {
    with_core(|c| {
        c.disc.ejected = ejected;
        true
    })
}
unsafe extern "C" fn get_eject() -> bool {
    with_core(|c| c.disc.ejected)
}
unsafe extern "C" fn get_index() -> c_uint {
    with_core(|c| c.disc.index as c_uint)
}
unsafe extern "C" fn set_index(index: c_uint) -> bool {
    with_core(|c| {
        if !c.disc.ejected || index > c.disc.images as c_uint {
            return false;
        }
        c.disc.index = index as u8;
        true
    })
}
unsafe extern "C" fn num_images() -> c_uint {
    with_core(|c| c.disc.images as c_uint)
}
unsafe extern "C" fn replace_image(index: c_uint, info: *const sys::retro_game_info) -> bool {
    let present = !info.is_null();
    with_core(|c| {
        if index >= c.disc.images as c_uint {
            return false;
        }
        if !present && index + 1 == c.disc.images as c_uint {
            c.disc.images -= 1;
        }
        true
    })
}
unsafe extern "C" fn add_image() -> bool {
    with_core(|c| {
        c.disc.images += 1;
        true
    })
}
unsafe extern "C" fn set_initial(_index: c_uint, _path: *const c_char) -> bool {
    true
}
unsafe extern "C" fn image_label(index: c_uint, s: *mut c_char, len: usize) -> bool {
    let label = format!("Disc {}", index + 1);
    if s.is_null() || len <= label.len() {
        return false;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(label.as_ptr() as *const c_char, s, label.len());
        *s.add(label.len()) = 0;
    }
    true
}

fn with_core<T: Default>(f: impl FnOnce(&mut Core) -> T) -> T {
    CORE.lock().unwrap().as_mut().map(f).unwrap_or_default()
}

static DISK: sys::retro_disk_control_ext_callback = sys::retro_disk_control_ext_callback {
    base: sys::retro_disk_control_callback {
        set_eject_state: Some(set_eject),
        get_eject_state: Some(get_eject),
        get_image_index: Some(get_index),
        set_image_index: Some(set_index),
        get_num_images: Some(num_images),
        replace_image_index: Some(replace_image),
        add_image_index: Some(add_image),
    },
    set_initial_image: Some(set_initial),
    get_image_path: None,
    get_image_label: Some(image_label),
};

/// The descriptors of a pad's controls, the console's own words for the RetroPad's.
const DESCRIPTORS: [(c_uint, &CStr); 6] = [(0, c"Cross"), (8, c"Circle"), (1, c"Square"), (9, c"Triangle"), (3, c"Start"), (2, c"Select")];

/// What the core says when it fails to start: at load, as an error; at its first frame, as PCEE2 says it, at info level.
pub const FAIL_AT_LOAD: &str = "the test machine found no firmware in its system folder";
pub const FAIL_AT_FIRST_FRAME: &str = "the test machine could not start";

#[unsafe(no_mangle)]
pub unsafe extern "C" fn retro_load_game(game: *const sys::retro_game_info) -> bool {
    let options = Options::read();
    if options.fail == "load" {
        log(sys::LOG_ERROR, FAIL_AT_LOAD);
        return false;
    }
    if options::get("tc_late_option") == "on" {
        options::declare_late();
    }
    *FRAME_TIME_SUM.lock().unwrap() = 0;
    if options.rumble {
        let mut r = sys::retro_rumble_interface { set_rumble_state: None };
        environment(env::GET_RUMBLE_INTERFACE, &mut r as *mut _ as *mut c_void);
        *RUMBLE.lock().unwrap() = r.set_rumble_state;
    }
    let mut core = Core::new(options.clone());
    if let Some(game) = unsafe { game.as_ref() }
        && !game.data.is_null()
        && game.size >= 4
    {
        let head = unsafe { std::slice::from_raw_parts(game.data as *const u8, 4) };
        if head != b"TCG1" {
            log(sys::LOG_ERROR, "not a test core game");
            return false;
        }
    }
    // The path a host passed, so a test can see which games reach a core by path (EmuSen_LibretroHost.md §2, E1).
    if let Some(game) = unsafe { game.as_ref() }
        && !game.path.is_null()
    {
        log(sys::LOG_INFO, &format!("game path: {}", unsafe { std::ffi::CStr::from_ptr(game.path) }.to_string_lossy()));
    }
    let mut format = if options.hw.starts_with("vulkan") { options.declared_format() } else { core.format };
    if !environment(env::SET_PIXEL_FORMAT, &mut format as *mut c_int as *mut c_void) {
        log(sys::LOG_WARN, "the pixel format was refused; drawing 0RGB1555");
        core.format = sys::PIXEL_FORMAT_0RGB1555;
    }
    let mut descriptors: Vec<sys::retro_input_descriptor> = DESCRIPTORS
        .iter()
        .flat_map(|&(id, d)| (0..2).map(move |port| sys::retro_input_descriptor { port, device: sys::DEVICE_JOYPAD, index: 0, id, description: d.as_ptr() }))
        .collect();
    descriptors.push(sys::retro_input_descriptor { port: 0, device: 0, index: 0, id: 0, description: std::ptr::null() });
    environment(env::SET_INPUT_DESCRIPTORS, descriptors.as_mut_ptr() as *mut c_void);
    let mut quirks = options.quirks();
    environment(env::SET_SERIALIZATION_QUIRKS, &mut quirks as *mut u64 as *mut c_void);
    let mut callback = sys::retro_frame_time_callback { callback: Some(frame_time), reference: 1_000_000 / 60 };
    environment(env::SET_FRAME_TIME_CALLBACK, &mut callback as *mut _ as *mut c_void);
    let mut disk_version: c_uint = 0;
    if environment(env::GET_DISK_CONTROL_INTERFACE_VERSION, &mut disk_version as *mut c_uint as *mut c_void) && disk_version >= 1 {
        environment(env::SET_DISK_CONTROL_EXT_INTERFACE, &DISK as *const _ as *mut c_void);
    }
    if options.files {
        core.files_root = directories();
    }
    if options.hw.starts_with("vulkan") && !vulkan::ask(&options.hw) {
        log(sys::LOG_ERROR, "the host refused a Vulkan context");
        return false;
    }
    *CORE.lock().unwrap() = Some(core);
    match options.memory {
        options::Memory::Maps => expose_maps(),
        options::Memory::SystemRam | options::Memory::None => {}
    }
    if options.log_thread {
        std::thread::spawn(|| log(sys::LOG_INFO, &format!("from a thread {} {}", "formatted", 42))).join().ok();
    }
    if options.probe {
        let results = probe::run();
        with_core(|c| c.ram[PROBE..PROBE + results.len()].copy_from_slice(&results));
    }
    if options.files {
        write_files(0);
    }
    true
}

/// The system and save directories the host answered, as text.
fn directories() -> Option<(String, String)> {
    let ask = |cmd| {
        let mut p: *const c_char = std::ptr::null();
        (environment(cmd, &mut p as *mut *const c_char as *mut c_void) && !p.is_null()).then(|| unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned())
    };
    Some((ask(env::GET_SYSTEM_DIRECTORY)?, ask(env::GET_SAVE_DIRECTORY)?))
}

/// The files the core keeps: a card in its system folder, notes in its save folder; changed at frame 10, the notes
/// removed at frame 20.
fn write_files(frame: u64) {
    let Some((system, save)) = CORE.lock().unwrap().as_ref().and_then(|c| c.files_root.clone()) else { return };
    let card = std::path::Path::new(&system).join("testcore").join("card.bin");
    let notes = std::path::Path::new(&save).join("testcore").join("notes.txt");
    let result = (|| -> std::io::Result<()> {
        match frame {
            0 => {
                std::fs::create_dir_all(card.parent().unwrap())?;
                std::fs::create_dir_all(notes.parent().unwrap())?;
                if !card.exists() {
                    std::fs::write(&card, vec![0xCAu8; 1024])?;
                }
                std::fs::write(&notes, b"first notes\n")
            }
            10 => {
                let mut bytes = std::fs::read(&card)?;
                bytes[0] = bytes[0].wrapping_add(1);
                std::fs::write(&card, bytes)
            }
            20 => std::fs::remove_file(&notes),
            _ => Ok(()),
        }
    })();
    if let Err(e) = result {
        log(sys::LOG_ERROR, &format!("files at frame {frame}: {e}"));
    }
}

fn expose_maps() {
    let (ram, vram) = {
        let mut guard = CORE.lock().unwrap();
        let core = guard.as_mut().unwrap();
        (core.ram.as_mut_ptr(), core.vram.as_mut_ptr())
    };
    let descriptors = [
        sys::retro_memory_descriptor { flags: 0, ptr: ram as *mut c_void, offset: 0, start: MAP_RAM_START, select: 0, disconnect: 0, len: RAM_SIZE, addrspace: std::ptr::null() },
        sys::retro_memory_descriptor { flags: 0, ptr: vram as *mut c_void, offset: 0, start: MAP_VRAM_START, select: 0, disconnect: 0, len: VRAM_SIZE, addrspace: c"VRAM".as_ptr() },
    ];
    let map = sys::retro_memory_map { descriptors: descriptors.as_ptr(), num_descriptors: 2 };
    environment(env::SET_MEMORY_MAPS, &map as *const _ as *mut c_void);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn retro_load_game_special(_kind: c_uint, _info: *const sys::retro_game_info, _num: usize) -> bool {
    false
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_unload_game() {
    *CORE.lock().unwrap() = None;
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_get_region() -> c_uint {
    0
}

fn read_input(core: &mut Core, cb: &Callbacks) {
    if let Some(poll) = cb.poll {
        unsafe { poll() };
    }
    let Some(input) = cb.input else { return };
    let device = core.ram[PORT_DEVICE] as c_uint;
    let pad = unsafe { input(0, sys::DEVICE_JOYPAD, 0, sys::DEVICE_ID_JOYPAD_MASK) } as u16;
    core.ram[PAD..PAD + 2].copy_from_slice(&pad.to_le_bytes());
    if device & sys::DEVICE_MASK == sys::DEVICE_ANALOG {
        let axes = [(0, 0), (0, 1), (1, 0), (1, 1)].map(|(index, id)| unsafe { input(0, sys::DEVICE_ANALOG, index, id) });
        for (i, v) in axes.iter().enumerate() {
            core.ram[AXES + i * 2..AXES + i * 2 + 2].copy_from_slice(&v.to_le_bytes());
        }
        let triggers = [sys::DEVICE_ID_JOYPAD_L2, sys::DEVICE_ID_JOYPAD_R2].map(|id| unsafe { input(0, sys::DEVICE_ANALOG, sys::DEVICE_INDEX_ANALOG_BUTTON, id) });
        for (i, v) in triggers.iter().enumerate() {
            core.ram[TRIGGERS + i * 2..TRIGGERS + i * 2 + 2].copy_from_slice(&v.to_le_bytes());
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_run() {
    let cb = callbacks();
    let mut updated = false;
    environment(env::GET_VARIABLE_UPDATE, &mut updated as *mut bool as *mut c_void);
    let mut guard = CORE.lock().unwrap();
    let Some(core) = guard.as_mut() else { return };
    if updated {
        let count = u32::from_le_bytes(core.ram[OPTION_UPDATES..OPTION_UPDATES + 4].try_into().unwrap()) + 1;
        core.ram[OPTION_UPDATES..OPTION_UPDATES + 4].copy_from_slice(&count.to_le_bytes());
        let size = core.drawn();
        core.options.reread_run_options();
        if core.drawn() != size {
            let g = av_info(core).geometry;
            drop(guard);
            environment(env::SET_GEOMETRY, &g as *const _ as *mut c_void);
            guard = CORE.lock().unwrap();
        }
    }
    let Some(core) = guard.as_mut() else { return };
    read_input(core, &cb);
    core.step_game();
    let pending = std::mem::take(&mut *FRAME_TIME_SUM.lock().unwrap());
    let sum = i64::from_le_bytes(core.ram[FRAME_TIME..FRAME_TIME + 8].try_into().unwrap()) + pending;
    core.ram[FRAME_TIME..FRAME_TIME + 8].copy_from_slice(&sum.to_le_bytes());
    core.ram[DISC] = core.disc.index;
    core.ram[DISC + 1] = core.disc.ejected as u8;
    core.ram[DISC + 2] = core.disc.images;
    let frame = core.frame;
    let options = core.options.clone();

    let mut geometry = None;
    let mut av = None;
    if options.grow && frame == 3 {
        core.options.scale = 2;
    }
    let options = core.options.clone();
    if options.geometry_change && frame == 60 {
        core.width = 256;
        core.height = 224;
        geometry = Some(av_info(core).geometry);
    } else if options.geometry_change && frame == 120 {
        core.fps_num = 50;
        core.rate = (44_100, 1);
        core.samples_made = (frame * 44_100) / 50;
        av = Some(av_info(core));
    }
    drop(guard);
    if let Some(g) = geometry {
        environment(env::SET_GEOMETRY, &g as *const _ as *mut c_void);
    }
    if let Some(a) = av {
        environment(env::SET_SYSTEM_AV_INFO, &a as *const _ as *mut c_void);
    }
    if options.rumble
        && let Some(set) = *RUMBLE.lock().unwrap()
    {
        for (port, strong, weak) in rumble_pattern(frame as u64) {
            unsafe { set(port, sys::RUMBLE_STRONG, strong) };
            unsafe { set(port, sys::RUMBLE_WEAK, weak) };
        }
    }
    if options.set_variable && frame == 5 {
        let var = sys::retro_variable { key: c"tc_note".as_ptr(), value: c"changed".as_ptr() };
        environment(env::SET_VARIABLE, &var as *const _ as *mut c_void);
    }
    let mut guard = CORE.lock().unwrap();
    let core = guard.as_mut().unwrap();
    let dupe = options.dupes && frame % 2 == 1;
    let bpp = core.bytes_per_pixel();
    let (width, height) = core.drawn();
    if dupe {
        drop(guard);
        if let Some(video) = cb.video {
            unsafe { video(std::ptr::null(), width, height, 0) };
        }
    } else {
        let mut target = None;
        let mut hw_picture = None;
        if options.framebuffer {
            let mut fb = sys::retro_framebuffer { data: std::ptr::null_mut(), width, height, pitch: 0, format: 0, access_flags: sys::MEMORY_ACCESS_WRITE, memory_flags: 0 };
            let core_format = core.format;
            drop(guard);
            let given = environment(env::GET_CURRENT_SOFTWARE_FRAMEBUFFER, &mut fb as *mut _ as *mut c_void);
            guard = CORE.lock().unwrap();
            if given && !fb.data.is_null() && fb.format == core_format && fb.pitch >= width as usize * bpp {
                target = Some((fb.data as *mut u8, fb.pitch));
            }
        }
        let core = guard.as_mut().unwrap();
        core.drawing = if core.options.lag { std::mem::replace(&mut core.shown, core.frame) } else { core.frame };
        let (data, pitch) = match target {
            Some((data, pitch)) => {
                let out = unsafe { std::slice::from_raw_parts_mut(data, pitch * height as usize) };
                core.draw(out, pitch);
                core.ram[DREW_IN_FRAMEBUFFER] = 1;
                (data as *const u8, pitch)
            }
            None if options.hw.starts_with("vulkan") => {
                let pitch = MAX_WIDTH as usize * bpp;
                let mut picture = std::mem::take(&mut core.picture);
                core.draw(&mut picture, pitch);
                hw_picture = Some(picture);
                (std::ptr::null(), pitch)
            }
            None => {
                let pitch = MAX_WIDTH as usize * bpp;
                let mut picture = std::mem::take(&mut core.picture);
                core.draw(&mut picture, pitch);
                core.picture = picture;
                core.ram[DREW_IN_FRAMEBUFFER] = 0;
                (core.picture.as_ptr(), pitch)
            }
        };
        drop(guard);
        if let Some(picture) = hw_picture {
            if let Some(video) = cb.video {
                vulkan::hand(&picture, pitch, width, height, video);
            }
            with_core(|c| c.picture = picture);
        } else if let Some(video) = cb.video {
            unsafe { video(data as *const c_void, width, height, pitch) };
        }
    }

    let mut guard = CORE.lock().unwrap();
    let core = guard.as_mut().unwrap();
    let due = core.samples_due();
    let first = core.samples_made;
    core.samples_made += due;
    drop(guard);
    if options.single_samples {
        if let Some(sample) = cb.sample {
            for n in first..first + due {
                let (l, r) = sample_at(n);
                unsafe { sample(l, r) };
            }
        }
    } else if let Some(batch) = cb.batch {
        let buffer: Vec<i16> = (first..first + due).flat_map(|n| <[i16; 2]>::from(sample_at(n))).collect();
        let mut sent = 0;
        while sent < due as usize {
            let taken = unsafe { batch(buffer.as_ptr().add(sent * 2), due as usize - sent) };
            if taken == 0 {
                break;
            }
            sent += taken;
        }
    }

    if frame == 1 && options.fail == "first_frame" {
        log(sys::LOG_INFO, FAIL_AT_FIRST_FRAME);
    }
    if frame == 2 && options.fault != "none" {
        let found = fault::run(&options.fault);
        with_core(|c| c.ram[FAULT] = found);
    }
    if options.files && (frame == 10 || frame == 20) {
        write_files(frame);
    }
    if options.shutdown_at == Some(frame) {
        let sent = with_core(|c| std::mem::replace(&mut c.shutdown_sent, true));
        if !sent {
            environment(env::SHUTDOWN, std::ptr::null_mut());
        }
    }
}

/// The state: the memory, the counters and the frame, at a size fixed for the machine unless the quirk says otherwise.
fn state_bytes(core: &Core) -> Vec<u8> {
    let mut out = Vec::with_capacity(STATE_FIXED);
    out.extend_from_slice(b"TCST");
    out.extend_from_slice(&core.frame.to_le_bytes());
    out.extend_from_slice(&core.samples_made.to_le_bytes());
    out.extend_from_slice(&core.ram);
    out.extend_from_slice(&core.vram);
    out.extend_from_slice(&core.sram);
    out.extend_from_slice(&core.rtc);
    if core.options.quirk == options::Quirk::Variable {
        out.extend(std::iter::repeat_n(0u8, (core.frame / 100) as usize));
    }
    out
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_serialize_size() -> usize {
    CORE.lock().unwrap().as_ref().map_or(0, |c| state_bytes(c).len())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn retro_serialize(data: *mut c_void, size: usize) -> bool {
    let guard = CORE.lock().unwrap();
    let Some(core) = guard.as_ref() else { return false };
    let bytes = state_bytes(core);
    if data.is_null() || size < bytes.len() {
        return false;
    }
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), data as *mut u8, bytes.len()) };
    true
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn retro_unserialize(data: *const c_void, size: usize) -> bool {
    let mut guard = CORE.lock().unwrap();
    let Some(core) = guard.as_mut() else { return false };
    if data.is_null() || size < STATE_FIXED {
        return false;
    }
    let bytes = unsafe { std::slice::from_raw_parts(data as *const u8, size) };
    if &bytes[..4] != b"TCST" {
        return false;
    }
    let mut at = 4;
    let mut take = |n: usize| {
        let s = &bytes[at..at + n];
        at += n;
        s
    };
    let frame = u64::from_le_bytes(take(8).try_into().unwrap());
    let samples = u64::from_le_bytes(take(8).try_into().unwrap());
    let ram = take(RAM_SIZE).to_vec();
    if core.options.unserialize_fails && !std::mem::replace(&mut core.failed_once, true) {
        core.ram[..RAM_SIZE / 2].copy_from_slice(&ram[..RAM_SIZE / 2]);
        core.frame = frame;
        return false;
    }
    core.frame = frame;
    core.samples_made = samples;
    core.ram.copy_from_slice(&ram);
    core.vram.copy_from_slice(take(VRAM_SIZE));
    core.sram.copy_from_slice(take(SRAM_SIZE));
    core.rtc.copy_from_slice(take(RTC_SIZE));
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_cheat_reset() {
    with_core(|c| {
        c.ram[CHEAT_COUNT..CHEAT_COUNT + 4].fill(0);
        c.ram[CHEAT_TEXT..CHEAT_TEXT + CHEAT_TEXT_LEN].fill(0);
    });
}

/// A code the core takes itself: `tc:<offset>=<value>` in hex, written once, when the option says the core honours it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn retro_cheat_set(_index: c_uint, enabled: bool, code: *const c_char) {
    if code.is_null() {
        return;
    }
    let text = unsafe { CStr::from_ptr(code) }.to_string_lossy().into_owned();
    with_core(|c| {
        if !c.options.cheat_set {
            return;
        }
        let count = u32::from_le_bytes(c.ram[CHEAT_COUNT..CHEAT_COUNT + 4].try_into().unwrap()) + 1;
        c.ram[CHEAT_COUNT..CHEAT_COUNT + 4].copy_from_slice(&count.to_le_bytes());
        let n = text.len().min(CHEAT_TEXT_LEN - 1);
        c.ram[CHEAT_TEXT..CHEAT_TEXT + CHEAT_TEXT_LEN].fill(0);
        c.ram[CHEAT_TEXT..CHEAT_TEXT + n].copy_from_slice(&text.as_bytes()[..n]);
        if let Some((offset, value)) = text.strip_prefix("tc:").and_then(|t| t.split_once('='))
            && enabled
            && let (Ok(o), Ok(v)) = (usize::from_str_radix(offset, 16), u8::from_str_radix(value, 16))
            && o < RAM_SIZE
        {
            c.ram[o] = v;
        }
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_get_memory_data(id: c_uint) -> *mut c_void {
    let mut guard = CORE.lock().unwrap();
    let Some(core) = guard.as_mut() else { return std::ptr::null_mut() };
    match id {
        sys::MEMORY_SAVE_RAM => core.sram.as_mut_ptr() as *mut c_void,
        sys::MEMORY_RTC => core.rtc.as_mut_ptr() as *mut c_void,
        sys::MEMORY_SYSTEM_RAM if core.options.memory != options::Memory::None => core.ram.as_mut_ptr() as *mut c_void,
        sys::MEMORY_VIDEO_RAM if core.options.memory != options::Memory::None => core.vram.as_mut_ptr() as *mut c_void,
        _ => std::ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn retro_get_memory_size(id: c_uint) -> usize {
    let guard = CORE.lock().unwrap();
    let Some(core) = guard.as_ref() else { return 0 };
    match id {
        sys::MEMORY_SAVE_RAM => SRAM_SIZE,
        sys::MEMORY_RTC => RTC_SIZE,
        sys::MEMORY_SYSTEM_RAM if core.options.memory != options::Memory::None => RAM_SIZE,
        sys::MEMORY_VIDEO_RAM if core.options.memory != options::Memory::None => VRAM_SIZE,
        _ => 0,
    }
}
