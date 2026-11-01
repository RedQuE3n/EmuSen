//! The part of libretro's interface EmuSen uses, written by hand from `libretro.h` (MIT); see EmuSen_LibretroHost.md §1.2.
#![allow(non_camel_case_types)]

use std::ffi::{c_char, c_int, c_uint, c_void};

pub const RETRO_API_VERSION: c_uint = 1;

pub const DEVICE_NONE: c_uint = 0;
pub const DEVICE_JOYPAD: c_uint = 1;
pub const DEVICE_MOUSE: c_uint = 2;
pub const DEVICE_KEYBOARD: c_uint = 3;
pub const DEVICE_LIGHTGUN: c_uint = 4;
pub const DEVICE_ANALOG: c_uint = 5;
pub const DEVICE_POINTER: c_uint = 6;
pub const DEVICE_TYPE_SHIFT: c_uint = 8;
pub const DEVICE_MASK: c_uint = (1 << DEVICE_TYPE_SHIFT) - 1;
pub const DEVICE_ID_JOYPAD_MASK: c_uint = 256;
pub const DEVICE_INDEX_ANALOG_LEFT: c_uint = 0;
pub const DEVICE_INDEX_ANALOG_RIGHT: c_uint = 1;
pub const DEVICE_INDEX_ANALOG_BUTTON: c_uint = 2;
pub const DEVICE_ID_ANALOG_X: c_uint = 0;
pub const DEVICE_ID_ANALOG_Y: c_uint = 1;
pub const DEVICE_ID_JOYPAD_L2: c_uint = 12;
pub const DEVICE_ID_JOYPAD_R2: c_uint = 13;

pub const MEMORY_SAVE_RAM: c_uint = 0;
pub const MEMORY_RTC: c_uint = 1;
pub const MEMORY_SYSTEM_RAM: c_uint = 2;
pub const MEMORY_VIDEO_RAM: c_uint = 3;

pub const MEMDESC_CONST: u64 = 1 << 0;
pub const MEMDESC_BIGENDIAN: u64 = 1 << 1;

pub const QUIRK_INCOMPLETE: u64 = 1 << 0;
pub const QUIRK_MUST_INITIALIZE: u64 = 1 << 1;
pub const QUIRK_CORE_VARIABLE_SIZE: u64 = 1 << 2;
pub const QUIRK_FRONT_VARIABLE_SIZE: u64 = 1 << 3;
pub const QUIRK_SINGLE_SESSION: u64 = 1 << 4;
pub const QUIRK_ENDIAN_DEPENDENT: u64 = 1 << 5;
pub const QUIRK_PLATFORM_DEPENDENT: u64 = 1 << 6;

pub const AV_ENABLE_VIDEO: c_int = 1 << 0;
pub const AV_ENABLE_AUDIO: c_int = 1 << 1;

pub const PIXEL_FORMAT_0RGB1555: c_int = 0;
pub const PIXEL_FORMAT_XRGB8888: c_int = 1;
pub const PIXEL_FORMAT_RGB565: c_int = 2;

pub const LOG_DEBUG: c_int = 0;
pub const LOG_INFO: c_int = 1;
pub const LOG_WARN: c_int = 2;
pub const LOG_ERROR: c_int = 3;

pub const HW_CONTEXT_NONE: c_int = 0;
pub const HW_CONTEXT_VULKAN: c_int = 6;

pub const SAVESTATE_CONTEXT_NORMAL: c_int = 0;
pub const SAVESTATE_CONTEXT_RUNAHEAD_SAME_INSTANCE: c_int = 1;

pub const LANGUAGE_ENGLISH: c_uint = 0;
pub const THROTTLE_NONE: c_uint = 0;
pub const NUM_CORE_OPTION_VALUES_MAX: usize = 128;

/// `RETRO_HW_FRAME_BUFFER_VALID`, the picture pointer that says the frame is on the device.
pub const HW_FRAME_BUFFER_VALID: *const c_void = usize::MAX as *const c_void;

