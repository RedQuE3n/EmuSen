import QtQuick
import EmuSen.Theme 1.0
import "../parts"

ThemeScreen {
    objectName: "Games"
    role: ThemeScreen.Games
    Rows {
        rows: Library.recent
        role: "title"
    }
}
