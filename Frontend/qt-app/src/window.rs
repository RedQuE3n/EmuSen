//! How the program's window is first shown, as the start chose (`start::FULL_SCREEN`; EmuSen_Qt.md §7.7.H7).
//! `QWindow::showFullScreen` sets the full-screen state and then shows the window, so a Wayland compositor is asked for it
//! (xdg_toplevel `set_fullscreen`) before the first frame; what the compositor gives is read back as `visibility`.

use emusen_qt_play::window::ffi::QQuickWindow;
use std::pin::Pin;

/// Shows the program's window: full screen, else in a window.
pub fn show(window: Pin<&mut QQuickWindow>, full_screen: bool) {
    if full_screen {
        window.show_full_screen();
    } else {
        window.show();
    }
}
