//! SDL3, loaded at run time from the file the program ships, through the functions the device half calls and no others.
//!
//! A host that has SDL loaded already hands over its handle, so both sides of one process share one SDL: its
//! subsystems' counts, its hints and its devices. See EmuSen_RustPlatform.md §2.8 and §14.

use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::mem::ManuallyDrop;

/// `SDL_AudioSpec`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AudioSpec {
    pub format: i32,
    pub channels: i32,
    pub freq: i32,
}

/// `SDL_GUID`, sixteen bytes returned by value.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Guid {
    pub data: [u8; 16],
}

pub const INIT_AUDIO: u32 = 0x10;
pub const INIT_GAMEPAD: u32 = 0x2000;
pub const AUDIO_S16LE: i32 = 0x8010;
pub const AUDIO_S16BE: i32 = 0x9010;
pub const AUDIO_F32LE: i32 = 0x8120;
pub const AUDIO_DEVICE_DEFAULT_PLAYBACK: u32 = 0xFFFF_FFFF;
pub const EVENT_GAMEPAD_ADDED: u32 = 0x653;
pub const EVENT_GAMEPAD_REMOVED: u32 = 0x654;
pub const HINT_AUDIO_DEVICE_SAMPLE_FRAMES: &str = "SDL_AUDIO_DEVICE_SAMPLE_FRAMES";

macro_rules! functions {
    ($($field:ident = $name:literal: fn($($arg:ty),*) $(-> $ret:ty)?;)*) => {
        /// The SDL functions Endymion calls, by their C names.
        #[allow(clippy::type_complexity)]
        pub struct Functions {
            $(pub $field: unsafe extern "C" fn($($arg),*) $(-> $ret)?,)*
        }

        /// The names looked up, for the export check and for a report of what a library lacks.
        pub const NAMES: &[&str] = &[$($name),*];

        impl Functions {
            /// # Safety
            /// `library` must be SDL3, so that each name has the signature declared for it.
            unsafe fn load(library: &libloading::Library) -> Result<Functions, String> {
                Ok(Functions {
                    $($field: *unsafe { library.get::<unsafe extern "C" fn($($arg),*) $(-> $ret)?>($name) }
                        .map_err(|_| format!("SDL3 has no {}", $name))?,)*
                })
            }
        }
    };
}

functions! {
    init_sub_system = "SDL_InitSubSystem": fn(u32) -> bool;
    quit_sub_system = "SDL_QuitSubSystem": fn(u32);
    set_hint = "SDL_SetHint": fn(*const c_char, *const c_char) -> bool;
    get_hint = "SDL_GetHint": fn(*const c_char) -> *const c_char;
    free = "SDL_free": fn(*mut c_void);
    open_audio_device_stream = "SDL_OpenAudioDeviceStream": fn(u32, *const AudioSpec, *const c_void, *mut c_void) -> *mut c_void;
    destroy_audio_stream = "SDL_DestroyAudioStream": fn(*mut c_void);
    get_audio_stream_queued = "SDL_GetAudioStreamQueued": fn(*mut c_void) -> c_int;
    put_audio_stream_data = "SDL_PutAudioStreamData": fn(*mut c_void, *const c_void, c_int) -> bool;
    set_audio_stream_gain = "SDL_SetAudioStreamGain": fn(*mut c_void, f32) -> bool;
    resume_audio_stream_device = "SDL_ResumeAudioStreamDevice": fn(*mut c_void) -> bool;
    clear_audio_stream = "SDL_ClearAudioStream": fn(*mut c_void) -> bool;
    load_wav = "SDL_LoadWAV": fn(*const c_char, *mut AudioSpec, *mut *mut u8, *mut u32) -> bool;
    convert_audio_samples = "SDL_ConvertAudioSamples": fn(*const AudioSpec, *const u8, c_int, *const AudioSpec, *mut *mut u8, *mut c_int) -> bool;
    get_gamepads = "SDL_GetGamepads": fn(*mut c_int) -> *mut u32;
    open_gamepad = "SDL_OpenGamepad": fn(u32) -> *mut c_void;
    close_gamepad = "SDL_CloseGamepad": fn(*mut c_void);
    gamepad_connected = "SDL_GamepadConnected": fn(*mut c_void) -> bool;
    has_events = "SDL_HasEvents": fn(u32, u32) -> bool;
    flush_events = "SDL_FlushEvents": fn(u32, u32);
    update_gamepads = "SDL_UpdateGamepads": fn();
    get_gamepad_button = "SDL_GetGamepadButton": fn(*mut c_void, c_int) -> bool;
    get_gamepad_axis = "SDL_GetGamepadAxis": fn(*mut c_void, c_int) -> i16;
    get_gamepad_name = "SDL_GetGamepadName": fn(*mut c_void) -> *const c_char;
    get_gamepad_type = "SDL_GetGamepadType": fn(*mut c_void) -> c_int;
    get_gamepad_button_label = "SDL_GetGamepadButtonLabel": fn(*mut c_void, c_int) -> c_int;
    get_gamepad_joystick = "SDL_GetGamepadJoystick": fn(*mut c_void) -> *mut c_void;
    get_joystick_guid = "SDL_GetJoystickGUID": fn(*mut c_void) -> Guid;
    get_gamepad_path = "SDL_GetGamepadPath": fn(*mut c_void) -> *const c_char;
    set_gamepad_player_index = "SDL_SetGamepadPlayerIndex": fn(*mut c_void, c_int) -> bool;
}

