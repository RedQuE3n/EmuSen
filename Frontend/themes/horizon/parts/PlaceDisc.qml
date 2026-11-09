import QtQuick
import EmuSen.Theme

// A place in the strip: a glass disc of 56 with its mark at 24, and the attention mark at its top right when the place asks for the player (EmuSen_Qt.md §3.4.2).
Item {
    id: root
    property string mark
    property bool attention: false
    readonly property bool night: Tokens.variant === Tokens.Dark
    width: 56
    height: 56
    Rectangle {
        anchors.fill: parent
        radius: 28
        color: root.night ? "#8026262B" : "#99FFFFFF"
        border.width: 1
        border.color: root.night ? "#2E8C96DC" : "#241F2A44"
    }
    Glyph {
        x: 16
        y: 16
        width: 24
        height: 24
        path: root.mark
        ink: Tokens.textStrong
        strokeWidth: 1.8
    }
    Glyph {
        objectName: "placeAttention"
        visible: root.attention
        x: root.width - 14
        y: -2
        width: 16
        height: 16
        path: Marks.attention
        filled: true
        ink: Tokens.attention
    }
}
