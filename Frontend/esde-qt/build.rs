use std::path::PathBuf;
use std::process::Command;

use cxx_qt_build::{CxxQtBuilder, QmlModule};

#[path = "build/freetype.rs"]
mod freetype;

// EmuSen.Esde 1.3, the painter's module (EmuSen_Qt.md §4.8); its .qmltypes is the module's surface.
const ESDE_FILES: [&str; 21] = [
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
    "qml/GlyphLine.qml",
    "qml/GlyphText.qml",
    "qml/ClipElement.qml",
    "qml/VideoElement.qml",
    "qml/ManualElement.qml",
];

/// The painter's fragment shaders: the picture's one pass, and the media viewer's video blur.
const SHADERS: [&str; 2] = ["picture.frag", "hblur.frag"];

/// Qt's shader baker over the painter's fragment shaders, into a resource the module carries at qrc:/emusen/esde/.
fn baked_shader() -> PathBuf {
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let qsb = ["/usr/lib64/qt6/bin/qsb", "/usr/lib/qt6/bin/qsb", "qsb"]
        .into_iter()
        .find(|p| Command::new(p).arg("--version").output().is_ok())
        .expect("Qt's qsb (qt6-qtshadertools)");
    for shader in SHADERS {
        println!("cargo:rerun-if-changed=shaders/{shader}");
        let baked = out.join(format!("{shader}.qsb"));
        let status = Command::new(qsb).args(["--qt6", "-o"]).arg(&baked).arg(format!("shaders/{shader}")).status().expect("qsb runs");
        assert!(status.success(), "qsb failed on shaders/{shader}");
    }
    // Named for the crate: Qt names a resource's initialiser after its file, and the kit's Shaders.qrc links into the same program.
    let qrc = out.join("EsdeShaders.qrc");
    let files: String = SHADERS.iter().map(|s| format!("    <file>{s}.qsb</file>\n")).collect();
    std::fs::write(&qrc, format!("<RCC>\n  <qresource prefix=\"/emusen/esde\">\n{files}  </qresource>\n</RCC>\n"))
        .expect("the shader's resource file");
    qrc
}

/// The system's FreeType linked by its run-time name, the library Qt already loads, so only libfreetype.so.6 need be
/// installed; pkg-config, where it knows freetype2, adds the folder it is in, and a missing library is named (EmuSen_Qt.md §4.12.4).
fn link_freetype() {
    let pkg_config = match Command::new("pkg-config").args(["--libs-only-L", "freetype2"]).output() {
        Ok(out) if out.status.success() => Ok(String::from_utf8_lossy(&out.stdout).into_owned()),
        Ok(out) => Err(String::from_utf8_lossy(&out.stderr).trim().to_owned()),
        Err(e) => Err(format!("it did not run: {e}")),
    };
    let ldconfig = ["ldconfig", "/sbin/ldconfig"]
        .iter()
        .find_map(|cmd| Command::new(cmd).arg("-p").output().ok().filter(|o| o.status.success()))
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned());
    match freetype::plan(pkg_config, ldconfig) {
        Ok((dirs, warning)) => {
            for dir in dirs {
                println!("cargo:rustc-link-search=native={dir}");
            }
            if let Some(words) = warning {
                println!("cargo:warning={words}");
            }
        }
        Err(words) => panic!("{words}"),
    }
    println!("cargo:rustc-link-lib=dylib:+verbatim={}", freetype::LIBRARY);
}

fn main() {
    link_freetype();
    let shaders = baked_shader();
    CxxQtBuilder::new_qml_module(
        QmlModule::new("EmuSen.Esde").version(1, 3).qml_files(ESDE_FILES).depend("QtQuick").depend("QtQuick.Shapes").depend("QtQml.Models").depend("QtQuick.Pdf").depend("QtMultimedia").depend("EmuSen.Kit"),
    )
    .qrc(shaders)
    .files(["src/marks.rs", "src/model.rs", "src/lists.rs", "src/text.rs"])
    .qt_module("Quick")
    .build()
    .export();
}
