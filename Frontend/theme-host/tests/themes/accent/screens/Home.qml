import QtQuick
import EmuSen.Theme

// A stub of the host's tests whose theme.json overrides the dark accent: the probe is drawn in the theme's accent.
ThemeScreen {
    objectName: "accentHome"
    role: ThemeScreen.Home
    Rectangle {
        objectName: "accentProbe"
        width: 40
        height: 40
        color: Tokens.accent
    }
}
