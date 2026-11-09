//! emusen-theme-qt: the QML module `EmuSen.Theme` 1.0, all a theme sees (EmuSen_Qt.md §3.2). Each singleton is a
//! curated view over emusen-qt-models that the host feeds through the functions of `host`; a theme only reads them and
//! asks for changes through `Actions`. The kit's types are re-exported by name from `qml/`, never the whole kit.

pub mod actions;
pub mod bells;
pub mod browse;
pub mod cues;
pub mod device;
pub mod glue;
pub mod help;
pub mod library;
pub mod marks;
pub mod rows;
pub mod session;
pub mod view;
pub mod words;

/// The host's way to the singletons of the engine a theme runs in: given that engine or any object it made, each
/// singleton to feed, made by the engine on the first ask.
pub mod host {
    use crate::actions::qobject::ThemeActions;
    use crate::bells::qobject::ThemeBells;
    use crate::browse::qobject::ThemeBrowse;
    use crate::cues::qobject::ThemeCues;
    use crate::device::qobject::ThemeDevice;
    use crate::glue::ffi;
    use crate::help::qobject::ThemeHelp;
    use crate::library::qobject::ThemeLibrary;
    use crate::session::qobject::ThemeSession;
    use crate::words::qobject::ThemeWords;
    use core::pin::Pin;
    use cxx_qt::casting::Upcast;
    use cxx_qt::QObject;
    use cxx_qt_lib::QString;

    fn find<'a, T: Upcast<QObject>>(within: Pin<&'a mut QObject>, name: &str) -> Option<Pin<&'a mut T>> {
        typed(ffi::singleton(within, &QString::from(name)))
    }

    fn typed<'a, T: Upcast<QObject>>(object: *mut QObject) -> Option<Pin<&'a mut T>> {
        if object.is_null() {
            return None;
        }
        // Safety: the module registers `name` as a T, and the engine keeps it alive as long as itself.
        let typed = unsafe { T::from_base_ptr(object as *const QObject) } as *mut T;
        unsafe { typed.as_mut() }.map(|t| unsafe { Pin::new_unchecked(t) })
    }

    pub fn library(within: Pin<&mut QObject>) -> Option<Pin<&mut ThemeLibrary>> {
        find(within, "Library")
    }

    pub fn session(within: Pin<&mut QObject>) -> Option<Pin<&mut ThemeSession>> {
        find(within, "Session")
    }

    pub fn device(within: Pin<&mut QObject>) -> Option<Pin<&mut ThemeDevice>> {
        find(within, "Device")
    }

    pub fn actions(within: Pin<&mut QObject>) -> Option<Pin<&mut ThemeActions>> {
        find(within, "Actions")
    }

    pub fn cues(within: Pin<&mut QObject>) -> Option<Pin<&mut ThemeCues>> {
        find(within, "Cues")
    }

    pub fn bells(within: Pin<&mut QObject>) -> Option<Pin<&mut ThemeBells>> {
        find(within, "Bells")
    }

    pub fn words(within: Pin<&mut QObject>) -> Option<Pin<&mut ThemeWords>> {
        find(within, "Words")
    }

    pub fn help(within: Pin<&mut QObject>) -> Option<Pin<&mut ThemeHelp>> {
        find(within, "Help")
    }

    pub fn browse(within: Pin<&mut QObject>) -> Option<Pin<&mut ThemeBrowse>> {
        find(within, "Browse")
    }

    /// The kit's tokens in the theme's engine, which the host keeps in step with the program's (reduced motion).
    pub fn kit_tokens(within: Pin<&mut QObject>) -> Option<Pin<&mut emusen_kit_qt::tokens::qobject::Tokens>> {
        typed(ffi::kit_singleton(within, &QString::from("Tokens")))
    }

    /// The kit's pad router in the theme's engine, which the host hands the pad's presses while a theme's screen holds the focus.
    pub fn pad_router(within: Pin<&mut QObject>) -> Option<Pin<&mut emusen_kit_qt::pad::qobject::PadRouter>> {
        typed(ffi::kit_singleton(within, &QString::from("PadRouter")))
    }
}
