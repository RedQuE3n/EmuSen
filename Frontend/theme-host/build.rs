use cxx_qt_build::{CxxQtBuilder, QmlModule};

// EmuSen.ThemeHost 1.0, the program's host of a theme (EmuSen_Qt.md §3.5); the two C++ shells are cpp/sandbox.cpp's.
fn main() {
    CxxQtBuilder::new_qml_module(QmlModule::new("EmuSen.ThemeHost").version(1, 0).depend("QtQuick"))
        .files(["src/sandbox.rs", "src/host.rs"])
        .include_dir("cpp")
        .cpp_files(["cpp/sandbox.cpp"])
        .qt_module("Quick")
        .qt_module("Network")
        .build()
        .export();
}