/// The environment commands, each with the flags `libretro.h` gives it.
pub mod env {
    use std::ffi::c_uint;
    pub const EXPERIMENTAL: c_uint = 0x10000;
    pub const SET_ROTATION: c_uint = 1;
    pub const GET_OVERSCAN: c_uint = 2;
    pub const GET_CAN_DUPE: c_uint = 3;
    pub const SET_MESSAGE: c_uint = 6;
    pub const SHUTDOWN: c_uint = 7;
    pub const SET_PERFORMANCE_LEVEL: c_uint = 8;
    pub const GET_SYSTEM_DIRECTORY: c_uint = 9;
    pub const SET_PIXEL_FORMAT: c_uint = 10;
    pub const SET_INPUT_DESCRIPTORS: c_uint = 11;
    pub const SET_KEYBOARD_CALLBACK: c_uint = 12;
    pub const SET_DISK_CONTROL_INTERFACE: c_uint = 13;
    pub const SET_HW_RENDER: c_uint = 14;
    pub const GET_VARIABLE: c_uint = 15;
    pub const SET_VARIABLES: c_uint = 16;
    pub const GET_VARIABLE_UPDATE: c_uint = 17;
    pub const SET_SUPPORT_NO_GAME: c_uint = 18;
    pub const GET_LIBRETRO_PATH: c_uint = 19;
    pub const SET_FRAME_TIME_CALLBACK: c_uint = 21;
    pub const SET_AUDIO_CALLBACK: c_uint = 22;
    pub const GET_RUMBLE_INTERFACE: c_uint = 23;
    pub const GET_INPUT_DEVICE_CAPABILITIES: c_uint = 24;
    pub const GET_SENSOR_INTERFACE: c_uint = 25 | EXPERIMENTAL;
    pub const GET_CAMERA_INTERFACE: c_uint = 26 | EXPERIMENTAL;
    pub const GET_LOG_INTERFACE: c_uint = 27;
    pub const GET_PERF_INTERFACE: c_uint = 28;
    pub const GET_LOCATION_INTERFACE: c_uint = 29;
    /// Also `GET_CONTENT_DIRECTORY`, its former name.
    pub const GET_CORE_ASSETS_DIRECTORY: c_uint = 30;
    pub const GET_SAVE_DIRECTORY: c_uint = 31;
    pub const SET_SYSTEM_AV_INFO: c_uint = 32;
    pub const SET_PROC_ADDRESS_CALLBACK: c_uint = 33;
    pub const SET_SUBSYSTEM_INFO: c_uint = 34;
    pub const SET_CONTROLLER_INFO: c_uint = 35;
    pub const SET_MEMORY_MAPS: c_uint = 36 | EXPERIMENTAL;
    pub const SET_GEOMETRY: c_uint = 37;
    pub const GET_USERNAME: c_uint = 38;
    pub const GET_LANGUAGE: c_uint = 39;
    pub const GET_CURRENT_SOFTWARE_FRAMEBUFFER: c_uint = 40 | EXPERIMENTAL;
    pub const GET_HW_RENDER_INTERFACE: c_uint = 41 | EXPERIMENTAL;
    pub const SET_SUPPORT_ACHIEVEMENTS: c_uint = 42 | EXPERIMENTAL;
    pub const SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE: c_uint = 43 | EXPERIMENTAL;
    pub const SET_SERIALIZATION_QUIRKS: c_uint = 44;
    pub const SET_HW_SHARED_CONTEXT: c_uint = 44 | EXPERIMENTAL;
    pub const GET_VFS_INTERFACE: c_uint = 45 | EXPERIMENTAL;
    pub const GET_LED_INTERFACE: c_uint = 46 | EXPERIMENTAL;
    pub const GET_AUDIO_VIDEO_ENABLE: c_uint = 47 | EXPERIMENTAL;
    pub const GET_MIDI_INTERFACE: c_uint = 48 | EXPERIMENTAL;
    pub const GET_FASTFORWARDING: c_uint = 49 | EXPERIMENTAL;
    pub const GET_TARGET_REFRESH_RATE: c_uint = 50 | EXPERIMENTAL;
    pub const GET_INPUT_BITMASKS: c_uint = 51 | EXPERIMENTAL;
    pub const GET_CORE_OPTIONS_VERSION: c_uint = 52;
    pub const SET_CORE_OPTIONS: c_uint = 53;
    pub const SET_CORE_OPTIONS_INTL: c_uint = 54;
    pub const SET_CORE_OPTIONS_DISPLAY: c_uint = 55;
    pub const GET_PREFERRED_HW_RENDER: c_uint = 56;
    pub const GET_DISK_CONTROL_INTERFACE_VERSION: c_uint = 57;
    pub const SET_DISK_CONTROL_EXT_INTERFACE: c_uint = 58;
    pub const GET_MESSAGE_INTERFACE_VERSION: c_uint = 59;
    pub const SET_MESSAGE_EXT: c_uint = 60;
    pub const GET_INPUT_MAX_USERS: c_uint = 61;
    pub const SET_AUDIO_BUFFER_STATUS_CALLBACK: c_uint = 62;
    pub const SET_MINIMUM_AUDIO_LATENCY: c_uint = 63;
    pub const SET_FASTFORWARDING_OVERRIDE: c_uint = 64;
    pub const SET_CONTENT_INFO_OVERRIDE: c_uint = 65;
    pub const GET_GAME_INFO_EXT: c_uint = 66;
    pub const SET_CORE_OPTIONS_V2: c_uint = 67;
    pub const SET_CORE_OPTIONS_V2_INTL: c_uint = 68;
    pub const SET_CORE_OPTIONS_UPDATE_DISPLAY_CALLBACK: c_uint = 69;
    pub const SET_VARIABLE: c_uint = 70;
    pub const GET_THROTTLE_STATE: c_uint = 71 | EXPERIMENTAL;
    pub const GET_SAVESTATE_CONTEXT: c_uint = 72 | EXPERIMENTAL;
    pub const GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT: c_uint = 73 | EXPERIMENTAL;
    pub const GET_JIT_CAPABLE: c_uint = 74;
    pub const GET_MICROPHONE_INTERFACE: c_uint = 75 | EXPERIMENTAL;
    pub const GET_DEVICE_POWER: c_uint = 77 | EXPERIMENTAL;
    pub const SET_NETPACKET_INTERFACE: c_uint = 78;
    pub const GET_PLAYLIST_DIRECTORY: c_uint = 79;
    pub const GET_FILE_BROWSER_START_DIRECTORY: c_uint = 80;
    pub const GET_TARGET_SAMPLE_RATE: c_uint = 81 | EXPERIMENTAL;
    pub const GET_NETPLAY_CLIENT_INDEX: c_uint = 82 | EXPERIMENTAL;
    pub const EXEC_MEM_ALLOC: c_uint = 83;
    pub const EXEC_MEM_FREE: c_uint = 84;
    pub const GET_AUDIO_SAMPLE_BATCH_FLOAT: c_uint = 85 | EXPERIMENTAL;
    pub const GET_MEMORY_STATUS: c_uint = 86 | EXPERIMENTAL;
    pub const SET_SERIALIZATION_QUIRKS_2: c_uint = 87;
    pub const GET_VFS_AUTHORIZED_LOCATIONS: c_uint = 93 | EXPERIMENTAL;
    pub const GET_AUDIO_SAMPLE_BATCH_MULTI: c_uint = 94 | EXPERIMENTAL;
    pub const SET_VIDEO_VIEWS: c_uint = 95 | EXPERIMENTAL;
    pub const GET_VIDEO_VIEWS_STATUS: c_uint = 96 | EXPERIMENTAL;
    pub const GET_VR_FRAME_STATE: c_uint = 97 | EXPERIMENTAL;
    pub const GET_VR_HEAD_POSE: c_uint = 98 | EXPERIMENTAL;
    pub const GET_VIDEO_DECODE_INTERFACE: c_uint = 99 | EXPERIMENTAL;
    pub const GET_VR_CONTROLLER_POSES: c_uint = 100 | EXPERIMENTAL;
    pub const GET_HDR_METADATA: c_uint = 101 | EXPERIMENTAL;
    pub const GET_DISPLAY_METADATA: c_uint = 102 | EXPERIMENTAL;
    pub const GET_HW_RENDER_UPSCALER_INTERFACE: c_uint = 103 | EXPERIMENTAL;
}

