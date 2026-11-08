use cxx_qt_build::{CxxQtBuilder, QmlModule};

// EmuSen.Endymion 1.0, Endymion's pads and interface player for the interface (EmuSen_Kit.md §7.15); its .qmltypes is the module's surface.
fn main() {
    CxxQtBuilder::new_qml_module(QmlModule::new("EmuSen.Endymion").version(1, 0).depend("QtQml").depend("EmuSen.Kit"))
        .files(["src/pad_reader.rs", "src/bell_player.rs"])
        .qt_module("Qml")
        .build()
        .export();
}
