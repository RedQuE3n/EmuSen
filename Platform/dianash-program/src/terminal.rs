//! The terminal's raw mode for the line editor, and Ctrl-C while a command runs: termios on Linux and macOS, the
//! console API with virtual-terminal sequences on Windows. See EmuSen_DianaSh.md §12.3.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};

/// The running line's cancellation flag, which Ctrl-C sets.
static CANCEL: OnceLock<Arc<AtomicBool>> = OnceLock::new();

#[cfg(unix)]
mod system {
    use std::io::IsTerminal;

    pub fn is_terminal() -> bool {
        std::io::stdin().is_terminal()
    }

    /// The terminal's settings before raw mode, put back when the guard drops.
    pub struct Raw {
        saved: libc::termios,
    }

    impl Raw {
        pub fn enter() -> Option<Raw> {
            let mut saved: libc::termios = unsafe { std::mem::zeroed() };
            if unsafe { libc::tcgetattr(libc::STDIN_FILENO, &mut saved) } != 0 {
                return None;
            }
            let mut raw = saved;
            raw.c_lflag &= !(libc::ICANON | libc::ECHO | libc::ISIG | libc::IEXTEN);
            raw.c_iflag &= !(libc::IXON | libc::ICRNL);
            raw.c_cc[libc::VMIN] = 1;
            raw.c_cc[libc::VTIME] = 0;
            if unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &raw) } != 0 {
                return None;
            }
            Some(Raw { saved })
        }
    }

    impl Drop for Raw {
        fn drop(&mut self) {
            unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &self.saved) };
        }
    }

    /// One byte of input, or none at the end of input.
    pub fn read_byte() -> Option<u8> {
        let mut byte = 0u8;
        loop {
            let n = unsafe { libc::read(libc::STDIN_FILENO, (&mut byte as *mut u8).cast(), 1) };
            if n == 1 {
                return Some(byte);
            }
            if n == 0 || std::io::Error::last_os_error().kind() != std::io::ErrorKind::Interrupted {
                return None;
            }
        }
    }

    /// Whether a byte arrives within `millis`, which tells a lone Escape from the start of a key's sequence.
    pub fn byte_waiting(millis: i32) -> bool {
        let mut poll = libc::pollfd { fd: libc::STDIN_FILENO, events: libc::POLLIN, revents: 0 };
        unsafe { libc::poll(&mut poll, 1, millis) > 0 }
    }

    extern "C" fn on_interrupt(_: libc::c_int) {
        super::interrupt();
    }

    pub fn install_interrupt() {
        unsafe { libc::signal(libc::SIGINT, on_interrupt as *const () as libc::sighandler_t) };
    }

    pub fn utf8_console() {}

    /// Whether standard output is a terminal too, as a full-screen program needs.
    pub fn output_is_terminal() -> bool {
        std::io::stdout().is_terminal()
    }

    /// The terminal's columns and rows, where it says.
    pub fn size() -> Option<(usize, usize)> {
        let mut size: libc::winsize = unsafe { std::mem::zeroed() };
        (unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut size) } == 0 && size.ws_col > 0).then_some((usize::from(size.ws_col), usize::from(size.ws_row)))
    }
}

#[cfg(windows)]
mod system {
    #![allow(non_snake_case)]
    use std::ffi::c_void;
    use std::io::IsTerminal;

    const STD_INPUT_HANDLE: u32 = -10i32 as u32;
    const ENABLE_PROCESSED_INPUT: u32 = 0x1;
    const ENABLE_LINE_INPUT: u32 = 0x2;
    const ENABLE_ECHO_INPUT: u32 = 0x4;
    const ENABLE_VIRTUAL_TERMINAL_INPUT: u32 = 0x200;
    const WAIT_OBJECT_0: u32 = 0;
    const CP_UTF8: u32 = 65001;

