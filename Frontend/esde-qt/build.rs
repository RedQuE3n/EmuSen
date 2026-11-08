use std::path::PathBuf;
use std::process::Command;

use cxx_qt_build::{CxxQtBuilder, QmlModule};

// EmuSen.Esde 1.0, the painter's module (EmuSen_Qt.md §4.8); its .qmltypes is the module's surface.
const ESDE_FILES: [&str; 16] = [
    "qml/Picture.qml",
    "qml/Mark.qml",
    "qml/TextBlock.qml",
    "qml/Icon.qml",
    "qml/ElementBox.qml",
    "qml/PictureElement.qml",
    "qml/TextElement.qml",
    "qml/RatingElement.qml",
    "qml/BadgesElement.qml",
    "qml/InfoElement.qml",
    "qml/StatusElement.qml",
    "qml/HelpElement.qml",
    "qml/CarouselElement.qml",
    "qml/TextListView.qml",
    "qml/GridElement.qml",
    "qml/View.qml",
];

/// Qt's shader baker over the picture's fragment shader, into a resource the module carries at qrc:/emusen/esde/.
fn baked_shader() -> PathBuf {
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let qsb = ["/usr/lib64/qt6/bin/qsb", "/usr/lib/qt6/bin/qsb", "qsb"]
        .into_iter()
        .find(|p| Command::new(p).arg("--version").output().is_ok())
        .expect("Qt's qsb (qt6-qtshadertools)");
    println!("cargo:rerun-if-changed=shaders/picture.frag");
    let baked = out.join("picture.frag.qsb");
    let status = Command::new(qsb).args(["--qt6", "-o"]).arg(&baked).arg("shaders/picture.frag").status().expect("qsb runs");
    assert!(status.success(), "qsb failed on shaders/picture.frag");
    let qrc = out.join("Shaders.qrc");
    std::fs::write(&qrc, "<RCC>\n  <qresource prefix=\"/emusen/esde\">\n    <file>picture.frag.qsb</file>\n  </qresource>\n</RCC>\n")
        .expect("the shader's resource file");
    qrc
}

fn main() {
    let shaders = baked_shader();
    CxxQtBuilder::new_qml_module(
        QmlModule::new("EmuSen.Esde").version(1, 0).qml_files(ESDE_FILES).depend("QtQuick").depend("QtQuick.Shapes").depend("QtQml.Models").depend("EmuSen.Kit"),
    )
    .qrc(shaders)
    .files(["src/base.rs", "src/marks.rs", "src/model.rs", "src/lists.rs"])
    .qt_module("Quick")
    .build()
    .export();
}
