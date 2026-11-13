use cxx_qt_build::{CxxQtBuilder, QmlModule};

// EmuSen.Models 1.0, the view models' module (EmuSen_Qt.md §1.2); its .qmltypes is the module's surface.
/// The controller drawings, one file each in data/controllers named by the controller's id, as a table the crate
/// includes (EmuSen_Qt.md §7.7.H): a drawing added is a file, and no code.
fn drawings() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("data/controllers");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut files: Vec<_> = std::fs::read_dir(&dir).expect("data/controllers").filter_map(Result::ok).map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "json")).collect();
    files.sort();
    let mut out = String::from("pub const DRAWINGS: &[(&str, &str)] = &[\n");
    for f in &files {
        println!("cargo:rerun-if-changed={}", f.display());
        let id = f.file_stem().and_then(|s| s.to_str()).expect("a drawing's id");
        out.push_str(&format!("    ({id:?}, include_str!({:?})),\n", f.display().to_string()));
    }
    out.push_str("];\n");
    std::fs::write(std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR")).join("drawings.rs"), out).expect("the drawings' table");
}

fn main() {
    drawings();
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
            "src/settings_model.rs",
            "src/rows_model.rs",
            "src/tool_model.rs",
            "src/launch_model.rs",
            "src/folders.rs",
            "src/game_page_model.rs",
        ])
        .include_dir("cpp")
        .cpp_files(["cpp/session_pictures.cpp", "cpp/folders.cpp"])
        .qt_module("Quick")
        .qt_module("Qml")
        .build()
        .export();
}
