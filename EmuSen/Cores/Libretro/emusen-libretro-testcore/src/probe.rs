//! Every environment command a host is asked about, each with data of the shape `libretro.h` gives it; `run` records
//! whether the host answered each, one byte per entry in `PROBES`' order (EmuSen_LibretroHost.md §9).

use std::ffi::{c_char, c_int, c_uint, c_void};

use emusen_libretro_sys::{self as sys, env};

use crate::layout::{BASE_HEIGHT, BASE_WIDTH, MAX_HEIGHT, MAX_WIDTH};

#[derive(Clone, Copy, Debug)]
pub enum Data {
    OutBool,
    OutStr,
    OutUint,
    OutInt,
    OutU64,
    OutF32,
    /// A zeroed block the host may write or must leave alone.
    Zeroed,
    Null,
    InUint(u32),
    InBool(bool),
    InU64(u64),
    PixelFormat,
    Message,
    MessageExt,
    Descriptors,
    Keyboard,
    FrameTime,
    AudioCallback,
    AudioBufferCallback,
    ProcAddress,
    Subsystem,
    Controllers,
    Geometry,
    AvInfo,
    Display,
    UpdateDisplay,
    Variable,
    SetVariable,
    HwRenderGl,
    Framebuffer,
    Disk,
}

/// (name, command, data), in the order the results are written.
pub const PROBES: &[(&str, c_uint, Data)] = &[
    ("SET_ROTATION", env::SET_ROTATION, Data::InUint(1)),
    ("GET_OVERSCAN", env::GET_OVERSCAN, Data::OutBool),
    ("GET_CAN_DUPE", env::GET_CAN_DUPE, Data::OutBool),
    ("SET_MESSAGE", env::SET_MESSAGE, Data::Message),
    ("SET_PERFORMANCE_LEVEL", env::SET_PERFORMANCE_LEVEL, Data::InUint(3)),
    ("GET_SYSTEM_DIRECTORY", env::GET_SYSTEM_DIRECTORY, Data::OutStr),
    ("SET_PIXEL_FORMAT", env::SET_PIXEL_FORMAT, Data::PixelFormat),
    ("SET_INPUT_DESCRIPTORS", env::SET_INPUT_DESCRIPTORS, Data::Descriptors),
    ("SET_KEYBOARD_CALLBACK", env::SET_KEYBOARD_CALLBACK, Data::Keyboard),
    ("SET_DISK_CONTROL_INTERFACE", env::SET_DISK_CONTROL_INTERFACE, Data::Disk),
    ("SET_HW_RENDER (OpenGL)", env::SET_HW_RENDER, Data::HwRenderGl),
    ("GET_VARIABLE", env::GET_VARIABLE, Data::Variable),
    ("GET_VARIABLE_UPDATE", env::GET_VARIABLE_UPDATE, Data::OutBool),
    ("SET_SUPPORT_NO_GAME", env::SET_SUPPORT_NO_GAME, Data::InBool(false)),
    ("GET_LIBRETRO_PATH", env::GET_LIBRETRO_PATH, Data::OutStr),
    ("SET_FRAME_TIME_CALLBACK", env::SET_FRAME_TIME_CALLBACK, Data::FrameTime),
    ("SET_AUDIO_CALLBACK", env::SET_AUDIO_CALLBACK, Data::AudioCallback),
    ("GET_RUMBLE_INTERFACE", env::GET_RUMBLE_INTERFACE, Data::Zeroed),
    ("GET_INPUT_DEVICE_CAPABILITIES", env::GET_INPUT_DEVICE_CAPABILITIES, Data::OutU64),
    ("GET_SENSOR_INTERFACE", env::GET_SENSOR_INTERFACE, Data::Zeroed),
    ("GET_CAMERA_INTERFACE", env::GET_CAMERA_INTERFACE, Data::Zeroed),
    ("GET_LOG_INTERFACE", env::GET_LOG_INTERFACE, Data::Zeroed),
    ("GET_PERF_INTERFACE", env::GET_PERF_INTERFACE, Data::Zeroed),
    ("GET_LOCATION_INTERFACE", env::GET_LOCATION_INTERFACE, Data::Zeroed),
    ("GET_CORE_ASSETS_DIRECTORY", env::GET_CORE_ASSETS_DIRECTORY, Data::OutStr),
    ("GET_SAVE_DIRECTORY", env::GET_SAVE_DIRECTORY, Data::OutStr),
    ("SET_SYSTEM_AV_INFO", env::SET_SYSTEM_AV_INFO, Data::AvInfo),
    ("SET_PROC_ADDRESS_CALLBACK", env::SET_PROC_ADDRESS_CALLBACK, Data::ProcAddress),
    ("SET_SUBSYSTEM_INFO", env::SET_SUBSYSTEM_INFO, Data::Subsystem),
    ("SET_CONTROLLER_INFO", env::SET_CONTROLLER_INFO, Data::Controllers),
    ("SET_GEOMETRY", env::SET_GEOMETRY, Data::Geometry),
    ("GET_USERNAME", env::GET_USERNAME, Data::OutStr),
    ("GET_LANGUAGE", env::GET_LANGUAGE, Data::OutUint),
    ("GET_CURRENT_SOFTWARE_FRAMEBUFFER", env::GET_CURRENT_SOFTWARE_FRAMEBUFFER, Data::Framebuffer),
    ("GET_HW_RENDER_INTERFACE", env::GET_HW_RENDER_INTERFACE, Data::Zeroed),
    ("SET_SUPPORT_ACHIEVEMENTS", env::SET_SUPPORT_ACHIEVEMENTS, Data::InBool(true)),
    ("SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE", env::SET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE, Data::Zeroed),
    ("SET_SERIALIZATION_QUIRKS", env::SET_SERIALIZATION_QUIRKS, Data::InU64(0)),
    ("SET_HW_SHARED_CONTEXT", env::SET_HW_SHARED_CONTEXT, Data::Null),
    ("GET_VFS_INTERFACE", env::GET_VFS_INTERFACE, Data::Zeroed),
    ("GET_LED_INTERFACE", env::GET_LED_INTERFACE, Data::Zeroed),
    ("GET_AUDIO_VIDEO_ENABLE", env::GET_AUDIO_VIDEO_ENABLE, Data::OutInt),
    ("GET_MIDI_INTERFACE", env::GET_MIDI_INTERFACE, Data::Zeroed),
    ("GET_FASTFORWARDING", env::GET_FASTFORWARDING, Data::OutBool),
    ("GET_TARGET_REFRESH_RATE", env::GET_TARGET_REFRESH_RATE, Data::OutF32),
    ("GET_INPUT_BITMASKS", env::GET_INPUT_BITMASKS, Data::Null),
    ("GET_CORE_OPTIONS_VERSION", env::GET_CORE_OPTIONS_VERSION, Data::OutUint),
    ("SET_CORE_OPTIONS_DISPLAY", env::SET_CORE_OPTIONS_DISPLAY, Data::Display),
    ("GET_PREFERRED_HW_RENDER", env::GET_PREFERRED_HW_RENDER, Data::OutUint),
    ("GET_DISK_CONTROL_INTERFACE_VERSION", env::GET_DISK_CONTROL_INTERFACE_VERSION, Data::OutUint),
    ("SET_DISK_CONTROL_EXT_INTERFACE", env::SET_DISK_CONTROL_EXT_INTERFACE, Data::Disk),
    ("GET_MESSAGE_INTERFACE_VERSION", env::GET_MESSAGE_INTERFACE_VERSION, Data::OutUint),
    ("SET_MESSAGE_EXT", env::SET_MESSAGE_EXT, Data::MessageExt),
    ("GET_INPUT_MAX_USERS", env::GET_INPUT_MAX_USERS, Data::OutUint),
    ("SET_AUDIO_BUFFER_STATUS_CALLBACK", env::SET_AUDIO_BUFFER_STATUS_CALLBACK, Data::AudioBufferCallback),
    ("SET_MINIMUM_AUDIO_LATENCY", env::SET_MINIMUM_AUDIO_LATENCY, Data::InUint(64)),
    ("SET_FASTFORWARDING_OVERRIDE", env::SET_FASTFORWARDING_OVERRIDE, Data::Zeroed),
    ("GET_GAME_INFO_EXT", env::GET_GAME_INFO_EXT, Data::Zeroed),
    ("SET_CORE_OPTIONS_UPDATE_DISPLAY_CALLBACK", env::SET_CORE_OPTIONS_UPDATE_DISPLAY_CALLBACK, Data::UpdateDisplay),
    ("SET_VARIABLE", env::SET_VARIABLE, Data::SetVariable),
    ("GET_THROTTLE_STATE", env::GET_THROTTLE_STATE, Data::Zeroed),
    ("GET_SAVESTATE_CONTEXT", env::GET_SAVESTATE_CONTEXT, Data::OutInt),
    ("GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT", env::GET_HW_RENDER_CONTEXT_NEGOTIATION_INTERFACE_SUPPORT, Data::Zeroed),
    ("GET_JIT_CAPABLE", env::GET_JIT_CAPABLE, Data::OutBool),
    ("GET_MICROPHONE_INTERFACE", env::GET_MICROPHONE_INTERFACE, Data::Zeroed),
    ("GET_DEVICE_POWER", env::GET_DEVICE_POWER, Data::Zeroed),
    ("SET_NETPACKET_INTERFACE", env::SET_NETPACKET_INTERFACE, Data::Zeroed),
    ("GET_PLAYLIST_DIRECTORY", env::GET_PLAYLIST_DIRECTORY, Data::OutStr),
    ("GET_FILE_BROWSER_START_DIRECTORY", env::GET_FILE_BROWSER_START_DIRECTORY, Data::OutStr),
    ("GET_TARGET_SAMPLE_RATE", env::GET_TARGET_SAMPLE_RATE, Data::OutUint),
    ("GET_NETPLAY_CLIENT_INDEX", env::GET_NETPLAY_CLIENT_INDEX, Data::OutUint),
    ("EXEC_MEM_ALLOC", env::EXEC_MEM_ALLOC, Data::Zeroed),
    ("EXEC_MEM_FREE", env::EXEC_MEM_FREE, Data::Zeroed),
    ("GET_AUDIO_SAMPLE_BATCH_FLOAT", env::GET_AUDIO_SAMPLE_BATCH_FLOAT, Data::Zeroed),
    ("GET_MEMORY_STATUS", env::GET_MEMORY_STATUS, Data::Zeroed),
    ("SET_SERIALIZATION_QUIRKS_2", env::SET_SERIALIZATION_QUIRKS_2, Data::Zeroed),
    ("GET_VFS_AUTHORIZED_LOCATIONS", env::GET_VFS_AUTHORIZED_LOCATIONS, Data::Zeroed),
    ("GET_AUDIO_SAMPLE_BATCH_MULTI", env::GET_AUDIO_SAMPLE_BATCH_MULTI, Data::Zeroed),
    ("SET_VIDEO_VIEWS", env::SET_VIDEO_VIEWS, Data::Zeroed),
    ("GET_VIDEO_VIEWS_STATUS", env::GET_VIDEO_VIEWS_STATUS, Data::Zeroed),
    ("GET_VR_FRAME_STATE", env::GET_VR_FRAME_STATE, Data::Zeroed),
    ("GET_VR_HEAD_POSE", env::GET_VR_HEAD_POSE, Data::Zeroed),
    ("GET_VIDEO_DECODE_INTERFACE", env::GET_VIDEO_DECODE_INTERFACE, Data::Zeroed),
    ("GET_VR_CONTROLLER_POSES", env::GET_VR_CONTROLLER_POSES, Data::Zeroed),
    ("GET_HDR_METADATA", env::GET_HDR_METADATA, Data::Zeroed),
    ("GET_DISPLAY_METADATA", env::GET_DISPLAY_METADATA, Data::Zeroed),
    ("GET_HW_RENDER_UPSCALER_INTERFACE", env::GET_HW_RENDER_UPSCALER_INTERFACE, Data::Zeroed),
    ("an unknown command", 9_999, Data::Zeroed),
    ("an unknown experimental command", 9_998 | env::EXPERIMENTAL, Data::Zeroed),
];

