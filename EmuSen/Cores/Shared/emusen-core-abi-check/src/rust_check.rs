//! §5.2's check, at compile time: a generated export whose signature differs from the header's function type, a
//! struct field at another offset or of another type, or a constant of another value, fails this crate's build.
//! The lists below are compared with the header's own at run time, so that nothing is left out.

use std::mem::{align_of, offset_of, size_of};

use emusen_native::core::sys::{self, caps, event, flags, kind, pixel, status};
use emusen_native::core::{Axis, Control};
use emusen_native::debug;

use crate::{c, sig, test_core};

macro_rules! signatures {
    ($($f:ident : $t:ident),* $(,)?) => {
        $( const _: sig::$t = Some(test_core::$f); )*
        /// The exports assigned to the header's function types.
        pub const ASSIGNED: &[&str] = &[$(stringify!($f)),*];
    };
}

signatures! {
    emusen_core_abi_version: emusen_core_abi_version_fn,
    emusen_core_capabilities: emusen_core_capabilities_fn,
    emusen_core_info: emusen_core_info_fn,
    emusen_core_settings_schema: emusen_core_settings_schema_fn,
    emusen_core_firmware_for: emusen_core_firmware_for_fn,
    emusen_core_status_text: emusen_core_status_text_fn,
    emusen_core_set_crash_log: emusen_core_set_crash_log_fn,
    emusen_core_log_drain: emusen_core_log_drain_fn,
    emusen_core_create: emusen_core_create_fn,
    emusen_core_free: emusen_core_free_fn,
    emusen_core_reset: emusen_core_reset_fn,
    emusen_core_machine_info: emusen_core_machine_info_fn,
    emusen_core_last_error: emusen_core_last_error_fn,
    emusen_core_advance: emusen_core_advance_fn,
    emusen_core_present: emusen_core_present_fn,
    emusen_core_set_options: emusen_core_set_options_fn,
    emusen_core_frame_count: emusen_core_frame_count_fn,
    emusen_core_phases: emusen_core_phases_fn,
    emusen_core_events: emusen_core_events_fn,
    emusen_core_frame_info: emusen_core_frame_info_fn,
    emusen_core_frame_copy: emusen_core_frame_copy_fn,
    emusen_core_audio_rate: emusen_core_audio_rate_fn,
    emusen_core_audio_buffered: emusen_core_audio_buffered_fn,
    emusen_core_audio_drain: emusen_core_audio_drain_fn,
    emusen_core_set_audio_limit: emusen_core_set_audio_limit_fn,
    emusen_core_audio_peek: emusen_core_audio_peek_fn,
    emusen_core_set_mutes: emusen_core_set_mutes_fn,
    emusen_core_set_buttons: emusen_core_set_buttons_fn,
    emusen_core_set_axis: emusen_core_set_axis_fn,
    emusen_core_state_size: emusen_core_state_size_fn,
    emusen_core_state_save: emusen_core_state_save_fn,
    emusen_core_state_load: emusen_core_state_load_fn,
    emusen_core_state_layout: emusen_core_state_layout_fn,
    emusen_core_space_size: emusen_core_space_size_fn,
    emusen_core_space_read: emusen_core_space_read_fn,
    emusen_core_space_write: emusen_core_space_write_fn,
    emusen_core_battery: emusen_core_battery_fn,
    emusen_core_battery_saved: emusen_core_battery_saved_fn,
    emusen_core_set_rom_patches: emusen_core_set_rom_patches_fn,
    emusen_core_set_cheat_pokes: emusen_core_set_cheat_pokes_fn,
    emusen_core_set_settings: emusen_core_set_settings_fn,
    emusen_core_setting_notes: emusen_core_setting_notes_fn,
    emusen_core_debug_set: emusen_core_debug_set_fn,
    emusen_core_debug_set_stack: emusen_core_debug_set_stack_fn,
    emusen_core_debug_set_breakpoints: emusen_core_debug_set_breakpoints_fn,
    emusen_core_debug_set_ranges: emusen_core_debug_set_ranges_fn,
    emusen_core_debug_run_frame: emusen_core_debug_run_frame_fn,
    emusen_core_debug_writes: emusen_core_debug_writes_fn,
    emusen_core_debug_calls: emusen_core_debug_calls_fn,
    emusen_core_debug_profile: emusen_core_debug_profile_fn,
    emusen_core_debug_coverage: emusen_core_debug_coverage_fn,
    emusen_core_debug_counters: emusen_core_debug_counters_fn,
    emusen_core_debug_pc: emusen_core_debug_pc_fn,
    emusen_core_debug_registers: emusen_core_debug_registers_fn,
    emusen_core_debug_disassemble: emusen_core_debug_disassemble_fn,
}

