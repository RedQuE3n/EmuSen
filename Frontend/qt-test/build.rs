use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("EmuSen.Scenes")
            .version(1, 0)
            .qml_files(["qml/MenuScene.qml", "qml/ControlsScene.qml", "qml/SheetScene.qml"])
            .depend("QtQuick")
            .depend("EmuSen.Kit"),
    )
    .files(["src/ffi.rs"])
    .include_dir("cpp")
    .cpp_files(["cpp/harness.cpp"])
    .qt_module("Quick")
    .qt_module("QuickControls2")
    .qt_module("Qml")
    .qt_module("Test")
    .qt_module("QuickTest")
    .build()
    .export();
}
