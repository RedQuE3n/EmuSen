//! The bridge to cpp/audit_glue.cpp: a kit singleton in the engine an object of a theme lives in.

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("cxx-qt-lib/qobject.h");
        type QObject = cxx_qt::QObject;

        include!("audit_glue.h");
        /// The kit singleton `name` of the engine that made `within`; null when there is none.
        #[cxx_name = "auditKitSingleton"]
        fn kit_singleton(within: Pin<&mut QObject>, name: &QString) -> *mut QObject;
    }
}
