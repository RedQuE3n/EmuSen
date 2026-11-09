use cxx_qt_build::{CxxQtBuilder, QmlModule};

// The kit's own components (EmuSen_Qt.md §1.4); the EmuSen style travels as a resource module, style/EmuSen.qrc (§1.5), and the splash's two faces as fonts/Fonts.qrc (EmuSen_Kit.md §7.10).
const KIT_FILES: [&str; 23] = [
    "qml/Walkable.qml",
    "qml/Body.qml",
    "qml/Sky.qml",
    "qml/Moonrise.qml",
    "qml/Frame.qml",
    "qml/Card.qml",
    "qml/Sheet.qml",
    "qml/Question.qml",
    "qml/HelpBar.qml",
    "qml/MenuPanel.qml",
    "qml/MenuRow.qml",
    "qml/WholeRow.qml",
    "qml/Carousel.qml",
    "qml/NormalizedCanvas.qml",
    "qml/PadGlyph.qml",
    "qml/Scene.qml",
    "qml/Keyboard.qml",
    "qml/NoticeLayer.qml",
    "qml/RoundPicture.qml",
    "qml/CoverTile.qml",
    "qml/OrbitSlot.qml",
    "qml/EdgeRamp.qml",
    "qml/EdgeFade.qml",
];

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("EmuSen.Kit").version(1, 0).qml_files(KIT_FILES).depend("QtQuick").depend("QtQuick.Shapes"),
    )
    .qrc("style/EmuSen.qrc")
    .qrc("fonts/Fonts.qrc")
    .files([
        "src/base.rs",
        "src/item_model.rs",
        "src/kinds.rs",
        "src/tokens.rs",
        "src/frame.rs",
        "src/pad.rs",
        "src/body.rs",
        "src/sky.rs",
        "src/moonrise.rs",
        "src/bells.rs",
        "src/glyph.rs",
        "src/planets.rs",
        "src/orbits.rs",
        "src/figures.rs",
        "src/wheels.rs",
        "src/stack.rs",
        "src/picture.rs",
        "src/keyboard.rs",
        "src/notices.rs",
        "src/keypass.rs",
    ])
    .include_dir("cpp")
    .cpp_files(["cpp/kit_glue.cpp", "cpp/game_picture.h", "cpp/game_picture.cpp"])
    .qt_module("Quick")
    .qt_module("QuickControls2")
    .build()
    .export();
}