macro_rules! layouts {
    ($($ours:ty = $theirs:ident { $($f:ident),* } [$($p:ident),*]);* $(;)?) => {
        $(
            const _: () = {
                assert!(size_of::<$ours>() == size_of::<c::$theirs>(), stringify!($theirs));
                assert!(align_of::<$ours>() == align_of::<c::$theirs>(), stringify!($theirs));
                $( assert!(offset_of!($ours, $f) == offset_of!(c::$theirs, $f), concat!(stringify!($theirs), ".", stringify!($f))); )*
                $( assert!(offset_of!($ours, $p) == offset_of!(c::$theirs, $p), concat!(stringify!($theirs), ".", stringify!($p))); )*
            };
            // The same type field by field: an array holds one type only. Pointers to the header's own structs
            // are compared as pointers of the same mutability.
            const _: fn(&$ours, &c::$theirs) = |a, b| {
                $( let _ = [a.$f, b.$f]; )*
                $( let _ = [a.$p.cast::<u8>(), b.$p.cast::<u8>()]; )*
            };
        )*
        /// Every field checked, as `struct.field`.
        pub const FIELDS: &[&str] = &[$($(concat!(stringify!($theirs), ".", stringify!($f)),)* $(concat!(stringify!($theirs), ".", stringify!($p)),)*)*];
    };
}

layouts! {
    sys::FileEntry = emusen_file { size, which, data, len } [];
    sys::CreateParams = emusen_create_params { size, host_abi_version, image, image_len, settings, settings_len, file_count, file_size, pixel_formats, error, error_len } [files];
    sys::FrameInfo = emusen_frame_info { size, format, width, height, stride, row_repeat, flags, aspect_num, aspect_den, reserved, serial, bytes } [];
    sys::Event = emusen_event { size, kind, a, b } [];
}

macro_rules! constants {
    ($($name:ident = $ours:expr),* $(,)?) => {
        $( const _: () = assert!(c::$name as i128 == ($ours) as i128, stringify!($name)); )*
        /// Every macro and enumeration constant checked, under bindgen's name.
        pub const CONSTANTS: &[&str] = &[$(stringify!($name)),*];
    };
}

