use cxx_qt_build::{CxxQtBuilder, QmlModule};

// EmuSen.App 1.0: the program's window and its screens (EmuSen_Qt.md §7, stage 1).
fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("EmuSen.App")
            .version(1, 0)
            .qml_files(["qml/Main.qml", "qml/Home.qml"])
            .depend("QtQuick")
            .depend("EmuSen.Kit")
            .depend("EmuSen.Models")
            .depend("EmuSen.Endymion"),
    )
    .qt_module("Quick")
    .qt_module("QuickControls2")
    .build()
    .export();
}
