//! The few calls of Qt's window and application the program makes that cxx-qt-lib does not bind.

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++Qt" {
        include!(<QtQuick/QQuickWindow>);
        #[qobject]
        #[base = QObject]
        type QQuickWindow;

        fn show(self: Pin<&mut QQuickWindow>);
        #[cxx_name = "showFullScreen"]
        fn show_full_screen(self: Pin<&mut QQuickWindow>);

        #[qsignal]
        #[cxx_name = "frameSwapped"]
        fn frame_swapped(self: Pin<&mut QQuickWindow>);
    }

    unsafe extern "C++" {
        include!(<QtGui/QGuiApplication>);
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        type QGuiApplication;
        #[Self = "QGuiApplication"]
        #[cxx_name = "platformName"]
        fn platform_name() -> QString;
    }

    unsafe extern "C++" {
        include!(<QtCore/QCoreApplication>);
        type QCoreApplication;
        #[Self = "QCoreApplication"]
        fn quit();
    }
}
