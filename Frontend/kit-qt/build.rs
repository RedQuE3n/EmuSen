use cxx_qt_build::{CxxQtBuilder, QmlModule};

// The kit's own components (EmuSen_Qt.md §1.4); the EmuSen style travels as a resource module, style/EmuSen.qrc (§1.5), and the splash's two faces as fonts/Fonts.qrc (EmuSen_Kit.md §7.10).
const KIT_FILES: [&str; 14] = [
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
];

fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("EmuSen.Kit").version(1, 0).qml_files(KIT_FILES).depend("QtQuick").depend("QtQuick.Shapes"),
    )
    .qrc("style/EmuSen.qrc")
    .qrc("fonts/Fonts.qrc")
    .files([
        "src/base.rs",
        "src/kinds.rs",
        "src/tokens.rs",
        "src/frame.rs",
        "src/pad.rs",
        "src/body.rs",
        "src/sky.rs",
        "src/moonrise.rs",
        "src/bells.rs",
    ])
    .include_dir("cpp")
    .cpp_files(["cpp/kit_glue.cpp"])
    .qt_module("Quick")
    .qt_module("QuickControls2")
    .build()
    .export();
}
