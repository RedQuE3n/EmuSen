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
}
