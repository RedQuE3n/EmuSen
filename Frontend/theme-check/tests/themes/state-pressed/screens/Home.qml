import QtQuick
import EmuSen.Theme 1.0
import "../parts"

// A fixture of theme-audit's states: Home's places as rows, the pressed row grown over its neighbours.
ThemeScreen {
    objectName: "Home"
    role: ThemeScreen.Home
    Rows {
        rows: Library.places
        role: "name"
    }
}
