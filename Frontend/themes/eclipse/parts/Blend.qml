import QtQuick

// Two colours mixed channel by channel, alpha too, at a share `t` of the second: the hand-over's colours (EmuSen_Qt.md §3.4.8, the splash).
QtObject {
    property color from
    property color to
    property real t
    readonly property color value: Qt.rgba(from.r * (1 - t) + to.r * t, from.g * (1 - t) + to.g * t, from.b * (1 - t) + to.b * t, from.a * (1 - t) + to.a * t)
}
