//! The program's one picture: its window's first swapped frame, saved and the program ended, so a bundle is checked by what it draws (EmuSen_Qt.md §9.2, EmuSen_Kit.md §7.13).

/// Qt's `QImage` under a name of this crate's, so cxx binds `QImage::save` on it, which cxx-qt-lib's `QImage` lacks.
#[repr(transparent)]
pub struct Picture(cxx_qt_lib::QImage);

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
