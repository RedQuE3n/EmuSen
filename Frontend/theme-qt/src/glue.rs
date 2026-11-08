//! The C++ glue of `cpp/theme_glue.cpp` (EmuSen_Qt.md §3.2): a list made as a singleton's child, and a singleton found in
//! the engine an object lives in, which is how the host reaches the views it feeds.

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("theme_glue.h");
        include!(<QtCore/QObject>);
        type QObject = cxx_qt::QObject;
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
        include!("emusen-theme-qt/src/rows.cxxqt.h");
        type ThemeRows = crate::rows::qobject::ThemeRows;
        include!("emusen-theme-qt/src/help.cxxqt.h");
        type ThemeHelpEntries = crate::help::qobject::ThemeHelpEntries;

        /// A list owned by `parent`, kept from every engine's collector.
        #[cxx_name = "themeNewRows"]
        fn new_rows(parent: Pin<&mut QObject>) -> *mut ThemeRows;

        /// A help bar's entries owned by `parent`.
        #[cxx_name = "themeNewHelpEntries"]
        fn new_help_entries(parent: Pin<&mut QObject>) -> *mut ThemeHelpEntries;

        /// The module's singleton `name` in `within`'s engine, or `within` itself where it is the engine; null where there is none.
        #[cxx_name = "themeSingleton"]
        fn singleton(within: Pin<&mut QObject>, name: &QString) -> *mut QObject;
    }
}