    unsafe extern "system" {
        fn GetStdHandle(which: u32) -> *mut c_void;
        fn GetConsoleMode(handle: *mut c_void, mode: *mut u32) -> i32;
        fn SetConsoleMode(handle: *mut c_void, mode: u32) -> i32;
        fn ReadFile(handle: *mut c_void, buffer: *mut u8, len: u32, read: *mut u32, overlapped: *mut c_void) -> i32;
        fn WaitForSingleObject(handle: *mut c_void, millis: u32) -> u32;
        fn SetConsoleCP(page: u32) -> i32;
        fn SetConsoleOutputCP(page: u32) -> i32;
        fn SetConsoleCtrlHandler(handler: Option<unsafe extern "system" fn(u32) -> i32>, add: i32) -> i32;
    }

    pub fn is_terminal() -> bool {
        std::io::stdin().is_terminal()
    }

    pub struct Raw {
        saved: u32,
    }

    impl Raw {
        pub fn enter() -> Option<Raw> {
            let input = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
            let mut saved = 0;
            if unsafe { GetConsoleMode(input, &mut saved) } == 0 {
                return None;
            }
            let raw = (saved & !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT)) | ENABLE_VIRTUAL_TERMINAL_INPUT;
            if unsafe { SetConsoleMode(input, raw) } == 0 {
                return None;
            }
            Some(Raw { saved })
        }
    }

    impl Drop for Raw {
        fn drop(&mut self) {
            unsafe { SetConsoleMode(GetStdHandle(STD_INPUT_HANDLE), self.saved) };
        }
    }

    pub fn read_byte() -> Option<u8> {
        let mut byte = 0u8;
        let mut read = 0u32;
        let ok = unsafe { ReadFile(GetStdHandle(STD_INPUT_HANDLE), &mut byte, 1, &mut read, std::ptr::null_mut()) };
        (ok != 0 && read == 1).then_some(byte)
    }

    pub fn byte_waiting(millis: i32) -> bool {
        unsafe { WaitForSingleObject(GetStdHandle(STD_INPUT_HANDLE), millis as u32) == WAIT_OBJECT_0 }
    }

    unsafe extern "system" fn on_control(_: u32) -> i32 {
        super::interrupt();
        1
    }

    pub fn install_interrupt() {
        unsafe { SetConsoleCtrlHandler(Some(on_control), 1) };
    }

    /// The console's code page set to UTF-8 for the shell's life (§12.3).
    pub fn utf8_console() {
        unsafe {
            SetConsoleCP(CP_UTF8);
            SetConsoleOutputCP(CP_UTF8);
        }
    }

    const STD_OUTPUT_HANDLE: u32 = -11i32 as u32;
    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x4;

    #[repr(C)]
    struct ScreenBufferInfo {
        size: [i16; 2],
        cursor: [i16; 2],
        attributes: u16,
        window: [i16; 4],
        maximum: [i16; 2],
    }

    unsafe extern "system" {
        fn GetConsoleScreenBufferInfo(handle: *mut c_void, info: *mut ScreenBufferInfo) -> i32;
    }

    /// Whether standard output is a console too, as a full-screen program needs; it then reads escape sequences.
    pub fn output_is_terminal() -> bool {
        if !std::io::stdout().is_terminal() {
            return false;
        }
        let output = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
        let mut mode = 0;
        unsafe { GetConsoleMode(output, &mut mode) != 0 && SetConsoleMode(output, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING) != 0 }
    }

    /// The console window's columns and rows.
    pub fn size() -> Option<(usize, usize)> {
        let mut info: ScreenBufferInfo = unsafe { std::mem::zeroed() };
        if unsafe { GetConsoleScreenBufferInfo(GetStdHandle(STD_OUTPUT_HANDLE), &mut info) } == 0 {
            return None;
        }
        let [left, top, right, bottom] = info.window;
        Some(((right - left + 1).max(1) as usize, (bottom - top + 1).max(1) as usize))
    }
}

pub use system::{byte_waiting, is_terminal, output_is_terminal, read_byte, size, Raw};

fn interrupt() {
    if let Some(cancel) = CANCEL.get() {
        cancel.store(true, Ordering::Relaxed);
    }
}

/// Ctrl-C while a line runs stops it at its next check, as `dianash_cancel` does (§10.4).
pub fn on_interrupt_cancel(cancel: Arc<AtomicBool>) {
    let _ = CANCEL.set(cancel);
    system::install_interrupt();
    system::utf8_console();
}