unsafe extern "C" fn keyboard(_down: bool, _keycode: c_uint, _character: u32, _modifiers: u16) {}
unsafe extern "C" fn audio_callback() {}
unsafe extern "C" fn audio_state(_enabled: bool) {}
unsafe extern "C" fn buffer_status(_active: bool, _occupancy: c_uint, _underrun: bool) {}
unsafe extern "C" fn frame_time(_usec: i64) {}
unsafe extern "C" fn update_display() -> bool {
    true
}
unsafe extern "C" fn proc_address(_sym: *const c_char) -> sys::retro_proc_address_t {
    None
}

/// Calls every probe in order and gives each result.
pub fn run() -> Vec<u8> {
    PROBES.iter().map(|&(_, cmd, data)| call(cmd, data) as u8).collect()
}

fn call(cmd: c_uint, data: Data) -> bool {
    let env = crate::environment;
    let mut block = [0u64; 512];
    let zeroed = block.as_mut_ptr() as *mut c_void;
    match data {
        Data::OutBool | Data::OutUint | Data::OutInt | Data::OutU64 | Data::OutF32 | Data::OutStr | Data::Zeroed => env(cmd, zeroed),
        Data::Null => env(cmd, std::ptr::null_mut()),
        Data::InUint(v) => {
            let mut v: c_uint = v;
            env(cmd, &mut v as *mut c_uint as *mut c_void)
        }
        Data::InBool(v) => {
            let mut v = v;
            env(cmd, &mut v as *mut bool as *mut c_void)
        }
        Data::InU64(v) => {
            let mut v = v;
            env(cmd, &mut v as *mut u64 as *mut c_void)
        }
        Data::PixelFormat => {
            let mut f: c_int = crate::with_core(|c| c.format);
            env(cmd, &mut f as *mut c_int as *mut c_void)
        }
        Data::Message => {
            let m = sys::retro_message { msg: c"a message from the probe".as_ptr(), frames: 60 };
            env(cmd, &m as *const _ as *mut c_void)
        }
        Data::MessageExt => {
            let m = sys::retro_message_ext { msg: c"an extended message from the probe".as_ptr(), duration: 1000, priority: 1, level: sys::LOG_INFO, target: 0, kind: 0, progress: -1 };
            env(cmd, &m as *const _ as *mut c_void)
        }
        Data::Descriptors => {
            let d = [
                sys::retro_input_descriptor { port: 0, device: sys::DEVICE_JOYPAD, index: 0, id: 0, description: c"Cross".as_ptr() },
                sys::retro_input_descriptor { port: 0, device: 0, index: 0, id: 0, description: std::ptr::null() },
            ];
            env(cmd, d.as_ptr() as *mut c_void)
        }
        Data::Keyboard => {
            let k = sys::retro_keyboard_callback { callback: Some(keyboard) };
            env(cmd, &k as *const _ as *mut c_void)
        }
        Data::FrameTime => {
            let f = sys::retro_frame_time_callback { callback: Some(frame_time), reference: 16_666 };
            let answered = env(cmd, &f as *const _ as *mut c_void);
            let mut again = sys::retro_frame_time_callback { callback: Some(super::frame_time), reference: 1_000_000 / 60 };
            env(cmd, &mut again as *mut _ as *mut c_void);
            answered
        }
        Data::AudioCallback => {
            let a = sys::retro_audio_callback { callback: Some(audio_callback), set_state: Some(audio_state) };
            env(cmd, &a as *const _ as *mut c_void)
        }
        Data::AudioBufferCallback => {
            let a = sys::retro_audio_buffer_status_callback { callback: Some(buffer_status) };
            env(cmd, &a as *const _ as *mut c_void)
        }
        Data::ProcAddress => {
            let p = sys::retro_get_proc_address_interface { get_proc_address: Some(proc_address) };
            env(cmd, &p as *const _ as *mut c_void)
        }
        Data::Subsystem => env(cmd, zeroed),
        Data::Controllers => {
            let types = [sys::retro_controller_description { desc: c"RetroPad".as_ptr(), id: sys::DEVICE_JOYPAD }];
            let ports = [sys::retro_controller_info { types: types.as_ptr(), num_types: 1 }, sys::retro_controller_info { types: std::ptr::null(), num_types: 0 }];
            env(cmd, ports.as_ptr() as *mut c_void)
        }
        Data::Geometry => {
            let g = sys::retro_game_geometry { base_width: BASE_WIDTH, base_height: BASE_HEIGHT, max_width: MAX_WIDTH, max_height: MAX_HEIGHT, aspect_ratio: 4.0 / 3.0 };
            env(cmd, &g as *const _ as *mut c_void)
        }
        Data::AvInfo => {
            let av = crate::with_core(|c| Some(crate::av_info(c))).unwrap();
            env(cmd, &av as *const _ as *mut c_void)
        }
        Data::Display => {
            let d = sys::retro_core_option_display { key: c"tc_note".as_ptr(), visible: true };
            env(cmd, &d as *const _ as *mut c_void)
        }
        Data::UpdateDisplay => {
            let u = sys::retro_core_options_update_display_callback { callback: Some(update_display) };
            env(cmd, &u as *const _ as *mut c_void)
        }
        Data::Variable => {
            let mut v = sys::retro_variable { key: c"tc_format".as_ptr(), value: std::ptr::null() };
            env(cmd, &mut v as *mut _ as *mut c_void) && !v.value.is_null()
        }
        Data::SetVariable => {
            let v = sys::retro_variable { key: c"tc_note".as_ptr(), value: c"plain".as_ptr() };
            env(cmd, &v as *const _ as *mut c_void)
        }
        Data::HwRenderGl => {
            let mut h = sys::retro_hw_render_callback {
                context_type: 1,
                context_reset: None,
                get_current_framebuffer: None,
                get_proc_address: None,
                depth: false,
                stencil: false,
                bottom_left_origin: true,
                version_major: 3,
                version_minor: 3,
                cache_context: false,
                context_destroy: None,
                debug_context: false,
            };
            env(cmd, &mut h as *mut _ as *mut c_void)
        }
        Data::Framebuffer => {
            let mut fb = sys::retro_framebuffer { data: std::ptr::null_mut(), width: BASE_WIDTH, height: BASE_HEIGHT, pitch: 0, format: 0, access_flags: sys::MEMORY_ACCESS_WRITE, memory_flags: 0 };
            env(cmd, &mut fb as *mut _ as *mut c_void) && !fb.data.is_null()
        }
        Data::Disk => env(cmd, &super::DISK as *const _ as *mut c_void),
    }
}
