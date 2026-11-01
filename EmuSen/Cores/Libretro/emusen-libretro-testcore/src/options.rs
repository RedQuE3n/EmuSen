//! The test core's options, declared through v2, v1 or the oldest `SET_VARIABLES` as the host's version and
//! `EMUSEN_TESTCORE_OPTIONS` allow, and read back at load.

use std::ffi::{CStr, CString, c_char, c_uint, c_void};

use emusen_libretro_sys::{self as sys, env};

use crate::environment;

/// (key, label, category, values, information); the first value is the default.
pub const OPTIONS: [(&str, &str, &str, &[&str], &str); 23] = [
    ("tc_format", "Pixel format", "picture", &["xrgb8888", "rgb565", "0rgb1555"], "The format the picture is drawn in."),
    ("tc_draw", "Drawing", "picture", &["copy", "framebuffer"], "Into the core's own buffer, or the host's."),
    ("tc_dupes", "Duplicate frames", "picture", &["off", "on"], "Every odd frame handed as a duplicate."),
    ("tc_geometry", "Geometry changes", "picture", &["off", "on", "grow"], "The picture's size changed at frame 60, the timing at 120; or, grow, maxima declared at the native size and the picture drawn twice as large from frame 3 with nothing said."),
    ("tc_rate", "Sound rate", "sound", &["48000", "44100", "32040.5"], "The rate sound is made at."),
    ("tc_sound", "Sound path", "sound", &["batch", "single"], "Sound through the batch or the single-sample call."),
    ("tc_memory", "Memory", "memory", &["system_ram", "maps", "none"], "How the core's memory is exposed."),
    ("tc_cheat_set", "Its own cheats", "memory", &["ignored", "honoured"], "Whether retro_cheat_set does anything."),
    ("tc_files", "Files", "memory", &["off", "on"], "Files written in the system and save folders."),
    ("tc_quirk", "State quirk", "states", &["none", "incomplete", "variable", "single", "platform"], "The serialization quirk declared."),
    ("tc_unserialize", "State loads", "states", &["ok", "fail"], "The first load fails half-way."),
    ("tc_shutdown", "Shutdown", "states", &["off", "30"], "The core asks to stop at frame 30."),
    ("tc_probe", "Environment probe", "probe", &["off", "on"], "Every environment command called at load."),
    ("tc_log_thread", "Log from a thread", "probe", &["off", "on"], "A formatted log line from a thread of the core's."),
    ("tc_late_option", "Declares at load", "probe", &["off", "on"], "An option declared only while loading the game."),
    ("tc_fault", "A fault at frame 2", "probe", &["none", "handled_segv", "segv", "abort", "write_outside", "socket", "hang", "read_system"], "What the core does at its second frame, for the broker's proofs."),
    ("tc_fail", "Fails to start", "probe", &["none", "load", "first_frame"], "The core refuses the game at load, or takes it and says at its first frame that it cannot start."),
    ("tc_set_variable", "Changes its option", "probe", &["off", "on"], "The core sets tc_note itself at frame 5."),
    ("tc_note", "Note", "probe", &["plain", "changed"], "Changed by the core when it changes its own option."),
    ("tc_rumble", "Rumble", "probe", &["off", "on"], "Both motors of ports 0 and 1 driven in a pattern a test reads back."),
    ("tc_scale", "Internal scale", "picture", &["1", "2"], "The picture drawn at a multiple of its native size, as a console core's upscaling is."),
    ("tc_deinterlace", "Deinterlacing", "picture", &["off", "blend"], "Each odd row a copy of the row above, as a console core's deinterlacing blends two fields."),
    ("tc_hw", "Vulkan picture", "picture", &["off", "vulkan", "vulkan_bgra", "vulkan_v1", "vulkan_v1_loader"], "The same picture drawn into an image on a Vulkan device of the core's, RGBA or BGRA; v1 its device made through negotiation v1, by the host's vkCreateDevice or the loader's."),
];

