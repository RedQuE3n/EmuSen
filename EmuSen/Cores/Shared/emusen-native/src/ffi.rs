//! A core's C ABI for its state: the status codes this crate owns, the helpers every export uses, and the four exports as a macro.

/// The codes -1 to -8, shared by every core; a core's own are `FIRST_CORE` to `LAST_CORE`, and below them is the interface's (EmuSen_NativeCores.md §3.3).
pub mod status {
    /// A null handle or buffer.
    pub const NULL: i32 = -1;
    /// The input ended before a field.
    pub const TRUNCATED: i32 = -2;
    /// Another core's state, or none: the magic is not this core's.
    pub const FOREIGN: i32 = -3;
    /// A version this core does not read.
    pub const VERSION: i32 = -4;
    /// A string length `BinaryReader` refuses.
    pub const BAD_STRING: i32 = -5;
    /// The caller's buffer is shorter than the state.
    pub const BUFFER_TOO_SMALL: i32 = -7;
    /// -6 and -8 are reserved; the first code a core may give its own refusals.
    pub const FIRST_CORE: i32 = -9;
    /// The last code of a core's own band.
    pub const LAST_CORE: i32 = -255;
}

/// An error that crosses the C ABI as a negative status.
pub trait Status {
    fn status(&self) -> i32;
}

/// A count, or the error's status.
pub fn result<E: Status>(result: Result<usize, E>) -> i64 {
    match result {
        Ok(n) => n as i64,
        Err(e) => e.status() as i64,
    }
}

/// The bytes a caller passed.
///
/// # Safety
/// `data` must be valid for `len` bytes, or null.
pub unsafe fn input<'a>(data: *const u8, len: usize) -> &'a [u8] {
    if data.is_null() || len == 0 { &[] } else { unsafe { std::slice::from_raw_parts(data, len) } }
}

/// `text` copied up to `len` bytes into `out`, which may be null; its whole length.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
pub unsafe fn copy_text(text: &str, out: *mut u8, len: usize) -> i64 {
    if !out.is_null() {
        let n = text.len().min(len);
        unsafe { std::ptr::copy_nonoverlapping(text.as_ptr(), out, n) };
    }
    text.len() as i64
}

/// A machine whose whole state is one `State` walk.
pub trait StateMachine {
    type Error: Status;
    /// The state's fields; a failed load changes nothing.
    fn load_state(&mut self, data: &[u8]) -> Result<(), Self::Error>;
    fn state_size(&self) -> usize;
    /// The bytes written.
    fn save_state(&self, out: &mut [u8]) -> Result<usize, Self::Error>;
    /// One line per field, `offset length type path`.
    fn layout(&self) -> String;
}

/// The value returned across an `extern "C"` boundary when a body panics.
pub trait PanicFallback {
    fn panic_fallback() -> Self;
}

impl PanicFallback for i32 {
    fn panic_fallback() -> Self {
        crate::abi::status::FAULT_BASE
    }
}

impl PanicFallback for i64 {
    fn panic_fallback() -> Self {
        crate::abi::status::FAULT_BASE as i64
    }
}

impl PanicFallback for u32 {
    fn panic_fallback() -> Self {
        crate::core::sys::ABI_VERSION
    }
}

impl PanicFallback for u64 {
    fn panic_fallback() -> Self {
        0
    }
}

impl PanicFallback for *mut crate::core::sys::Machine {
    fn panic_fallback() -> Self {
        std::ptr::null_mut()
    }
}

/// Extracts a readable panic description from a caught unwind payload.
pub fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_owned()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "core panicked".to_owned()
    }
}

/// Catches any panic before it unwinds across an `extern "C"` boundary.
pub fn guard<R: PanicFallback>(f: impl FnOnce() -> R) -> R {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(v) => v,
        Err(payload) => {
            crate::core::outbox::library_log().push(crate::core::Level::Error, "panic", &panic_message(payload));
            R::panic_fallback()
        }
    }
}

/// The four state exports over a `StateMachine`, under the names given: load, size, save, layout.
#[macro_export]
macro_rules! state_exports {
    ($machine:ty, $load:ident, $size:ident, $save:ident, $layout:ident) => {
        /// The state's fields; zero, or a negative status, and a failed load changes nothing.
        ///
        /// # Safety
        /// `machine` must be live or null; `data` valid for `len` bytes.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $load(machine: *mut $machine, data: *const u8, len: usize) -> i32 {
            $crate::ffi::guard(|| {
                let Some(m) = (unsafe { machine.as_mut() }) else { return $crate::ffi::status::NULL };
                match $crate::ffi::StateMachine::load_state(m, unsafe { $crate::ffi::input(data, len) }) {
                    Ok(()) => 0,
                    Err(e) => $crate::ffi::Status::status(&e),
                }
            })
        }

        /// # Safety
        /// `machine` must be live or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $size(machine: *const $machine) -> i64 {
            $crate::ffi::guard(|| unsafe { machine.as_ref() }.map_or($crate::ffi::status::NULL as i64, |m| $crate::ffi::StateMachine::state_size(m) as i64))
        }

        /// The bytes written, or a negative status.
        ///
        /// # Safety
        /// `machine` must be live or null; `out` valid for `len` bytes.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $save(machine: *const $machine, out: *mut u8, len: usize) -> i64 {
            $crate::ffi::guard(|| {
                let Some(m) = (unsafe { machine.as_ref() }) else { return $crate::ffi::status::NULL as i64 };
                if out.is_null() {
                    return $crate::ffi::status::NULL as i64;
                }
                $crate::ffi::result($crate::ffi::StateMachine::save_state(m, unsafe { std::slice::from_raw_parts_mut(out, len) }))
            })
        }

        /// The state's layout as UTF-8 text, copied up to `len` bytes; returns its whole length, or a negative status.
        ///
        /// # Safety
        /// `machine` must be live or null; `out` valid for `len` bytes, or null.
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn $layout(machine: *const $machine, out: *mut u8, len: usize) -> i64 {
            $crate::ffi::guard(|| {
                let Some(m) = (unsafe { machine.as_ref() }) else { return $crate::ffi::status::NULL as i64 };
                unsafe { $crate::ffi::copy_text(&$crate::ffi::StateMachine::layout(m), out, len) }
            })
        }
    };
}
