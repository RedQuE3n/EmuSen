//! The header's half in Rust: `emusen_core.h`'s structs and constants, kept identical to it by
//! `emusen-core-abi-check` (EmuSen_CoreAPI.md §5.2). Names follow Rust; the C name is on each item.

/// `EMUSEN_CORE_ABI_MAJOR`.
pub const ABI_MAJOR: u32 = 1;
/// `EMUSEN_CORE_ABI_MINOR`.
pub const ABI_MINOR: u32 = 11;
/// `EMUSEN_CORE_ABI_VERSION`.
pub const ABI_VERSION: u32 = (ABI_MAJOR << 16) | ABI_MINOR;

/// `emusen_machine`, never dereferenced: the exports cast it to the macro's own instance.
#[repr(C)]
pub struct Machine {
    _opaque: [u8; 0],
    _not_send: std::marker::PhantomData<*mut u8>,
}

/// `emusen_file`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FileEntry {
    pub size: u32,
    pub which: u32,
    pub data: *const u8,
    pub len: usize,
}

/// `emusen_create_params`. `flags` is since 1.3; a host of an earlier minor passes the 88 bytes before it.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CreateParams {
    pub size: u32,
    pub host_abi_version: u32,
    pub image: *const u8,
    pub image_len: usize,
    pub settings: *const u8,
    pub settings_len: usize,
    pub files: *const FileEntry,
    pub file_count: usize,
    pub file_size: usize,
    pub pixel_formats: u64,
    pub error: *mut u8,
    pub error_len: usize,
    pub flags: u32,
    /// Since 1.9, `DEVICE_PICTURE`: the host's device lent, or null; absent from a shorter struct.
    pub device: *const DeviceLend,
    /// Since 1.11: with an image of no bytes, the NUL-terminated id of the system to start empty; absent from a shorter struct.
    pub system: *const std::ffi::c_char,
}

impl CreateParams {
    /// The struct's size at 1.0, the least a core accepts (EmuSen_CoreAPI.md §4.2).
    pub const SIZE_1_0: usize = std::mem::offset_of!(CreateParams, flags);
}

/// `emusen_device_lend`, since 1.9: the host's Vulkan device and a queue the core alone submits to, as Vulkan's
/// handles in integers, and the host's timeline semaphore `done` (EmuSen_CoreAPI.md §6.29).
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DeviceLend {
    pub size: u32,
    pub api_version: u32,
    pub get_instance_proc_addr: Option<unsafe extern "C" fn()>,
    pub instance: u64,
    pub physical_device: u64,
    pub device: u64,
    pub queue_family: u32,
    pub queue_index: u32,
    pub host_queue_family: u32,
    pub reserved: u32,
    pub done: u64,
}

/// `emusen_device_frame`, since 1.9: the frame's picture where it lies on the lent device, and the core's timeline
/// semaphore with the value it reaches once the picture is written. A core fills everything but `size`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DeviceFrame {
    pub size: u32,
    pub format: u32,
    pub buffer: u64,
    pub offset: u64,
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub reserved: u32,
    pub ready: u64,
    pub value: u64,
}

/// `emusen_media`, since 1.1: a disc for a drive, with its companions.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Media {
    pub size: u32,
    pub flags: u32,
    pub image: *const u8,
    pub image_len: usize,
    pub files: *const FileEntry,
    pub file_count: usize,
    pub file_size: usize,
}

/// `emusen_frame_info`. A core fills everything but `size`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FrameInfo {
    pub size: u32,
    pub format: u32,
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub row_repeat: i32,
    pub flags: u32,
    pub aspect_num: u32,
    pub aspect_den: u32,
    pub reserved: u32,
    pub serial: i64,
    pub bytes: i64,
}

