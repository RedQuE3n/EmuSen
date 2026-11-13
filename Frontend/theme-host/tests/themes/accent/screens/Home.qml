import QtQuick
import EmuSen.Theme

// A stub of the host's tests whose theme.json overrides the dark accent, shadow colour, a number and a switch: each probe reads the theme's.
ThemeScreen {
    objectName: "accentHome"
    role: ThemeScreen.Home
    Rectangle {
        objectName: "accentProbe"
        width: 40
        height: 40
        color: Tokens.accent
    }
    Rectangle {
        objectName: "shadowProbe"
        y: 40
        width: 40
        height: 40
        color: Tokens.shadowColor
    }
    Item {
        objectName: "tokenProbe"
        readonly property real tileRadius: Tokens.tileRadius
        readonly property bool breathes: Tokens.breathes
    }
}
