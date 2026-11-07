use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("EmuSen.Kit")
            .version(1, 0)
            .qml_files(["qml/Walkable.qml", "qml/Body.qml", "qml/Sky.qml", "qml/Moonrise.qml"])
            .depend("QtQuick")
            .depend("QtQuick.Shapes"),
    )
    .files([
        "src/base.rs",
        "src/kinds.rs",
        "src/tokens.rs",
        "src/frame.rs",
        "src/pad.rs",
        "src/body.rs",
        "src/sky.rs",
        "src/moonrise.rs",
        "src/bells.rs",
    ])
    .include_dir("cpp")
    .cpp_files(["cpp/kit_glue.cpp"])
    .qt_module("Quick")
    .qt_module("QuickControls2")
    .build()
    .export();
    // Both lines are for GNU ld.bfd alone and go once lld is installed (EmuSen_Qt.md §10.4, EmuSen_Kit.md §7.6).
    println!("cargo:rustc-link-arg=-Wl,--no-as-needed");
    for module in ["Quick", "QuickControls2", "Qml", "Gui", "Core"] {
        println!("cargo:rustc-link-arg=-lQt6{module}");
    }
    for symbol in ["crate_emusen_kit_qt", "qml_module_EmuSen_Kit", "crate_cxx_qt_lib", "crate_cxx_qt"] {
        println!("cargo:rustc-link-arg=-Wl,--undefined=cxx_qt_init_{symbol}");
    }
}