impl FrameInfo {
    /// An RGBA8888 picture of square pixels shown once a row; the serial is the macro's to fill without `FRAME_SERIAL`.
    pub fn rgba(width: i32, height: i32) -> FrameInfo {
        FrameInfo {
            size: std::mem::size_of::<FrameInfo>() as u32,
            format: pixel::RGBA8888,
            width,
            height,
            stride: width * 4,
            row_repeat: 1,
            bytes: width as i64 * height as i64 * 4,
            ..FrameInfo::default()
        }
    }
}

/// `emusen_raw_frame`, since 1.10: the shape of a frame's raw picture in one encoding (EmuSen_CoreAPI.md §6.30). A core
/// fills everything but `size`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RawFrame {
    pub size: u32,
    pub element: u32,
    pub width: i32,
    pub height: i32,
    pub stride: i32,
    pub reserved: u32,
    pub bytes: i64,
}

impl RawFrame {
    /// `width` by `height` values of `element` bytes, rows packed; `bytes` is the crate's to fill.
    pub fn packed(element: u32, width: i32, height: i32) -> RawFrame {
        RawFrame { size: std::mem::size_of::<RawFrame>() as u32, element, width, height, stride: width * element as i32, ..RawFrame::default() }
    }
}

/// `emusen_event`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Event {
    pub size: u32,
    pub kind: u32,
    pub a: i64,
    pub b: i64,
}

/// The status codes (§6.15): the state crate's band, the interface's, and the fault band's base.
pub mod status {
    pub use crate::ffi::status::{BAD_STRING, BUFFER_TOO_SMALL, FIRST_CORE as CORE_BAND_FIRST, FOREIGN, LAST_CORE as CORE_BAND_LAST, NULL, TRUNCATED, VERSION as STATE_VERSION};
    pub use crate::abi::status::{BAD_FILE, BAD_SETTING, FAULT_BASE, NO_SUCH_PORT, NO_SUCH_SPACE, NOT_SUPPORTED, READ_ONLY, UNKNOWN_SETTING};
    pub const BAD_STRUCT: i32 = -263;
    pub const BAD_IMAGE: i32 = -264;
    /// Since 1.1, `MEDIA`.
    pub const LID_CLOSED: i32 = -265;
    pub const WRONG_MEDIA: i32 = -266;
    pub const NO_SUCH_DRIVE: i32 = -267;
}

/// The capability bits; 0-13 keep the pre-stable numbering.
pub mod caps {
    pub use crate::abi::caps::{AUDIO_PEEK, AXES, BATTERY_DIRTY, DEBUG, DEBUG_STACK, FRAME_SERIAL, MUTES, PHASES, PRESENT, RESET, ROM_PATCHES, ROW_REPEAT, SETTINGS, SNAPSHOT};
    pub const CHEAT_POKES: u64 = 1 << 14;
    pub const SETTING_NOTES: u64 = 1 << 15;
    pub const DEBUG_REGISTERS: u64 = 1 << 16;
    pub const DEBUG_DISASSEMBLE: u64 = 1 << 17;
    /// Since 1.1.
    pub const MEDIA: u64 = 1 << 18;
    /// Since 1.2: read watches and read data breakpoints.
    pub const DEBUG_READS: u64 = 1 << 19;
    /// Since 1.3: the core keeps pointers into an image and media files a host lends; `core::keep` is how.
    pub const BORROW: u64 = 1 << 20;
    /// Since 1.9: the core draws on a device the host lends and hands its picture there (`frame_device`).
    pub const DEVICE_PICTURE: u64 = 1 << 21;
    /// Since 1.10: the core hands the picture before its palette in the encodings machine info lists (`raw_*`).
    pub const RAW_PICTURE: u64 = 1 << 22;
    /// Bits 23-47, for later minors; a core may not claim them.
    pub const UNASSIGNED: u64 = ((1 << 48) - 1) & !((1 << 23) - 1);
}

