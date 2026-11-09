use cxx_qt_build::{CxxQtBuilder, QmlModule};

// EmuSen.Models 1.0, the view models' module (EmuSen_Qt.md §1.2); its .qmltypes is the module's surface.
fn main() {
    // The project's version, whose owner is still MSBuild's Directory.Build.props until the build host moves to cargo (EmuSen_Qt.md §1.9).
    let props = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Directory.Build.props");
    println!("cargo:rerun-if-changed={}", props.display());
    let text = std::fs::read_to_string(&props).unwrap_or_default();
    let version = text.split("<Version>").nth(1).and_then(|r| r.split("</Version>").next()).unwrap_or(env!("CARGO_PKG_VERSION"));
    println!("cargo:rustc-env=EMUSEN_VERSION={}", version.trim());
    CxxQtBuilder::new_qml_module(QmlModule::new("EmuSen.Models").version(1, 0).depend("QtQml"))
        .files([
            "src/library_model.rs",
            "src/game_model.rs",
            "src/help_model.rs",
            "src/words_object.rs",
            "src/marks_object.rs",
            "src/screens.rs",
            "src/clock.rs",
            "src/session_model.rs",
            "src/question_model.rs",
            "src/pause_model.rs",
            "src/card_model.rs",
            "src/pane_model.rs",
            "src/pictures.rs",
            "src/themes_model.rs",
        ])
        .include_dir("cpp")
        .cpp_files(["cpp/session_pictures.cpp"])
        .qt_module("Quick")
        .qt_module("Qml")
        .build()
        .export();
}