constants! {
    EMUSEN_CORE_ABI_MAJOR = sys::ABI_MAJOR,
    EMUSEN_CORE_ABI_MINOR = sys::ABI_MINOR,
    EMUSEN_CORE_ABI_VERSION = sys::ABI_VERSION,
    EMUSEN_NULL = status::NULL,
    EMUSEN_TRUNCATED = status::TRUNCATED,
    EMUSEN_FOREIGN = status::FOREIGN,
    EMUSEN_STATE_VERSION = status::STATE_VERSION,
    EMUSEN_BAD_STRING = status::BAD_STRING,
    EMUSEN_BUFFER_TOO_SMALL = status::BUFFER_TOO_SMALL,
    EMUSEN_CORE_BAND_FIRST = status::CORE_BAND_FIRST,
    EMUSEN_CORE_BAND_LAST = status::CORE_BAND_LAST,
    EMUSEN_NOT_SUPPORTED = status::NOT_SUPPORTED,
    EMUSEN_NO_SUCH_SPACE = status::NO_SUCH_SPACE,
    EMUSEN_READ_ONLY = status::READ_ONLY,
    EMUSEN_UNKNOWN_SETTING = status::UNKNOWN_SETTING,
    EMUSEN_BAD_SETTING = status::BAD_SETTING,
    EMUSEN_NO_SUCH_PORT = status::NO_SUCH_PORT,
    EMUSEN_BAD_FILE = status::BAD_FILE,
    EMUSEN_BAD_STRUCT = status::BAD_STRUCT,
    EMUSEN_BAD_IMAGE = status::BAD_IMAGE,
    EMUSEN_FAULT_BASE = status::FAULT_BASE,
    EMUSEN_CAP_RESET = caps::RESET,
    EMUSEN_CAP_PRESENT = caps::PRESENT,
    EMUSEN_CAP_SNAPSHOT = caps::SNAPSHOT,
    EMUSEN_CAP_AXES = caps::AXES,
    EMUSEN_CAP_AUDIO_PEEK = caps::AUDIO_PEEK,
    EMUSEN_CAP_MUTES = caps::MUTES,
    EMUSEN_CAP_SETTINGS = caps::SETTINGS,
    EMUSEN_CAP_PHASES = caps::PHASES,
    EMUSEN_CAP_FRAME_SERIAL = caps::FRAME_SERIAL,
    EMUSEN_CAP_ROW_REPEAT = caps::ROW_REPEAT,
    EMUSEN_CAP_BATTERY_DIRTY = caps::BATTERY_DIRTY,
    EMUSEN_CAP_ROM_PATCHES = caps::ROM_PATCHES,
    EMUSEN_CAP_DEBUG = caps::DEBUG,
    EMUSEN_CAP_DEBUG_STACK = caps::DEBUG_STACK,
    EMUSEN_CAP_CHEAT_POKES = caps::CHEAT_POKES,
    EMUSEN_CAP_SETTING_NOTES = caps::SETTING_NOTES,
    EMUSEN_CAP_DEBUG_REGISTERS = caps::DEBUG_REGISTERS,
    EMUSEN_CAP_DEBUG_DISASSEMBLE = caps::DEBUG_DISASSEMBLE,
    EMUSEN_OPTION_SKIP_RENDERING = flags::OPTION_SKIP_RENDERING,
    EMUSEN_FRAME_WALKED = flags::FRAME_WALKED,
    EMUSEN_BATTERY_CHANGED = flags::BATTERY_CHANGED,
    EMUSEN_BATTERY_TRACKED = flags::BATTERY_TRACKED,
    EMUSEN_FILE_BATTERY = flags::FILE_BATTERY,
    EMUSEN_NO_COMPARE = flags::NO_COMPARE,
    EMUSEN_DEBUG_CALLS = debug::flag::CALLS,
    EMUSEN_DEBUG_WRITES = debug::flag::WRITES,
    EMUSEN_DEBUG_INTERRUPTS = debug::flag::INTERRUPTS,
    EMUSEN_DEBUG_EACH = debug::flag::EACH,
    EMUSEN_DEBUG_PROFILING = debug::flag::PROFILING,
    EMUSEN_DEBUG_COVERAGE = debug::flag::COVERAGE,
    EMUSEN_STOP_BREAKPOINT = debug::stop::BREAKPOINT,
    EMUSEN_STOP_EACH = debug::stop::EACH,
    EMUSEN_STOP_DEPTH = debug::stop::DEPTH,
    EMUSEN_STOP_DATA = debug::stop::DATA,
    EMUSEN_STOP_INTERRUPT = debug::stop::INTERRUPT,
    EMUSEN_STOP_RING = debug::stop::RING,
    EMUSEN_RUN_UNCHECKED = debug::run::UNCHECKED,
    EMUSEN_RUN_CONTINUE = debug::run::CONTINUE,
    EMUSEN_CALL_RETURN = debug::kind::RETURN,
    EMUSEN_CALL_CALL = debug::kind::CALL,
    EMUSEN_CALL_IRQ = debug::kind::IRQ,
    EMUSEN_CALL_NMI = debug::kind::NMI,
    EMUSEN_CALL_BRK = debug::kind::BRK,
    EMUSEN_CALL_COP = debug::kind::COP,
    emusen_pixel_format_EMUSEN_PIXEL_RGBA8888 = pixel::RGBA8888,
    emusen_pixel_format_EMUSEN_PIXEL_BGRA8888 = pixel::BGRA8888,
    emusen_pixel_format_EMUSEN_PIXEL_RGB565 = pixel::RGB565,
    emusen_state_kind_EMUSEN_STATE_FULL = kind::FULL,
    emusen_state_kind_EMUSEN_STATE_SNAPSHOT = kind::SNAPSHOT,
    emusen_event_kind_EMUSEN_EVENT_AUDIO_RATE = event::AUDIO_RATE,
    emusen_event_kind_EMUSEN_EVENT_GEOMETRY = event::GEOMETRY,
    emusen_event_kind_EMUSEN_EVENT_STATE_SIZE = event::STATE_SIZE,
    emusen_event_kind_EMUSEN_EVENT_BATTERY = event::BATTERY,
    emusen_event_kind_EMUSEN_EVENT_LOG = event::LOG,
    emusen_event_kind_EMUSEN_EVENT_MACHINE_INFO = event::MACHINE_INFO,
    emusen_control_EMUSEN_CONTROL_B = Control::B,
    emusen_control_EMUSEN_CONTROL_Y = Control::Y,
    emusen_control_EMUSEN_CONTROL_SELECT = Control::Select,
    emusen_control_EMUSEN_CONTROL_START = Control::Start,
    emusen_control_EMUSEN_CONTROL_UP = Control::Up,
    emusen_control_EMUSEN_CONTROL_DOWN = Control::Down,
    emusen_control_EMUSEN_CONTROL_LEFT = Control::Left,
    emusen_control_EMUSEN_CONTROL_RIGHT = Control::Right,
    emusen_control_EMUSEN_CONTROL_A = Control::A,
    emusen_control_EMUSEN_CONTROL_X = Control::X,
    emusen_control_EMUSEN_CONTROL_L = Control::L,
    emusen_control_EMUSEN_CONTROL_R = Control::R,
    emusen_control_EMUSEN_CONTROL_L2 = Control::L2,
    emusen_control_EMUSEN_CONTROL_R2 = Control::R2,
    emusen_control_EMUSEN_CONTROL_L3 = Control::L3,
    emusen_control_EMUSEN_CONTROL_R3 = Control::R3,
    emusen_axis_EMUSEN_AXIS_LEFT_X = Axis::LeftX,
    emusen_axis_EMUSEN_AXIS_LEFT_Y = Axis::LeftY,
    emusen_axis_EMUSEN_AXIS_RIGHT_X = Axis::RightX,
    emusen_axis_EMUSEN_AXIS_RIGHT_Y = Axis::RightY,
    emusen_axis_EMUSEN_AXIS_LEFT_TRIGGER = Axis::LeftTrigger,
    emusen_axis_EMUSEN_AXIS_RIGHT_TRIGGER = Axis::RightTrigger,
}
