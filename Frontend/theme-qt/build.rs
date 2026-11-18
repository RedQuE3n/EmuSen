use cxx_qt_build::{CxxQtBuilder, QmlFile, QmlModule};

// EmuSen.Theme 1.10, what a theme sees, each file at the minor it was added in (EmuSen_Qt.md §3.2, rule T).
fn main() {
    let qml = [
        QmlFile::from("qml/ThemeScreen.qml").version(1, 0),
        QmlFile::from("qml/Tokens.qml").singleton(true).version(1, 0),
        QmlFile::from("qml/Frame.qml").version(1, 0),
        QmlFile::from("qml/HelpBar.qml").version(1, 0),
        QmlFile::from("qml/Walkable.qml").version(1, 0),
        QmlFile::from("qml/Body.qml").version(1, 0),
        QmlFile::from("qml/Sky.qml").version(1, 0),
        QmlFile::from("qml/Scene.qml").version(1, 0),
        QmlFile::from("qml/Carousel.qml").version(1, 0),
        QmlFile::from("qml/NormalizedCanvas.qml").version(1, 0),
        QmlFile::from("qml/Moonrise.qml").version(1, 0),
        QmlFile::from("qml/CoverTile.qml").version(1, 0),
        QmlFile::from("qml/RoundPicture.qml").version(1, 0),
        QmlFile::from("qml/PadGlyph.qml").version(1, 0),
        QmlFile::from("qml/PlanetLayout.qml").version(1, 0),
        QmlFile::from("qml/OrbitLayout.qml").version(1, 0),
        QmlFile::from("qml/FigureLayout.qml").version(1, 0),
        QmlFile::from("qml/WheelLayout.qml").version(1, 0),
        QmlFile::from("qml/OrbitSlot.qml").version(1, 0),
        QmlFile::from("qml/EdgeFade.qml").version(1, 0),
        QmlFile::from("qml/FocusRing.qml").version(1, 1),
        QmlFile::from("qml/PlanetMark.qml").version(1, 2),
        QmlFile::from("qml/CoronaGlow.qml").version(1, 2),
        QmlFile::from("qml/CoronaDisc.qml").version(1, 2),
        QmlFile::from("qml/BackdropField.qml").version(1, 4),
        QmlFile::from("qml/Backdrop.qml").version(1, 4),
        QmlFile::from("qml/Glow.qml").version(1, 4),
        QmlFile::from("qml/GroupGridLayout.qml").version(1, 5),
        QmlFile::from("qml/MeasuredWalk.qml").version(1, 9),
    ];
    CxxQtBuilder::new_qml_module(QmlModule::new("EmuSen.Theme").version(1, 10).qml_files(qml).depend("QtQuick").depend("EmuSen.Kit"))
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
            "src/settings.rs",
            "src/page.rs",
            "src/splash.rs",
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
