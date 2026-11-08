import QtQuick
import EmuSen.Esde

// A theme's picture file or a built-in mark in one box, multiplied by one colour; the one not named draws nothing.
Item {
    id: root
    property url source
    property string mark
    property color tint: Qt.rgba(1, 1, 1, 1)
    Picture {
        anchors.fill: parent
        source: root.source
        tint: root.tint
    }
    Mark {
        anchors.fill: parent
        mark: root.mark
        color: root.tint
    }
}
