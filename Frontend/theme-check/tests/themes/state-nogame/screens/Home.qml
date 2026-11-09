import QtQuick
import EmuSen.Theme 1.0
import "../parts"

// A fixture of theme-audit's states: Home's places as rows under a Continue entry drawn whether or not a game is suspended.
ThemeScreen {
    objectName: "Home"
    role: ThemeScreen.Home
    Rows {
        rows: Library.places
        role: "name"
    }
}