/// SDL3 and its functions; the library stays loaded while this lives, and one the host lent is never closed here.
pub struct Sdl {
    pub f: Functions,
    _library: ManuallyDrop<libloading::Library>,
    owned: bool,
}

// SDL's functions are its own to make thread-safe, as they are for the C# that calls the same ones.
unsafe impl Send for Sdl {}
unsafe impl Sync for Sdl {}

impl Sdl {
    /// SDL3 from a file.
    pub fn open(path: &str) -> Result<Sdl, String> {
        let library = unsafe { libloading::Library::new(path) }.map_err(|e| format!("{path} could not be loaded: {e}"))?;
        let f = unsafe { Functions::load(&library) }?;
        Ok(Sdl { f, _library: ManuallyDrop::new(library), owned: true })
    }

    /// SDL3 as the host already has it loaded, by the handle its loader returned.
    ///
    /// # Safety
    /// `handle` must be a live handle to SDL3 from the platform's loader (`dlopen`, `LoadLibrary`), which stays loaded
    /// for as long as this lives.
    pub unsafe fn lent(handle: *mut c_void) -> Result<Sdl, String> {
        if handle.is_null() {
            return Err("no SDL3 handle".to_string());
        }
        #[cfg(unix)]
        let library: libloading::Library = unsafe { libloading::os::unix::Library::from_raw(handle) }.into();
        #[cfg(windows)]
        let library: libloading::Library = unsafe { libloading::os::windows::Library::from_raw(handle as isize) }.into();
        let f = unsafe { Functions::load(&library) }?;
        Ok(Sdl { f, _library: ManuallyDrop::new(library), owned: false })
    }

    pub fn set_hint(&self, name: &str, value: &str) -> bool {
        match (CString::new(name), CString::new(value)) {
            (Ok(name), Ok(value)) => unsafe { (self.f.set_hint)(name.as_ptr(), value.as_ptr()) },
            _ => false,
        }
    }

    pub fn hint(&self, name: &str) -> Option<String> {
        let name = CString::new(name).ok()?;
        unsafe { text((self.f.get_hint)(name.as_ptr())) }
    }
}

impl Drop for Sdl {
    fn drop(&mut self) {
        if self.owned {
            unsafe { ManuallyDrop::drop(&mut self._library) };
        }
    }
}

/// A C string SDL returned, decoded as the C# marshaller decodes it: UTF-8, with each bad sequence replaced. None for null.
///
/// # Safety
/// `text` must be null or a NUL-terminated string that lives for the call.
pub unsafe fn text(text: *const c_char) -> Option<String> {
    if text.is_null() {
        return None;
    }
    Some(String::from_utf8_lossy(unsafe { CStr::from_ptr(text) }.to_bytes()).into_owned())
}
