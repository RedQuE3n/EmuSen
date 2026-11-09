use cxx_qt_build::{CxxQtBuilder, QmlFile, QmlModule};

// EmuSen.Theme 1.0, what a theme sees (EmuSen_Qt.md §3.2); its .qmltypes and qmldir are the module's surface.
fn main() {
    let qml = [
        QmlFile::from("qml/ThemeScreen.qml"),
        QmlFile::from("qml/Tokens.qml").singleton(true),
        QmlFile::from("qml/Frame.qml"),
        QmlFile::from("qml/HelpBar.qml"),
        QmlFile::from("qml/Walkable.qml"),
        QmlFile::from("qml/Body.qml"),
        QmlFile::from("qml/Sky.qml"),
        QmlFile::from("qml/Scene.qml"),
        QmlFile::from("qml/Carousel.qml"),
        QmlFile::from("qml/NormalizedCanvas.qml"),
        QmlFile::from("qml/Moonrise.qml"),
        QmlFile::from("qml/CoverTile.qml"),
        QmlFile::from("qml/RoundPicture.qml"),
        QmlFile::from("qml/PadGlyph.qml"),
        QmlFile::from("qml/PlanetLayout.qml"),
    ];
    CxxQtBuilder::new_qml_module(QmlModule::new("EmuSen.Theme").version(1, 0).qml_files(qml).depend("QtQuick").depend("EmuSen.Kit"))
        .files([
            "src/rows.rs",
            "src/library.rs",
            "src/session.rs",
            "src/device.rs",
            "src/actions.rs",
            "src/cues.rs",
            "src/bells.rs",
            "src/words.rs",
            "src/help.rs",
            "src/browse.rs",
            "src/marks.rs",
            "src/glue.rs",
        ])
        .include_dir("cpp")
        .cpp_files(["cpp/theme_glue.cpp"])
        .qt_module("Quick")
        .qt_module("Qml")
        .build()
        .export();
}
