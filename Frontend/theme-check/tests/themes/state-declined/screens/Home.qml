import QtQuick
import EmuSen.Theme 1.0
import "../parts"

// A fixture of theme-audit's states: Home's places as rows, a declined place's words at 38% as a disabled look, and the last place's words at 38% though it is offered.
ThemeScreen {
    objectName: "Home"
    role: ThemeScreen.Home
    Rows {
        rows: Library.places
        role: "name"
    }
}
