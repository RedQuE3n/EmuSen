import QtQuick
import EmuSen.Theme

// A stub of the host's tests whose theme.json overrides the dark accent and shadow colour: each probe is drawn in the theme's.
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
}