pub type retro_environment_t = Option<unsafe extern "C" fn(cmd: c_uint, data: *mut c_void) -> bool>;
pub type retro_video_refresh_t = Option<unsafe extern "C" fn(data: *const c_void, width: c_uint, height: c_uint, pitch: usize)>;
pub type retro_audio_sample_t = Option<unsafe extern "C" fn(left: i16, right: i16)>;
pub type retro_audio_sample_batch_t = Option<unsafe extern "C" fn(data: *const i16, frames: usize) -> usize>;
pub type retro_input_poll_t = Option<unsafe extern "C" fn()>;
pub type retro_input_state_t = Option<unsafe extern "C" fn(port: c_uint, device: c_uint, index: c_uint, id: c_uint) -> i16>;
pub type retro_log_printf_t = Option<unsafe extern "C" fn(level: c_int, fmt: *const c_char, ...)>;
pub type retro_frame_time_callback_t = Option<unsafe extern "C" fn(usec: i64)>;
pub type retro_audio_callback_t = Option<unsafe extern "C" fn()>;
pub type retro_audio_set_state_callback_t = Option<unsafe extern "C" fn(enabled: bool)>;
pub type retro_audio_buffer_status_callback_t = Option<unsafe extern "C" fn(active: bool, occupancy: c_uint, underrun_likely: bool)>;
pub type retro_keyboard_event_t = Option<unsafe extern "C" fn(down: bool, keycode: c_uint, character: u32, modifiers: u16)>;
pub type retro_set_rumble_state_t = Option<unsafe extern "C" fn(port: c_uint, effect: c_int, strength: u16) -> bool>;
pub type retro_core_options_update_display_callback_t = Option<unsafe extern "C" fn() -> bool>;
pub type retro_proc_address_t = Option<unsafe extern "C" fn()>;
pub type retro_get_proc_address_t = Option<unsafe extern "C" fn(sym: *const c_char) -> retro_proc_address_t>;
pub type retro_hw_context_reset_t = Option<unsafe extern "C" fn()>;
pub type retro_hw_get_current_framebuffer_t = Option<unsafe extern "C" fn() -> usize>;
pub type retro_hw_get_proc_address_t = Option<unsafe extern "C" fn(sym: *const c_char) -> retro_proc_address_t>;

