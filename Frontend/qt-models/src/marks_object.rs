//! `Marks`, the drawn marks of marks.rs as a QML singleton, so a QML file draws a mark and never writes its path.

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
        #[qproperty(QString, heart, READ, CONSTANT)]
        #[qproperty(QString, sparkle, READ, CONSTANT)]
        #[qproperty(QString, crescent, READ, CONSTANT)]
        #[qproperty(QString, attention, READ, CONSTANT)]
        #[qproperty(QString, badge, READ, CONSTANT)]
        #[qproperty(QString, alien, READ, CONSTANT)]
        type Marks = super::MarksRust;
    }

    extern "RustQt" {
        /// A place's icon or an engine's mark by its name; empty for none.
        #[qinvokable]
        fn path(self: &Marks, name: &QString) -> QString;
    }
}

use crate::marks;
use cxx_qt_lib::QString;

/// The marks, each from marks.rs.
pub struct MarksRust {
    heart: QString,
    sparkle: QString,
    crescent: QString,
    attention: QString,
    badge: QString,
    alien: QString,
}

impl Default for MarksRust {
    fn default() -> Self {
        MarksRust {
            heart: QString::from(marks::HEART),
            sparkle: QString::from(marks::SPARKLE),
            crescent: QString::from(marks::CRESCENT),
            attention: QString::from(marks::ATTENTION),
            badge: QString::from(marks::BADGE),
            alien: QString::from(marks::alien().as_str()),
        }
    }
}

impl qobject::Marks {
    pub fn path(&self, name: &QString) -> QString {
        QString::from(marks::path(&name.to_string()))
    }
}
