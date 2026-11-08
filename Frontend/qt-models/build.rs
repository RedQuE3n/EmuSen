use cxx_qt_build::{CxxQtBuilder, QmlModule};

// EmuSen.Models 1.0, the view models' module (EmuSen_Qt.md §1.2); its .qmltypes is the module's surface.
fn main() {
    CxxQtBuilder::new_qml_module(QmlModule::new("EmuSen.Models").version(1, 0).depend("QtQml"))
        .files(["src/library_model.rs", "src/session_model.rs"])
        .qt_module("Qml")
        .build()
        .export();
}
