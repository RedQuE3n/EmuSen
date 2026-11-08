//! The program's one picture: its window's first swapped frame that is the window's own size, saved and the program ended, so a bundle is checked by what it draws (EmuSen_Qt.md §9.2, EmuSen_Kit.md §7.13).

/// Qt's `QImage` under a name of this crate's, so cxx binds `QImage::save` on it, which cxx-qt-lib's `QImage` lacks.
#[repr(transparent)]
pub struct Picture(cxx_qt_lib::QImage);

impl Picture {
    /// The picture's size in pixels.
    pub fn pixels(&self) -> (i32, i32) {
        (self.0.width(), self.0.height())
    }
}

/// How many frames the grab waits for one of the window's own size before it saves what it has: five seconds at 60 Hz.
pub const FRAMES: u32 = 300;

/// Whether a frame is the window's own: its pixels the window's size at its ratio, as Qt rounds them; a frame of
/// another size was drawn while a resize the window manager made had not yet reached Qt (EmuSen_Kit.md §7.13).
pub fn settled(frame: (i32, i32), window: (i32, i32), ratio: f64) -> bool {
    let near = |pixels: i32, size: i32| (f64::from(pixels) - f64::from(size) * ratio).abs() <= 1.0;
    near(frame.0, window.0) && near(frame.1, window.1)
}

// SAFETY: the same C++ type as cxx-qt-lib's `QImage`, which is trivially relocatable and wraps it with no other field.
unsafe impl cxx::ExternType for Picture {
    type Id = cxx::type_id!("QImage");
    type Kind = cxx::kind::Trivial;
}

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qimage.h");
        type QImage = super::Picture;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;

        /// `QImage::save(fileName, format, quality)`, the format taken from the name when none is given.
        ///
        /// # Safety
        /// `format` is null or a NUL-terminated string that outlives the call.
        unsafe fn save(self: &QImage, file: &QString, format: *const c_char, quality: i32) -> bool;
    }

    unsafe extern "C++Qt" {
        include!(<QtQuick/QQuickWindow>);
        #[qobject]
        #[base = QObject]
        type QQuickWindow;

        #[cxx_name = "grabWindow"]
        fn grab_window(self: Pin<&mut QQuickWindow>) -> QImage;

        /// Asks for another frame.
        fn update(self: Pin<&mut QQuickWindow>);

        fn width(self: &QQuickWindow) -> i32;

        fn height(self: &QQuickWindow) -> i32;

        #[cxx_name = "devicePixelRatio"]
        fn device_pixel_ratio(self: &QQuickWindow) -> f64;

        #[qsignal]
        #[cxx_name = "frameSwapped"]
        fn frame_swapped(self: Pin<&mut QQuickWindow>);
    }

    unsafe extern "C++" {
        include!(<QtCore/QCoreApplication>);
        type QCoreApplication;
        #[Self = "QCoreApplication"]
        fn quit();
    }
}

#[cfg(test)]
mod tests {
    use super::settled;

    #[test]
    fn a_frame_is_the_window_s_own_at_its_size_and_ratio() {
        assert!(settled((1920, 1200), (1920, 1200), 1.0));
        assert!(settled((1920, 1200), (1280, 800), 1.5), "a window at a ratio of 1.5");
        assert!(settled((2400, 1349), (1920, 1079), 1.25), "Qt rounds a fractional ratio");
        assert!(!settled((1920, 1200), (1280, 800), 1.0), "the swapchain resized by the window manager before Qt was told");
        assert!(!settled((1280, 800), (1920, 1200), 1.0), "a frame drawn before the resize");
    }
}