/// `EMUSEN_OPTION_*`, `EMUSEN_FRAME_*`, `EMUSEN_BATTERY_*`, `EMUSEN_FILE_BATTERY`, `EMUSEN_NO_COMPARE`.
pub mod flags {
    pub const OPTION_SKIP_RENDERING: u32 = 1;
    pub const FRAME_WALKED: u32 = 1;
    pub const BATTERY_CHANGED: u32 = 1;
    pub const BATTERY_TRACKED: u32 = 2;
    pub const FILE_BATTERY: u32 = 0;
    pub const NO_COMPARE: u32 = u32::MAX;
    /// Since 1.1: the first companion of an image set; the nth file a sheet names is `FILE_MEDIA + n`.
    pub const FILE_MEDIA: u32 = 65_536;
    /// Since 1.1: `emusen_media.flags`, the drive set for a state about to load.
    pub const MEDIA_RESTORE: u32 = 1;
    /// Since 1.3: `emusen_create_params.flags`, the image and its media files lent until the loan ends.
    pub const CREATE_LENT: u32 = 1;
    /// Since 1.3: `emusen_media.flags`, the same for an inserted disc.
    pub const MEDIA_LENT: u32 = 2;
    /// Since 1.7: `emusen_create_params.flags`, the removable battery files kept by the host apart from the game.
    pub const CREATE_SHARED_REMOVABLE: u32 = 2;
    /// Since 1.8: `emusen_create_params.flags`, the console's own battery files kept by the host apart from the game.
    pub const CREATE_SHARED_CONSOLE: u32 = 4;
    /// Since 1.4: in `MEDIA_SEEK`'s `a`, the call's image itself, which no file entry numbers.
    pub const FILE_IMAGE: u32 = u32::MAX;
}

/// `enum emusen_pixel_format`.
pub mod pixel {
    pub const RGBA8888: u32 = 0;
    pub const BGRA8888: u32 = 1;
    pub const RGB565: u32 = 2;
}

/// `enum emusen_state_kind`.
pub mod kind {
    pub const FULL: u32 = 0;
    pub const SNAPSHOT: u32 = 1;
}

/// `enum emusen_event_kind`.
pub mod event {
    pub const AUDIO_RATE: u32 = 1;
    pub const GEOMETRY: u32 = 2;
    pub const STATE_SIZE: u32 = 3;
    pub const BATTERY: u32 = 4;
    pub const LOG: u32 = 5;
    pub const MACHINE_INFO: u32 = 6;
    /// Since 1.4: a drive is about to read its disc at a place; `core::seek` raises it. 7 is kept for outputs.
    pub const MEDIA_SEEK: u32 = 8;
}

