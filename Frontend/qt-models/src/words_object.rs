//! `Words`, the screens' fixed words as a QML singleton read from words.rs, so a QML file names a word and never writes one.

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(QString, title, READ, CONSTANT)]
        type Words = super::WordsRust;
    }
}

use cxx_qt_lib::QString;

/// The words, each from words.rs.
pub struct WordsRust {
    title: QString,
}

impl Default for WordsRust {
    fn default() -> Self {
        WordsRust { title: QString::from(crate::words::TITLE) }
    }
}
