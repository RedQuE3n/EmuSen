import QtQuick
import EmuSen.Theme 1.0
import "../parts"

// A fixture of theme-audit's states: Home's places as rows, each enabled only while the host would take it, so a declined place cannot hold the focus.
ThemeScreen {
    objectName: "Home"
    role: ThemeScreen.Home
    Rows {
        rows: Library.places
        role: "name"
    }
}