/// Every export, and the capability bit that claims it; zero for a required one.
pub const EXPORTS: &[(&str, u64)] = &[
    ("emusen_core_abi_version", 0),
    ("emusen_core_capabilities", 0),
    ("emusen_core_info", 0),
    ("emusen_core_settings_schema", 0),
    ("emusen_core_firmware_for", 0),
    ("emusen_core_status_text", 0),
    ("emusen_core_set_crash_log", 0),
    ("emusen_core_log_drain", 0),
    ("emusen_core_create", 0),
    ("emusen_core_free", 0),
    ("emusen_core_reset", caps::RESET),
    ("emusen_core_machine_info", 0),
    ("emusen_core_last_error", 0),
    ("emusen_core_advance", 0),
    ("emusen_core_present", caps::PRESENT),
    ("emusen_core_set_options", 0),
    ("emusen_core_frame_count", 0),
    ("emusen_core_phases", caps::PHASES),
    ("emusen_core_events", 0),
    ("emusen_core_frame_info", 0),
    ("emusen_core_frame_copy", 0),
    ("emusen_core_audio_rate", 0),
    ("emusen_core_audio_buffered", 0),
    ("emusen_core_audio_drain", 0),
    ("emusen_core_set_audio_limit", 0),
    ("emusen_core_audio_peek", caps::AUDIO_PEEK),
    ("emusen_core_set_mutes", caps::MUTES),
    ("emusen_core_set_buttons", 0),
    ("emusen_core_set_axis", caps::AXES),
    ("emusen_core_state_size", 0),
    ("emusen_core_state_save", 0),
    ("emusen_core_state_load", 0),
    ("emusen_core_state_layout", 0),
    ("emusen_core_space_size", 0),
    ("emusen_core_space_read", 0),
    ("emusen_core_space_write", 0),
    ("emusen_core_battery", 0),
    ("emusen_core_battery_saved", 0),
    ("emusen_core_set_rom_patches", caps::ROM_PATCHES),
    ("emusen_core_set_cheat_pokes", caps::CHEAT_POKES),
    ("emusen_core_set_settings", caps::SETTINGS),
    ("emusen_core_setting_notes", caps::SETTING_NOTES),
    ("emusen_core_debug_set", caps::DEBUG),
    ("emusen_core_debug_set_stack", caps::DEBUG_STACK),
    ("emusen_core_debug_set_breakpoints", caps::DEBUG),
    ("emusen_core_debug_set_ranges", caps::DEBUG),
    ("emusen_core_debug_run_frame", caps::DEBUG),
    ("emusen_core_debug_writes", caps::DEBUG),
    ("emusen_core_debug_calls", caps::DEBUG),
    ("emusen_core_debug_profile", caps::DEBUG),
    ("emusen_core_debug_coverage", caps::DEBUG),
    ("emusen_core_debug_counters", caps::DEBUG),
    ("emusen_core_debug_pc", caps::DEBUG),
    ("emusen_core_debug_registers", caps::DEBUG_REGISTERS),
    ("emusen_core_debug_disassemble", caps::DEBUG_DISASSEMBLE),
    ("emusen_core_media_lid", caps::MEDIA),
    ("emusen_core_media_insert", caps::MEDIA),
    ("emusen_core_debug_reads", caps::DEBUG_READS),
    ("emusen_core_frame_device", caps::DEVICE_PICTURE),
    ("emusen_core_raw_select", caps::RAW_PICTURE),
    ("emusen_core_raw_info", caps::RAW_PICTURE),
    ("emusen_core_raw_copy", caps::RAW_PICTURE),
];

/// Each assigned bit's name, as core info's `capabilities` lists it.
pub const CAPABILITY_NAMES: &[(u64, &str)] = &[
    (caps::RESET, "RESET"),
    (caps::PRESENT, "PRESENT"),
    (caps::SNAPSHOT, "SNAPSHOT"),
    (caps::AXES, "AXES"),
    (caps::AUDIO_PEEK, "AUDIO_PEEK"),
    (caps::MUTES, "MUTES"),
    (caps::SETTINGS, "SETTINGS"),
    (caps::PHASES, "PHASES"),
    (caps::FRAME_SERIAL, "FRAME_SERIAL"),
    (caps::ROW_REPEAT, "ROW_REPEAT"),
    (caps::BATTERY_DIRTY, "BATTERY_DIRTY"),
    (caps::ROM_PATCHES, "ROM_PATCHES"),
    (caps::DEBUG, "DEBUG"),
    (caps::DEBUG_STACK, "DEBUG_STACK"),
    (caps::CHEAT_POKES, "CHEAT_POKES"),
    (caps::SETTING_NOTES, "SETTING_NOTES"),
    (caps::DEBUG_REGISTERS, "DEBUG_REGISTERS"),
    (caps::DEBUG_DISASSEMBLE, "DEBUG_DISASSEMBLE"),
    (caps::MEDIA, "MEDIA"),
    (caps::DEBUG_READS, "DEBUG_READS"),
    (caps::BORROW, "BORROW"),
    (caps::DEVICE_PICTURE, "DEVICE_PICTURE"),
    (caps::RAW_PICTURE, "RAW_PICTURE"),
];