#[repr(C)]
pub struct retro_system_info {
    pub library_name: *const c_char,
    pub library_version: *const c_char,
    pub valid_extensions: *const c_char,
    pub need_fullpath: bool,
    pub block_extract: bool,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct retro_game_geometry {
    pub base_width: c_uint,
    pub base_height: c_uint,
    pub max_width: c_uint,
    pub max_height: c_uint,
    pub aspect_ratio: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct retro_system_timing {
    pub fps: f64,
    pub sample_rate: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct retro_system_av_info {
    pub geometry: retro_game_geometry,
    pub timing: retro_system_timing,
}

#[repr(C)]
pub struct retro_game_info {
    pub path: *const c_char,
    pub data: *const c_void,
    pub size: usize,
    pub meta: *const c_char,
}

#[repr(C)]
pub struct retro_game_info_ext {
    pub full_path: *const c_char,
    pub archive_path: *const c_char,
    pub archive_file: *const c_char,
    pub dir: *const c_char,
    pub name: *const c_char,
    pub ext: *const c_char,
    pub meta: *const c_char,
    pub data: *const c_void,
    pub size: usize,
    pub file_in_archive: bool,
    pub persistent_data: bool,
}

#[repr(C)]
pub struct retro_system_content_info_override {
    pub extensions: *const c_char,
    pub need_fullpath: bool,
    pub persistent_data: bool,
}

#[repr(C)]
pub struct retro_variable {
    pub key: *const c_char,
    pub value: *const c_char,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct retro_core_option_value {
    pub value: *const c_char,
    pub label: *const c_char,
}

#[repr(C)]
pub struct retro_core_option_definition {
    pub key: *const c_char,
    pub desc: *const c_char,
    pub info: *const c_char,
    pub values: [retro_core_option_value; NUM_CORE_OPTION_VALUES_MAX],
    pub default_value: *const c_char,
}

#[repr(C)]
pub struct retro_core_options_intl {
    pub us: *const retro_core_option_definition,
    pub local: *const retro_core_option_definition,
}

#[repr(C)]
pub struct retro_core_option_v2_category {
    pub key: *const c_char,
    pub desc: *const c_char,
    pub info: *const c_char,
}

#[repr(C)]
pub struct retro_core_option_v2_definition {
    pub key: *const c_char,
    pub desc: *const c_char,
    pub desc_categorized: *const c_char,
    pub info: *const c_char,
    pub info_categorized: *const c_char,
    pub category_key: *const c_char,
    pub values: [retro_core_option_value; NUM_CORE_OPTION_VALUES_MAX],
    pub default_value: *const c_char,
}

#[repr(C)]
pub struct retro_core_options_v2 {
    pub categories: *const retro_core_option_v2_category,
    pub definitions: *const retro_core_option_v2_definition,
}

#[repr(C)]
pub struct retro_core_options_v2_intl {
    pub us: *const retro_core_options_v2,
    pub local: *const retro_core_options_v2,
}

#[repr(C)]
pub struct retro_core_option_display {
    pub key: *const c_char,
    pub visible: bool,
}

#[repr(C)]
pub struct retro_core_options_update_display_callback {
    pub callback: retro_core_options_update_display_callback_t,
}

#[repr(C)]
pub struct retro_log_callback {
    pub log: retro_log_printf_t,
}

#[repr(C)]
pub struct retro_message {
    pub msg: *const c_char,
    pub frames: c_uint,
}

#[repr(C)]
pub struct retro_message_ext {
    pub msg: *const c_char,
    pub duration: c_uint,
    pub priority: c_uint,
    pub level: c_int,
    pub target: c_int,
    pub kind: c_int,
    pub progress: i8,
}

#[repr(C)]
pub struct retro_input_descriptor {
    pub port: c_uint,
    pub device: c_uint,
    pub index: c_uint,
    pub id: c_uint,
    pub description: *const c_char,
}

#[repr(C)]
pub struct retro_controller_description {
    pub desc: *const c_char,
    pub id: c_uint,
}

#[repr(C)]
pub struct retro_controller_info {
    pub types: *const retro_controller_description,
    pub num_types: c_uint,
}

#[repr(C)]
pub struct retro_memory_descriptor {
    pub flags: u64,
    pub ptr: *mut c_void,
    pub offset: usize,
    pub start: usize,
    pub select: usize,
    pub disconnect: usize,
    pub len: usize,
    pub addrspace: *const c_char,
}

#[repr(C)]
pub struct retro_memory_map {
    pub descriptors: *const retro_memory_descriptor,
    pub num_descriptors: c_uint,
}

#[repr(C)]
pub struct retro_frame_time_callback {
    pub callback: retro_frame_time_callback_t,
    pub reference: i64,
}

#[repr(C)]
pub struct retro_audio_callback {
    pub callback: retro_audio_callback_t,
    pub set_state: retro_audio_set_state_callback_t,
}

#[repr(C)]
pub struct retro_audio_buffer_status_callback {
    pub callback: retro_audio_buffer_status_callback_t,
}

#[repr(C)]
pub struct retro_keyboard_callback {
    pub callback: retro_keyboard_event_t,
}

#[repr(C)]
pub struct retro_rumble_interface {
    pub set_rumble_state: retro_set_rumble_state_t,
}

#[repr(C)]
pub struct retro_throttle_state {
    pub mode: c_uint,
    pub rate: f32,
}

#[repr(C)]
pub struct retro_framebuffer {
    pub data: *mut c_void,
    pub width: c_uint,
    pub height: c_uint,
    pub pitch: usize,
    pub format: c_int,
    pub access_flags: c_uint,
    pub memory_flags: c_uint,
}
pub const MEMORY_ACCESS_WRITE: c_uint = 1 << 0;
pub const MEMORY_ACCESS_READ: c_uint = 1 << 1;
pub const MEMORY_TYPE_CACHED: c_uint = 1 << 0;

#[repr(C)]
pub struct retro_perf_counter {
    pub ident: *const c_char,
    pub start: u64,
    pub total: u64,
    pub call_cnt: u64,
    pub registered: bool,
}

#[repr(C)]
pub struct retro_perf_callback {
    pub get_time_usec: Option<unsafe extern "C" fn() -> i64>,
    pub get_cpu_features: Option<unsafe extern "C" fn() -> u64>,
    pub get_perf_counter: Option<unsafe extern "C" fn() -> u64>,
    pub perf_register: Option<unsafe extern "C" fn(counter: *mut retro_perf_counter)>,
    pub perf_start: Option<unsafe extern "C" fn(counter: *mut retro_perf_counter)>,
    pub perf_stop: Option<unsafe extern "C" fn(counter: *mut retro_perf_counter)>,
    pub perf_log: Option<unsafe extern "C" fn()>,
}

#[repr(C)]
pub struct retro_get_proc_address_interface {
    pub get_proc_address: retro_get_proc_address_t,
}

#[repr(C)]
pub struct retro_hw_render_callback {
    pub context_type: c_int,
    pub context_reset: retro_hw_context_reset_t,
    pub get_current_framebuffer: retro_hw_get_current_framebuffer_t,
    pub get_proc_address: retro_hw_get_proc_address_t,
    pub depth: bool,
    pub stencil: bool,
    pub bottom_left_origin: bool,
    pub version_major: c_uint,
    pub version_minor: c_uint,
    pub cache_context: bool,
    pub context_destroy: retro_hw_context_reset_t,
    pub debug_context: bool,
}

/// The disk control interface of version 0, which the extended one begins with.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct retro_disk_control_callback {
    pub set_eject_state: Option<unsafe extern "C" fn(ejected: bool) -> bool>,
    pub get_eject_state: Option<unsafe extern "C" fn() -> bool>,
    pub get_image_index: Option<unsafe extern "C" fn() -> c_uint>,
    pub set_image_index: Option<unsafe extern "C" fn(index: c_uint) -> bool>,
    pub get_num_images: Option<unsafe extern "C" fn() -> c_uint>,
    pub replace_image_index: Option<unsafe extern "C" fn(index: c_uint, info: *const retro_game_info) -> bool>,
    pub add_image_index: Option<unsafe extern "C" fn() -> bool>,
}

/// An image's path or label written into the core's buffer.
pub type retro_get_image_text_t = Option<unsafe extern "C" fn(index: c_uint, s: *mut c_char, len: usize) -> bool>;

/// `retro_disk_control_ext_callback`: version 0's seven entries, laid out first as the header lays them, and three more.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct retro_disk_control_ext_callback {
    pub base: retro_disk_control_callback,
    pub set_initial_image: Option<unsafe extern "C" fn(index: c_uint, path: *const c_char) -> bool>,
    pub get_image_path: retro_get_image_text_t,
    pub get_image_label: retro_get_image_text_t,
}

/// The exports every libretro core has, by name.
pub const CORE_EXPORTS: [&str; 25] = [
    "retro_set_environment",
    "retro_set_video_refresh",
    "retro_set_audio_sample",
    "retro_set_audio_sample_batch",
    "retro_set_input_poll",
    "retro_set_input_state",
    "retro_init",
    "retro_deinit",
    "retro_api_version",
    "retro_get_system_info",
    "retro_get_system_av_info",
    "retro_set_controller_port_device",
    "retro_reset",
    "retro_run",
    "retro_serialize_size",
    "retro_serialize",
    "retro_unserialize",
    "retro_cheat_reset",
    "retro_cheat_set",
    "retro_load_game",
    "retro_load_game_special",
    "retro_unload_game",
    "retro_get_region",
    "retro_get_memory_data",
    "retro_get_memory_size",
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, offset_of, size_of};

    /// Sizes and offsets gcc 16 gave for `libretro.h` on x86-64, measured 2026-10-31.
    #[test]
    #[cfg(target_pointer_width = "64")]
    fn layouts_match_the_header_on_64_bit_targets() {
        assert_eq!(size_of::<retro_system_info>(), 32);
        assert_eq!(size_of::<retro_game_geometry>(), 20);
        assert_eq!(size_of::<retro_system_av_info>(), 40);
        assert_eq!(offset_of!(retro_system_av_info, timing), 24);
        assert_eq!(size_of::<retro_game_info>(), 32);
        assert_eq!(size_of::<retro_game_info_ext>(), 80);
        assert_eq!(size_of::<retro_core_option_definition>(), 24 + 128 * 16 + 8);
        assert_eq!(size_of::<retro_core_option_v2_definition>(), 48 + 128 * 16 + 8);
        assert_eq!(size_of::<retro_input_descriptor>(), 24);
        assert_eq!(size_of::<retro_memory_descriptor>(), 64);
        assert_eq!(size_of::<retro_message_ext>(), 32);
        assert_eq!(offset_of!(retro_message_ext, progress), 28);
        assert_eq!(size_of::<retro_hw_render_callback>(), 64);
        assert_eq!(offset_of!(retro_hw_render_callback, version_major), 36);
        assert_eq!(offset_of!(retro_hw_render_callback, context_destroy), 48);
        assert_eq!(size_of::<retro_system_content_info_override>(), 16);
        assert_eq!(size_of::<retro_throttle_state>(), 8);
        assert_eq!(size_of::<retro_frame_time_callback>(), 16);
        assert_eq!(size_of::<retro_framebuffer>(), 40);
        assert_eq!(size_of::<retro_perf_counter>(), 40);
        assert_eq!(size_of::<retro_disk_control_ext_callback>(), 80);
        assert_eq!(align_of::<retro_frame_time_callback>(), 8);
    }
}
