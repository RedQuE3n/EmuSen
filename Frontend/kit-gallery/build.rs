use cxx_qt_build::{CxxQtBuilder, QmlModule};

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("EmuSen.Gallery")
            .version(1, 0)
            .qml_files(["qml/Gallery.qml", "qml/Screens.qml"])
            .depend("QtQuick")
            .depend("EmuSen.Kit")
            .depend("EmuSen.Endymion"),
    )
    .files(["src/grab.rs"])
    .qt_module("Quick")
    .qt_module("QuickControls2")
    .build()
    .export();
}
