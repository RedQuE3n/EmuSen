//! `Marks`, the drawn marks a theme may place (EmuSen_Qt.md §3.2): the curated face of emusen-qt-models' marks.rs, each
//! an SVG path in a 24-unit box (libretro's badge and alien in an 11-unit one), so a theme draws a mark and never writes it.

use cxx_qt_lib::QString;
use emusen_qt_models::marks;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element = "Marks"]
        #[qml_singleton]
        #[qproperty(QString, heart, READ, CONSTANT)]
        #[qproperty(QString, sparkle, READ, CONSTANT)]
        #[qproperty(QString, crescent, READ, CONSTANT)]
        #[qproperty(QString, attention, READ, CONSTANT)]
        #[qproperty(QString, badge, READ, CONSTANT)]
        #[qproperty(QString, alien, READ, CONSTANT)]
        type ThemeMarks = super::ThemeMarksRust;
    }

    extern "RustQt" {
        /// A shelf's icon by its key, or an engine's mark by a console's `engineMark`; empty for none.
        #[qinvokable]
        fn path(self: &ThemeMarks, name: &QString) -> QString;
    }
}

pub struct ThemeMarksRust {
    heart: QString,
    sparkle: QString,
    crescent: QString,
    attention: QString,
    badge: QString,
    alien: QString,
}

impl Default for ThemeMarksRust {
    fn default() -> Self {
        ThemeMarksRust {
            heart: QString::from(marks::HEART),
            sparkle: QString::from(marks::SPARKLE),
            crescent: QString::from(marks::CRESCENT),
            attention: QString::from(marks::ATTENTION),
            badge: QString::from(marks::BADGE),
            alien: QString::from(marks::alien().as_str()),
        }
    }
}

impl qobject::ThemeMarks {
    pub fn path(&self, name: &QString) -> QString {
        QString::from(marks::path(&name.to_string()))
    }
}
