//! `CanvasUnits`: the big screen's design height from kit-core, the one owner `NormalizedCanvas` scales by (EmuSen_Kit.md §7.112).

#[cxx_qt::bridge]
pub mod qobject {
    #[auto_cxx_name]
    extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[qml_singleton]
        #[qproperty(f64, design_height, READ, CONSTANT)]
        type CanvasUnits = super::CanvasUnitsRust;
    }
}

pub struct CanvasUnitsRust {
    design_height: f64,
}

impl Default for CanvasUnitsRust {
    fn default() -> CanvasUnitsRust {
        CanvasUnitsRust { design_height: emusen_kit_core::canvas::DESIGN_HEIGHT }
    }
}
