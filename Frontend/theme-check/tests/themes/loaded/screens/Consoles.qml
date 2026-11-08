import QtQuick
import EmuSen.Theme 1.0
import "../parts"

ThemeScreen {
    objectName: "Consoles"
    role: ThemeScreen.Consoles
    Rows {
        rows: Library.consoles
        role: "name"
    }
}
