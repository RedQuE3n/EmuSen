use cxx_qt_build::{CxxQtBuilder, QmlModule};

// EmuSen.Play 1.0: the program's one window (EmuSen_Qt.md §1.8).
fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("EmuSen.Play")
            .version(1, 0)
            .qml_files(["qml/Play.qml", "qml/Game.qml"])
            .depend("QtQuick")
            .depend("EmuSen.Kit")
            .depend("EmuSen.Models"),
    )
    .files(["src/window.rs"])
    .qt_module("Quick")
    .build()
    .export();
}
