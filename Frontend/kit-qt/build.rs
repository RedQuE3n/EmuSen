use std::path::PathBuf;
use std::process::Command;

use cxx_qt_build::{CxxQtBuilder, QmlModule};

// The kit's own components (EmuSen_Qt.md §1.4); the EmuSen style travels as a resource module, style/EmuSen.qrc (§1.5), and the splash's two faces as fonts/Fonts.qrc (EmuSen_Kit.md §7.10).
const KIT_FILES: [&str; 34] = [
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
    "qml/EdgeFade.qml",
    "qml/SettingsFrame.qml",
    "qml/ChoiceRow.qml",
    "qml/SliderRow.qml",
    "qml/SectionStrip.qml",
    "qml/ToolScreen.qml",
    "qml/PageFrame.qml",
    "qml/ToneText.qml",
    "qml/ActionButton.qml",
    "qml/VideoTile.qml",
    "qml/VideoClip.qml",
    "qml/RaisedShadow.qml",
    "qml/ControllerDrawing.qml",
];

/// Qt's shader baker over EdgeFade's fragment shader, into a resource beside the module's own at qrc:/qt/qml/EmuSen/Kit/shaders/ (EmuSen_Kit.md §7.35).
fn baked_shader() -> PathBuf {
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    let qsb = ["/usr/lib64/qt6/bin/qsb", "/usr/lib/qt6/bin/qsb", "qsb"]
        .into_iter()
        .find(|p| Command::new(p).arg("--version").output().is_ok())
        .expect("Qt's qsb (qt6-qtshadertools)");
    println!("cargo:rerun-if-changed=shaders/edge_fade.frag");
    let baked = out.join("edge_fade.frag.qsb");
    let status = Command::new(qsb).args(["--qt6", "-o"]).arg(&baked).arg("shaders/edge_fade.frag").status().expect("qsb runs");
    assert!(status.success(), "qsb failed on shaders/edge_fade.frag");
    let qrc = out.join("Shaders.qrc");
    std::fs::write(&qrc, "<RCC>\n  <qresource prefix=\"/qt/qml/EmuSen/Kit/shaders\">\n    <file>edge_fade.frag.qsb</file>\n  </qresource>\n</RCC>\n")
        .expect("the shader's resource file");
    qrc
}

fn main() {
    let shaders = baked_shader();
    CxxQtBuilder::new_qml_module(
        QmlModule::new("EmuSen.Kit").version(1, 2).qml_files(KIT_FILES).depend("QtQuick").depend("QtQuick.Shapes").depend("QtMultimedia"),
    )
    .qrc("style/EmuSen.qrc")
    .qrc("fonts/Fonts.qrc")
    .qrc(shaders)
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
        "src/current.rs",
        "src/settings.rs",
        "src/tones.rs",
        "src/tool.rs",
        "src/page.rs",
        "src/table.rs",
        "src/reset_gate.rs",
        "src/controller.rs",
    ])
    .include_dir("cpp")
    .cpp_files(["cpp/kit_glue.cpp", "cpp/game_picture.h", "cpp/game_picture.cpp", "cpp/edge_mask.h", "cpp/edge_mask.cpp", "cpp/inside_clips.h", "cpp/inside_clips.cpp", "cpp/text_width.h", "cpp/text_width.cpp", "cpp/video_frames.h", "cpp/video_frames.cpp"])
    .qt_module("Quick")
    .qt_module("QuickControls2")
    // The clip's frames on the software backend (VideoFrames); the decoder is Qt Multimedia's, over the system's FFmpeg (EmuSen_Qt.md §1.8.9).
    .qt_module("Multimedia")
    .build()
    .export();
}
