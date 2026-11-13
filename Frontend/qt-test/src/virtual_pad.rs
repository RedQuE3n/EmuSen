//! SDL's own virtual joysticks, as SdlVirtualPadsTests attaches them, for tests whose pads are read by Endymion through a
//! `PadReader` shown these alone (EmuSen_Kit.md §7.15); no pad on the desk is opened.

use std::ffi::{c_char, c_void, CString};
use std::sync::atomic::{AtomicU32, Ordering};

/// SDL_VirtualJoystickDesc (SDL_joystick.h, SDL 3.4), 136 bytes on a 64-bit target.
#[repr(C)]
struct VirtualJoystickDesc {
    version: u32,
    kind: u16,
    padding: u16,
    vendor_id: u16,
    product_id: u16,
    naxes: u16,
    nbuttons: u16,
    nballs: u16,
    nhats: u16,
    ntouchpads: u16,
    nsensors: u16,
    padding2: [u16; 2],
    button_mask: u32,
    axis_mask: u32,
    name: *const c_char,
    touchpads: *const c_void,
    sensors: *const c_void,
    userdata: *mut c_void,
    callbacks: [*const c_void; 8],
}

const SDL_INIT_GAMEPAD: u32 = 0x2000;
const SDL_JOYSTICK_TYPE_GAMEPAD: u16 = 1;
const GAMEPAD_AXES: u16 = 6;
const GAMEPAD_BUTTONS: u16 = 26;

/// SDL3, the same library the platform library is lent, with the calls that attach, press, move and detach a virtual pad.
pub struct Sdl {
    lib: libloading::Library,
}

/// A virtual pad: its id, the joystick the test opened to press it, and the motors' last levels SDL gave it.
pub struct VirtualPad {
    pub id: u32,
    joystick: *mut c_void,
    _name: CString,
    motors: &'static AtomicU32,
}

impl VirtualPad {
    /// The levels of the last rumble SDL sent the pad, its low-frequency (strong) motor then its high (weak).
    pub fn rumble(&self) -> (u16, u16) {
        let both = self.motors.load(Ordering::Acquire);
        ((both >> 16) as u16, both as u16)
    }
}

/// SDL_VirtualJoystickDesc's Rumble: the levels kept where the pad's userdata points.
extern "C" fn rumbled(userdata: *mut c_void, low: u16, high: u16) -> bool {
    // SAFETY: userdata is the pad's `motors`, leaked, so alive as long as SDL may call.
    let motors = unsafe { &*(userdata as *const AtomicU32) };
    motors.store(u32::from(low) << 16 | u32::from(high), Ordering::Release);
    true
}

// SDL takes its joystick lock in every call on a joystick, so a pad may be pressed from another thread than its own.
unsafe impl Send for VirtualPad {}

impl Sdl {
    pub fn open() -> Sdl {
        std::env::set_var("SDL_AUDIODRIVER", "dummy");
        let lib = unsafe { libloading::Library::new("libSDL3.so.0") }.expect("SDL3");
        let sdl = Sdl { lib };
        let init: libloading::Symbol<unsafe extern "C" fn(u32) -> bool> = unsafe { sdl.lib.get(b"SDL_InitSubSystem") }.unwrap();
        assert!(unsafe { init(SDL_INIT_GAMEPAD) }, "SDL's gamepad subsystem");
        sdl
    }

    pub fn attach(&self, name: &str, vendor_id: u16, product_id: u16) -> VirtualPad {
        let name = CString::new(name).unwrap();
        let motors: &'static AtomicU32 = Box::leak(Box::new(AtomicU32::new(0)));
        let mut callbacks = [std::ptr::null(); 8];
        // Update, SetPlayerIndex, then Rumble.
        callbacks[2] = rumbled as *const c_void;
        let desc = VirtualJoystickDesc {
            version: std::mem::size_of::<VirtualJoystickDesc>() as u32,
            kind: SDL_JOYSTICK_TYPE_GAMEPAD,
            padding: 0,
            vendor_id,
            product_id,
            naxes: GAMEPAD_AXES,
            nbuttons: GAMEPAD_BUTTONS,
            nballs: 0,
            nhats: 0,
            ntouchpads: 0,
            nsensors: 0,
            padding2: [0; 2],
            button_mask: 0,
            axis_mask: 0,
            name: name.as_ptr(),
            touchpads: std::ptr::null(),
            sensors: std::ptr::null(),
            userdata: motors as *const AtomicU32 as *mut c_void,
            callbacks,
        };
        unsafe {
            let attach: libloading::Symbol<unsafe extern "C" fn(*const VirtualJoystickDesc) -> u32> =
                self.lib.get(b"SDL_AttachVirtualJoystick").unwrap();
            let open: libloading::Symbol<unsafe extern "C" fn(u32) -> *mut c_void> = self.lib.get(b"SDL_OpenJoystick").unwrap();
            let id = attach(&desc);
            assert_ne!(id, 0, "a virtual pad attached");
            let joystick = open(id);
            assert!(!joystick.is_null(), "the virtual pad opened");
            VirtualPad { id, joystick, _name: name, motors }
        }
    }

    pub fn press(&self, pad: &VirtualPad, button: i32, down: bool) {
        let set: libloading::Symbol<unsafe extern "C" fn(*mut c_void, i32, bool) -> bool> =
            unsafe { self.lib.get(b"SDL_SetJoystickVirtualButton") }.unwrap();
        assert!(unsafe { set(pad.joystick, button, down) });
    }

    /// An axis of the pad moved, SDL's sixteen bits (the sticks -32768 to 32767, the triggers 0 to 32767).
    pub fn axis(&self, pad: &VirtualPad, axis: i32, value: i16) {
        let set: libloading::Symbol<unsafe extern "C" fn(*mut c_void, i32, i16) -> bool> =
            unsafe { self.lib.get(b"SDL_SetJoystickVirtualAxis") }.unwrap();
        assert!(unsafe { set(pad.joystick, axis, value) });
    }

    pub fn detach(&self, pad: VirtualPad) {
        unsafe {
            let close: libloading::Symbol<unsafe extern "C" fn(*mut c_void)> = self.lib.get(b"SDL_CloseJoystick").unwrap();
            let detach: libloading::Symbol<unsafe extern "C" fn(u32) -> bool> = self.lib.get(b"SDL_DetachVirtualJoystick").unwrap();
            close(pad.joystick);
            assert!(detach(pad.id));
        }
    }
}