pub const CATEGORIES: [(&str, &str); 5] = [("picture", "Picture"), ("sound", "Sound"), ("memory", "Memory"), ("states", "States"), ("probe", "Probe")];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Memory {
    SystemRam,
    Maps,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quirk {
    None,
    Incomplete,
    Variable,
    Single,
    Platform,
}

#[derive(Clone, Debug)]
pub struct Options {
    pub format: String,
    pub framebuffer: bool,
    pub dupes: bool,
    pub geometry_change: bool,
    pub grow: bool,
    pub rate: String,
    pub single_samples: bool,
    pub memory: Memory,
    pub cheat_set: bool,
    pub files: bool,
    pub quirk: Quirk,
    pub unserialize_fails: bool,
    pub shutdown_at: Option<u64>,
    pub probe: bool,
    pub log_thread: bool,
    pub set_variable: bool,
    pub note: String,
    pub fault: String,
    pub fail: String,
    pub rumble: bool,
    pub hw: String,
    pub scale: u32,
    pub blend: bool,
}

/// A C string kept for the process's life, as libretro's declarations must be.
fn keep(s: &str) -> *const c_char {
    CString::new(s).unwrap().into_raw()
}

/// The options declared in the newest form the host takes.
pub fn declare() {
    let forced = std::env::var("EMUSEN_TESTCORE_OPTIONS").unwrap_or_default();
    let mut version: c_uint = 0;
    environment(env::GET_CORE_OPTIONS_VERSION, &mut version as *mut c_uint as *mut c_void);
    if version >= 2 && forced.is_empty() {
        declare_v2();
    } else if version >= 1 && forced != "legacy" {
        declare_v1();
    } else {
        declare_legacy();
    }
}

fn values(list: &[&str]) -> [sys::retro_core_option_value; sys::NUM_CORE_OPTION_VALUES_MAX] {
    let mut out = [sys::retro_core_option_value { value: std::ptr::null(), label: std::ptr::null() }; sys::NUM_CORE_OPTION_VALUES_MAX];
    for (slot, v) in out.iter_mut().zip(list) {
        *slot = sys::retro_core_option_value { value: keep(v), label: keep(&format!("{v} (label)")) };
    }
    out
}

/// The options again, with one more declared only while a game loads.
pub fn declare_late() {
    let late: &[(&str, &str, &str, &[&str], &str)] = &[("tc_late", "Declared at load", "probe", &["first", "second"], "Known only after a game loaded.")];
    declare_v2_from(&OPTIONS.iter().chain(late).copied().collect::<Vec<_>>());
}

fn declare_v2() {
    declare_v2_from(&OPTIONS);
}

fn declare_v2_from(list: &[(&str, &str, &str, &[&str], &str)]) {
    let mut categories: Vec<sys::retro_core_option_v2_category> =
        CATEGORIES.iter().map(|&(k, d)| sys::retro_core_option_v2_category { key: keep(k), desc: keep(d), info: keep(&format!("The {k} options.")) }).collect();
    categories.push(sys::retro_core_option_v2_category { key: std::ptr::null(), desc: std::ptr::null(), info: std::ptr::null() });
    let mut definitions: Vec<sys::retro_core_option_v2_definition> = list
        .iter()
        .map(|&(key, label, category, list, info)| sys::retro_core_option_v2_definition {
            key: keep(key),
            desc: keep(label),
            desc_categorized: keep(&format!("{label} (in its category)")),
            info: keep(info),
            info_categorized: std::ptr::null(),
            category_key: keep(category),
            values: values(list),
            default_value: keep(list[0]),
        })
        .collect();
    definitions.push(sys::retro_core_option_v2_definition {
        key: std::ptr::null(),
        desc: std::ptr::null(),
        desc_categorized: std::ptr::null(),
        info: std::ptr::null(),
        info_categorized: std::ptr::null(),
        category_key: std::ptr::null(),
        values: values(&[]),
        default_value: std::ptr::null(),
    });
    let options = Box::leak(Box::new(sys::retro_core_options_v2 { categories: Box::leak(categories.into_boxed_slice()).as_ptr(), definitions: Box::leak(definitions.into_boxed_slice()).as_ptr() }));
    environment(env::SET_CORE_OPTIONS_V2, options as *mut _ as *mut c_void);
}

fn declare_v1() {
    let mut definitions: Vec<sys::retro_core_option_definition> = OPTIONS
        .iter()
        .map(|&(key, label, _, list, info)| sys::retro_core_option_definition { key: keep(key), desc: keep(label), info: keep(info), values: values(list), default_value: keep(list[0]) })
        .collect();
    definitions.push(sys::retro_core_option_definition { key: std::ptr::null(), desc: std::ptr::null(), info: std::ptr::null(), values: values(&[]), default_value: std::ptr::null() });
    let list = Box::leak(definitions.into_boxed_slice());
    environment(env::SET_CORE_OPTIONS, list.as_mut_ptr() as *mut c_void);
}

fn declare_legacy() {
    let mut variables: Vec<sys::retro_variable> =
        OPTIONS.iter().map(|&(key, label, _, list, _)| sys::retro_variable { key: keep(key), value: keep(&format!("{label}; {}", list.join("|"))) }).collect();
    variables.push(sys::retro_variable { key: std::ptr::null(), value: std::ptr::null() });
    let list = Box::leak(variables.into_boxed_slice());
    environment(env::SET_VARIABLES, list.as_mut_ptr() as *mut c_void);
}

/// An option's value as the host answers it, or its default.
pub fn get(key: &str) -> String {
    let key_c = CString::new(key).unwrap();
    let mut var = sys::retro_variable { key: key_c.as_ptr(), value: std::ptr::null() };
    let default = OPTIONS.iter().find(|o| o.0 == key).map_or("", |o| o.3[0]);
    if environment(env::GET_VARIABLE, &mut var as *mut _ as *mut c_void) && !var.value.is_null() {
        unsafe { CStr::from_ptr(var.value) }.to_string_lossy().into_owned()
    } else {
        default.to_owned()
    }
}

impl Options {
    pub fn read() -> Options {
        let on = |k: &str| get(k) == "on";
        let options = Options {
            format: get("tc_format"),
            framebuffer: get("tc_draw") == "framebuffer",
            dupes: on("tc_dupes"),
            geometry_change: on("tc_geometry"),
            grow: get("tc_geometry") == "grow",
            rate: get("tc_rate"),
            single_samples: get("tc_sound") == "single",
            memory: match get("tc_memory").as_str() {
                "maps" => Memory::Maps,
                "none" => Memory::None,
                _ => Memory::SystemRam,
            },
            cheat_set: get("tc_cheat_set") == "honoured",
            files: on("tc_files"),
            quirk: match get("tc_quirk").as_str() {
                "incomplete" => Quirk::Incomplete,
                "variable" => Quirk::Variable,
                "single" => Quirk::Single,
                "platform" => Quirk::Platform,
                _ => Quirk::None,
            },
            unserialize_fails: get("tc_unserialize") == "fail",
            shutdown_at: get("tc_shutdown").parse().ok(),
            probe: on("tc_probe"),
            log_thread: on("tc_log_thread"),
            set_variable: on("tc_set_variable"),
            note: get("tc_note"),
            fault: get("tc_fault"),
            fail: get("tc_fail"),
            rumble: on("tc_rumble"),
            hw: get("tc_hw"),
            scale: get("tc_scale").parse().unwrap_or(1),
            blend: get("tc_deinterlace") == "blend",
        };
        let display = sys::retro_core_option_display { key: c"tc_note".as_ptr(), visible: options.set_variable };
        environment(env::SET_CORE_OPTIONS_DISPLAY, &display as *const _ as *mut c_void);
        options
    }

    pub fn reread_run_options(&mut self) {
        self.note = get("tc_note");
        self.scale = get("tc_scale").parse().unwrap_or(1);
        self.blend = get("tc_deinterlace") == "blend";
    }

    pub fn pixel_format(&self) -> i32 {
        if self.hw != "off" && !self.hw.is_empty() {
            return sys::PIXEL_FORMAT_XRGB8888;
        }
        self.declared_format()
    }

    /// The format `tc_format` names, which a Vulkan core declares though its hardware frames are 8-bit (§10o).
    pub fn declared_format(&self) -> i32 {
        match self.format.as_str() {
            "rgb565" => sys::PIXEL_FORMAT_RGB565,
            "0rgb1555" => sys::PIXEL_FORMAT_0RGB1555,
            _ => sys::PIXEL_FORMAT_XRGB8888,
        }
    }

    /// The rate as a ratio of whole numbers.
    pub fn rate(&self) -> (u64, u64) {
        match self.rate.as_str() {
            "44100" => (44_100, 1),
            "32040.5" => (64_081, 2),
            _ => (48_000, 1),
        }
    }

    pub fn quirks(&self) -> u64 {
        match self.quirk {
            Quirk::None => 0,
            Quirk::Incomplete => sys::QUIRK_INCOMPLETE,
            Quirk::Variable => sys::QUIRK_CORE_VARIABLE_SIZE,
            Quirk::Single => sys::QUIRK_SINGLE_SESSION,
            Quirk::Platform => sys::QUIRK_PLATFORM_DEPENDENT,
        }
    }
}
