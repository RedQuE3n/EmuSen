import QtQuick
import EmuSen.Theme

// The inks of words drawn on the world, the night sky in both variants: the kit's own by night, the dark variant's by day (EmuSen_Qt.md §3.4.1).
QtObject {
    readonly property bool day: Tokens.variant === Tokens.Light
    readonly property color textStrong: day ? "#EEEEF0" : Tokens.textStrong
    readonly property color text: day ? "#D4D4D4" : Tokens.text
    readonly property color muted: day ? "#8D8D91" : Tokens.muted
}
