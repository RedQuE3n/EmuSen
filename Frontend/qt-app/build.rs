use cxx_qt_build::{CxxQtBuilder, QmlModule};

// EmuSen.App 1.0: the program's window and its screens (EmuSen_Qt.md §7, stage 1).
fn main() {
    CxxQtBuilder::new_qml_module(
        QmlModule::new("EmuSen.App")
            .version(1, 0)
            .qml_files(["qml/Main.qml", "qml/PauseCard.qml", "qml/MenuCard.qml", "qml/MenuCardReference.qml", "qml/MenuCardEclipse.qml", "qml/MenuRowEclipse.qml", "qml/EclipsePillHighlight.qml", "qml/EclipseFooter.qml", "qml/QuestionDialogView.qml", "qml/QuestionEclipse.qml", "qml/QuestionHost.qml", "qml/PowerDialogEclipse.qml", "qml/NoticeEclipse.qml", "qml/CardLine.qml", "qml/CardGlyph.qml", "qml/CardChevron.qml", "qml/CardPlanet.qml", "qml/CardPane.qml", "qml/CardPaneEclipse.qml", "qml/ThemeHost.qml", "qml/Themes.qml", "qml/Settings.qml", "qml/SettingsScreen.qml", "qml/ControllersEclipse.qml", "qml/Cores.qml", "qml/GamePage.qml", "qml/SearchScreen.qml", "qml/ListCardScreen.qml", "qml/ThemesScreen.qml", "qml/ToolScreen.qml", "qml/PageScreen.qml", "qml/HomeCardScreen.qml", "qml/PauseScreen.qml", "qml/ResumeQuestion.qml", "qml/LaunchQuestion.qml", "qml/NoticeScreen.qml", "qml/ThemeSplash.qml"])
            .depend("QtQuick")
            .depend("QtQuick.Shapes")
            .depend("QtQuick.Effects")
            .depend("QtQuick.Dialogs")
            .depend("EmuSen.Kit")
            .depend("EmuSen.Models")
            .depend("EmuSen.Endymion")
            .depend("EmuSen.Play")
            .depend("EmuSen.ThemeHost")
            .depend("EmuSen.Esde"),
    )
    .files(["src/theme.rs", "src/style.rs", "src/cues.rs", "src/splash_sound.rs"])
    .include_dir("cpp")
    .cpp_files(["cpp/compositor_answer.h", "cpp/compositor_answer.cpp", "cpp/sound_file.h", "cpp/sound_file.cpp"])
    .qrc("themes.qrc")
    .qt_module("Quick")
    .qt_module("QuickControls2")
    // A theme's sound read through Qt Multimedia's decoder (cpp/sound_file.cpp), the module the kit's VideoClip already links.
    .qt_module("Multimedia")
    .build()
    .export();
}
