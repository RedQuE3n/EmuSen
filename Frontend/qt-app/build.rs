use cxx_qt_build::{CxxQtBuilder, QmlModule};

// EmuSen.App 1.0: the program's window and its screens (EmuSen_Qt.md §7, stage 1).
fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("EmuSen.App")
            .version(1, 0)
            .qml_files(["qml/Main.qml", "qml/Home.qml", "qml/Consoles.qml", "qml/GameGrid.qml", "qml/CoverFan.qml", "qml/Faces.qml", "qml/Cover.qml", "qml/Glyph.qml", "qml/EngineMark.qml", "qml/PauseCard.qml", "qml/MenuCard.qml", "qml/CardPane.qml", "qml/ThemeHost.qml"])
            .depend("QtQuick")
            .depend("QtQuick.Shapes")
            .depend("QtQuick.Effects")
            .depend("EmuSen.Kit")
            .depend("EmuSen.Models")
            .depend("EmuSen.Endymion")
            .depend("EmuSen.Play")
            .depend("EmuSen.ThemeHost"),
    )
    .files(["src/theme.rs"])
    .qt_module("Quick")
    .qt_module("QuickControls2")
    .build()
    .export();
}
