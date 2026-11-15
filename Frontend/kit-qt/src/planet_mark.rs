//! `PlanetMarks`: a core's planet mark from kit-core's `planet_mark`, its one owner, for the kit's `PlanetMark` (EmuSen_Kit.md §7.115).

use cxx_qt_lib::QString;
use emusen_kit_core::planet_mark;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(f64, box_size, READ, CONSTANT)]
        #[qproperty(f64, stroke, READ, CONSTANT)]
        #[qproperty(f64, badge_box, READ, CONSTANT)]
        #[qproperty(QString, badge, READ, CONSTANT)]
        #[qproperty(QString, alien, READ, CONSTANT)]
        #[qproperty(QString, libretro, READ, CONSTANT)]
        type PlanetMarks = super::PlanetMarksRust;
    }

    #[auto_cxx_name]
    extern "RustQt" {
        /// A mark's path in the box by its name, a console row's `engineMark`; empty for libretro's and for none.
        #[qinvokable]
        fn path(self: &PlanetMarks, mark: &QString) -> QString;
    }
}

pub struct PlanetMarksRust {
    box_size: f64,
    stroke: f64,
    badge_box: f64,
    badge: QString,
    alien: QString,
    libretro: QString,
}

impl Default for PlanetMarksRust {
    fn default() -> PlanetMarksRust {
        PlanetMarksRust {
            box_size: planet_mark::BOX,
            stroke: planet_mark::STROKE,
            badge_box: planet_mark::BADGE_BOX,
            badge: QString::from(planet_mark::BADGE),
            alien: QString::from(planet_mark::alien().as_str()),
            libretro: QString::from(planet_mark::LIBRETRO),
        }
    }
}

impl qobject::PlanetMarks {
    pub fn path(&self, mark: &QString) -> QString {
        QString::from(planet_mark::path(&mark.to_string()))
    }
}
